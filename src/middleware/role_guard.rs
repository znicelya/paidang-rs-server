//! Role guard extractors: `Admin` (role ≥ 2) and `Photographer` (role ≥ 1).
//!
//! Usage: add `auth: Admin` or `auth: Photographer` as a handler parameter.
//! If the JWT `AuthUser` is already extracted and the role is too low, returns
//! 7004/403. Owner checks (e.g.摄影师 can only touch their own slots) are done
//! in the service layer, not here.

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};

use crate::error::AppError;
use crate::middleware::auth::AuthUser;

/// Extractor: requires `role >= 2` (管理员).
pub struct Admin(pub AuthUser);

/// Extractor: requires `role >= 1` (摄影师 or above).
pub struct Photographer(pub AuthUser);

fn check_role(auth: AuthUser, min: i8, label: &str) -> Result<AuthUser, RoleRejection> {
    if auth.role >= min {
        Ok(auth)
    } else {
        Err(RoleRejection(AppError::Forbidden(format!(
            "需要{label}权限"
        ))))
    }
}

pub struct RoleRejection(AppError);

impl IntoResponse for RoleRejection {
    fn into_response(self) -> Response {
        self.0.into_response()
    }
}

impl<S: Send + Sync> FromRequestParts<S> for Admin {
    type Rejection = RoleRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth = AuthUser::from_request_parts(parts, state)
            .await
            .map_err(|_| RoleRejection(AppError::Unauthorized("未登录".into())))?;
        check_role(auth, 2, "管理员").map(Admin)
    }
}

impl<S: Send + Sync> FromRequestParts<S> for Photographer {
    type Rejection = RoleRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth = AuthUser::from_request_parts(parts, state)
            .await
            .map_err(|_| RoleRejection(AppError::Unauthorized("未登录".into())))?;
        check_role(auth, 1, "摄影师").map(Photographer)
    }
}
