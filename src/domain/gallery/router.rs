//! Gallery — read public, write admin. Tags as nested resource.

use axum::extract::{Path, Query, State};
use axum::Json;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};
use serde::Deserialize;
use utoipa::ToSchema;
use utoipa_axum::routes;
use utoipa_axum::router::OpenApiRouter;
use validator::Validate;

use crate::app_state::AppState;
use crate::entity::{gallery, gallery_tag};
use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use crate::response::{ApiResponse, PaginatedData};

// ── Gallery DTOs ─────────────────────────────────────

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateGalleryReq {
    pub group_id: Option<i32>,
    #[validate(length(min = 1))]
    pub title: String,
    pub subtitle: Option<String>,
    pub cover_image: Option<String>,
    pub image_url: Option<String>,
    pub image_list: Option<serde_json::Value>,
    pub video_url: Option<String>,
    pub media_type: Option<String>,
    pub tags: Option<String>,
    pub photographer_id: Option<i32>,
    pub photographer_name: Option<String>,
    pub shooting_location: Option<String>,
    pub shooting_date: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub file_size: Option<i32>,
    pub sort_order: Option<i32>,
    pub is_cover: Option<i8>,
    pub status: Option<i8>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams, ToSchema)]
pub struct UpdateGalleryReq {
    pub group_id: Option<i32>,
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub cover_image: Option<String>,
    pub image_url: Option<String>,
    pub image_list: Option<serde_json::Value>,
    pub video_url: Option<String>,
    pub media_type: Option<String>,
    pub tags: Option<String>,
    pub photographer_id: Option<i32>,
    pub photographer_name: Option<String>,
    pub shooting_location: Option<String>,
    pub shooting_date: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub file_size: Option<i32>,
    pub sort_order: Option<i32>,
    pub is_cover: Option<i8>,
    pub status: Option<i8>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams, ToSchema)]
pub struct GalleryListQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub group_id: Option<i32>,
    pub photographer_id: Option<i32>,
    pub status: Option<i8>,
}

// ── Gallery Tag DTOs ─────────────────────────────────

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateTagReq {
    #[validate(length(min = 1))]
    pub tag_name: String,
    pub tag_type: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams, ToSchema)]
pub struct UpdateTagReq {
    pub tag_name: Option<String>,
    pub tag_type: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams, ToSchema)]
pub struct TagListQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub tag_type: Option<String>,
}

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list, create))
        .routes(routes!(read, update, delete_one))
        .routes(routes!(list_tags, create_tag))
        .routes(routes!(read_tag, update_tag, delete_tag))
}

fn require_admin(auth: &AuthUser) -> Result<(), AppError> {
    if auth.role >= 2 {
        Ok(())
    } else {
        Err(AppError::Forbidden("需要管理员权限".into()))
    }
}

// ── Gallery handlers ──────────────────────────────────

/// GET /gallery — list gallery items.
#[utoipa::path(
    get,
    path = "/gallery",
    params(GalleryListQuery),
    responses(
        (status = 200, body = ApiResponse<PaginatedData<serde_json::Value>>),
    ),
    tag = "gallery",
)]
async fn list(
    State(state): State<AppState>,
    Query(q): Query<GalleryListQuery>,
) -> Result<Json<ApiResponse<PaginatedData<serde_json::Value>>>, AppError> {
    let page = q.page.unwrap_or(1);
    let ps = q.page_size.unwrap_or(20).min(100);
    let mut s = gallery::Entity::find()
        .filter(gallery::Column::Status.eq(q.status.unwrap_or(1)));
    if let Some(gid) = q.group_id {
        s = s.filter(gallery::Column::GroupId.eq(gid));
    }
    if let Some(pid) = q.photographer_id {
        s = s.filter(gallery::Column::PhotographerId.eq(pid));
    }
    let total = s
        .clone()
        .count(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?;
    let rows = s
        .order_by_desc(gallery::Column::CreateTime)
        .offset(((page.saturating_sub(1)) * ps) as u64)
        .limit(ps)
        .all(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?;
    let list: Vec<_> = rows.iter().map(|r| serde_json::to_value(r).unwrap()).collect();
    Ok(Json(ApiResponse::ok(PaginatedData::new(list, total, page, ps))))
}

/// GET /gallery/{id} — read a gallery item.
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
async fn read(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let r = gallery::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?
        .ok_or(AppError::NotFound("不存在".into()))?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(r).unwrap())))
}

/// POST /gallery — create a gallery item (admin).
#[utoipa::path(
    post,
    path = "/gallery",
    request_body = CreateGalleryReq,
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 400, description = "Input validation error"),
        (status = 403, description = "Forbidden — admin only"),
    ),
    tag = "gallery",
)]
async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<CreateGalleryReq>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    require_admin(&auth)?;
    body.validate()
        .map_err(|e| AppError::InputValidation(e.to_string()))?;
    let m = gallery::ActiveModel {
        group_id: Set(body.group_id),
        title: Set(body.title),
        subtitle: Set(body.subtitle),
        cover_image: Set(body.cover_image),
        image_url: Set(body.image_url),
        image_list: Set(body.image_list),
        video_url: Set(body.video_url),
        media_type: Set(body.media_type),
        tags: Set(body.tags),
        photographer_id: Set(body.photographer_id),
        photographer_name: Set(body.photographer_name),
        shooting_location: Set(body.shooting_location),
        shooting_date: Set(body.shooting_date),
        width: Set(body.width),
        height: Set(body.height),
        file_size: Set(body.file_size),
        sort_order: Set(body.sort_order),
        is_cover: Set(body.is_cover),
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

/// PUT /gallery/{id} — update a gallery item (admin).
#[utoipa::path(
    put,
    path = "/gallery/{id}",
    params(("id" = i32, Path, description = "Gallery ID")),
    request_body = UpdateGalleryReq,
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 403, description = "Forbidden — admin only"),
        (status = 404, description = "Not found"),
    ),
    tag = "gallery",
)]
async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(body): Json<UpdateGalleryReq>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    require_admin(&auth)?;
    let rec = gallery::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?
        .ok_or(AppError::NotFound("不存在".into()))?;
    let mut a: gallery::ActiveModel = rec.into();
    if let Some(v) = body.group_id { a.group_id = Set(Some(v)); }
    if let Some(v) = body.title { a.title = Set(v); }
    if let Some(v) = body.subtitle { a.subtitle = Set(Some(v)); }
    if let Some(v) = body.cover_image { a.cover_image = Set(Some(v)); }
    if let Some(v) = body.image_url { a.image_url = Set(Some(v)); }
    if let Some(v) = body.image_list { a.image_list = Set(Some(v)); }
    if let Some(v) = body.video_url { a.video_url = Set(Some(v)); }
    if let Some(v) = body.media_type { a.media_type = Set(Some(v)); }
    if let Some(v) = body.tags { a.tags = Set(Some(v)); }
    if let Some(v) = body.photographer_id { a.photographer_id = Set(Some(v)); }
    if let Some(v) = body.photographer_name { a.photographer_name = Set(Some(v)); }
    if let Some(v) = body.shooting_location { a.shooting_location = Set(Some(v)); }
    if let Some(v) = body.shooting_date { a.shooting_date = Set(Some(v)); }
    if let Some(v) = body.width { a.width = Set(Some(v)); }
    if let Some(v) = body.height { a.height = Set(Some(v)); }
    if let Some(v) = body.file_size { a.file_size = Set(Some(v)); }
    if let Some(v) = body.sort_order { a.sort_order = Set(Some(v)); }
    if let Some(v) = body.is_cover { a.is_cover = Set(Some(v)); }
    if let Some(v) = body.status { a.status = Set(Some(v)); }
    a.update_by = Set(Some(auth.user_id));
    let r = a.update(&state.db).await.map_err(|e| AppError::Internal(format!("DB:{e}")))?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(r).unwrap())))
}

/// DELETE /gallery/{id} — delete a gallery item (admin).
#[utoipa::path(
    delete,
    path = "/gallery/{id}",
    params(("id" = i32, Path, description = "Gallery ID")),
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 403, description = "Forbidden — admin only"),
    ),
    tag = "gallery",
)]
async fn delete_one(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    require_admin(&auth)?;
    gallery::Entity::delete_by_id(id)
        .exec(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?;
    Ok(Json(ApiResponse::ok(())))
}

// ── Tag handlers ──────────────────────────────────────

/// GET /gallery-tags — list gallery tags.
#[utoipa::path(
    get,
    path = "/gallery-tags",
    params(TagListQuery),
    responses(
        (status = 200, body = ApiResponse<PaginatedData<serde_json::Value>>),
    ),
    tag = "gallery",
)]
async fn list_tags(
    State(state): State<AppState>,
    Query(q): Query<TagListQuery>,
) -> Result<Json<ApiResponse<PaginatedData<serde_json::Value>>>, AppError> {
    let page = q.page.unwrap_or(1);
    let ps = q.page_size.unwrap_or(50).min(200);
    let mut s = gallery_tag::Entity::find();
    if let Some(ref tt) = q.tag_type {
        s = s.filter(gallery_tag::Column::TagType.eq(tt));
    }
    let total = s
        .clone()
        .count(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?;
    let rows = s
        .order_by_desc(gallery_tag::Column::UseCount)
        .offset(((page.saturating_sub(1)) * ps) as u64)
        .limit(ps)
        .all(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?;
    let list: Vec<_> = rows.iter().map(|r| serde_json::to_value(r).unwrap()).collect();
    Ok(Json(ApiResponse::ok(PaginatedData::new(list, total, page, ps))))
}

/// GET /gallery-tags/{id} — read a gallery tag.
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
async fn read_tag(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let r = gallery_tag::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?
        .ok_or(AppError::NotFound("不存在".into()))?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(r).unwrap())))
}

/// POST /gallery-tags — create a gallery tag (admin).
#[utoipa::path(
    post,
    path = "/gallery-tags",
    request_body = CreateTagReq,
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 400, description = "Input validation error"),
        (status = 403, description = "Forbidden — admin only"),
    ),
    tag = "gallery",
)]
async fn create_tag(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<CreateTagReq>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    require_admin(&auth)?;
    body.validate()
        .map_err(|e| AppError::InputValidation(e.to_string()))?;
    let m = gallery_tag::ActiveModel {
        tag_name: Set(body.tag_name),
        tag_type: Set(body.tag_type),
        use_count: Set(Some(0)),
        sort_order: Set(body.sort_order),
        ..Default::default()
    }
    .insert(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("DB:{e}")))?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(m).unwrap())))
}

/// PUT /gallery-tags/{id} — update a gallery tag (admin).
#[utoipa::path(
    put,
    path = "/gallery-tags/{id}",
    params(("id" = i32, Path, description = "Tag ID")),
    request_body = UpdateTagReq,
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 403, description = "Forbidden — admin only"),
        (status = 404, description = "Not found"),
    ),
    tag = "gallery",
)]
async fn update_tag(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(body): Json<UpdateTagReq>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    require_admin(&auth)?;
    let rec = gallery_tag::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?
        .ok_or(AppError::NotFound("不存在".into()))?;
    let mut a: gallery_tag::ActiveModel = rec.into();
    if let Some(v) = body.tag_name { a.tag_name = Set(v); }
    if let Some(v) = body.tag_type { a.tag_type = Set(Some(v)); }
    if let Some(v) = body.sort_order { a.sort_order = Set(Some(v)); }
    let r = a.update(&state.db).await.map_err(|e| AppError::Internal(format!("DB:{e}")))?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(r).unwrap())))
}

/// DELETE /gallery-tags/{id} — delete a gallery tag (admin).
#[utoipa::path(
    delete,
    path = "/gallery-tags/{id}",
    params(("id" = i32, Path, description = "Tag ID")),
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 403, description = "Forbidden — admin only"),
    ),
    tag = "gallery",
)]
async fn delete_tag(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    require_admin(&auth)?;
    gallery_tag::Entity::delete_by_id(id)
        .exec(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?;
    Ok(Json(ApiResponse::ok(())))
}
