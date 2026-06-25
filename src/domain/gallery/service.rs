//! Gallery service.

use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};

use crate::app_state::AppState;
use crate::entity::{gallery, gallery_tag};
use crate::error::AppError;

use super::dto::{CreateGalleryReq, CreateTagReq, GalleryListQuery, TagListQuery, UpdateGalleryReq, UpdateTagReq};

/// List gallery items with pagination.
pub async fn list(
    state: &AppState,
    q: &GalleryListQuery,
) -> Result<(Vec<gallery::Model>, u64), AppError> {
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
    Ok((rows, total))
}

/// Read a single gallery item by id.
pub async fn read(state: &AppState, id: i32) -> Result<gallery::Model, AppError> {
    gallery::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?
        .ok_or(AppError::NotFound("不存在".into()))
}

/// Create a gallery item.
pub async fn create(
    state: &AppState,
    body: CreateGalleryReq,
    user_id: i32,
) -> Result<gallery::Model, AppError> {
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
        create_by: Set(Some(user_id)),
        update_by: Set(Some(user_id)),
        ..Default::default()
    }
    .insert(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("DB:{e}")))?;
    Ok(m)
}

/// Update a gallery item by id.
pub async fn update(
    state: &AppState,
    id: i32,
    body: UpdateGalleryReq,
    user_id: i32,
) -> Result<gallery::Model, AppError> {
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
    a.update_by = Set(Some(user_id));
    let r = a.update(&state.db).await.map_err(|e| AppError::Internal(format!("DB:{e}")))?;
    Ok(r)
}

/// Delete a gallery item by id.
pub async fn delete_one(state: &AppState, id: i32) -> Result<(), AppError> {
    gallery::Entity::delete_by_id(id)
        .exec(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?;
    Ok(())
}

/// List gallery tags with pagination.
pub async fn list_tags(
    state: &AppState,
    q: &TagListQuery,
) -> Result<(Vec<gallery_tag::Model>, u64), AppError> {
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
    Ok((rows, total))
}

/// Read a single gallery tag by id.
pub async fn read_tag(state: &AppState, id: i32) -> Result<gallery_tag::Model, AppError> {
    gallery_tag::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?
        .ok_or(AppError::NotFound("不存在".into()))
}

/// Create a gallery tag.
pub async fn create_tag(
    state: &AppState,
    body: CreateTagReq,
) -> Result<gallery_tag::Model, AppError> {
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
    Ok(m)
}

/// Update a gallery tag by id.
pub async fn update_tag(
    state: &AppState,
    id: i32,
    body: UpdateTagReq,
) -> Result<gallery_tag::Model, AppError> {
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
    Ok(r)
}

/// Delete a gallery tag by id.
pub async fn delete_tag(state: &AppState, id: i32) -> Result<(), AppError> {
    gallery_tag::Entity::delete_by_id(id)
        .exec(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?;
    Ok(())
}
