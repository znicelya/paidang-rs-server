use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use serde_json::json;

/// Error codes — compatible with the existing TS backend where applicable,
/// plus two new codes for JWT auth (spec §5.2).
pub const CODE_INTERNAL: u16 = 7000;
pub const CODE_VALIDATION: u16 = 7001;
pub const CODE_NOT_FOUND: u16 = 7002;
pub const CODE_UNAUTHORIZED: u16 = 7003;
pub const CODE_FORBIDDEN: u16 = 7004;

/// Serializable error body for OpenAPI schema.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ErrorBody {
    pub success: bool,
    pub errors: Vec<ErrorItem>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ErrorItem {
    pub code: u16,
    pub message: String,
}

/// Application error type. Every variant renders into the existing failure
/// envelope: `{ "success": false, "errors": [{ "code", "message" }] }`.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    InputValidation(String),

    #[error("{0}")]
    NotFound(String),

    #[error("{0}")]
    Unauthorized(String),

    #[error("{0}")]
    Forbidden(String),

    /// External service failure (WeChat / Qiniu / COS). Maps to 500/7000.
    #[error("{0}")]
    External(String),

    #[error("{0}")]
    Internal(String),
}

impl AppError {
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }

    /// Map a SeaORM DB error, translating known constraint violations into
    /// user-facing validation messages instead of leaking raw SQL errors.
    pub fn from_db(e: sea_orm::DbErr) -> Self {
        let s = e.to_string();
        if s.contains("1062") || s.contains("Duplicate entry") {
            if s.contains("uq_dateslot") {
                return Self::InputValidation("该日期的此时间段已存在，请勿重复添加".into());
            }
            return Self::InputValidation("记录已存在，请勿重复添加".into());
        }
        Self::Internal(format!("DB:{s}"))
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            AppError::InputValidation(m) => (StatusCode::BAD_REQUEST, CODE_VALIDATION, m.clone()),
            AppError::NotFound(m) => (StatusCode::NOT_FOUND, CODE_NOT_FOUND, m.clone()),
            AppError::Unauthorized(m) => (StatusCode::UNAUTHORIZED, CODE_UNAUTHORIZED, m.clone()),
            AppError::Forbidden(m) => (StatusCode::FORBIDDEN, CODE_FORBIDDEN, m.clone()),
            AppError::External(m) => (StatusCode::INTERNAL_SERVER_ERROR, CODE_INTERNAL, m.clone()),
            AppError::Internal(m) => (StatusCode::INTERNAL_SERVER_ERROR, CODE_INTERNAL, m.clone()),
        };

        let body = json!({
            "success": false,
            "errors": [{ "code": code, "message": message }]
        });

        (status, Json(body)).into_response()
    }
}

/// Convenience alias used by handlers.
pub type AppResult<T> = Result<T, AppError>;
