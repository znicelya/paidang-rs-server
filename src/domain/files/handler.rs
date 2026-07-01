//! Files handlers - upload / list / download (proxy) / delete via COS.

use axum::Json;
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{StatusCode, header};
use axum::response::IntoResponse;

use crate::app_state::AppState;
use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use crate::response::ApiResponse;

use super::dto::{
    Base64UploadRequest, DeleteQuery, ListQuery, ModerateUploadRequest, SignQuery,
    UploadPolicyRequest,
};
use super::service;

/// POST /files - JSON upload to COS after moderation.
#[utoipa::path(
    post,
    path = "/files",
    request_body = Base64UploadRequest,
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 400, description = "Invalid upload payload"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "COS not configured"),
    ),
    tag = "files",
)]
pub async fn upload(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<Base64UploadRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let data = service::decode_base64_upload(&body.data_base64)?;
    if data.is_empty() {
        return Err(AppError::InputValidation("empty file".into()));
    }

    let original_name = body.file_name.as_deref().unwrap_or("upload.jpg");
    let content_type = body
        .content_type
        .as_deref()
        .unwrap_or("application/octet-stream");
    let prefix = body
        .prefix
        .as_deref()
        .or(body.folder.as_deref())
        .unwrap_or("files");
    let normalized_prefix = format!("{}/", prefix.trim().trim_matches('/'));
    let stored_file_name = service::storage_file_name(auth.user_id, original_name);
    let mut value = service::upload(
        &state,
        data,
        &stored_file_name,
        content_type,
        &normalized_prefix,
    )
    .await?;

    if normalized_prefix == "avatars/" {
        if let Some(path) = value
            .get("path")
            .and_then(|v| v.as_str())
            .map(str::to_string)
        {
            if let Some(obj) = value.as_object_mut() {
                obj.insert("avatar_url".into(), serde_json::Value::String(path));
            }
        }
    }

    Ok(Json(ApiResponse::ok(value)))
}

/// POST /files/upload-policy - disabled. Direct COS upload cannot satisfy pre-upload moderation.
#[utoipa::path(
    post,
    path = "/files/upload-policy",
    request_body = UploadPolicyRequest,
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "COS not configured"),
    ),
    tag = "files",
)]
pub async fn upload_policy(
    State(_state): State<AppState>,
    _auth: AuthUser,
    Json(_body): Json<UploadPolicyRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    Err(AppError::InputValidation(
        "直传 COS 已禁用，请通过 /files 上传并完成审核".into(),
    ))
}

/// POST /files/moderate - disabled. Files must be moderated before COS upload.
#[utoipa::path(
    post,
    path = "/files/moderate",
    request_body = ModerateUploadRequest,
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "COS not configured"),
    ),
    tag = "files",
)]
pub async fn moderate_upload(
    State(_state): State<AppState>,
    _auth: AuthUser,
    Json(_body): Json<ModerateUploadRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    Err(AppError::InputValidation(
        "上传后审核接口已禁用，请通过 /files 上传并完成审核".into(),
    ))
}

/// GET /files/*path - proxy download from COS.
#[utoipa::path(
    get,
    path = "/files/{*path}",
    params(("path" = String, Path, description = "File path in COS bucket")),
    responses(
        (status = 200, description = "File content"),
        (status = 500, description = "COS not configured"),
    ),
    tag = "files",
)]
pub async fn download(
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let (body, content_type) = service::download(&state, &path).await?;

    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, content_type),
            (
                header::CONTENT_DISPOSITION,
                format!(
                    "inline; filename=\"{}\"",
                    path.rsplit('/').next().unwrap_or(&path)
                ),
            ),
        ],
        Body::from(body),
    ))
}

/// GET /files - list objects by prefix.
#[utoipa::path(
    get,
    path = "/files",
    params(ListQuery),
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "COS not configured"),
    ),
    tag = "files",
)]
pub async fn list(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(q): Query<ListQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let prefix = q.prefix.unwrap_or_else(|| "files/".to_string());
    let value = service::list(&state, &prefix).await?;
    Ok(Json(ApiResponse::ok(value)))
}

/// GET /files/sign?key= - create a temporary signed COS URL.
#[utoipa::path(
    get,
    path = "/files/sign",
    params(SignQuery),
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 500, description = "COS not configured"),
    ),
    tag = "files",
)]
pub async fn sign_url(
    State(state): State<AppState>,
    Query(q): Query<SignQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let value = service::sign_url(&state, &q.key)?;
    Ok(Json(ApiResponse::ok(value)))
}

/// DELETE /files?key= - delete an object, provider login required.
#[utoipa::path(
    delete,
    path = "/files",
    params(DeleteQuery),
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 403, description = "Forbidden - login required"),
        (status = 500, description = "COS not configured"),
    ),
    tag = "files",
)]
pub async fn delete_file(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(q): Query<DeleteQuery>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    service::delete(&state, &q.key).await?;
    Ok(Json(ApiResponse::ok(())))
}
