//! Date settings service.

use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};

use crate::app_state::AppState;
use crate::entity::date_setting;
use crate::error::AppError;

use super::dto::{CreateReq, ListQuery, UpdateReq};

/// List date settings with pagination, owner-scoped by photographer_id.
pub async fn list(
    state: &AppState,
    q: &ListQuery,
    photographer_id: Option<i32>,
) -> Result<(Vec<date_setting::Model>, u64), AppError> {
    let page = q.page.unwrap_or(1);
    let ps = q.page_size.unwrap_or(20);
    let mut s = date_setting::Entity::find();

    // Provider scoping is applied by the caller when needed.
    if let Some(pid) = photographer_id {
        s = s.filter(date_setting::Column::PhotographerId.eq(pid));
    }

    let total = s
        .clone()
        .count(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?;
    let rows = s
        .order_by_desc(date_setting::Column::TargetDate)
        .offset(((page - 1) * ps) as u64)
        .limit(ps)
        .all(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?;
    Ok((rows, total))
}

/// Read a single date setting by id.
pub async fn read(state: &AppState, id: i32) -> Result<date_setting::Model, AppError> {
    date_setting::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?
        .ok_or(AppError::NotFound("不存在".into()))
}

/// Create a new date setting.
pub async fn create(state: &AppState, body: CreateReq) -> Result<date_setting::Model, AppError> {
    let m = date_setting::ActiveModel {
        photographer_id: Set(body.photographer_id),
        target_date: Set(body.target_date),
        start_time: Set(body.start_time),
        end_time: Set(body.end_time),
        is_available: Set(body.is_available),
        use_template_id: Set(body.use_template_id),
        reason: Set(body.reason),
        ..Default::default()
    }
    .insert(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("DB:{e}")))?;
    Ok(m)
}

/// Update a date setting by id.
pub async fn update(
    state: &AppState,
    id: i32,
    body: UpdateReq,
) -> Result<date_setting::Model, AppError> {
    let rec = date_setting::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?
        .ok_or(AppError::NotFound("不存在".into()))?;
    let mut a: date_setting::ActiveModel = rec.into();
    if let Some(v) = body.target_date {
        a.target_date = Set(v);
    }
    if let Some(v) = body.start_time {
        a.start_time = Set(Some(v));
    }
    if let Some(v) = body.end_time {
        a.end_time = Set(Some(v));
    }
    if let Some(v) = body.is_available {
        a.is_available = Set(Some(v));
    }
    if let Some(v) = body.use_template_id {
        a.use_template_id = Set(Some(v));
    }
    if let Some(v) = body.reason {
        a.reason = Set(Some(v));
    }
    let r = a
        .update(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?;
    Ok(r)
}

/// Delete a date setting by id.
pub async fn delete_one(state: &AppState, id: i32) -> Result<(), AppError> {
    date_setting::Entity::delete_by_id(id)
        .exec(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?;
    Ok(())
}

/// Check date availability for a photographer, returning is_rest_day + blocks.
pub async fn check(
    state: &AppState,
    photographer_id: i32,
    target_date: &str,
) -> Result<serde_json::Value, AppError> {
    let day_blocked = date_setting::Entity::find()
        .filter(date_setting::Column::PhotographerId.eq(photographer_id))
        .filter(date_setting::Column::TargetDate.eq(target_date))
        .filter(date_setting::Column::IsAvailable.eq(0))
        .filter(date_setting::Column::StartTime.is_null())
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?;
    let blocks = date_setting::Entity::find()
        .filter(date_setting::Column::PhotographerId.eq(photographer_id))
        .filter(date_setting::Column::TargetDate.eq(target_date))
        .filter(date_setting::Column::IsAvailable.eq(0))
        .filter(date_setting::Column::StartTime.is_not_null())
        .all(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?;
    let blist: Vec<_> = blocks
        .iter()
        .map(|r| serde_json::to_value(r).unwrap())
        .collect();
    Ok(serde_json::json!({
        "is_rest_day": day_blocked.is_some(), "blocks": blist
    }))
}
