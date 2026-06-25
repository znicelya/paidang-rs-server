//! Files router — upload / list / download (proxy) / delete via COS.

use utoipa_axum::routes;
use utoipa_axum::router::OpenApiRouter;

use crate::app_state::AppState;

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(super::handler::upload, super::handler::list, super::handler::delete_file))
        .routes(routes!(super::handler::download))
}
