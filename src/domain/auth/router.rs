//! Auth router — `POST /auth/login`.

use axum::routing::post;
use axum::Router;

use super::login_handler::login_handler;
use crate::app_state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/auth/login", post(login_handler))
}