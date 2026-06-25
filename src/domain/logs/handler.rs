//! Logs handlers — dev-mode live log viewer.

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use axum::Json;

use crate::app_state::AppState;
use crate::error::AppError;

use super::dto::SinceQuery;
use super::service;

/// GET /logs — the HTML viewer page (dev mode only).
#[utoipa::path(
    get,
    path = "/logs",
    responses(
        (status = 200, description = "HTML log viewer page"),
        (status = 404, description = "Not available in production"),
    ),
    tag = "logs",
)]
pub async fn log_page(State(state): State<AppState>) -> Result<Response, AppError> {
    if !service::is_dev(&state.settings) {
        return Ok(StatusCode::NOT_FOUND.into_response());
    }
    Ok(Html(service::LOG_HTML).into_response())
}

/// GET /logs/api — long-poll new entries since `since` index (dev mode only).
#[utoipa::path(
    get,
    path = "/logs/api",
    params(("since" = Option<usize>, Query, description = "Index to poll from")),
    responses(
        (status = 200, body = serde_json::Value),
        (status = 404, description = "Not available in production"),
    ),
    tag = "logs",
)]
pub async fn log_api(
    State(state): State<AppState>,
    Query(q): Query<SinceQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    if !service::is_dev(&state.settings) {
        return Err(AppError::NotFound("不在 dev 模式".into()));
    }
    let since = q.since.unwrap_or(0);
    let value = service::poll(&state, since).await?;
    Ok(Json(value))
}
