// Scaffolding: foundation types are built ahead of their first consumers.
// TODO(M7): remove this `allow` and enforce `cargo clippy -- -D warnings`.
#![allow(dead_code)]

mod app_state;
mod config;
mod domain;
mod entity;
mod error;
mod external;
mod migration;
mod middleware;
mod response;

use std::sync::Arc;

use app_state::AppState;
use axum::routing::get;
use axum::Router;
use migration::MigratorTrait;
use sea_orm::{ConnectOptions, Database};
use tracing_subscriber::EnvFilter;

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

    // ── JWT secret — injected into request extensions for the AuthUser extractor
    let jwt_secret = settings
        .jwt_secret
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("JWT_SECRET is not set"))?
        .clone();

    let app = Router::new()
        .route("/", get(health))
        .merge(domain::auth::router::routes())
        .merge(domain::user::router::routes())
        .merge(domain::bookings::routes())
        .merge(domain::booking_logs::routes())
        .merge(domain::time_slot_templates::routes())
        .merge(domain::date_slots::routes())
        .merge(domain::date_settings::routes())
        .layer(axum::Extension(middleware::auth::JwtSecret(jwt_secret)))
        .with_state(state);

    let addr = format!("{}:{}", settings.server.host, settings.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("listening on {addr}");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> &'static str {
    "ok"
}