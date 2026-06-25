//! Dev-mode live log viewer — ports `logStore.ts` / `GET /logs`.
//!
//! Provides:
//! - `GET /logs`       — HTML page with dark-themed log table + long-polling JS
//! - `GET /logs/api`   — long-polling endpoint: `?since=N` returns new entries
//!
//! Activated only in `development` / `dev` environments. Returns `404` in production.

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use utoipa::ToSchema;
use utoipa_axum::routes;
use utoipa_axum::router::OpenApiRouter;

use crate::app_state::AppState;
use crate::error::AppError;

/// Only serve /logs in dev mode.
fn is_dev(settings: &Arc<crate::config::Settings>) -> bool {
    matches!(settings.env.as_str(), "development" | "dev")
}

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(log_page))
        .routes(routes!(log_api))
}

#[derive(Debug, Deserialize, utoipa::IntoParams, ToSchema)]
struct SinceQuery {
    since: Option<usize>,
}

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
async fn log_page(
    State(state): State<AppState>,
) -> Result<Response, AppError> {
    if !is_dev(&state.settings) {
        return Ok(StatusCode::NOT_FOUND.into_response());
    }
    Ok(Html(LOG_HTML).into_response())
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
async fn log_api(
    State(state): State<AppState>,
    Query(q): Query<SinceQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    if !is_dev(&state.settings) {
        return Err(AppError::NotFound("不在 dev 模式".into()));
    }

    let buf = state
        .log_buffer
        .as_ref()
        .ok_or_else(|| AppError::Internal("log buffer not initialized".into()))?;

    let since = q.since.unwrap_or(0);

    // Long-poll: wait up to 30s for new entries
    let start = tokio::time::Instant::now();
    let timeout = tokio::time::Duration::from_secs(30);
    let poll_interval = tokio::time::Duration::from_millis(500);

    loop {
        let entries = {
            let ring = buf.lock().await;
            ring.since(since)
        };

        if !entries.is_empty() || start.elapsed() >= timeout {
            let total = buf.lock().await.entries.len();
            return Ok(Json(serde_json::json!({
                "entries": entries,
                "total": total,
            })));
        }

        tokio::time::sleep(poll_interval).await;
    }
}

// ── HTML template (ported from existing log viewer: dark theme + long-polling JS) ──

const LOG_HTML: &str = r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Realtime Logs — paidang-rs-server</title>
<style>
* { margin:0; padding:0; box-sizing:border-box; }
body { background:#1a1a2e; color:#e0e0e0; font-family:'Fira Code',monospace; padding:16px; }
h1 { color:#00d4aa; margin-bottom:16px; font-size:18px; }
table { width:100%; border-collapse:collapse; font-size:12px; }
th { background:#16213e; padding:8px 12px; text-align:left; color:#00d4aa; position:sticky; top:0; }
td { padding:6px 12px; border-bottom:1px solid #2a2a4a; }
tr:hover { background:#16213e; }
.method-get { color:#4caf50; }
.method-post { color:#ff9800; }
.method-put { color:#2196f3; }
.method-delete { color:#f44336; }
.status-ok { color:#4caf50; }
.status-err { color:#f44336; }
#stats { color:#888; margin-bottom:12px; font-size:12px; }
</style>
</head>
<body>
<h1>📡 Realtime Logs — paidang-rs-server</h1>
<div id="stats">Connecting…</div>
<table>
<thead><tr><th>Time</th><th>Method</th><th>URL</th><th>Status</th><th>Duration</th></tr></thead>
<tbody id="tbody"></tbody>
</table>
<script>
let since = 0;
let total = 0;
const tbody = document.getElementById('tbody');
const stats = document.getElementById('stats');

function methodClass(m) {
    const map = {GET:'method-get',POST:'method-post',PUT:'method-put',DELETE:'method-delete'};
    return map[m] || '';
}
function statusClass(s) {
    return s < 400 ? 'status-ok' : 'status-err';
}

function render(entries) {
    for (const e of entries) {
        const tr = document.createElement('tr');
        tr.innerHTML = `<td>${e.timestamp}</td>
            <td class="${methodClass(e.method)}">${e.method}</td>
            <td>${e.url}</td>
            <td class="${statusClass(e.status)}">${e.status}</td>
            <td>${e.duration_ms}ms</td>`;
        tbody.appendChild(tr);
    }
    since += entries.length;
    total += entries.length;
    stats.textContent = `Entries: ${total} | Auto-refreshing…`;
    // Keep only last 200 rows in DOM
    while (tbody.children.length > 200) tbody.firstChild.remove();
}

async function poll() {
    try {
        const resp = await fetch(`/logs/api?since=${since}`);
        const data = await resp.json();
        if (data.entries && data.entries.length > 0) {
            render(data.entries);
        }
        stats.textContent = `Entries: ${total} | Watching…`;
    } catch(e) {
        stats.textContent = `Error: ${e.message}`;
    }
    setTimeout(poll, 1000);
}

poll();
</script>
</body>
</html>
"#;
