//! Gallery groups — read public, write admin.

use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};
use serde::Deserialize;
use validator::Validate;

use crate::app_state::AppState;
use crate::entity::gallery_group;
use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use crate::response::{ApiResponse, PaginatedData};

// ── DTOs ─────────────────────────────────────────────

#[derive(Debug, Deserialize, Validate)]
pub struct CreateReq {
    #[validate(length(min = 1))]
    pub name: String,
    pub cover_image: Option<String>,
    pub description: Option<String>,
    pub sort_order: Option<i32>,
    pub is_visible: Option<i8>,
    pub status: Option<i8>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateReq {
    pub name: Option<String>,
    pub cover_image: Option<String>,
    pub description: Option<String>,
    pub sort_order: Option<i32>,
    pub is_visible: Option<i8>,
    pub status: Option<i8>,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/gallery-groups", get(list).post(create))
        .route(
            "/gallery-groups/{id}",
            get(read).put(update).delete(delete_one),
        )
}

fn require_admin(auth: &AuthUser) -> Result<(), AppError> {
    if auth.role >= 2 {
        Ok(())
    } else {
        Err(AppError::Forbidden("需要管理员权限".into()))
    }
}

async fn list(
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> Result<Json<ApiResponse<PaginatedData<serde_json::Value>>>, AppError> {
    let page = q.page.unwrap_or(1);
    let ps = q.page_size.unwrap_or(20);
    let s = gallery_group::Entity::find()
        .filter(gallery_group::Column::Status.eq(1));
    let total = s
        .clone()
        .count(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?;
    let rows = s
        .order_by_asc(gallery_group::Column::SortOrder)
        .offset(((page - 1) * ps) as u64)
        .limit(ps)
        .all(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?;
    let list: Vec<_> = rows.iter().map(|r| serde_json::to_value(r).unwrap()).collect();
    Ok(Json(ApiResponse::ok(PaginatedData::new(list, total, page, ps))))
}

async fn read(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let r = gallery_group::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?
        .ok_or(AppError::NotFound("不存在".into()))?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(r).unwrap())))
}

async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<CreateReq>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    require_admin(&auth)?;
    body.validate()
        .map_err(|e| AppError::InputValidation(e.to_string()))?;
    let m = gallery_group::ActiveModel {
        name: Set(body.name),
        cover_image: Set(body.cover_image),
        description: Set(body.description),
        sort_order: Set(body.sort_order),
        is_visible: Set(body.is_visible),
        status: Set(body.status),
        create_by: Set(Some(auth.user_id)),
        update_by: Set(Some(auth.user_id)),
        ..Default::default()
    }
    .insert(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("DB:{e}")))?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(m).unwrap())))
}

async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(body): Json<UpdateReq>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    require_admin(&auth)?;
    let rec = gallery_group::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?
        .ok_or(AppError::NotFound("不存在".into()))?;
    let mut a: gallery_group::ActiveModel = rec.into();
    if let Some(v) = body.name { a.name = Set(v); }
    if let Some(v) = body.cover_image { a.cover_image = Set(Some(v)); }
    if let Some(v) = body.description { a.description = Set(Some(v)); }
    if let Some(v) = body.sort_order { a.sort_order = Set(Some(v)); }
    if let Some(v) = body.is_visible { a.is_visible = Set(Some(v)); }
    if let Some(v) = body.status { a.status = Set(Some(v)); }
    a.update_by = Set(Some(auth.user_id));
    let r = a.update(&state.db).await.map_err(|e| AppError::Internal(format!("DB:{e}")))?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(r).unwrap())))
}

async fn delete_one(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    require_admin(&auth)?;
    gallery_group::Entity::delete_by_id(id)
        .exec(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?;
    Ok(Json(ApiResponse::ok(())))
}
