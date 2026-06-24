//! Bookings handlers + router.

use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use validator::Validate;

use crate::app_state::AppState;
use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use crate::response::{ApiResponse, PaginatedData};

use super::dto::*;
use super::service;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/bookings", get(list).post(create))
        .route("/bookings/stats", get(stats))
        .route("/bookings/today", get(today))
        .route("/bookings/{id}", get(read).put(update).delete(delete_booking))
}

/// Require that the authenticated user is the owner of the resource or an admin.
fn require_owner(auth: &AuthUser, photographer_id: i32) -> Result<(), AppError> {
    if auth.role >= 2 || auth.user_id == photographer_id {
        Ok(())
    } else {
        Err(AppError::Forbidden("无权操作此资源".into()))
    }
}

// ── Handlers ────────────────────────────────────────────────

async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<BookingListQuery>,
) -> Result<Json<ApiResponse<PaginatedData<serde_json::Value>>>, AppError> {
    let page = q.page.unwrap_or(1);
    let page_size = q.page_size.unwrap_or(20);
    let (rows, total) = service::list(&state, &q, Some(auth.user_id), auth.role).await?;
    let list: Vec<serde_json::Value> =
        rows.iter().map(|r| serde_json::to_value(r).unwrap()).collect();
    Ok(Json(ApiResponse::ok(PaginatedData::new(
        list, total, page, page_size,
    ))))
}

async fn read(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let b = service::read(&state, id).await?;
    require_owner(&auth, b.photographer_id)?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(b).unwrap())))
}

async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<CreateBookingRequest>,
) -> Result<Json<ApiResponse<CreateBookingData>>, AppError> {
    body.validate()
        .map_err(|e| AppError::InputValidation(e.to_string()))?;
    let data = service::create(&state, &body, Some(auth.user_id)).await?;
    Ok(Json(ApiResponse::ok(data)))
}

async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(body): Json<UpdateBookingRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    body.validate()
        .map_err(|e| AppError::InputValidation(e.to_string()))?;
    // Check ownership before update
    let existing = service::read(&state, id).await?;
    require_owner(&auth, existing.photographer_id)?;
    let b = service::update(&state, id, &body, Some(auth.user_id)).await?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(b).unwrap())))
}

async fn delete_booking(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    let existing = service::read(&state, id).await?;
    require_owner(&auth, existing.photographer_id)?;
    service::delete(&state, id).await?;
    Ok(Json(ApiResponse::ok(())))
}

async fn stats(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<StatsQuery>,
) -> Result<Json<ApiResponse<StatsData>>, AppError> {
    // Enforce owner scoping: non-admin can only see own stats
    let pid = if auth.role >= 2 {
        q.photographer_id
    } else {
        Some(auth.user_id)
    };
    let data = service::stats(&state, pid).await?;
    Ok(Json(ApiResponse::ok(data)))
}

async fn today(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<TodayQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    use chrono::Local;
    let today_str = Local::now().format("%Y-%m-%d").to_string();

    let query = BookingListQuery {
        page: Some(1),
        page_size: Some(200),
        booking_date: Some(today_str),
        photographer_id: q.photographer_id,
        status: None,
    };
    // Use real auth role instead of hardcoded admin (2)
    let (rows, _) =
        service::list(&state, &query, Some(auth.user_id), auth.role).await?;
    let list: Vec<serde_json::Value> =
        rows.iter().map(|r| serde_json::to_value(r).unwrap()).collect();
    Ok(Json(ApiResponse::ok(
        serde_json::json!({ "list": list, "total": list.len() }),
    )))
}

