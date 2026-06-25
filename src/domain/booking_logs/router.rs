//! Booking logs router — read-only list. JWT-protected.

use utoipa_axum::routes;
use utoipa_axum::router::OpenApiRouter;

use crate::app_state::AppState;

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(super::handler::list))
        .routes(routes!(super::handler::read))
}
