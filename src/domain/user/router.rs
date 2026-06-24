//! User router — profile read/update + avatar upload (JWT-protected).

use axum::routing::{get, post};
use axum::Router;

use super::handlers::{get_profile, update_profile, upload_avatar};
use crate::app_state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/user/profile", get(get_profile).put(update_profile))
        .route("/user/avatar", post(upload_avatar))
}