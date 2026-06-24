//! Booking logs — read-only list. JWT-protected.

use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect};
use serde::Deserialize;

use crate::app_state::AppState;
use crate::entity::booking_log;
use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use crate::response::{ApiResponse, PaginatedData};

#[derive(Debug, Deserialize)]
pub struct ListQuery { pub page: Option<u64>, pub page_size: Option<u64>, pub booking_id: Option<i32> }

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/booking-logs", get(list))
        .route("/booking-logs/{id}", get(read))
}

async fn list(State(state): State<AppState>, _auth: AuthUser, Query(q): Query<ListQuery>) -> Result<Json<ApiResponse<PaginatedData<serde_json::Value>>>, AppError> {
    let page = q.page.unwrap_or(1); let ps = q.page_size.unwrap_or(20);
    let mut s = booking_log::Entity::find();
    if let Some(bid) = q.booking_id { s = s.filter(booking_log::Column::BookingId.eq(bid)); }
    let total = s.clone().count(&state.db).await.map_err(|e| AppError::Internal(format!("DB:{e}")))?;
    let rows = s.order_by_desc(booking_log::Column::CreateTime).offset(((page-1)*ps) as u64).limit(ps).all(&state.db).await.map_err(|e| AppError::Internal(format!("DB:{e}")))?;
    let list: Vec<_> = rows.iter().map(|r| serde_json::to_value(r).unwrap()).collect();
    Ok(Json(ApiResponse::ok(PaginatedData::new(list,total,page,ps))))
}

async fn read(State(state): State<AppState>, _auth: AuthUser, Path(id): Path<i32>) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let r = booking_log::Entity::find_by_id(id).one(&state.db).await.map_err(|e| AppError::Internal(format!("DB:{e}")))?.ok_or(AppError::NotFound("不存在".into()))?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(r).unwrap())))
}
