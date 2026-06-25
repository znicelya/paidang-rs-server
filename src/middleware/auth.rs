//! JWT authentication middleware.
//!
//! Extracts `Authorization: Bearer <token>`, verifies it, and injects
//! `AuthUser { user_id, openid }` into the request extensions.
//! On failure returns 7003/401 per spec 5.2.

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use crate::error::AppError;

/// JWT claims. `sub` = user_id. A valid token represents a logged-in provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32,
    pub openid: String,
    pub exp: u64,
}

/// The authenticated provider injected into request extensions.
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: i32,
    pub openid: String,
}

/// Sign a new JWT. Called from the login handler.
pub fn sign_jwt(claims: Claims, secret: &str) -> Result<String, AppError> {
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("JWT sign error: {e}")))
}

/// Verify and decode a JWT string.
pub fn verify_jwt(token: &str, secret: &str) -> Result<Claims, AppError> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| AppError::Unauthorized(format!("token invalid: {e}")))?;
    Ok(data.claims)
}

/// Wrapper to stash the JWT secret in request extensions.
/// Set once per app startup, read by the AuthUser extractor.
#[derive(Clone)]
pub struct JwtSecret(pub String);

// Axum extractor: AuthUser
// Using FromRequestParts so it can coexist with body extractors.

/// Error response when JWT extraction fails.
pub struct AuthRejection(AppError);

impl IntoResponse for AuthRejection {
    fn into_response(self) -> Response {
        self.0.into_response()
    }
}

impl<S: Send + Sync> FromRequestParts<S> for AuthUser {
    type Rejection = AuthRejection;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| {
                AuthRejection(AppError::Unauthorized(
                    "missing Authorization header".into(),
                ))
            })?;

        let token = auth_header.strip_prefix("Bearer ").ok_or_else(|| {
            AuthRejection(AppError::Unauthorized(
                "invalid Authorization header format".into(),
            ))
        })?;

        let secret = parts
            .extensions
            .get::<JwtSecret>()
            .ok_or_else(|| AuthRejection(AppError::Internal("JWT secret not configured".into())))?
            .0
            .clone();

        let claims = verify_jwt(token, &secret).map_err(AuthRejection)?;

        Ok(AuthUser {
            user_id: claims.sub,
            openid: claims.openid,
        })
    }
}
