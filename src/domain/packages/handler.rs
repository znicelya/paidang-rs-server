//! Packages handlers - read public, write provider.

use axum::Json;
use axum::extract::{Path, Query, State};
use validator::Validate;

use crate::app_state::AppState;
use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use crate::response::{ApiResponse, PaginatedData};

use super::dto::*;
use super::service;

// Package handlers

/// GET /packages - list packages with filters.
#[utoipa::path(
    get,
    path = "/packages",
    params(ListQuery),
    responses(
        (status = 200, body = ApiResponse<PaginatedData<serde_json::Value>>),
    ),
    tag = "packages",
)]
pub async fn list(
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> Result<Json<ApiResponse<PaginatedData<serde_json::Value>>>, AppError> {
    let page = q.page.unwrap_or(1);
    let ps = q.page_size.unwrap_or(20);
    let (rows, total) = service::list_packages(&state, &q).await?;
    let list: Vec<_> = rows
        .iter()
        .map(|r| serde_json::to_value(r).unwrap())
        .collect();
    Ok(Json(ApiResponse::ok(PaginatedData::new(
        list, total, page, ps,
    ))))
}

/// GET /packages/{id} - read a single package.
#[utoipa::path(
    get,
    path = "/packages/{id}",
    params(("id" = i32, Path, description = "Package ID")),
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 404, description = "Not found"),
    ),
    tag = "packages",
)]
pub async fn read(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let p = service::read_package(&state, id).await?;
    let items = service::list_items(&state, id).await?;
    let mut value = serde_json::to_value(p).unwrap();
    if let Some(obj) = value.as_object_mut() {
        obj.insert("items".to_string(), serde_json::to_value(items).unwrap());
    }
    Ok(Json(ApiResponse::ok(value)))
}

/// POST /packages - create a new package (provider login required).
#[utoipa::path(
    post,
    path = "/packages",
    request_body = CreatePackageReq,
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 400, description = "Input validation error"),
        (status = 403, description = "Forbidden - login required"),
    ),
    tag = "packages",
)]
pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<CreatePackageReq>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    body.validate()
        .map_err(|e| AppError::InputValidation(e.to_string()))?;
    let p = service::create_package(&state, &body, auth.user_id).await?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(p).unwrap())))
}

/// PUT /packages/{id} - update a package (provider login required).
#[utoipa::path(
    put,
    path = "/packages/{id}",
    params(("id" = i32, Path, description = "Package ID")),
    request_body = UpdatePackageReq,
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 403, description = "Forbidden - login required"),
        (status = 404, description = "Not found"),
    ),
    tag = "packages",
)]
pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(body): Json<UpdatePackageReq>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let p = service::update_package(&state, id, &body, auth.user_id).await?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(p).unwrap())))
}

/// DELETE /packages/{id} - delete a package (provider login required).
#[utoipa::path(
    delete,
    path = "/packages/{id}",
    params(("id" = i32, Path, description = "Package ID")),
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 403, description = "Forbidden - login required"),
    ),
    tag = "packages",
)]
pub async fn delete_one(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    service::delete_package(&state, id).await?;
    Ok(Json(ApiResponse::ok(())))
}

// Item handlers

/// GET /packages/{package_id}/items - list package items.
#[utoipa::path(
    get,
    path = "/packages/{package_id}/items",
    params(("package_id" = i32, Path, description = "Package ID")),
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
    ),
    tag = "packages",
)]
pub async fn list_items(
    State(state): State<AppState>,
    Path(package_id): Path<i32>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let rows = service::list_items(&state, package_id).await?;
    let list: Vec<_> = rows
        .iter()
        .map(|r| serde_json::to_value(r).unwrap())
        .collect();
    Ok(Json(ApiResponse::ok(serde_json::json!({ "list": list }))))
}

/// POST /packages/{package_id}/items - create a package item (provider login required).
#[utoipa::path(
    post,
    path = "/packages/{package_id}/items",
    params(("package_id" = i32, Path, description = "Package ID")),
    request_body = CreateItemReq,
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 400, description = "Input validation error"),
        (status = 403, description = "Forbidden - login required"),
    ),
    tag = "packages",
)]
pub async fn create_item(
    State(state): State<AppState>,
    _auth: AuthUser,
    Json(body): Json<CreateItemReq>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    body.validate()
        .map_err(|e| AppError::InputValidation(e.to_string()))?;
    let m = service::create_item(&state, &body).await?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(m).unwrap())))
}

/// PUT /package-items/{item_id} - update a package item (provider login required).
#[utoipa::path(
    put,
    path = "/package-items/{item_id}",
    params(("item_id" = i32, Path, description = "Item ID")),
    request_body = UpdateItemReq,
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 403, description = "Forbidden - login required"),
        (status = 404, description = "Not found"),
    ),
    tag = "packages",
)]
pub async fn update_item(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(item_id): Path<i32>,
    Json(body): Json<UpdateItemReq>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let m = service::update_item(&state, item_id, &body).await?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(m).unwrap())))
}

/// DELETE /package-items/{item_id} - delete a package item (provider login required).
#[utoipa::path(
    delete,
    path = "/package-items/{item_id}",
    params(("item_id" = i32, Path, description = "Item ID")),
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 403, description = "Forbidden - login required"),
    ),
    tag = "packages",
)]
pub async fn delete_item(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(item_id): Path<i32>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    service::delete_item(&state, item_id).await?;
    Ok(Json(ApiResponse::ok(())))
}

// Gallery handlers

/// GET /packages/{package_id}/gallery - list package gallery images.
#[utoipa::path(
    get,
    path = "/packages/{package_id}/gallery",
    params(("package_id" = i32, Path, description = "Package ID")),
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
    ),
    tag = "packages",
)]
pub async fn list_gallery(
    State(state): State<AppState>,
    Path(package_id): Path<i32>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let rows = service::list_gallery(&state, package_id).await?;
    let list: Vec<_> = rows
        .iter()
        .map(|r| serde_json::to_value(r).unwrap())
        .collect();
    Ok(Json(ApiResponse::ok(serde_json::json!({ "list": list }))))
}

/// POST /packages/{package_id}/gallery - add a gallery image (provider login required).
#[utoipa::path(
    post,
    path = "/packages/{package_id}/gallery",
    params(("package_id" = i32, Path, description = "Package ID")),
    request_body = CreateGalleryReq,
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 400, description = "Input validation error"),
        (status = 403, description = "Forbidden - login required"),
    ),
    tag = "packages",
)]
pub async fn create_gallery(
    State(state): State<AppState>,
    _auth: AuthUser,
    Json(body): Json<CreateGalleryReq>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    body.validate()
        .map_err(|e| AppError::InputValidation(e.to_string()))?;
    let m = service::create_gallery(&state, &body).await?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(m).unwrap())))
}

/// PUT /package-gallery/{gallery_id} - update a gallery image (provider login required).
#[utoipa::path(
    put,
    path = "/package-gallery/{gallery_id}",
    params(("gallery_id" = i32, Path, description = "Gallery ID")),
    request_body = UpdateGalleryReq,
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 403, description = "Forbidden - login required"),
        (status = 404, description = "Not found"),
    ),
    tag = "packages",
)]
pub async fn update_gallery(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(gallery_id): Path<i32>,
    Json(body): Json<UpdateGalleryReq>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let m = service::update_gallery(&state, gallery_id, &body).await?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(m).unwrap())))
}

/// DELETE /package-gallery/{gallery_id} - delete a gallery image (provider login required).
#[utoipa::path(
    delete,
    path = "/package-gallery/{gallery_id}",
    params(("gallery_id" = i32, Path, description = "Gallery ID")),
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 403, description = "Forbidden - login required"),
    ),
    tag = "packages",
)]
pub async fn delete_gallery(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(gallery_id): Path<i32>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    service::delete_gallery(&state, gallery_id).await?;
    Ok(Json(ApiResponse::ok(())))
}
