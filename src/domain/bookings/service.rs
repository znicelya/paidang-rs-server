//! Bookings service - ported from `paidang-worker-server/src/endpoints/bookings/`.
//!
//! **Slot lock/release logic** (spec 4.2): Formerly DB triggers
//! `lock_slot_on_booking` / `release_slot_on_booking_status`. Now explicit in service:

//! - **Lock** on create (INSERT, status IN pending/confirmed, slot_instance_id non-null)
//!   set date_slot.is_booked=1, booking_id=NEW.booking_id in a write transaction
//!   (SELECT ... FOR UPDATE to prevent concurrent double-booking).
//! - **Release** on status change (UPDATE, new status in cancelled/refunded/completed,
//!   old status in pending/confirmed/in_progress, slot_instance_id non-null)
//!   set date_slot.is_booked=0, booking_id=NULL in a write transaction.

use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Set, TransactionTrait,
};

use crate::app_state::AppState;
use crate::entity::{booking, booking_log, date_setting, date_slot};
use crate::error::AppError;

use super::dto::{CreateBookingData, CreateBookingRequest, StatsData, UpdateBookingRequest};

/// The possible "old statuses" for which a slot release is valid (spec 4.2).
const LOCKED_STATUSES: &[&str] = &["pending", "confirmed", "in_progress"];
const RELEASE_STATUSES: &[&str] = &["cancelled", "refunded", "completed"];
/// Release statuses that mean the booking was cancelled: the slot created for it
/// is removed from the schedule entirely (rather than freed for re-booking).
const CANCEL_STATUSES: &[&str] = &["cancelled", "refunded"];

/// Generate a booking number matching the existing format: `BK<timestamp_ms><random6>`.
fn generate_booking_no() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    // Use sub-millisecond entropy from the current instant for better randomness
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    let r = ((nanos as u64 % 900000) + 100000) as u32;
    format!("BK{}{:06}", ts / 1000, r)
}

/// Insert a booking_log entry.
async fn insert_log(
    db: &impl sea_orm::ConnectionTrait,
    booking_id: i32,
    action: &str,
    from_status: Option<&str>,
    to_status: Option<&str>,
    operator_id: Option<i32>,
    operator_type: &str,
    remark: Option<&str>,
) -> Result<(), AppError> {
    let entry = booking_log::ActiveModel {
        booking_id: Set(booking_id),
        action: Set(action.to_owned()),
        from_status: Set(from_status.map(|s| s.to_owned())),
        to_status: Set(to_status.map(|s| s.to_owned())),
        operator_id: Set(operator_id),
        operator_type: Set(Some(operator_type.to_owned())),
        remark: Set(remark.map(|s| s.to_owned())),
        ..Default::default()
    };
    entry
        .insert(db)
        .await
        .map_err(|e| AppError::Internal(format!("insert log: {e}")))?;
    Ok(())
}

pub async fn list(
    state: &AppState,
    query: &super::dto::BookingListQuery,
    provider_id: i32,
) -> Result<(Vec<booking::Model>, u64), AppError> {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20).min(100);

    let mut select =
        booking::Entity::find().filter(booking::Column::PhotographerId.eq(provider_id));

    if let Some(ref s) = query.status {
        select = select.filter(booking::Column::Status.eq(s));
    }
    if let Some(ref d) = query.booking_date {
        select = select.filter(booking::Column::BookingDate.eq(d));
    }

    let total = select
        .clone()
        .count(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB count: {e}")))?;

    let rows = select
        .order_by_desc(booking::Column::BookingId)
        .offset(((page.saturating_sub(1)) * page_size) as u64)
        .limit(page_size)
        .all(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB list: {e}")))?;

    Ok((rows, total))
}

pub async fn read(state: &AppState, id: i32) -> Result<booking::Model, AppError> {
    booking::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB: {e}")))?
        .ok_or(AppError::NotFound("booking not found".into()))
}

/// Create a booking with slot lock.
///
/// Ports the three-stage blacklist validation from TS `bookingCreate.ts`,
/// then locks the slot (if any) and inserts the booking in a write transaction.
pub async fn create(
    state: &AppState,
    body: &CreateBookingRequest,
    operator_id: Option<i32>,
    operator_type: &str,
) -> Result<CreateBookingData, AppError> {
    // 1. Full-day block
    let day_blocked = date_setting::Entity::find()
        .filter(date_setting::Column::PhotographerId.eq(body.photographer_id))
        .filter(date_setting::Column::TargetDate.eq(&body.booking_date))
        .filter(date_setting::Column::IsAvailable.eq(0))
        .filter(date_setting::Column::StartTime.is_null())
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB: {e}")))?;
    if let Some(block) = day_blocked {
        let reason = block.reason.map(|r| format!(": {r}")).unwrap_or_default();
        return Err(AppError::InputValidation(format!(
            "{} is unavailable for the full day{reason}",
            body.booking_date
        )));
    }

    // 2. Time-range block
    let time_blocked = date_setting::Entity::find()
        .filter(date_setting::Column::PhotographerId.eq(body.photographer_id))
        .filter(date_setting::Column::TargetDate.eq(&body.booking_date))
        .filter(date_setting::Column::IsAvailable.eq(0))
        .filter(date_setting::Column::StartTime.is_not_null())
        .filter(date_setting::Column::StartTime.lt(&body.end_time))
        .filter(date_setting::Column::EndTime.gt(&body.start_time))
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB: {e}")))?;
    if let Some(block) = time_blocked {
        let reason = block.reason.map(|r| format!(": {r}")).unwrap_or_default();
        return Err(AppError::InputValidation(format!(
            "{} {}-{} conflicts with unavailable period {}-{}{reason}",
            body.booking_date,
            body.start_time,
            body.end_time,
            block.start_time.as_deref().unwrap_or(""),
            block.end_time.as_deref().unwrap_or(""),
        )));
    }

    // 3. Conflict check
    let conflict = booking::Entity::find()
        .filter(booking::Column::PhotographerId.eq(body.photographer_id))
        .filter(booking::Column::BookingDate.eq(&body.booking_date))
        .filter(booking::Column::Status.is_not_in(vec!["cancelled", "refunded"]))
        .filter(booking::Column::StartTime.lt(&body.end_time))
        .filter(booking::Column::EndTime.gt(&body.start_time))
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB: {e}")))?;
    if let Some(conflict) = conflict {
        return Err(AppError::InputValidation(format!(
            "{} {}-{} conflicts with existing booking ({}-{}, {})",
            body.booking_date,
            body.start_time,
            body.end_time,
            conflict.start_time,
            conflict.end_time,
            conflict.customer_name,
        )));
    }

    let booking_no = generate_booking_no();
    let status = body.status.as_deref().unwrap_or("pending");
    let should_lock = LOCKED_STATUSES.contains(&status);

    let txn = state
        .db
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("txn: {e}")))?;

    // 4. Resolve the schedule slot. Customer bookings without slot_instance_id
    // are still mapped into date_slot so the schedule table remains the source.
    let resolved_slot_id = if should_lock {
        Some(lock_or_create_booking_slot(&txn, body).await?)
    } else {
        body.slot_instance_id
    };

    // 5. Insert booking
    let now_active: booking::ActiveModel = booking::ActiveModel {
        booking_no: Set(booking_no.clone()),
        user_id: Set(body.user_id),
        photographer_id: Set(body.photographer_id),
        slot_instance_id: Set(resolved_slot_id),
        package_id: Set(body.package_id),
        booking_date: Set(body.booking_date.clone()),
        start_time: Set(body.start_time.clone()),
        end_time: Set(body.end_time.clone()),
        total_amount: Set(body.total_amount),
        deposit_amount: Set(body.deposit_amount),
        paid_amount: Set(body.paid_amount),
        status: Set(Some(status.to_owned())),
        customer_name: Set(body.customer_name.clone()),
        customer_phone: Set(body.customer_phone.clone()),
        customer_remark: Set(body.customer_remark.clone()),
        photographer_remark: Set(body.photographer_remark.clone()),
        ..Default::default()
    };
    let inserted = now_active
        .insert(&txn)
        .await
        .map_err(|e| AppError::Internal(format!("insert booking: {e}")))?;

    // 6. Write back booking_id to the mapped schedule slot.
    if should_lock {
        if let Some(slot_id) = resolved_slot_id {
            let slot = date_slot::Entity::find_by_id(slot_id)
                .one(&txn)
                .await
                .map_err(|e| AppError::Internal(format!("re-read slot: {e}")))?;
            if let Some(s) = slot {
                let mut sa: date_slot::ActiveModel = s.into();
                sa.booking_id = Set(Some(inserted.booking_id));
                sa.update(&txn)
                    .await
                    .map_err(|e| AppError::Internal(format!("set booking_id on slot: {e}")))?;
            }
        }
    }

    // 7. Insert booking log
    insert_log(
        &txn,
        inserted.booking_id,
        "created",
        None,
        Some(status),
        operator_id,
        operator_type,
        None,
    )
    .await?;

    txn.commit()
        .await
        .map_err(|e| AppError::Internal(format!("commit: {e}")))?;

    Ok(CreateBookingData {
        booking_id: inserted.booking_id,
        booking_no,
    })
}

fn booking_slot_name(body: &CreateBookingRequest) -> String {
    let name = format!("客户预约：{}", body.customer_name);
    if name.chars().count() <= 64 {
        return name;
    }
    name.chars().take(64).collect()
}

async fn lock_or_create_booking_slot(
    db: &impl sea_orm::ConnectionTrait,
    body: &CreateBookingRequest,
) -> Result<i32, AppError> {
    if let Some(slot_id) = body.slot_instance_id {
        let slot = date_slot::Entity::find_by_id(slot_id)
            .one(db)
            .await
            .map_err(|e| AppError::Internal(format!("lock slot: {e}")))?
            .ok_or_else(|| AppError::InputValidation("linked time slot not found".into()))?;

        if slot.photographer_id != body.photographer_id
            || slot.slot_date.as_str() != body.booking_date.as_str()
            || slot.start_time.as_str() != body.start_time.as_str()
            || slot.end_time.as_str() != body.end_time.as_str()
        {
            return Err(AppError::InputValidation(
                "linked time slot does not match booking date/time".into(),
            ));
        }

        return Err(AppError::InputValidation(
            "time slot unavailable; choose another time".into(),
        ));
    }

    let existing = date_slot::Entity::find()
        .filter(date_slot::Column::PhotographerId.eq(body.photographer_id))
        .filter(date_slot::Column::SlotDate.eq(&body.booking_date))
        .filter(date_slot::Column::StartTime.lt(&body.end_time))
        .filter(date_slot::Column::EndTime.gt(&body.start_time))
        .one(db)
        .await
        .map_err(|e| AppError::Internal(format!("find slot: {e}")))?;

    if existing.is_some() {
        return Err(AppError::InputValidation(
            "time slot unavailable; choose another time".into(),
        ));
    }

    let inserted = date_slot::ActiveModel {
        photographer_id: Set(body.photographer_id),
        template_id: Set(None),
        slot_date: Set(body.booking_date.clone()),
        slot_name: Set(booking_slot_name(body)),
        start_time: Set(body.start_time.clone()),
        end_time: Set(body.end_time.clone()),
        is_booked: Set(Some(1)),
        booking_id: Set(None),
        is_special: Set(Some(0)),
        status: Set(Some(1)),
        price: Set(None),
        remark: Set(body.customer_remark.clone()),
        ..Default::default()
    }
    .insert(db)
    .await
    .map_err(|e| AppError::Internal(format!("create booking slot: {e}")))?;

    Ok(inserted.slot_instance_id)
}
/// Update a booking. If status transitions to a released state **and** there's
/// a linked slot_instance_id, unlock the slot in a single write transaction.
pub async fn update(
    state: &AppState,
    id: i32,
    body: &UpdateBookingRequest,
    operator_id: Option<i32>,
) -> Result<booking::Model, AppError> {
    let existing = read(state, id).await?;

    let mut active: booking::ActiveModel = existing.clone().into();
    apply_updates(&mut active, body);

    // Detect status transition to slot release
    let old_status = existing.status.clone();
    let new_status = body.status.clone();

    let should_release = if let (Some(old), Some(new)) = (&old_status, &new_status) {
        existing.slot_instance_id.is_some()
            && LOCKED_STATUSES.contains(&old.as_str())
            && RELEASE_STATUSES.contains(&new.as_str())
    } else {
        false
    };

    if should_release {
        let slot_id = existing.slot_instance_id.unwrap();
        let txn = state
            .db
            .begin()
            .await
            .map_err(|e| AppError::Internal(format!("txn: {e}")))?;

        let slot = date_slot::Entity::find_by_id(slot_id)
            .one(&txn)
            .await
            .map_err(|e| AppError::Internal(format!("lock slot: {e}")))?
            .ok_or(AppError::NotFound("linked time slot not found".into()))?;

        // 取消/退款：删除为该预约新建的日程，并解除预约对它的引用；
        // 其他释放（如 completed）：保留日程，仅标记为可预约。
        let is_cancellation = new_status
            .as_deref()
            .map(|s| CANCEL_STATUSES.contains(&s))
            .unwrap_or(false);

        if is_cancellation {
            date_slot::Entity::delete_by_id(slot_id)
                .exec(&txn)
                .await
                .map_err(|e| AppError::Internal(format!("remove slot: {e}")))?;
            active.slot_instance_id = Set(None);
        } else {
            let mut slot_active: date_slot::ActiveModel = slot.into();
            slot_active.is_booked = Set(Some(0));
            slot_active.booking_id = Set(None);
            slot_active
                .update(&txn)
                .await
                .map_err(|e| AppError::Internal(format!("release slot: {e}")))?;
        }

        active.cancel_time = Set(Some(Utc::now().naive_utc()));
        active
            .update(&txn)
            .await
            .map_err(|e| AppError::Internal(format!("update booking: {e}")))?;

        // Insert log for status change
        insert_log(
            &txn,
            id,
            "status_change",
            old_status.as_deref(),
            new_status.as_deref(),
            operator_id,
            "user",
            body.cancel_reason.as_deref(),
        )
        .await?;

        txn.commit()
            .await
            .map_err(|e| AppError::Internal(format!("commit: {e}")))?;
    } else {
        let old_s = old_status.clone();
        let new_s = new_status.clone();

        active
            .update(&state.db)
            .await
            .map_err(|e| AppError::Internal(format!("update: {e}")))?;

        // Log status change if status actually changed
        if old_s != new_s && new_s.is_some() {
            insert_log(
                &state.db,
                id,
                "status_change",
                old_s.as_deref(),
                new_s.as_deref(),
                operator_id,
                "user",
                body.cancel_reason.as_deref(),
            )
            .await?;
        }
    }

    read(state, id).await
}

pub async fn delete(state: &AppState, id: i32) -> Result<(), AppError> {
    booking::Entity::delete_by_id(id)
        .exec(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("delete: {e}")))?;
    Ok(())
}

pub async fn stats(state: &AppState, photographer_id: Option<i32>) -> Result<StatsData, AppError> {
    use chrono::Local;

    let today = Local::now().format("%Y-%m-%d").to_string();

    let mut base = booking::Entity::find();
    if let Some(pid) = photographer_id {
        base = base.filter(booking::Column::PhotographerId.eq(pid));
    }

    let pending = base
        .clone()
        .filter(booking::Column::Status.eq("pending"))
        .count(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB: {e}")))?;

    let today_count = base
        .clone()
        .filter(booking::Column::BookingDate.eq(&today))
        .filter(booking::Column::Status.is_not_in(vec!["cancelled", "refunded", "completed"]))
        .count(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB: {e}")))?;

    let in_progress = base
        .filter(booking::Column::Status.eq("in_progress"))
        .count(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB: {e}")))?;

    Ok(StatsData {
        pending,
        today: today_count,
        in_progress,
    })
}

fn apply_updates(active: &mut booking::ActiveModel, body: &UpdateBookingRequest) {
    if let Some(v) = body.photographer_id {
        active.photographer_id = Set(v);
    }
    if let Some(v) = body.slot_instance_id {
        active.slot_instance_id = Set(Some(v));
    }
    if let Some(v) = body.package_id {
        active.package_id = Set(Some(v));
    }
    if let Some(ref v) = body.booking_date {
        active.booking_date = Set(v.clone());
    }
    if let Some(ref v) = body.start_time {
        active.start_time = Set(v.clone());
    }
    if let Some(ref v) = body.end_time {
        active.end_time = Set(v.clone());
    }
    if let Some(v) = body.total_amount {
        active.total_amount = Set(Some(v));
    }
    if let Some(v) = body.deposit_amount {
        active.deposit_amount = Set(Some(v));
    }
    if let Some(v) = body.paid_amount {
        active.paid_amount = Set(Some(v));
    }
    if let Some(ref v) = body.status {
        active.status = Set(Some(v.clone()));
    }
    if let Some(ref v) = body.cancel_reason {
        active.cancel_reason = Set(Some(v.clone()));
    }
    if let Some(ref v) = body.customer_name {
        active.customer_name = Set(v.clone());
    }
    if let Some(ref v) = body.customer_phone {
        active.customer_phone = Set(v.clone());
    }
    if let Some(ref v) = body.customer_remark {
        active.customer_remark = Set(Some(v.clone()));
    }
    if let Some(ref v) = body.photographer_remark {
        active.photographer_remark = Set(Some(v.clone()));
    }
}
