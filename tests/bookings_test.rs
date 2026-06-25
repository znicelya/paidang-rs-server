//! Bookings integration tests: create validation, slot locking, and log writes.

mod common;

use common::setup;
use paidang_rs_server::domain::bookings::dto::{
    BookingListQuery, CreateBookingRequest, UpdateBookingRequest,
};
use paidang_rs_server::domain::bookings::service;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

/// Insert a minimal user so FK-like references pass service-layer checks.
async fn seed_photographer(ctx: &common::TestContext) -> i32 {
    use paidang_rs_server::entity::user;
    use sea_orm::{ActiveModelTrait, Set};
    let m = user::ActiveModel {
        openid: Set("wx-photographer".into()),
        role: Set(1),
        status: Set(1),
        ..Default::default()
    }
    .insert(&ctx.state.db)
    .await
    .unwrap();
    m.user_id
}

// Creation with validation.

#[tokio::test]
async fn create_booking_success() {
    let ctx = setup().await;
    let pid = seed_photographer(&ctx).await;

    let body = CreateBookingRequest {
        photographer_id: pid,
        user_id: None,
        slot_instance_id: None,
        package_id: None,
        booking_date: "2026-08-01".into(),
        start_time: "10:00".into(),
        end_time: "11:00".into(),
        total_amount: Some(10000),
        deposit_amount: Some(2000),
        paid_amount: Some(0),
        status: Some("pending".into()),
        customer_name: "CustomerA".into(),
        customer_phone: "13800138000".into(),
        customer_remark: None,
        photographer_remark: None,
    };

    let result = service::create(&ctx.state, &body, None, "customer")
        .await
        .unwrap();
    assert!(result.booking_id > 0);
    assert!(result.booking_no.starts_with("BK"));

    // Verify log was written
    use paidang_rs_server::entity::booking_log;
    let logs = booking_log::Entity::find()
        .filter(booking_log::Column::BookingId.eq(result.booking_id))
        .all(&ctx.state.db)
        .await
        .unwrap();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].action, "created");
    assert_eq!(logs[0].to_status.as_deref(), Some("pending"));
    assert_eq!(logs[0].operator_id, None);
    assert_eq!(logs[0].operator_type.as_deref(), Some("customer"));
}

#[tokio::test]
async fn create_booking_conflict_detected() {
    let ctx = setup().await;
    let pid = seed_photographer(&ctx).await;

    let body = CreateBookingRequest {
        photographer_id: pid,
        user_id: None,
        slot_instance_id: None,
        package_id: None,
        booking_date: "2026-08-01".into(),
        start_time: "14:00".into(),
        end_time: "16:00".into(),
        total_amount: None,
        deposit_amount: None,
        paid_amount: None,
        status: None,
        customer_name: "CustomerB".into(),
        customer_phone: "13900139000".into(),
        customer_remark: None,
        photographer_remark: None,
    };

    // First booking succeeds
    service::create(&ctx.state, &body, Some(pid), "provider")
        .await
        .unwrap();

    // Overlapping booking should fail
    let conflict = CreateBookingRequest {
        start_time: "15:00".into(),
        end_time: "17:00".into(),
        customer_name: "CustomerConflict".into(),
        customer_phone: "13700137000".into(),
        ..body.clone()
    };
    let result = service::create(&ctx.state, &conflict, Some(pid), "provider").await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("conflicts"),
        "expected conflict error, got: {err}"
    );
}

#[tokio::test]
async fn create_booking_full_day_block() {
    let ctx = setup().await;
    let pid = seed_photographer(&ctx).await;

    // Insert a full-day block
    use paidang_rs_server::entity::date_setting;
    use sea_orm::{ActiveModelTrait, Set};
    date_setting::ActiveModel {
        photographer_id: Set(pid),
        target_date: Set("2026-08-01".into()),
        is_available: Set(Some(0)),
        ..Default::default()
    }
    .insert(&ctx.state.db)
    .await
    .unwrap();

    let body = CreateBookingRequest {
        photographer_id: pid,
        user_id: None,
        slot_instance_id: None,
        package_id: None,
        booking_date: "2026-08-01".into(),
        start_time: "10:00".into(),
        end_time: "11:00".into(),
        total_amount: None,
        deposit_amount: None,
        paid_amount: None,
        status: None,
        customer_name: "CustomerBlocked".into(),
        customer_phone: "13600136000".into(),
        customer_remark: None,
        photographer_remark: None,
    };

    let result = service::create(&ctx.state, &body, Some(pid), "provider").await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("unavailable for the full day"),
        "expected full-day block error"
    );
}

// List with pagination.

#[tokio::test]
async fn list_bookings_pagination() {
    let ctx = setup().await;
    let pid = seed_photographer(&ctx).await;

    // Create 3 bookings
    for i in 0..3 {
        let body = CreateBookingRequest {
            photographer_id: pid,
            user_id: None,
            slot_instance_id: None,
            package_id: None,
            booking_date: format!("2026-08-{:02}", 10 + i),
            start_time: "10:00".into(),
            end_time: "11:00".into(),
            total_amount: None,
            deposit_amount: None,
            paid_amount: None,
            status: Some("pending".into()),
            customer_name: format!("User{i}"),
            customer_phone: format!("1380000000{i}"),
            customer_remark: None,
            photographer_remark: None,
        };
        service::create(&ctx.state, &body, Some(pid), "provider")
            .await
            .unwrap();
    }

    let query = BookingListQuery {
        page: Some(1),
        page_size: Some(10),
        photographer_id: Some(pid),
        status: Some("pending".into()),
        booking_date: None,
    };
    let (rows, total) = service::list(&ctx.state, &query, pid).await.unwrap();
    assert_eq!(total, 3);
    assert_eq!(rows.len(), 3);
}

// Read.

#[tokio::test]
async fn read_booking_not_found() {
    let ctx = setup().await;
    let result = service::read(&ctx.state, 99999).await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("booking not found")
    );
}

// Status change and booking log.

#[tokio::test]
async fn update_status_writes_log() {
    let ctx = setup().await;
    let pid = seed_photographer(&ctx).await;

    let body = CreateBookingRequest {
        photographer_id: pid,
        user_id: None,
        slot_instance_id: None,
        package_id: None,
        booking_date: "2026-08-15".into(),
        start_time: "09:00".into(),
        end_time: "10:00".into(),
        total_amount: None,
        deposit_amount: None,
        paid_amount: None,
        status: Some("pending".into()),
        customer_name: "CustomerStatus".into(),
        customer_phone: "13500135000".into(),
        customer_remark: None,
        photographer_remark: None,
    };
    let created = service::create(&ctx.state, &body, Some(pid), "provider")
        .await
        .unwrap();

    // Update status to confirmed
    let update = UpdateBookingRequest {
        status: Some("confirmed".into()),
        ..Default::default()
    };
    service::update(&ctx.state, created.booking_id, &update, Some(pid))
        .await
        .unwrap();

    // Verify two logs: created + status_change
    use paidang_rs_server::entity::booking_log;
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
    let logs = booking_log::Entity::find()
        .filter(booking_log::Column::BookingId.eq(created.booking_id))
        .all(&ctx.state.db)
        .await
        .unwrap();
    assert_eq!(logs.len(), 2, "expected 2 logs (created + status_change)");
    let actions: Vec<_> = logs.iter().map(|l| l.action.clone()).collect();
    assert!(actions.contains(&"created".to_string()));
    assert!(actions.contains(&"status_change".to_string()));
}

// Stats.

#[tokio::test]
async fn stats_returns_counts() {
    let ctx = setup().await;
    let pid = seed_photographer(&ctx).await;

    let body = CreateBookingRequest {
        photographer_id: pid,
        user_id: None,
        slot_instance_id: None,
        package_id: None,
        booking_date: chrono::Local::now().format("%Y-%m-%d").to_string(),
        start_time: "09:00".into(),
        end_time: "10:00".into(),
        total_amount: None,
        deposit_amount: None,
        paid_amount: None,
        status: Some("pending".into()),
        customer_name: "Stats".into(),
        customer_phone: "13400134000".into(),
        customer_remark: None,
        photographer_remark: None,
    };
    service::create(&ctx.state, &body, Some(pid), "provider")
        .await
        .unwrap();

    let stats = service::stats(&ctx.state, Some(pid)).await.unwrap();
    assert!(stats.pending >= 1);
}

// Delete.

#[tokio::test]
async fn delete_booking_removes_record() {
    let ctx = setup().await;
    let pid = seed_photographer(&ctx).await;

    let body = CreateBookingRequest {
        photographer_id: pid,
        user_id: None,
        slot_instance_id: None,
        package_id: None,
        booking_date: "2026-08-20".into(),
        start_time: "08:00".into(),
        end_time: "09:00".into(),
        total_amount: None,
        deposit_amount: None,
        paid_amount: None,
        status: None,
        customer_name: "Delete".into(),
        customer_phone: "13300133000".into(),
        customer_remark: None,
        photographer_remark: None,
    };
    let created = service::create(&ctx.state, &body, Some(pid), "provider")
        .await
        .unwrap();

    service::delete(&ctx.state, created.booking_id)
        .await
        .unwrap();

    let result = service::read(&ctx.state, created.booking_id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn list_bookings_ignores_query_photographer_and_scopes_to_provider() {
    let ctx = setup().await;
    let pid_a = seed_photographer(&ctx).await;
    let pid_b = {
        use paidang_rs_server::entity::user;
        use sea_orm::{ActiveModelTrait, Set};
        user::ActiveModel {
            openid: Set("wx-photographer-b".into()),
            role: Set(2),
            status: Set(1),
            ..Default::default()
        }
        .insert(&ctx.state.db)
        .await
        .unwrap()
        .user_id
    };

    for (pid, name, date) in [
        (pid_a, "Scoped A", "2026-09-01"),
        (pid_b, "Scoped B", "2026-09-02"),
    ] {
        let body = CreateBookingRequest {
            photographer_id: pid,
            user_id: None,
            slot_instance_id: None,
            package_id: None,
            booking_date: date.into(),
            start_time: "10:00".into(),
            end_time: "11:00".into(),
            total_amount: None,
            deposit_amount: None,
            paid_amount: None,
            status: Some("pending".into()),
            customer_name: name.into(),
            customer_phone: format!("13800000{pid:03}"),
            customer_remark: None,
            photographer_remark: None,
        };
        service::create(&ctx.state, &body, Some(pid), "provider")
            .await
            .unwrap();
    }

    let query = BookingListQuery {
        page: Some(1),
        page_size: Some(10),
        photographer_id: Some(pid_b),
        status: None,
        booking_date: None,
    };

    let (rows, total) = service::list(&ctx.state, &query, pid_a).await.unwrap();
    assert_eq!(total, 1);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].photographer_id, pid_a);
}
