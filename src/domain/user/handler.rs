//! User handlers: GET profile, PUT profile, POST avatar upload.

use axum::Json;
use axum::extract::{Multipart, Query, State};
use validator::Validate;

use crate::app_state::AppState;
use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use crate::response::ApiResponse;

use super::dto::{self, ProfileQuery, UpdateProfileRequest};
use super::service;

/// GET /user/profile — read a public user profile by user_id.
#[utoipa::path(
    get,
    path = "/user/profile",
    params(ProfileQuery),
    responses(
        (status = 200, body = ApiResponse<dto::ProfileData>),
        (status = 400, description = "Missing user_id"),
    ),
    tag = "user",
)]
pub async fn get_profile(
    State(state): State<AppState>,
    Query(query): Query<ProfileQuery>,
) -> Result<Json<ApiResponse<dto::ProfileData>>, AppError> {
    let user_id = query
        .user_id
        .ok_or_else(|| AppError::InputValidation("user_id is required".into()))?;
    let data = service::get_profile(&state, user_id).await?;
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
/// Multipart form with `avatar` field (mini-program contract) or `file` field.
#[utoipa::path(
    post,
    path = "/user/avatar",
    request_body(content_type = "multipart/form-data", description = "Multipart form with `avatar` or `file` field"),
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 400, description = "No file provided"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "COS not configured"),
    ),
    tag = "user",
)]
pub async fn upload_avatar(
    State(state): State<AppState>,
    auth: AuthUser,
    mut multipart: Multipart,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let mut file_data: Option<Vec<u8>> = None;
    let mut file_name = avatar_storage_name(auth.user_id, "avatar.jpg");
    let mut content_type = String::from("application/octet-stream");

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::InputValidation(format!("upload parse error: {e}")))?
    {
        if matches!(field.name(), Some("avatar") | Some("file")) {
            file_name =
                avatar_storage_name(auth.user_id, field.file_name().unwrap_or("avatar.jpg"));
            content_type = field
                .content_type()
                .unwrap_or("application/octet-stream")
                .to_string();
            file_data = Some(
                field
                    .bytes()
                    .await
                    .map_err(|e| AppError::InputValidation(format!("read avatar: {e}")))?
                    .to_vec(),
            );
        }
    }

    let data = file_data.ok_or_else(|| AppError::InputValidation("No file provided".into()))?;
    if data.is_empty() {
        return Err(AppError::InputValidation("empty file".into()));
    }

    let upload =
        crate::domain::files::service::upload(&state, data, &file_name, &content_type, "avatars/")
            .await?;

    if upload
        .get("blocked")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        return Ok(Json(ApiResponse::ok(upload)));
    }

    let key = upload
        .get("key")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Internal("upload response missing key".into()))?;
    let avatar_url = format!("/files/{key}");

    Ok(Json(ApiResponse::ok(serde_json::json!({
        "avatar_url": avatar_url,
    }))))
}

fn avatar_storage_name(user_id: i32, original_name: &str) -> String {
    let ext = original_name
        .rsplit_once('.')
        .map(|(_, ext)| ext)
        .filter(|ext| {
            !ext.is_empty() && ext.len() <= 10 && ext.chars().all(|c| c.is_ascii_alphanumeric())
        })
        .unwrap_or("jpg")
        .to_ascii_lowercase();
    format!(
        "{user_id}_{}.{}",
        chrono::Utc::now().timestamp_millis(),
        ext
    )
}
