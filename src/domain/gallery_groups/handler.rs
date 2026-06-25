//! Gallery groups handlers.

use axum::extract::{Path, Query, State};
use axum::Json;
use validator::Validate;

use crate::app_state::AppState;
use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use crate::util::require_admin;
use crate::response::{ApiResponse, PaginatedData};

use super::dto::{CreateReq, ListQuery, UpdateReq};
use super::service;

/// GET /gallery-groups — list gallery groups.
#[utoipa::path(
    get,
    path = "/gallery-groups",
    params(ListQuery),
    responses(
        (status = 200, body = ApiResponse<PaginatedData<serde_json::Value>>),
    ),
    tag = "gallery-groups",
)]
pub async fn list(
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> Result<Json<ApiResponse<PaginatedData<serde_json::Value>>>, AppError> {
    let page = q.page.unwrap_or(1);
    let ps = q.page_size.unwrap_or(20);
    let (rows, total) = service::list(&state, &q).await?;
    let list: Vec<_> = rows.iter().map(|r| serde_json::to_value(r).unwrap()).collect();
    Ok(Json(ApiResponse::ok(PaginatedData::new(list, total, page, ps))))
}

/// GET /gallery-groups/{id} — read a single gallery group.
#[utoipa::path(
    get,
    path = "/gallery-groups/{id}",
    params(("id" = i32, Path, description = "Gallery group ID")),
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 404, description = "Not found"),
    ),
    tag = "gallery-groups",
)]
pub async fn read(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let r = service::read(&state, id).await?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(r).unwrap())))
}

/// POST /gallery-groups — create a gallery group (admin).
#[utoipa::path(
    post,
    path = "/gallery-groups",
    request_body = CreateReq,
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 400, description = "Input validation error"),
        (status = 403, description = "Forbidden — admin only"),
    ),
    tag = "gallery-groups",
)]
pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<CreateReq>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    require_admin(&auth)?;
    body.validate()
        .map_err(|e| AppError::InputValidation(e.to_string()))?;
    let m = service::create(&state, body, auth.user_id).await?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(m).unwrap())))
}

/// PUT /gallery-groups/{id} — update a gallery group (admin).
#[utoipa::path(
    put,
    path = "/gallery-groups/{id}",
    params(("id" = i32, Path, description = "Gallery group ID")),
    request_body = UpdateReq,
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 403, description = "Forbidden — admin only"),
        (status = 404, description = "Not found"),
    ),
    tag = "gallery-groups",
)]
pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(body): Json<UpdateReq>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    require_admin(&auth)?;
    let r = service::update(&state, id, body, auth.user_id).await?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(r).unwrap())))
}

/// DELETE /gallery-groups/{id} — delete a gallery group (admin).
#[utoipa::path(
    delete,
    path = "/gallery-groups/{id}",
    params(("id" = i32, Path, description = "Gallery group ID")),
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 403, description = "Forbidden — admin only"),
    ),
    tag = "gallery-groups",
)]
pub async fn delete_one(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    require_admin(&auth)?;
    service::delete_one(&state, id).await?;
    Ok(Json(ApiResponse::ok(())))
}
