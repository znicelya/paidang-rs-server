//! Bookings handlers - JWT-protected, ownership scoped.

use axum::Json;
use axum::extract::{Path, Query, State};
use validator::Validate;

use crate::app_state::AppState;
use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use crate::response::{ApiResponse, PaginatedData};
use crate::util::require_owner;

use super::dto::*;
use super::service;

/// GET /bookings - list bookings with pagination and filters.
#[utoipa::path(
    get,
    path = "/bookings",
    params(BookingListQuery),
    responses(
        (status = 200, body = ApiResponse<PaginatedData<serde_json::Value>>),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "bookings",
)]
pub async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<BookingListQuery>,
) -> Result<Json<ApiResponse<PaginatedData<serde_json::Value>>>, AppError> {
    let page = q.page.unwrap_or(1);
    let page_size = q.page_size.unwrap_or(20);
    let (rows, total) = service::list(&state, &q, auth.user_id).await?;
    let list: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| serde_json::to_value(r).unwrap())
        .collect();
    Ok(Json(ApiResponse::ok(PaginatedData::new(
        list, total, page, page_size,
    ))))
}

/// GET /bookings/{id} - read a single booking by id.
#[utoipa::path(
    get,
    path = "/bookings/{id}",
    params(("id" = i32, Path, description = "Booking ID")),
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    ),
    tag = "bookings",
)]
pub async fn read(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let b = service::read(&state, id).await?;
    require_owner(&auth, b.photographer_id)?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(b).unwrap())))
}

/// POST /bookings - create a new booking.
#[utoipa::path(
    post,
    path = "/bookings",
    request_body = CreateBookingRequest,
    responses(
        (status = 200, body = ApiResponse<CreateBookingData>),
        (status = 400, description = "Input validation error"),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "bookings",
)]
pub async fn create(
    State(state): State<AppState>,
    Json(body): Json<CreateBookingRequest>,
) -> Result<Json<ApiResponse<CreateBookingData>>, AppError> {
    body.validate()
        .map_err(|e| AppError::InputValidation(e.to_string()))?;
    let data = service::create(&state, &body, None, "customer").await?;
    Ok(Json(ApiResponse::ok(data)))
}

/// PUT /bookings/{id} - update an existing booking.
#[utoipa::path(
    put,
    path = "/bookings/{id}",
    params(("id" = i32, Path, description = "Booking ID")),
    request_body = UpdateBookingRequest,
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 400, description = "Input validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    ),
    tag = "bookings",
)]
pub async fn update(
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

/// DELETE /bookings/{id} - delete a booking.
#[utoipa::path(
    delete,
    path = "/bookings/{id}",
    params(("id" = i32, Path, description = "Booking ID")),
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    ),
    tag = "bookings",
)]
pub async fn delete_booking(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    let existing = service::read(&state, id).await?;
    require_owner(&auth, existing.photographer_id)?;
    service::delete(&state, id).await?;
    Ok(Json(ApiResponse::ok(())))
}

/// GET /bookings/stats - booking statistics.
#[utoipa::path(
    get,
    path = "/bookings/stats",
    params(StatsQuery),
    responses(
        (status = 200, body = ApiResponse<StatsData>),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "bookings",
)]
pub async fn stats(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(_q): Query<StatsQuery>,
) -> Result<Json<ApiResponse<StatsData>>, AppError> {
    let data = service::stats(&state, Some(auth.user_id)).await?;
    Ok(Json(ApiResponse::ok(data)))
}

/// GET /bookings/today - bookings for today.
#[utoipa::path(
    get,
    path = "/bookings/today",
    params(TodayQuery),
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "bookings",
)]
pub async fn today(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(_q): Query<TodayQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    use chrono::Local;
    let today_str = Local::now().format("%Y-%m-%d").to_string();

    let query = BookingListQuery {
        page: Some(1),
        page_size: Some(200),
        booking_date: Some(today_str),
        photographer_id: Some(auth.user_id),
        status: None,
    };
    let (rows, _) = service::list(&state, &query, auth.user_id).await?;
    let list: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| serde_json::to_value(r).unwrap())
        .collect();
    Ok(Json(ApiResponse::ok(
        serde_json::json!({ "list": list, "total": list.len() }),
    )))
}
