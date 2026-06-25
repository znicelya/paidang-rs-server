//! Date slots — CRUD + day + monthly. JWT-protected, photographer-owner scoped.

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
use crate::entity::date_slot;
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
    pub template_id: Option<i32>,
    #[validate(custom(function = "valid_date_format"))]
    pub slot_date: String,
    #[validate(length(min = 1))]
    pub slot_name: String,
    pub start_time: String,
    pub end_time: String,
    pub is_special: Option<i8>,
    pub status: Option<i8>,
    pub price: Option<i32>,
    pub remark: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams, ToSchema)]
pub struct UpdateReq {
    pub slot_name: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub is_special: Option<i8>,
    pub status: Option<i8>,
    pub price: Option<i32>,
    pub remark: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams, ToSchema)]
pub struct ListQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub photographer_id: Option<i32>,
    pub slot_date: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams, ToSchema)]
pub struct DayQuery {
    pub photographer_id: i32,
    pub slot_date: String,
}

#[derive(Debug, Deserialize, utoipa::IntoParams, ToSchema)]
pub struct MonthlyQuery {
    pub photographer_id: i32,
    pub year_month: String,
}

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list, create))
        .routes(routes!(read, update, delete_one))
        .routes(routes!(day))
        .routes(routes!(monthly))
}

/// GET /date-slots — list date slots.
#[utoipa::path(
    get,
    path = "/date-slots",
    params(ListQuery),
    responses(
        (status = 200, body = ApiResponse<PaginatedData<serde_json::Value>>),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "date-slots",
)]
async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<ListQuery>,
) -> Result<Json<ApiResponse<PaginatedData<serde_json::Value>>>, AppError> {
    let page = q.page.unwrap_or(1);
    let ps = q.page_size.unwrap_or(20);
    let mut s = date_slot::Entity::find();

    // Owner scoping: non-admin can only see their own
    let pid = if auth.role >= 2 {
        q.photographer_id
    } else {
        Some(auth.user_id)
    };
    if let Some(pid) = pid {
        s = s.filter(date_slot::Column::PhotographerId.eq(pid));
    }
    if let Some(ref d) = q.slot_date {
        s = s.filter(date_slot::Column::SlotDate.eq(d));
    }
    let total = s
        .clone()
        .count(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?;
    let rows = s
        .order_by_asc(date_slot::Column::SlotDate)
        .order_by_asc(date_slot::Column::StartTime)
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

/// GET /date-slots/{id} — read a single date slot.
#[utoipa::path(
    get,
    path = "/date-slots/{id}",
    params(("id" = i32, Path, description = "Date slot ID")),
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    ),
    tag = "date-slots",
)]
async fn read(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let r = date_slot::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?
        .ok_or(AppError::NotFound("不存在".into()))?;
    require_owner(&auth, r.photographer_id)?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(r).unwrap())))
}

/// POST /date-slots — create a new date slot.
#[utoipa::path(
    post,
    path = "/date-slots",
    request_body = CreateReq,
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 400, description = "Input validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    tag = "date-slots",
)]
async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<CreateReq>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    body.validate()
        .map_err(|e| AppError::InputValidation(e.to_string()))?;
    require_owner(&auth, body.photographer_id)?;
    let m = date_slot::ActiveModel {
        photographer_id: Set(body.photographer_id),
        template_id: Set(body.template_id),
        slot_date: Set(body.slot_date),
        slot_name: Set(body.slot_name),
        start_time: Set(body.start_time),
        end_time: Set(body.end_time),
        is_special: Set(body.is_special),
        status: Set(body.status),
        price: Set(body.price),
        remark: Set(body.remark),
        ..Default::default()
    }
    .insert(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("DB:{e}")))?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(m).unwrap())))
}

/// PUT /date-slots/{id} — update a date slot.
#[utoipa::path(
    put,
    path = "/date-slots/{id}",
    params(("id" = i32, Path, description = "Date slot ID")),
    request_body = UpdateReq,
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    ),
    tag = "date-slots",
)]
async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(body): Json<UpdateReq>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let rec = date_slot::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?
        .ok_or(AppError::NotFound("不存在".into()))?;
    require_owner(&auth, rec.photographer_id)?;
    let mut a: date_slot::ActiveModel = rec.into();
    if let Some(v) = body.slot_name {
        a.slot_name = Set(v);
    }
    if let Some(v) = body.start_time {
        a.start_time = Set(v);
    }
    if let Some(v) = body.end_time {
        a.end_time = Set(v);
    }
    if let Some(v) = body.is_special {
        a.is_special = Set(Some(v));
    }
    if let Some(v) = body.status {
        a.status = Set(Some(v));
    }
    if let Some(v) = body.price {
        a.price = Set(Some(v));
    }
    if let Some(v) = body.remark {
        a.remark = Set(Some(v));
    }
    let r = a
        .update(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?;
    Ok(Json(ApiResponse::ok(serde_json::to_value(r).unwrap())))
}

/// DELETE /date-slots/{id} — delete a date slot.
#[utoipa::path(
    delete,
    path = "/date-slots/{id}",
    params(("id" = i32, Path, description = "Date slot ID")),
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    ),
    tag = "date-slots",
)]
async fn delete_one(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    let rec = date_slot::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?
        .ok_or(AppError::NotFound("不存在".into()))?;
    require_owner(&auth, rec.photographer_id)?;
    date_slot::Entity::delete_by_id(id)
        .exec(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?;
    Ok(Json(ApiResponse::ok(())))
}

/// GET /date-slots/day — slots for a specific day.
#[utoipa::path(
    get,
    path = "/date-slots/day",
    params(DayQuery),
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    tag = "date-slots",
)]
async fn day(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<DayQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    require_owner(&auth, q.photographer_id)?;
    let rows = date_slot::Entity::find()
        .filter(date_slot::Column::PhotographerId.eq(q.photographer_id))
        .filter(date_slot::Column::SlotDate.eq(&q.slot_date))
        .order_by_asc(date_slot::Column::StartTime)
        .all(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?;
    let list: Vec<_> = rows
        .iter()
        .map(|r| serde_json::to_value(r).unwrap())
        .collect();
    Ok(Json(ApiResponse::ok(serde_json::json!({"list":list}))))
}

/// GET /date-slots/monthly — slots for a month.
#[utoipa::path(
    get,
    path = "/date-slots/monthly",
    params(MonthlyQuery),
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    tag = "date-slots",
)]
async fn monthly(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<MonthlyQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    require_owner(&auth, q.photographer_id)?;
    let prefix = format!("{}%", q.year_month);
    let rows = date_slot::Entity::find()
        .filter(date_slot::Column::PhotographerId.eq(q.photographer_id))
        .filter(date_slot::Column::SlotDate.like(&prefix))
        .order_by_asc(date_slot::Column::SlotDate)
        .order_by_asc(date_slot::Column::StartTime)
        .all(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB:{e}")))?;
    let list: Vec<_> = rows
        .iter()
        .map(|r| serde_json::to_value(r).unwrap())
        .collect();
    Ok(Json(ApiResponse::ok(serde_json::json!({"list":list}))))
}
