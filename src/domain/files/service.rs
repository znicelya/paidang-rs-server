//! Files service — COS upload/download/list/delete + Qiniu moderation.

use crate::app_state::AppState;
use crate::domain::files::dto::{ModerateUploadRequest, UploadPolicyRequest};
use crate::error::AppError;
use crate::external::qiniu_moderation;

/// Upload bytes to COS under `prefix/file_name`, with optional image moderation.
/// Returns the JSON payload (`{ blocked?, key, url }`) to be wrapped in `ApiResponse`.
pub async fn upload(
    state: &AppState,
    data: Vec<u8>,
    file_name: &str,
    content_type: &str,
    prefix: &str,
) -> Result<serde_json::Value, AppError> {
    let cos_client = state
        .cos_client
        .as_ref()
        .ok_or_else(|| AppError::Internal("COS not configured".into()))?;
    let moderation = state
        .moderation
        .as_ref()
        .cloned()
        .unwrap_or_else(|| qiniu_moderation::QiniuModeration::new(None, None));

    // Moderation (images only)
    if qiniu_moderation::QiniuModeration::should_moderate(content_type)
        && moderation.is_configured()
    {
        let b64 = {
            use base64::Engine;
            base64::engine::general_purpose::STANDARD.encode(&data)
        };
        let result = moderation.moderate(&b64, content_type).await?;
        if let qiniu_moderation::ModerationResult::Block(reason) = result {
            return Ok(serde_json::json!({ "blocked": true, "reason": reason }));
        }
    }

    let key = format!("{prefix}{file_name}");
    cos_client.put_object(&key, data, content_type).await?;
    let url = cos_client.signed_get_url(&key, 24 * 60 * 60);

    Ok(serde_json::json!({ "key": key, "path": format!("/files/{key}"), "url": url }))
}

pub fn upload_policy(
    state: &AppState,
    user_id: i32,
    req: UploadPolicyRequest,
) -> Result<serde_json::Value, AppError> {
    let cos_client = state
        .cos_client
        .as_ref()
        .ok_or_else(|| AppError::Internal("COS not configured".into()))?;
    let prefix = normalize_prefix(req.prefix.as_deref().unwrap_or("files"));
    let file_name = storage_file_name(user_id, &req.file_name);
    let key = format!("{prefix}{file_name}");
    Ok(cos_client.post_upload_policy(&key, 10 * 60))
}

pub async fn moderate_uploaded_object(
    state: &AppState,
    req: ModerateUploadRequest,
) -> Result<serde_json::Value, AppError> {
    let cos_client = state
        .cos_client
        .as_ref()
        .ok_or_else(|| AppError::Internal("COS not configured".into()))?;
    let clean_key = normalize_key(&req.key);
    let content_type = req
        .content_type
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| guess_content_type(&clean_key));

    if !qiniu_moderation::QiniuModeration::should_moderate(&content_type) {
        let url = cos_client.signed_get_url(&clean_key, 24 * 60 * 60);
        return Ok(serde_json::json!({
            "key": clean_key,
            "path": format!("/files/{clean_key}"),
            "url": url,
        }));
    }

    let moderation = state
        .moderation
        .as_ref()
        .cloned()
        .unwrap_or_else(|| qiniu_moderation::QiniuModeration::new(None, None));
    let url = cos_client.signed_get_url(&clean_key, 10 * 60);

    if moderation.is_configured() {
        let result = moderation.moderate_uri(&url).await?;
        if let qiniu_moderation::ModerationResult::Block(reason) = result {
            let _ = cos_client.delete_object(&clean_key).await;
            return Ok(serde_json::json!({
                "blocked": true,
                "reason": reason,
                "key": clean_key,
                "path": format!("/files/{clean_key}"),
            }));
        }
    }

    Ok(serde_json::json!({
        "key": clean_key,
        "path": format!("/files/{clean_key}"),
        "url": cos_client.signed_get_url(&clean_key, 24 * 60 * 60),
    }))
}

/// Create a temporary signed COS URL. The client can load the image directly
/// from COS without proxying bytes through this service.
pub fn sign_url(state: &AppState, key: &str) -> Result<serde_json::Value, AppError> {
    let cos_client = state
        .cos_client
        .as_ref()
        .ok_or_else(|| AppError::Internal("COS not configured".into()))?;
    let clean_key = normalize_key(key);
    let url = cos_client.signed_get_url(&clean_key, 24 * 60 * 60);
    Ok(serde_json::json!({ "key": clean_key, "url": url }))
}

fn normalize_key(value: &str) -> String {
    let mut key = value.trim().to_string();
    if let Some(pos) = key.find("/files/") {
        key = key[(pos + "/files/".len())..].to_string();
    }
    if let Some(pos) = key.find('?') {
        key.truncate(pos);
    }
    if let Some(pos) = key.find('#') {
        key.truncate(pos);
    }
    key.trim_start_matches('/').to_string()
}

fn normalize_prefix(value: &str) -> String {
    let clean = value.trim().trim_matches('/');
    if clean.is_empty() {
        "files/".to_string()
    } else {
        format!("{clean}/")
    }
}

fn storage_file_name(user_id: i32, original_name: &str) -> String {
    let base_name = original_name
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or("upload.jpg");
    let ext = base_name
        .rsplit_once('.')
        .map(|(_, ext)| ext)
        .filter(|ext| {
            !ext.is_empty() && ext.len() <= 10 && ext.chars().all(|c| c.is_ascii_alphanumeric())
        })
        .unwrap_or("jpg")
        .to_ascii_lowercase();
    let millis = chrono::Utc::now().timestamp_millis();
    format!("{user_id}_{millis}.{ext}")
}

fn guess_content_type(key: &str) -> String {
    let lower = key.to_ascii_lowercase();
    if lower.ends_with(".png") {
        "image/png"
    } else if lower.ends_with(".webp") {
        "image/webp"
    } else if lower.ends_with(".gif") {
        "image/gif"
    } else if lower.ends_with(".heic") {
        "image/heic"
    } else if lower.ends_with(".svg") || lower.ends_with(".svgz") {
        "image/svg+xml"
    } else {
        "image/jpeg"
    }
    .to_string()
}

/// Proxy-download an object from COS. Returns `(body, content_type)`.
pub async fn download(state: &AppState, path: &str) -> Result<(Vec<u8>, String), AppError> {
    let cos_client = state
        .cos_client
        .as_ref()
        .ok_or_else(|| AppError::Internal("COS not configured".into()))?;
    cos_client.get_object(path).await
}

/// List objects by prefix. Returns the JSON payload `{ prefix, keys }`.
pub async fn list(state: &AppState, prefix: &str) -> Result<serde_json::Value, AppError> {
    let cos_client = state
        .cos_client
        .as_ref()
        .ok_or_else(|| AppError::Internal("COS not configured".into()))?;
    let keys = cos_client.list_objects(prefix).await?;
    Ok(serde_json::json!({ "prefix": prefix, "keys": keys }))
}

/// Delete an object by key.
pub async fn delete(state: &AppState, key: &str) -> Result<(), AppError> {
    let cos_client = state
        .cos_client
        .as_ref()
        .ok_or_else(|| AppError::Internal("COS not configured".into()))?;
    cos_client.delete_object(key).await
}
