//! User handlers: GET profile, PUT profile, POST avatar upload.

use axum::extract::{Multipart, State};
use axum::Json;
use validator::Validate;

use crate::app_state::AppState;
use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use crate::response::ApiResponse;

use super::dto::{self, UpdateProfileRequest};
use super::service;

/// GET /user/profile — read the authenticated user's profile.
#[utoipa::path(
    get,
    path = "/user/profile",
    responses(
        (status = 200, body = ApiResponse<dto::ProfileData>),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "user",
)]
pub async fn get_profile(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<ApiResponse<dto::ProfileData>>, AppError> {
    let data = service::get_profile(&state, auth.user_id).await?;
    Ok(Json(ApiResponse::ok(data)))
}

/// PUT /user/profile — update the authenticated user's profile.
#[utoipa::path(
    put,
    path = "/user/profile",
    request_body = UpdateProfileRequest,
    responses(
        (status = 200, body = ApiResponse<dto::ProfileData>),
        (status = 400, description = "Input validation error"),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "user",
)]
pub async fn update_profile(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<UpdateProfileRequest>,
) -> Result<Json<ApiResponse<dto::ProfileData>>, AppError> {
    body.validate()
        .map_err(|e| AppError::InputValidation(e.to_string()))?;
    let data = service::update_profile(&state, auth.user_id, &body).await?;
    Ok(Json(ApiResponse::ok(data)))
}

/// POST /user/avatar — upload avatar.
///
/// Ported from `paidang-worker-server/src/endpoints/user/avatarUpload.ts`.
/// Multipart form with `file` field. In M2 this is a stub; the actual COS
/// upload + moderation pipeline is wired in M5 (`domain/files`).
#[utoipa::path(
    post,
    path = "/user/avatar",
    request_body(content_type = "multipart/form-data", description = "Multipart form with `file` field"),
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 400, description = "No file provided"),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "user",
)]
pub async fn upload_avatar(
    State(_state): State<AppState>,
    auth: AuthUser,
    mut multipart: Multipart,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let mut file_found = false;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::InputValidation(format!("upload parse error: {e}")))?
    {
        if field.name() == Some("file") {
            file_found = true;
            // TODO(M5): wire up COS upload + Qiniu moderation
            let _ = field;
        }
    }
    if !file_found {
        return Err(AppError::InputValidation("No file provided".into()));
    }

    // Stub response for now — returns the old avatar_url unchanged.
    // TODO(M5): return actual uploaded URL + moderation result.
    Ok(Json(ApiResponse::ok(
        serde_json::json!({ "user_id": auth.user_id, "message": "avatar upload (M5)" }),
    )))
}