//! POST /auth/login — WeChat mini-program login + JWT issuance.

use axum::extract::State;
use axum::Json;
use validator::Validate;

use crate::app_state::AppState;
use crate::error::AppError;
use crate::external::wechat::ReqwestWechat;
use crate::response::ApiResponse;

use super::dto::{LoginData, LoginRequest};
use super::service;

/// POST /auth/login
///
/// Accepts `{ code, nickname?, avatar_url?, phone?, phone_code? }`.
/// Returns `{ user_id, openid, role, phone, is_new, token }`.
#[utoipa::path(
    post,
    path = "/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, body = ApiResponse<LoginData>),
        (status = 400, description = "Input validation error"),
        (status = 500, description = "External/Internal error"),
    ),
    tag = "auth",
)]
pub async fn login_handler(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<ApiResponse<super::dto::LoginData>>, AppError> {
    body.validate()
        .map_err(|e| AppError::InputValidation(e.to_string()))?;

    // Build the real WeChat client from settings
    let appid = state
        .settings
        .wechat_appid
        .as_ref()
        .ok_or(AppError::External("WX_APPID not configured".into()))?;
    let secret = state
        .settings
        .wechat_secret
        .as_ref()
        .ok_or(AppError::External("WX_SECRET not configured".into()))?;

    let wechat = ReqwestWechat::new(appid.clone(), secret.clone());

    let data = service::login(
        &state,
        &wechat,
        &body.code,
        body.nickname.as_deref(),
        body.avatar_url.as_deref(),
        body.phone.as_deref(),
        body.phone_code.as_deref(),
    )
    .await?;

    Ok(Json(ApiResponse::ok(data)))
}