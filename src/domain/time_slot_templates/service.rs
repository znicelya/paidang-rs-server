//! Time slot templates service.

use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};

use crate::app_state::AppState;
use crate::entity::time_slot_template;
use crate::error::AppError;

use super::dto::{CreateReq, UpdateReq};

/// List templates with pagination, filtered by photographer.
pub async fn list(
    state: &AppState,
    photographer_id: i32,
    page: u64,
    ps: u64,
) -> Result<(Vec<time_slot_template::Model>, u64), AppError> {
    let select = time_slot_template::Entity::find()
        .filter(time_slot_template::Column::PhotographerId.eq(photographer_id));
    let total = select
        .clone()
        .count(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?;
    let rows = select
        .order_by_asc(time_slot_template::Column::SortOrder)
        .offset(((page - 1) * ps) as u64)
        .limit(ps)
        .all(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?;
    Ok((rows, total))
}

/// Read a single template by id.
pub async fn read(state: &AppState, id: i32) -> Result<time_slot_template::Model, AppError> {
    time_slot_template::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?
        .ok_or(AppError::NotFound("不存在".into()))
}

/// Create a new template.
pub async fn create(
    state: &AppState,
    photographer_id: i32,
    body: CreateReq,
) -> Result<time_slot_template::Model, AppError> {
    let m = time_slot_template::ActiveModel {
        photographer_id: Set(photographer_id),
        slot_name: Set(body.slot_name),
        start_time: Set(body.start_time),
        end_time: Set(body.end_time),
        sort_order: Set(body.sort_order),
        is_default: Set(body.is_default),
        status: Set(body.status),
        ..Default::default()
    }
    .insert(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("DB:{e}")))?;
    Ok(m)
}

/// Update an existing template.
pub async fn update(
    state: &AppState,
    id: i32,
    body: UpdateReq,
) -> Result<time_slot_template::Model, AppError> {
    let rec = time_slot_template::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?
        .ok_or(AppError::NotFound("不存在".into()))?;
    let mut a: time_slot_template::ActiveModel = rec.into();
    if let Some(v) = body.slot_name {
        a.slot_name = Set(v);
    }
    if let Some(v) = body.start_time {
        a.start_time = Set(v);
    }
    if let Some(v) = body.end_time {
        a.end_time = Set(v);
    }
    if let Some(v) = body.sort_order {
        a.sort_order = Set(Some(v));
    }
    if let Some(v) = body.is_default {
        a.is_default = Set(Some(v));
    }
    if let Some(v) = body.status {
        a.status = Set(Some(v));
    }
    let r = a
        .update(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?;
    Ok(r)
}

/// Delete a template by id.
pub async fn delete_one(state: &AppState, id: i32) -> Result<(), AppError> {
    time_slot_template::Entity::delete_by_id(id)
        .exec(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?;
    Ok(())
}
