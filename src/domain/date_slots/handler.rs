//! Date slots handlers.

use axum::extract::{Path, Query, State};
use axum::Json;
use validator::Validate;

use crate::app_state::AppState;
use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use crate::response::{ApiResponse, PaginatedData};

use super::dto::*;
use super::service;

/// Require that the authenticated user is the owner of the resource or an admin.
fn require_owner(auth: &AuthUser, photographer_id: i32) -> Result<(), AppError> {
    if auth.role >= 2 || auth.user_id == photographer_id {
        Ok(())
    } else {
        Err(AppError::Forbidden("无权操作此资源".into()))
    }
}

/// GET /date-slots — list date slots.
#[utoipa::path(
    get,
    path = "/date-slots",
    params(ListQuery),
    responses(
        (status = 200, body = ApiResponse<PaginatedData<serde_json::Value>>),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "date-slots",
)]
pub async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<ListQuery>,
) -> Result<Json<ApiResponse<PaginatedData<serde_json::Value>>>, AppError> {
    let page = q.page.unwrap_or(1);
    let ps = q.page_size.unwrap_or(20);
    // Owner scoping: non-admin can only see their own
    let photographer_id = if auth.role >= 2 {
        q.photographer_id
    } else {
        Some(auth.user_id)
    };
    let (rows, total) = service::list(&state, &q, photographer_id).await?;
    let list: Vec<_> = rows
        .iter()
        .map(|r| serde_json::to_value(r).unwrap())
        .collect();
    Ok(Json(ApiResponse::ok(PaginatedData::new(
        list, total, page, ps,
    ))))
}

/// GET /date-slots/{id} — read a single date slot.
#[utoipa::path(
    get,
    path = "/date-slots/{id}",
    params(("id" = i32, Path, description = "Date slot ID")),
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    ),
    tag = "date-slots",
)]
pub async fn read(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let r = service::read(&state, id).await?;
    require_owner(&auth, r.photographer_id)?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(r).unwrap())))
}

/// POST /date-slots — create a new date slot.
#[utoipa::path(
    post,
    path = "/date-slots",
    request_body = CreateReq,
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 400, description = "Input validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    tag = "date-slots",
)]
pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<CreateReq>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    body.validate()
        .map_err(|e| AppError::InputValidation(e.to_string()))?;
    require_owner(&auth, body.photographer_id)?;
    let m = service::create(&state, body).await?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(m).unwrap())))
}

/// PUT /date-slots/{id} — update a date slot.
#[utoipa::path(
    put,
    path = "/date-slots/{id}",
    params(("id" = i32, Path, description = "Date slot ID")),
    request_body = UpdateReq,
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    ),
    tag = "date-slots",
)]
pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(body): Json<UpdateReq>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let rec = service::read(&state, id).await?;
    require_owner(&auth, rec.photographer_id)?;
    let r = service::update(&state, id, body).await?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(r).unwrap())))
}

/// DELETE /date-slots/{id} — delete a date slot.
#[utoipa::path(
    delete,
    path = "/date-slots/{id}",
    params(("id" = i32, Path, description = "Date slot ID")),
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    ),
    tag = "date-slots",
)]
pub async fn delete_one(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    let rec = service::read(&state, id).await?;
    require_owner(&auth, rec.photographer_id)?;
    service::delete_one(&state, id).await?;
    Ok(Json(ApiResponse::ok(())))
}

/// GET /date-slots/day — slots for a specific day.
#[utoipa::path(
    get,
    path = "/date-slots/day",
    params(DayQuery),
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    tag = "date-slots",
)]
pub async fn day(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<DayQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    require_owner(&auth, q.photographer_id)?;
    let rows = service::day(&state, q.photographer_id, &q.slot_date).await?;
    let list: Vec<_> = rows
        .iter()
        .map(|r| serde_json::to_value(r).unwrap())
        .collect();
    Ok(Json(ApiResponse::ok(serde_json::json!({"list":list}))))
}

/// GET /date-slots/monthly — slots for a month.
#[utoipa::path(
    get,
    path = "/date-slots/monthly",
    params(MonthlyQuery),
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    tag = "date-slots",
)]
pub async fn monthly(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<MonthlyQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    require_owner(&auth, q.photographer_id)?;
    let rows = service::monthly(&state, q.photographer_id, &q.year_month).await?;
    let list: Vec<_> = rows
        .iter()
        .map(|r| serde_json::to_value(r).unwrap())
        .collect();
    Ok(Json(ApiResponse::ok(serde_json::json!({"list":list}))))
}
