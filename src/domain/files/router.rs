//! Files domain — upload / list / download (proxy) / delete.
//!
//! File metadata is stored in existing entity columns (avatar_url, cover_image, image_url…)
//! rather than in a dedicated `file` table. This module handles COS operations:
//! upload to COS after moderation, proxy-download from COS, list/delete in COS bucket.

use axum::body::Body;
use axum::extract::{Multipart, Path, Query, State};
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use utoipa::ToSchema;
use utoipa_axum::routes;
use utoipa_axum::router::OpenApiRouter;

use crate::app_state::AppState;
use crate::error::AppError;
use crate::external::qiniu_moderation;
use crate::middleware::auth::AuthUser;
use crate::response::ApiResponse;

// ── DTOs ─────────────────────────────────────────────

#[derive(Debug, Deserialize, utoipa::IntoParams, ToSchema)]
pub struct ListQuery {
    pub prefix: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams, ToSchema)]
pub struct DeleteQuery {
    pub key: String,
}

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(upload, list, delete_file))
        .routes(routes!(download))
}

// ── Handlers ──────────────────────────────────────────

/// POST /files — multipart upload to COS, with optional Qiniu moderation.
///
/// Fields:
/// - `file` (required): the binary file
/// - `prefix` (optional): storage path prefix, e.g. "gallery/" or "avatars/"
#[utoipa::path(
    post,
    path = "/files",
    request_body(content_type = "multipart/form-data", description = "Multipart form with file field"),
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 400, description = "No file provided"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "COS not configured"),
    ),
    tag = "files",
)]
async fn upload(
    State(state): State<AppState>,
    _auth: AuthUser,
    mut multipart: Multipart,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let cos_client = state.cos_client.as_ref().ok_or_else(|| {
        AppError::Internal("COS not configured".into())
    })?;
    let moderation = state
        .moderation
        .as_ref()
        .cloned()
        .unwrap_or_else(|| qiniu_moderation::QiniuModeration::new(None, None));

    let mut file_data: Option<Vec<u8>> = None;
    let mut file_name = String::new();
    let mut content_type = String::new();
    let mut prefix = String::from("files/");

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::InputValidation(format!("multipart: {e}")))?
    {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "file" => {
                file_name = field
                    .file_name()
                    .unwrap_or("unnamed")
                    .to_string();
                content_type = field
                    .content_type()
                    .unwrap_or("application/octet-stream")
                    .to_string();
                file_data = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|e| AppError::InputValidation(format!("read: {e}")))?
                        .to_vec(),
                );
            }
            "prefix" => {
                let p = field
                    .text()
                    .await
                    .map_err(|e| AppError::InputValidation(format!("prefix: {e}")))?;
                prefix = p.trim_end_matches('/').to_string() + "/";
            }
            _ => {}
        }
    }

    let data = file_data.ok_or_else(|| {
        AppError::InputValidation("未提供文件".into())
    })?;

    if data.is_empty() {
        return Err(AppError::InputValidation("文件为空".into()));
    }

    // ── Moderation (images only) ───────────────────────
    if qiniu_moderation::QiniuModeration::should_moderate(&content_type)
        && moderation.is_configured()
    {
        let b64 = {
            use base64::Engine;
            base64::engine::general_purpose::STANDARD.encode(&data)
        };
        let result = moderation.moderate(&b64, &content_type).await?;
        if let qiniu_moderation::ModerationResult::Block(reason) = result {
            return Ok(Json(ApiResponse::ok(serde_json::json!({
                "blocked": true,
                "reason": reason,
            }))));
        }
    }

    // ── Upload to COS ──────────────────────────────────
    let key = format!("{prefix}{file_name}");
    let url = cos_client.put_object(&key, data, &content_type).await?;

    Ok(Json(ApiResponse::ok(serde_json::json!({
        "key": key,
        "url": url,
    }))))
}

/// GET /files/*path — proxy download from COS.
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
async fn download(
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let cos_client = state.cos_client.as_ref().ok_or_else(|| {
        AppError::Internal("COS not configured".into())
    })?;

    let (body, content_type) = cos_client.get_object(&path).await?;

    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, content_type),
            (
                header::CONTENT_DISPOSITION,
                format!("inline; filename=\"{}\"", path.rsplit('/').next().unwrap_or(&path)),
            ),
        ],
        Body::from(body),
    ))
}

/// GET /files — list objects by prefix.
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
async fn list(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(q): Query<ListQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let cos_client = state.cos_client.as_ref().ok_or_else(|| {
        AppError::Internal("COS not configured".into())
    })?;

    let prefix = q.prefix.unwrap_or_else(|| "files/".to_string());
    let keys = cos_client.list_objects(&prefix).await?;

    Ok(Json(ApiResponse::ok(serde_json::json!({
        "prefix": prefix,
        "keys": keys,
    }))))
}

/// DELETE /files?key= — delete an object (admin only).
#[utoipa::path(
    delete,
    path = "/files",
    params(DeleteQuery),
    responses(
        (status = 200, body = ApiResponse<serde_json::Value>),
        (status = 403, description = "Forbidden — admin only"),
        (status = 500, description = "COS not configured"),
    ),
    tag = "files",
)]
async fn delete_file(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<DeleteQuery>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    // Only admins can delete files
    if auth.role < 2 {
        return Err(AppError::Forbidden("需要管理员权限".into()));
    }
    let cos_client = state.cos_client.as_ref().ok_or_else(|| {
        AppError::Internal("COS not configured".into())
    })?;
    cos_client.delete_object(&q.key).await?;
    Ok(Json(ApiResponse::ok(())))
}
