use std::sync::Arc;

use sea_orm::DatabaseConnection;

use crate::config::Settings;

/// Shared application state passed to every handler via `State<AppState>`.
///
/// `cos: CosClient` is added in the files milestone (M5).
#[derive(Clone)]
pub struct AppState {
    pub settings: Arc<Settings>,
    pub db: DatabaseConnection,
}

impl AppState {
    pub fn new(settings: Arc<Settings>, db: DatabaseConnection) -> Self {
        Self { settings, db }
    }
}
