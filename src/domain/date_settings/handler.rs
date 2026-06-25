//! Date settings handlers - CRUD + check. JWT-protected.

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

/// GET /date-settings - list date settings.
#[utoipa::path(
    get,
    path = "/date-settings",
    params(ListQuery),
    responses(
        (status = 200, body = ApiResponse<PaginatedData<serde_json::Value>>),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "date-settings",
)]
pub async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<ListQuery>,
) -> Result<Json<ApiResponse<PaginatedData<serde_json::Value>>>, AppError> {
    let page = q.page.unwrap_or(1);
    let ps = q.page_size.unwrap_or(20);
    let (rows, total) = service::list(&state, &q, Some(auth.user_id)).await?;
    let list: Vec<_> = rows
        .iter()
        .map(|r| serde_json::to_value(r).unwrap())
        .collect();
    Ok(Json(ApiResponse::ok(PaginatedData::new(
        list, total, page, ps,
    ))))
}

/// GET /date-settings/{id} - read a single date setting.
#[utoipa::path(
    get,
    path = "/date-settings/{id}",
    params(("id" = i32, Path, description = "Date setting ID")),
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    ),
    tag = "date-settings",
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

/// POST /date-settings - create a new date setting.
#[utoipa::path(
    post,
    path = "/date-settings",
    request_body = CreateReq,
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 400, description = "Input validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    tag = "date-settings",
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

/// PUT /date-settings/{id} - update a date setting.
#[utoipa::path(
    put,
    path = "/date-settings/{id}",
    params(("id" = i32, Path, description = "Date setting ID")),
    request_body = UpdateReq,
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    ),
    tag = "date-settings",
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

/// DELETE /date-settings/{id} - delete a date setting.
#[utoipa::path(
    delete,
    path = "/date-settings/{id}",
    params(("id" = i32, Path, description = "Date setting ID")),
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    ),
    tag = "date-settings",
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

/// GET /date-settings/check - check date availability.
#[utoipa::path(
    get,
    path = "/date-settings/check",
    params(CheckQuery),
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    tag = "date-settings",
)]
pub async fn check(
    State(state): State<AppState>,
    Query(q): Query<CheckQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let v = service::check(&state, q.photographer_id, &q.target_date).await?;
    Ok(Json(ApiResponse::ok(v)))
}
