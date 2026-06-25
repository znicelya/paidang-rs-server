//! Auth router — `POST /auth/login`.

use utoipa_axum::{routes, router::OpenApiRouter};

use crate::app_state::AppState;

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(super::handler::login_handler))
}
