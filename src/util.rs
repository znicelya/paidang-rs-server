//! Shared authorization guards.
//!
//! A valid JWT represents a logged-in provider. The database `user.role` column
//! is retained for compatibility but is never used for authorization.

use crate::error::AppError;
use crate::middleware::auth::AuthUser;

/// Require that the logged-in provider owns the photographer-scoped resource.
pub fn require_owner(auth: &AuthUser, photographer_id: i32) -> Result<(), AppError> {
    if auth.user_id == photographer_id {
        Ok(())
    } else {
        Err(AppError::Forbidden(
            "forbidden photographer resource".into(),
        ))
    }
}
