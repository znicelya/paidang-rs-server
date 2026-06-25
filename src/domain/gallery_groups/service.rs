//! Gallery groups service.

use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};

use crate::app_state::AppState;
use crate::entity::gallery_group;
use crate::error::AppError;

use super::dto::{CreateReq, ListQuery, UpdateReq};

/// List gallery groups with pagination, filtered to visible status.
pub async fn list(
    state: &AppState,
    q: &ListQuery,
) -> Result<(Vec<gallery_group::Model>, u64), AppError> {
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
    Ok((rows, total))
}

/// Read a single gallery group by id.
pub async fn read(state: &AppState, id: i32) -> Result<gallery_group::Model, AppError> {
    gallery_group::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?
        .ok_or(AppError::NotFound("不存在".into()))
}

/// Create a gallery group.
pub async fn create(
    state: &AppState,
    body: CreateReq,
    user_id: i32,
) -> Result<gallery_group::Model, AppError> {
    let m = gallery_group::ActiveModel {
        name: Set(body.name),
        cover_image: Set(body.cover_image),
        description: Set(body.description),
        sort_order: Set(body.sort_order),
        is_visible: Set(body.is_visible),
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

/// Update a gallery group by id.
pub async fn update(
    state: &AppState,
    id: i32,
    body: UpdateReq,
    user_id: i32,
) -> Result<gallery_group::Model, AppError> {
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
    a.update_by = Set(Some(user_id));
    let r = a.update(&state.db).await.map_err(|e| AppError::Internal(format!("DB:{e}")))?;
    Ok(r)
}

/// Delete a gallery group by id.
pub async fn delete_one(state: &AppState, id: i32) -> Result<(), AppError> {
    gallery_group::Entity::delete_by_id(id)
        .exec(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?;
    Ok(())
}
