//! User router — profile read/update + avatar upload (JWT-protected).

use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::app_state::AppState;

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(
        crate::domain::user::handler::get_profile,
        crate::domain::user::handler::update_profile,
        crate::domain::user::handler::upload_avatar
    ))
}
