//! Time slot templates — CRUD, photographer-owned (role >= 1).

use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};
use serde::Deserialize;
use validator::{Validate, ValidationError};

use crate::app_state::AppState;
use crate::entity::time_slot_template;
use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use crate::response::{ApiResponse, PaginatedData};

/// Validate HH:MM time format.
fn valid_time_format(s: &str) -> Result<(), ValidationError> {
    let re = regex::Regex::new(r"^\d{2}:\d{2}$")
        .map_err(|_| ValidationError::new("regex"))?;
    if re.is_match(s) { Ok(()) } else { Err(ValidationError::new("invalid_time_format")) }
}

/// Require that the authenticated user is the owner of the resource or an admin.
fn require_owner(auth: &AuthUser, photographer_id: i32) -> Result<(), AppError> {
    if auth.role >= 2 || auth.user_id == photographer_id {
        Ok(())
    } else {
        Err(AppError::Forbidden("无权操作此资源".into()))
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateReq {
    #[validate(length(min = 1))]
    pub slot_name: String,
    #[validate(custom(function = "valid_time_format"))]
    pub start_time: String,
    #[validate(custom(function = "valid_time_format"))]
    pub end_time: String,
    pub sort_order: Option<i32>,
    pub is_default: Option<i8>,
    pub status: Option<i8>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateReq {
    pub slot_name: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub sort_order: Option<i32>,
    pub is_default: Option<i8>,
    pub status: Option<i8>,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/time-slot-templates", get(list).post(create))
        .route(
            "/time-slot-templates/{id}",
            get(read).put(update).delete(delete_one),
        )
}

async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<ListQuery>,
) -> Result<Json<ApiResponse<PaginatedData<serde_json::Value>>>, AppError> {
    let page = q.page.unwrap_or(1);
    let ps = q.page_size.unwrap_or(20);
    let select = time_slot_template::Entity::find()
        .filter(time_slot_template::Column::PhotographerId.eq(auth.user_id));
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
    let list: Vec<_> = rows
        .iter()
        .map(|r| serde_json::to_value(r).unwrap())
        .collect();
    Ok(Json(ApiResponse::ok(PaginatedData::new(
        list, total, page, ps,
    ))))
}

async fn read(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let r = time_slot_template::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?
        .ok_or(AppError::NotFound("不存在".into()))?;
    require_owner(&auth, r.photographer_id)?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(r).unwrap())))
}

async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<CreateReq>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    body.validate()
        .map_err(|e| AppError::InputValidation(e.to_string()))?;
    let m = time_slot_template::ActiveModel {
        photographer_id: Set(auth.user_id),
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
    Ok(Json(ApiResponse::ok(serde_json::to_value(m).unwrap())))
}

async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(body): Json<UpdateReq>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let rec = time_slot_template::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?
        .ok_or(AppError::NotFound("不存在".into()))?;
    require_owner(&auth, rec.photographer_id)?;
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
    Ok(Json(ApiResponse::ok(serde_json::to_value(r).unwrap())))
}

async fn delete_one(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    let rec = time_slot_template::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?
        .ok_or(AppError::NotFound("不存在".into()))?;
    require_owner(&auth, rec.photographer_id)?;
    time_slot_template::Entity::delete_by_id(id)
        .exec(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?;
    Ok(Json(ApiResponse::ok(())))
}
