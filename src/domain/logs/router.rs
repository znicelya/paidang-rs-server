//! Logs router — dev-mode live log viewer.

use utoipa_axum::routes;
use utoipa_axum::router::OpenApiRouter;

use crate::app_state::AppState;

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(super::handler::log_page))
        .routes(routes!(super::handler::log_api))
}
