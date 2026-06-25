//! paidang-rs-server binary entrypoint.
//! Module declarations are in `lib.rs` so that integration tests can import the crate.

use std::sync::Arc;

use migration::MigratorTrait;
use paidang_rs_server::app_state::AppState;
use paidang_rs_server::{config, domain, middleware, migration};
use sea_orm::{ConnectOptions, Database};
use tracing_subscriber::EnvFilter;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let settings = Arc::new(config::Settings::load()?);
    tracing::info!(env = %settings.env, "starting paidang-rs-server");

    // ── Database ──────────────────────────────────────────────
    let db_url = settings
        .database_url
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("DATABASE_URL is not set"))?;
    let mut opt = ConnectOptions::new(db_url.clone());
    opt.max_connections(settings.database.pool_size);
    let db = Database::connect(opt).await?;
    tracing::info!("database connected");

    // Apply migrations (UTC+8 MySQL, no FK, no triggers).
    migration::Migrator::up(&db, None).await?;
    tracing::info!("migrations applied");

    let state = AppState::new(settings.clone(), db);

    // ── Log ring buffer for dev mode /logs endpoint
    let log_buffer = state.log_buffer.clone();

    // ── JWT secret — injected into request extensions for the AuthUser extractor
    let jwt_secret = settings
        .jwt_secret
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("JWT_SECRET is not set"))?
        .clone();

    // Health handler with OpenAPI annotation
    #[utoipa::path(get, path = "/", responses((status = OK, body = str)), tag = "health")]
    async fn health() -> &'static str {
        "ok"
    }

    // Build the root OpenApiRouter, merge ALL domain routers BEFORE split_for_parts
    // so that #[utoipa::path] annotations from every handler are collected into the OpenAPI spec.
    let (router, openapi) = OpenApiRouter::new()
        .routes(routes!(health))
        .merge(domain::auth::router::routes())
        .merge(domain::user::router::routes())
        .merge(domain::bookings::routes())
        .merge(domain::booking_logs::routes())
        .merge(domain::time_slot_templates::routes())
        .merge(domain::date_slots::routes())
        .merge(domain::date_settings::routes())
        .merge(domain::packages::routes())
        .merge(domain::gallery_groups::routes())
        .merge(domain::gallery::routes())
        .merge(domain::files::routes())
        .merge(domain::logs::routes())
        .split_for_parts();

    let mut app = router
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", openapi))
        .layer(axum::Extension(middleware::auth::JwtSecret(jwt_secret)));

    // Inject log buffer into request extensions for the request_log middleware
    if let Some(buf) = log_buffer {
        app = app.layer(axum::Extension(buf));
    }

    let app = app
        .layer(axum::middleware::from_fn(
            middleware::request_log::request_logger,
        ))
        .with_state(state);

    let addr = format!("{}:{}", settings.server.host, settings.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("listening on {addr}");
    axum::serve(listener, app).await?;
    Ok(())
}
