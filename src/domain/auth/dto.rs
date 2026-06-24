//! DTOs for the auth domain — request validation via `validator`.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

/// POST /auth/login request body.
/// Mirrors `paidang-worker-server/src/endpoints/auth/login.ts` schema.
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LoginReq {
    #[validate(length(min = 1))]
    pub code: String,
    pub nickname: Option<String>,
    pub avatar_url: Option<String>,
    pub phone: Option<String>,
    pub phone_code: Option<String>,
}

/// POST body for delegating phone-code lookup.
#[derive(Debug, Deserialize, ToSchema)]
pub struct PhoneCodeReq {
    pub code: String,
}

/// Login response data (placed inside ApiResponse::ok).
#[derive(Debug, Serialize, ToSchema)]
pub struct LoginResp {
    pub user_id: i32,
    pub openid: String,
    pub role: i8,
    pub phone: Option<String>,
    /// `is_new` retained for mini-program backward compat.
    pub is_new: bool,
    /// JWT token — the new auth credential.
    pub token: String,
}

/// Backward-compat alias.
pub type LoginData = LoginResp;
/// Backward-compat alias.
pub type LoginRequest = LoginReq;