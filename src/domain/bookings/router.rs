//! Bookings router — JWT-protected, ownership scoped.

use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::app_state::AppState;

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(super::handler::list, super::handler::create))
        .routes(routes!(
            super::handler::read,
            super::handler::update,
            super::handler::delete_booking
        ))
        .routes(routes!(super::handler::stats))
        .routes(routes!(super::handler::today))
}
