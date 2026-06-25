//! Booking logs service — read-only DB access.

use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect};

use crate::app_state::AppState;
use crate::entity::booking_log;
use crate::error::AppError;

use super::dto::ListQuery;

/// List booking logs with pagination, optionally filtered by booking_id.
pub async fn list(
    state: &AppState,
    q: &ListQuery,
) -> Result<(Vec<booking_log::Model>, u64), AppError> {
    let page = q.page.unwrap_or(1);
    let ps = q.page_size.unwrap_or(20);
    let mut s = booking_log::Entity::find();
    if let Some(bid) = q.booking_id {
        s = s.filter(booking_log::Column::BookingId.eq(bid));
    }
    let total = s
        .clone()
        .count(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?;
    let rows = s
        .order_by_desc(booking_log::Column::CreateTime)
        .offset(((page - 1) * ps) as u64)
        .limit(ps)
        .all(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?;
    Ok((rows, total))
}

/// Read a single booking log by id.
pub async fn read(state: &AppState, id: i32) -> Result<booking_log::Model, AppError> {
    booking_log::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?
        .ok_or(AppError::NotFound("不存在".into()))
}
