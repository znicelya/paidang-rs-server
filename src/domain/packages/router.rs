//! Packages router — read public, write admin.

use utoipa_axum::routes;
use utoipa_axum::router::OpenApiRouter;

use crate::app_state::AppState;

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(super::handler::list, super::handler::create))
        .routes(routes!(super::handler::read, super::handler::update, super::handler::delete_one))
        .routes(routes!(super::handler::list_items, super::handler::create_item))
        .routes(routes!(super::handler::update_item, super::handler::delete_item))
        .routes(routes!(super::handler::list_gallery, super::handler::create_gallery))
        .routes(routes!(super::handler::update_gallery, super::handler::delete_gallery))
}
