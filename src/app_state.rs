use std::sync::Arc;

use crate::config::Settings;

/// Shared application state passed to every handler via `State<AppState>`.
///
/// Starts with just `settings`; gains `db: DatabaseConnection` in the migrations
/// milestone and `cos: CosClient` in the files milestone.
#[derive(Clone)]
pub struct AppState {
    pub settings: Arc<Settings>,
}

impl AppState {
    pub fn new(settings: Arc<Settings>) -> Self {
        Self { settings }
    }
}
