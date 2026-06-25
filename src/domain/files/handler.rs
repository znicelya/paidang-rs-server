//! Files handlers — upload / list / download (proxy) / delete via COS.

use axum::body::Body;
use axum::extract::{Multipart, Path, Query, State};
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use axum::Json;

use crate::app_state::AppState;
use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use crate::response::ApiResponse;

use super::dto::{DeleteQuery, ListQuery};
use super::service;

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
pub async fn upload(
    State(state): State<AppState>,
    _auth: AuthUser,
    mut multipart: Multipart,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
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
                file_name = field.file_name().unwrap_or("unnamed").to_string();
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

    let data = file_data.ok_or_else(|| AppError::InputValidation("未提供文件".into()))?;
    if data.is_empty() {
        return Err(AppError::InputValidation("文件为空".into()));
    }

    let value = service::upload(&state, data, &file_name, &content_type, &prefix).await?;
    Ok(Json(ApiResponse::ok(value)))
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
pub async fn list(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(q): Query<ListQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let prefix = q.prefix.unwrap_or_else(|| "files/".to_string());
    let value = service::list(&state, &prefix).await?;
    Ok(Json(ApiResponse::ok(value)))
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
pub async fn delete_file(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<DeleteQuery>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    if auth.role < 2 {
        return Err(AppError::Forbidden("需要管理员权限".into()));
    }
    service::delete(&state, &q.key).await?;
    Ok(Json(ApiResponse::ok(())))
}
