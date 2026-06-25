//! Date settings — CRUD + check. JWT-protected.

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
use validator::{Validate, ValidationError};

use crate::app_state::AppState;
use crate::entity::date_setting;
use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use crate::response::{ApiResponse, PaginatedData};

/// Validate YYYY-MM-DD date format.
fn valid_date_format(s: &str) -> Result<(), ValidationError> {
    let re = regex::Regex::new(r"^\d{4}-\d{2}-\d{2}$")
        .map_err(|_| ValidationError::new("regex"))?;
    if re.is_match(s) {
        Ok(())
    } else {
        Err(ValidationError::new("invalid_date_format"))
    }
}

/// Require that the authenticated user is the owner of the resource or an admin.
fn require_owner(auth: &AuthUser, photographer_id: i32) -> Result<(), AppError> {
    if auth.role >= 2 || auth.user_id == photographer_id {
        Ok(())
    } else {
        Err(AppError::Forbidden("无权操作此资源".into()))
    }
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateReq {
    #[validate(range(min = 1))]
    pub photographer_id: i32,
    #[validate(custom(function = "valid_date_format"))]
    pub target_date: String,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub is_available: Option<i8>,
    pub use_template_id: Option<i32>,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams, ToSchema)]
pub struct UpdateReq {
    pub target_date: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub is_available: Option<i8>,
    pub use_template_id: Option<i32>,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams, ToSchema)]
pub struct ListQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub photographer_id: Option<i32>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams, ToSchema)]
pub struct CheckQuery {
    pub photographer_id: i32,
    pub target_date: String,
}

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list, create))
        .routes(routes!(read, update, delete_one))
        .routes(routes!(check))
}

/// GET /date-settings — list date settings.
#[utoipa::path(
    get,
    path = "/date-settings",
    params(ListQuery),
    responses(
        (status = 200, body = ApiResponse<PaginatedData<serde_json::Value>>),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "date-settings",
)]
async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<ListQuery>,
) -> Result<Json<ApiResponse<PaginatedData<serde_json::Value>>>, AppError> {
    let page = q.page.unwrap_or(1);
    let ps = q.page_size.unwrap_or(20);
    let mut s = date_setting::Entity::find();

    // Owner scoping: non-admin can only see their own
    let pid = if auth.role >= 2 {
        q.photographer_id
    } else {
        Some(auth.user_id)
    };
    if let Some(pid) = pid {
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
    let list: Vec<_> = rows
        .iter()
        .map(|r| serde_json::to_value(r).unwrap())
        .collect();
    Ok(Json(ApiResponse::ok(PaginatedData::new(
        list, total, page, ps,
    ))))
}

/// GET /date-settings/{id} — read a single date setting.
#[utoipa::path(
    get,
    path = "/date-settings/{id}",
    params(("id" = i32, Path, description = "Date setting ID")),
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    ),
    tag = "date-settings",
)]
async fn read(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let r = date_setting::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?
        .ok_or(AppError::NotFound("不存在".into()))?;
    require_owner(&auth, r.photographer_id)?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(r).unwrap())))
}

/// POST /date-settings — create a new date setting.
#[utoipa::path(
    post,
    path = "/date-settings",
    request_body = CreateReq,
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 400, description = "Input validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    tag = "date-settings",
)]
async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<CreateReq>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    body.validate()
        .map_err(|e| AppError::InputValidation(e.to_string()))?;
    require_owner(&auth, body.photographer_id)?;
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
    Ok(Json(ApiResponse::ok(serde_json::to_value(m).unwrap())))
}

/// PUT /date-settings/{id} — update a date setting.
#[utoipa::path(
    put,
    path = "/date-settings/{id}",
    params(("id" = i32, Path, description = "Date setting ID")),
    request_body = UpdateReq,
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    ),
    tag = "date-settings",
)]
async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(body): Json<UpdateReq>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let rec = date_setting::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?
        .ok_or(AppError::NotFound("不存在".into()))?;
    require_owner(&auth, rec.photographer_id)?;
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
    Ok(Json(ApiResponse::ok(serde_json::to_value(r).unwrap())))
}

/// DELETE /date-settings/{id} — delete a date setting.
#[utoipa::path(
    delete,
    path = "/date-settings/{id}",
    params(("id" = i32, Path, description = "Date setting ID")),
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    ),
    tag = "date-settings",
)]
async fn delete_one(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    let rec = date_setting::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?
        .ok_or(AppError::NotFound("不存在".into()))?;
    require_owner(&auth, rec.photographer_id)?;
    date_setting::Entity::delete_by_id(id)
        .exec(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?;
    Ok(Json(ApiResponse::ok(())))
}

/// GET /date-settings/check — check date availability.
#[utoipa::path(
    get,
    path = "/date-settings/check",
    params(CheckQuery),
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    tag = "date-settings",
)]
async fn check(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<CheckQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    require_owner(&auth, q.photographer_id)?;
    let day_blocked = date_setting::Entity::find()
        .filter(date_setting::Column::PhotographerId.eq(q.photographer_id))
        .filter(date_setting::Column::TargetDate.eq(&q.target_date))
        .filter(date_setting::Column::IsAvailable.eq(0))
        .filter(date_setting::Column::StartTime.is_null())
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?;
    let blocks = date_setting::Entity::find()
        .filter(date_setting::Column::PhotographerId.eq(q.photographer_id))
        .filter(date_setting::Column::TargetDate.eq(&q.target_date))
        .filter(date_setting::Column::IsAvailable.eq(0))
        .filter(date_setting::Column::StartTime.is_not_null())
        .all(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?;
    let blist: Vec<_> = blocks
        .iter()
        .map(|r| serde_json::to_value(r).unwrap())
        .collect();
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "is_rest_day": day_blocked.is_some(), "blocks": blist
    }))))
}
