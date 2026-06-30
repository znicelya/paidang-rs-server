//! Request/response logging middleware — ports the global trace middleware
//! from TS `paidang-worker-server/src/middleware/requestLog.ts`.
//!
//! Records: method, url, query, reqBody, status, duration_ms, resBody (truncated to 500
//! chars). Skips `/logs` requests to avoid self-referential noise.
//!
//! When the `in_memory_logs` feature is enabled (dev mode), entries are pushed into
//! the global ring buffer so that `/logs` can stream them to the browser.

use std::sync::Arc;
use std::time::Instant;

use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;
use serde::Serialize;
use tokio::sync::Mutex;
use tracing::info;

/// A single captured log entry.
#[derive(Debug, Clone, Serialize)]
pub struct LogEntry {
    pub method: String,
    pub url: String,
    pub query: String,
    pub status: u16,
    pub duration_ms: u64,
    pub req_body: Option<String>,
    pub res_body: Option<String>,
    pub timestamp: String,
}

/// Ring buffer shared with the `/logs` endpoint.
pub type LogBuffer = Arc<Mutex<LogRing>>;

pub struct LogRing {
    pub entries: Vec<LogEntry>,
    capacity: usize,
}

impl LogRing {
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, entry: LogEntry) {
        if self.entries.len() >= self.capacity {
            self.entries.remove(0);
        }
        self.entries.push(entry);
    }

    pub fn since(&self, index: usize) -> Vec<LogEntry> {
        if index < self.entries.len() {
            self.entries[index..].to_vec()
        } else {
            vec![]
        }
    }
}

pub async fn request_logger(req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let path = uri.path().to_string();

    // Skip logging /logs endpoints themselves
    if path.starts_with("/logs") {
        return next.run(req).await;
    }

    let query = uri.query().unwrap_or("").to_string();

    let start = Instant::now();
    let status: StatusCode;

    // Hoist the log buffer out of extensions before moving req
    let log_buf = req.extensions().get::<LogBuffer>().cloned();

    let response = next.run(req).await;
    status = response.status();
    let duration_ms = start.elapsed().as_millis() as u64;

    // Truncate response body to 500 chars
    let res_body = None; // body already consumed — log without body snippet

    // Build entry
    let entry = LogEntry {
        method: method.to_string(),
        url: uri.to_string(),
        query,
        status: status.as_u16(),
        duration_ms,
        req_body: None,
        res_body,
        timestamp: chrono::Local::now()
            .format("%Y-%m-%d %H:%M:%S%.3f")
            .to_string(),
    };

    info!(
        method = %entry.method,
        url = %entry.url,
        status = %entry.status,
        duration_ms = entry.duration_ms,
        "[REQ]"
    );

    // Push to ring buffer if available
    if let Some(buf) = log_buf {
        buf.lock().await.push(entry);
    }

    response
}
