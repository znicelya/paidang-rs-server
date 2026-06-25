//! Date slots router.

use utoipa_axum::routes;
use utoipa_axum::router::OpenApiRouter;

use crate::app_state::AppState;

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(super::handler::list, super::handler::create))
        .routes(routes!(super::handler::read, super::handler::update, super::handler::delete_one))
        .routes(routes!(super::handler::day))
        .routes(routes!(super::handler::monthly))
}
