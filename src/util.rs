//! Shared authorization guards.
//!
//! Thin helpers over [`crate::middleware::auth::AuthUser`] that return
//! `AppError::Forbidden` on failure. Role semantics (see `Claims`):
//! 0 = 普通用户, 1 = 摄影师, 2 = 管理员.

use crate::error::AppError;
use crate::middleware::auth::AuthUser;

/// Require an admin (role >= 2).
pub fn require_admin(auth: &AuthUser) -> Result<(), AppError> {
    if auth.role >= 2 {
        Ok(())
    } else {
        Err(AppError::Forbidden("需要管理员权限".into()))
    }
}

/// Require that the caller is the owner of the resource or an admin (role >= 2).
pub fn require_owner(auth: &AuthUser, photographer_id: i32) -> Result<(), AppError> {
    if auth.role >= 2 || auth.user_id == photographer_id {
        Ok(())
    } else {
        Err(AppError::Forbidden("无权操作此资源".into()))
    }
}
