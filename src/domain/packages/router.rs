//! Packages router — read public, write admin.

use axum::extract::{Path, Query, State};
use axum::routing::{get, put};
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
        // Packages
        .route("/packages", get(list).post(create))
        .route("/packages/{id}", get(read).put(update).delete(delete_one))
        // Package Items
        .route(
            "/packages/{package_id}/items",
            get(list_items).post(create_item),
        )
        .route("/package-items/{item_id}", put(update_item).delete(delete_item))
        // Package Gallery
        .route(
            "/packages/{package_id}/gallery",
            get(list_gallery).post(create_gallery),
        )
        .route(
            "/package-gallery/{gallery_id}",
            put(update_gallery).delete(delete_gallery),
        )
}

fn require_admin(auth: &AuthUser) -> Result<(), AppError> {
    if auth.role >= 2 {
        Ok(())
    } else {
        Err(AppError::Forbidden("需要管理员权限".into()))
    }
}

// ── Package handlers ────────────────────────────────────

async fn list(
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> Result<Json<ApiResponse<PaginatedData<serde_json::Value>>>, AppError> {
    let page = q.page.unwrap_or(1);
    let ps = q.page_size.unwrap_or(20);
    let (rows, total) = service::list_packages(&state, &q).await?;
    let list: Vec<_> = rows.iter().map(|r| serde_json::to_value(r).unwrap()).collect();
    Ok(Json(ApiResponse::ok(PaginatedData::new(list, total, page, ps))))
}

async fn read(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let p = service::read_package(&state, id).await?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(p).unwrap())))
}

async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<CreatePackageReq>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    require_admin(&auth)?;
    body.validate()
        .map_err(|e| AppError::InputValidation(e.to_string()))?;
    let p = service::create_package(&state, &body, auth.user_id).await?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(p).unwrap())))
}

async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(body): Json<UpdatePackageReq>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    require_admin(&auth)?;
    let p = service::update_package(&state, id, &body, auth.user_id).await?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(p).unwrap())))
}

async fn delete_one(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    require_admin(&auth)?;
    service::delete_package(&state, id).await?;
    Ok(Json(ApiResponse::ok(())))
}

// ── Item handlers ───────────────────────────────────────

async fn list_items(
    State(state): State<AppState>,
    Path(package_id): Path<i32>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let rows = service::list_items(&state, package_id).await?;
    let list: Vec<_> = rows.iter().map(|r| serde_json::to_value(r).unwrap()).collect();
    Ok(Json(ApiResponse::ok(serde_json::json!({ "list": list }))))
}

async fn create_item(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<CreateItemReq>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    require_admin(&auth)?;
    body.validate()
        .map_err(|e| AppError::InputValidation(e.to_string()))?;
    let m = service::create_item(&state, &body).await?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(m).unwrap())))
}

async fn update_item(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(item_id): Path<i32>,
    Json(body): Json<UpdateItemReq>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    require_admin(&auth)?;
    let m = service::update_item(&state, item_id, &body).await?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(m).unwrap())))
}

async fn delete_item(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(item_id): Path<i32>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    require_admin(&auth)?;
    service::delete_item(&state, item_id).await?;
    Ok(Json(ApiResponse::ok(())))
}

// ── Gallery handlers ────────────────────────────────────

async fn list_gallery(
    State(state): State<AppState>,
    Path(package_id): Path<i32>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let rows = service::list_gallery(&state, package_id).await?;
    let list: Vec<_> = rows.iter().map(|r| serde_json::to_value(r).unwrap()).collect();
    Ok(Json(ApiResponse::ok(serde_json::json!({ "list": list }))))
}

async fn create_gallery(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<CreateGalleryReq>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    require_admin(&auth)?;
    body.validate()
        .map_err(|e| AppError::InputValidation(e.to_string()))?;
    let m = service::create_gallery(&state, &body).await?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(m).unwrap())))
}

async fn update_gallery(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(gallery_id): Path<i32>,
    Json(body): Json<UpdateGalleryReq>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    require_admin(&auth)?;
    let m = service::update_gallery(&state, gallery_id, &body).await?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(m).unwrap())))
}

async fn delete_gallery(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(gallery_id): Path<i32>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    require_admin(&auth)?;
    service::delete_gallery(&state, gallery_id).await?;
    Ok(Json(ApiResponse::ok(())))
}
