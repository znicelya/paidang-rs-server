//! Time slot templates handlers.

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

/// GET /time-slot-templates — list templates.
#[utoipa::path(
    get,
    path = "/time-slot-templates",
    params(ListQuery),
    responses(
        (status = 200, body = ApiResponse<PaginatedData<serde_json::Value>>),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "time-slot-templates",
)]
pub async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<ListQuery>,
) -> Result<Json<ApiResponse<PaginatedData<serde_json::Value>>>, AppError> {
    let page = q.page.unwrap_or(1);
    let ps = q.page_size.unwrap_or(20);
    let (rows, total) = service::list(&state, auth.user_id, page, ps).await?;
    let list: Vec<_> = rows
        .iter()
        .map(|r| serde_json::to_value(r).unwrap())
        .collect();
    Ok(Json(ApiResponse::ok(PaginatedData::new(
        list, total, page, ps,
    ))))
}

/// GET /time-slot-templates/{id} — read a single template.
#[utoipa::path(
    get,
    path = "/time-slot-templates/{id}",
    params(("id" = i32, Path, description = "Template ID")),
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    ),
    tag = "time-slot-templates",
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

/// POST /time-slot-templates — create a new template.
#[utoipa::path(
    post,
    path = "/time-slot-templates",
    request_body = CreateReq,
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 400, description = "Input validation error"),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "time-slot-templates",
)]
pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<CreateReq>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    body.validate()
        .map_err(|e| AppError::InputValidation(e.to_string()))?;
    let m = service::create(&state, auth.user_id, body).await?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(m).unwrap())))
}

/// PUT /time-slot-templates/{id} — update a template.
#[utoipa::path(
    put,
    path = "/time-slot-templates/{id}",
    params(("id" = i32, Path, description = "Template ID")),
    request_body = UpdateReq,
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    ),
    tag = "time-slot-templates",
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

/// DELETE /time-slot-templates/{id} — delete a template.
#[utoipa::path(
    delete,
    path = "/time-slot-templates/{id}",
    params(("id" = i32, Path, description = "Template ID")),
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    ),
    tag = "time-slot-templates",
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
