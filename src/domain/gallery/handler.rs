//! Gallery handlers.

use axum::Json;
use axum::extract::{Path, Query, State};
use validator::Validate;

use crate::app_state::AppState;
use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use crate::response::{ApiResponse, PaginatedData};

use super::dto::*;
use super::service;

// Gallery handlers
/// GET /gallery - list gallery items.
#[utoipa::path(
    get,
    path = "/gallery",
    params(GalleryListQuery),
    responses(
        (status = 200, body = ApiResponse<PaginatedData<serde_json::Value>>),
    ),
    tag = "gallery",
)]
pub async fn list(
    State(state): State<AppState>,
    Query(q): Query<GalleryListQuery>,
) -> Result<Json<ApiResponse<PaginatedData<serde_json::Value>>>, AppError> {
    let page = q.page.unwrap_or(1);
    let ps = q.page_size.unwrap_or(20).min(100);
    let (rows, total) = service::list(&state, &q).await?;
    let list: Vec<_> = rows
        .iter()
        .map(|r| serde_json::to_value(r).unwrap())
        .collect();
    Ok(Json(ApiResponse::ok(PaginatedData::new(
        list, total, page, ps,
    ))))
}

/// GET /gallery/{id} - read a gallery item.
#[utoipa::path(
    get,
    path = "/gallery/{id}",
    params(("id" = i32, Path, description = "Gallery ID")),
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 404, description = "Not found"),
    ),
    tag = "gallery",
)]
pub async fn read(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let r = service::read(&state, id).await?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(r).unwrap())))
}

/// POST /gallery - create a gallery item (provider login required).
#[utoipa::path(
    post,
    path = "/gallery",
    request_body = CreateGalleryReq,
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 400, description = "Input validation error"),
        (status = 403, description = "Forbidden - login required"),
    ),
    tag = "gallery",
)]
pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<CreateGalleryReq>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    body.validate()
        .map_err(|e| AppError::InputValidation(e.to_string()))?;
    let m = service::create(&state, body, auth.user_id).await?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(m).unwrap())))
}

/// PUT /gallery/{id} - update a gallery item (provider login required).
#[utoipa::path(
    put,
    path = "/gallery/{id}",
    params(("id" = i32, Path, description = "Gallery ID")),
    request_body = UpdateGalleryReq,
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 403, description = "Forbidden - login required"),
        (status = 404, description = "Not found"),
    ),
    tag = "gallery",
)]
pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(body): Json<UpdateGalleryReq>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let r = service::update(&state, id, body, auth.user_id).await?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(r).unwrap())))
}

/// DELETE /gallery/{id} - delete a gallery item (provider login required).
#[utoipa::path(
    delete,
    path = "/gallery/{id}",
    params(("id" = i32, Path, description = "Gallery ID")),
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 403, description = "Forbidden - login required"),
    ),
    tag = "gallery",
)]
pub async fn delete_one(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    service::delete_one(&state, id).await?;
    Ok(Json(ApiResponse::ok(())))
}

// Tag handlers
/// GET /gallery-tags - list gallery tags.
#[utoipa::path(
    get,
    path = "/gallery-tags",
    params(TagListQuery),
    responses(
        (status = 200, body = ApiResponse<PaginatedData<serde_json::Value>>),
    ),
    tag = "gallery",
)]
pub async fn list_tags(
    State(state): State<AppState>,
    Query(q): Query<TagListQuery>,
) -> Result<Json<ApiResponse<PaginatedData<serde_json::Value>>>, AppError> {
    let page = q.page.unwrap_or(1);
    let ps = q.page_size.unwrap_or(50).min(200);
    let (rows, total) = service::list_tags(&state, &q).await?;
    let list: Vec<_> = rows
        .iter()
        .map(|r| serde_json::to_value(r).unwrap())
        .collect();
    Ok(Json(ApiResponse::ok(PaginatedData::new(
        list, total, page, ps,
    ))))
}

/// GET /gallery-tags/{id} - read a gallery tag.
#[utoipa::path(
    get,
    path = "/gallery-tags/{id}",
    params(("id" = i32, Path, description = "Tag ID")),
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 404, description = "Not found"),
    ),
    tag = "gallery",
)]
pub async fn read_tag(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let r = service::read_tag(&state, id).await?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(r).unwrap())))
}

/// POST /gallery-tags - create a gallery tag (provider login required).
#[utoipa::path(
    post,
    path = "/gallery-tags",
    request_body = CreateTagReq,
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 400, description = "Input validation error"),
        (status = 403, description = "Forbidden - login required"),
    ),
    tag = "gallery",
)]
pub async fn create_tag(
    State(state): State<AppState>,
    _auth: AuthUser,
    Json(body): Json<CreateTagReq>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    body.validate()
        .map_err(|e| AppError::InputValidation(e.to_string()))?;
    let m = service::create_tag(&state, body).await?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(m).unwrap())))
}

/// PUT /gallery-tags/{id} - update a gallery tag (provider login required).
#[utoipa::path(
    put,
    path = "/gallery-tags/{id}",
    params(("id" = i32, Path, description = "Tag ID")),
    request_body = UpdateTagReq,
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 403, description = "Forbidden - login required"),
        (status = 404, description = "Not found"),
    ),
    tag = "gallery",
)]
pub async fn update_tag(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
    Json(body): Json<UpdateTagReq>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let r = service::update_tag(&state, id, body).await?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(r).unwrap())))
}

/// DELETE /gallery-tags/{id} - delete a gallery tag (provider login required).
#[utoipa::path(
    delete,
    path = "/gallery-tags/{id}",
    params(("id" = i32, Path, description = "Tag ID")),
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 403, description = "Forbidden - login required"),
    ),
    tag = "gallery",
)]
pub async fn delete_tag(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    service::delete_tag(&state, id).await?;
    Ok(Json(ApiResponse::ok(())))
}
