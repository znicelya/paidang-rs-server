//! DTOs for the auth domain — request validation via `validator`.

use serde::{Deserialize, Serialize};
use validator::Validate;

/// POST /auth/login request body.
/// Mirrors `paidang-worker-server/src/endpoints/auth/login.ts` schema.
#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(length(min = 1))]
    pub code: String,
    pub nickname: Option<String>,
    pub avatar_url: Option<String>,
    pub phone: Option<String>,
    pub phone_code: Option<String>,
}

/// Login response data (placed inside ApiResponse::ok).
#[derive(Debug, Serialize)]
pub struct LoginData {
    pub user_id: i32,
    pub openid: String,
    pub role: i8,
    pub phone: Option<String>,
    /// `is_new` retained for mini-program backward compat.
    pub is_new: bool,
    /// JWT token — the new auth credential.
    pub token: String,
}