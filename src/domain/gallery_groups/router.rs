//! Gallery groups router.

use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::app_state::AppState;

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(super::handler::list, super::handler::create))
        .routes(routes!(
            super::handler::read,
            super::handler::update,
            super::handler::delete_one
        ))
}
