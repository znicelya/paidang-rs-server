//! Date slots service.

use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};

use crate::app_state::AppState;
use crate::entity::date_slot;
use crate::error::AppError;

use super::dto::{CreateReq, ListQuery, UpdateReq};

/// List date slots with pagination, optionally scoped to one provider.
pub async fn list(
    state: &AppState,
    q: &ListQuery,
    photographer_id: Option<i32>,
) -> Result<(Vec<date_slot::Model>, u64), AppError> {
    let page = q.page.unwrap_or(1);
    let ps = q.page_size.unwrap_or(20);
    let mut s = date_slot::Entity::find();

    // Provider scoping is applied by the caller when needed.
    if let Some(pid) = photographer_id {
        s = s.filter(date_slot::Column::PhotographerId.eq(pid));
    }
    if let Some(ref d) = q.slot_date {
        s = s.filter(date_slot::Column::SlotDate.eq(d));
    }
    let total = s
        .clone()
        .count(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?;
    let rows = s
        .order_by_asc(date_slot::Column::SlotDate)
        .order_by_asc(date_slot::Column::StartTime)
        .offset(((page - 1) * ps) as u64)
        .limit(ps)
        .all(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?;
    Ok((rows, total))
}

/// Read a single date slot by id.
pub async fn read(state: &AppState, id: i32) -> Result<date_slot::Model, AppError> {
    date_slot::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?
        .ok_or(AppError::NotFound("不存在".into()))
}

/// Create a new date slot.
pub async fn create(state: &AppState, body: CreateReq) -> Result<date_slot::Model, AppError> {
    let m = date_slot::ActiveModel {
        photographer_id: Set(body.photographer_id),
        template_id: Set(body.template_id),
        slot_date: Set(body.slot_date),
        slot_name: Set(body.slot_name),
        start_time: Set(body.start_time),
        end_time: Set(body.end_time),
        is_special: Set(body.is_special),
        status: Set(body.status),
        price: Set(body.price),
        remark: Set(body.remark),
        ..Default::default()
    }
    .insert(&state.db)
    .await
    .map_err(AppError::from_db)?;
    Ok(m)
}

/// Update a date slot by id.
pub async fn update(
    state: &AppState,
    id: i32,
    body: UpdateReq,
) -> Result<date_slot::Model, AppError> {
    let rec = date_slot::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?
        .ok_or(AppError::NotFound("不存在".into()))?;
    let mut a: date_slot::ActiveModel = rec.into();
    if let Some(v) = body.slot_name {
        a.slot_name = Set(v);
    }
    if let Some(v) = body.start_time {
        a.start_time = Set(v);
    }
    if let Some(v) = body.end_time {
        a.end_time = Set(v);
    }
    if let Some(v) = body.is_special {
        a.is_special = Set(Some(v));
    }
    if let Some(v) = body.status {
        a.status = Set(Some(v));
    }
    if let Some(v) = body.price {
        a.price = Set(Some(v));
    }
    if let Some(v) = body.remark {
        a.remark = Set(Some(v));
    }
    let r = a
        .update(&state.db)
        .await
        .map_err(AppError::from_db)?;
    Ok(r)
}

/// Delete a date slot by id.
pub async fn delete_one(state: &AppState, id: i32) -> Result<(), AppError> {
    date_slot::Entity::delete_by_id(id)
        .exec(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?;
    Ok(())
}

/// Slots for a specific day.
pub async fn day(
    state: &AppState,
    photographer_id: i32,
    slot_date: &str,
) -> Result<Vec<date_slot::Model>, AppError> {
    let rows = date_slot::Entity::find()
        .filter(date_slot::Column::PhotographerId.eq(photographer_id))
        .filter(date_slot::Column::SlotDate.eq(slot_date))
        .order_by_asc(date_slot::Column::StartTime)
        .all(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?;
    Ok(rows)
}

/// Slots for a month (year_month prefix match).
pub async fn monthly(
    state: &AppState,
    photographer_id: i32,
    year_month: &str,
) -> Result<Vec<date_slot::Model>, AppError> {
    let prefix = format!("{}%", year_month);
    let rows = date_slot::Entity::find()
        .filter(date_slot::Column::PhotographerId.eq(photographer_id))
        .filter(date_slot::Column::SlotDate.like(&prefix))
        .order_by_asc(date_slot::Column::SlotDate)
        .order_by_asc(date_slot::Column::StartTime)
        .all(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?;
    Ok(rows)
}
