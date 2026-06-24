//! Packages service.

use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};

use crate::app_state::AppState;
use crate::entity::{package, package_gallery, package_item};
use crate::error::AppError;

use super::dto::*;

// ── Package CRUD ─────────────────────────────────────────

pub async fn list_packages(
    state: &AppState,
    query: &ListQuery,
) -> Result<(Vec<package::Model>, u64), AppError> {
    let page = query.page.unwrap_or(1);
    let ps = query.page_size.unwrap_or(20).min(100);
    let mut s = package::Entity::find()
        .filter(package::Column::Status.eq(query.status.unwrap_or(1)));

    if let Some(ref cat) = query.category {
        s = s.filter(package::Column::Category.eq(cat));
    }

    let total = s
        .clone()
        .count(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB: {e}")))?;

    let rows = s
        .order_by_asc(package::Column::SortOrder)
        .offset(((page.saturating_sub(1)) * ps) as u64)
        .limit(ps)
        .all(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB: {e}")))?;

    Ok((rows, total))
}

pub async fn read_package(state: &AppState, id: i32) -> Result<package::Model, AppError> {
    package::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB: {e}")))?
        .ok_or(AppError::NotFound("套餐不存在".into()))
}

pub async fn create_package(
    state: &AppState,
    body: &CreatePackageReq,
    user_id: i32,
) -> Result<package::Model, AppError> {
    let m = package::ActiveModel {
        name: Set(body.name.clone()),
        subtitle: Set(body.subtitle.clone()),
        category: Set(body.category.clone()),
        price: Set(body.price),
        original_price: Set(body.original_price),
        deposit: Set(body.deposit),
        cover_image: Set(body.cover_image.clone()),
        description: Set(body.description.clone()),
        service_items: Set(body.service_items.clone()),
        suitable_people: Set(body.suitable_people.clone()),
        shooting_location: Set(body.shooting_location.clone()),
        validity_days: Set(body.validity_days),
        sort_order: Set(body.sort_order),
        is_hot: Set(body.is_hot),
        is_recommend: Set(body.is_recommend),
        status: Set(body.status),
        create_by: Set(Some(user_id)),
        update_by: Set(Some(user_id)),
        ..Default::default()
    }
    .insert(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("DB: {e}")))?;
    Ok(m)
}

pub async fn update_package(
    state: &AppState,
    id: i32,
    body: &UpdatePackageReq,
    user_id: i32,
) -> Result<package::Model, AppError> {
    let existing = read_package(state, id).await?;
    let mut a: package::ActiveModel = existing.into();
    if let Some(ref v) = body.name { a.name = Set(v.clone()); }
    if let Some(ref v) = body.subtitle { a.subtitle = Set(Some(v.clone())); }
    if let Some(ref v) = body.category { a.category = Set(Some(v.clone())); }
    if let Some(v) = body.price { a.price = Set(v); }
    if let Some(v) = body.original_price { a.original_price = Set(Some(v)); }
    if let Some(v) = body.deposit { a.deposit = Set(Some(v)); }
    if let Some(ref v) = body.cover_image { a.cover_image = Set(Some(v.clone())); }
    if let Some(ref v) = body.description { a.description = Set(Some(v.clone())); }
    if let Some(ref v) = body.service_items { a.service_items = Set(Some(v.clone())); }
    if let Some(ref v) = body.suitable_people { a.suitable_people = Set(Some(v.clone())); }
    if let Some(ref v) = body.shooting_location { a.shooting_location = Set(Some(v.clone())); }
    if let Some(v) = body.validity_days { a.validity_days = Set(Some(v)); }
    if let Some(v) = body.sort_order { a.sort_order = Set(Some(v)); }
    if let Some(v) = body.is_hot { a.is_hot = Set(Some(v)); }
    if let Some(v) = body.is_recommend { a.is_recommend = Set(Some(v)); }
    if let Some(v) = body.status { a.status = Set(Some(v)); }
    a.update_by = Set(Some(user_id));
    a.update(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB: {e}")))?;
    read_package(state, id).await
}

pub async fn delete_package(state: &AppState, id: i32) -> Result<(), AppError> {
    package::Entity::delete_by_id(id)
        .exec(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB: {e}")))?;
    Ok(())
}

// ── Package Items ───────────────────────────────────────

pub async fn list_items(
    state: &AppState,
    package_id: i32,
) -> Result<Vec<package_item::Model>, AppError> {
    package_item::Entity::find()
        .filter(package_item::Column::PackageId.eq(package_id))
        .order_by_asc(package_item::Column::SortOrder)
        .all(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB: {e}")))
}

pub async fn create_item(
    state: &AppState,
    body: &CreateItemReq,
) -> Result<package_item::Model, AppError> {
    let m = package_item::ActiveModel {
        package_id: Set(body.package_id),
        item_type: Set(body.item_type.clone()),
        item_name: Set(body.item_name.clone()),
        quantity: Set(body.quantity),
        unit: Set(body.unit.clone()),
        item_value: Set(body.item_value.clone()),
        sort_order: Set(body.sort_order),
        is_default: Set(body.is_default),
        ..Default::default()
    }
    .insert(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("DB: {e}")))?;
    Ok(m)
}

pub async fn update_item(
    state: &AppState,
    id: i32,
    body: &UpdateItemReq,
) -> Result<package_item::Model, AppError> {
    let existing = package_item::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB: {e}")))?
        .ok_or(AppError::NotFound("套餐项不存在".into()))?;
    let mut a: package_item::ActiveModel = existing.into();
    if let Some(ref v) = body.item_type { a.item_type = Set(v.clone()); }
    if let Some(ref v) = body.item_name { a.item_name = Set(v.clone()); }
    if let Some(v) = body.quantity { a.quantity = Set(Some(v)); }
    if let Some(ref v) = body.unit { a.unit = Set(Some(v.clone())); }
    if let Some(ref v) = body.item_value { a.item_value = Set(Some(v.clone())); }
    if let Some(v) = body.sort_order { a.sort_order = Set(Some(v)); }
    if let Some(v) = body.is_default { a.is_default = Set(Some(v)); }
    a.update(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB: {e}")))?;
    Ok(package_item::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB: {e}")))?
        .unwrap())
}

pub async fn delete_item(state: &AppState, id: i32) -> Result<(), AppError> {
    package_item::Entity::delete_by_id(id)
        .exec(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB: {e}")))?;
    Ok(())
}

// ── Package Gallery ─────────────────────────────────────

pub async fn list_gallery(
    state: &AppState,
    package_id: i32,
) -> Result<Vec<package_gallery::Model>, AppError> {
    package_gallery::Entity::find()
        .filter(package_gallery::Column::PackageId.eq(package_id))
        .order_by_asc(package_gallery::Column::SortOrder)
        .all(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB: {e}")))
}

pub async fn create_gallery(
    state: &AppState,
    body: &CreateGalleryReq,
) -> Result<package_gallery::Model, AppError> {
    let m = package_gallery::ActiveModel {
        package_id: Set(body.package_id),
        image_url: Set(body.image_url.clone()),
        image_type: Set(body.image_type.clone()),
        caption: Set(body.caption.clone()),
        sort_order: Set(body.sort_order),
        ..Default::default()
    }
    .insert(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("DB: {e}")))?;
    Ok(m)
}

pub async fn update_gallery(
    state: &AppState,
    id: i32,
    body: &UpdateGalleryReq,
) -> Result<package_gallery::Model, AppError> {
    let existing = package_gallery::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB: {e}")))?
        .ok_or(AppError::NotFound("套餐图片不存在".into()))?;
    let mut a: package_gallery::ActiveModel = existing.into();
    if let Some(ref v) = body.image_url { a.image_url = Set(v.clone()); }
    if let Some(ref v) = body.image_type { a.image_type = Set(Some(v.clone())); }
    if let Some(ref v) = body.caption { a.caption = Set(Some(v.clone())); }
    if let Some(v) = body.sort_order { a.sort_order = Set(Some(v)); }
    a.update(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB: {e}")))?;
    Ok(package_gallery::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB: {e}")))?
        .unwrap())
}

pub async fn delete_gallery(state: &AppState, id: i32) -> Result<(), AppError> {
    package_gallery::Entity::delete_by_id(id)
        .exec(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB: {e}")))?;
    Ok(())
}
