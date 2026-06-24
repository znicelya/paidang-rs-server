// Scaffolding: foundation types are built ahead of their first consumers.
// TODO(M7): remove this `allow` and enforce `cargo clippy -- -D warnings`.
#![allow(dead_code)]

mod app_state;
mod config;
mod error;
mod response;

use std::sync::Arc;

use app_state::AppState;
use axum::routing::get;
use axum::Router;
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

    let state = AppState::new(settings.clone());

    let app = Router::new().route("/", get(health)).with_state(state);

    let addr = format!("{}:{}", settings.server.host, settings.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("listening on {addr}");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> &'static str {
    "ok"
}
