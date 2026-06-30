//! Booking logs handlers — read-only list. JWT-protected.

use axum::Json;
use axum::extract::{Path, Query, State};

use crate::app_state::AppState;
use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use crate::response::{ApiResponse, PaginatedData};

use super::dto::ListQuery;
use super::service;

/// GET /booking-logs — list booking audit logs.
#[utoipa::path(
    get,
    path = "/booking-logs",
    params(ListQuery),
    responses(
        (status = 200, body = ApiResponse<PaginatedData<serde_json::Value>>),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "booking-logs",
)]
pub async fn list(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(q): Query<ListQuery>,
) -> Result<Json<ApiResponse<PaginatedData<serde_json::Value>>>, AppError> {
    let page = q.page.unwrap_or(1);
    let ps = q.page_size.unwrap_or(20);
    let (rows, total) = service::list(&state, &q).await?;
    let list: Vec<_> = rows
        .iter()
        .map(|r| serde_json::to_value(r).unwrap())
        .collect();
    Ok(Json(ApiResponse::ok(PaginatedData::new(
        list, total, page, ps,
    ))))
}

/// GET /booking-logs/{id} — read a single booking log entry.
#[utoipa::path(
    get,
    path = "/booking-logs/{id}",
    params(("id" = i32, Path, description = "Booking log ID")),
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found"),
    ),
    tag = "booking-logs",
)]
pub async fn read(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let r = service::read(&state, id).await?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(r).unwrap())))
}
