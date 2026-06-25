//! User router — profile read/update + avatar upload (JWT-protected).

use utoipa_axum::routes;
use utoipa_axum::router::OpenApiRouter;

use crate::app_state::AppState;

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(
        crate::domain::user::handlers::get_profile,
        crate::domain::user::handlers::update_profile,
        crate::domain::user::handlers::upload_avatar
    ))
}