use std::sync::Arc;

use sea_orm::DatabaseConnection;
use tokio::sync::Mutex;

use crate::config::Settings;
use crate::external::cos::CosClient;
use crate::external::qiniu_moderation::QiniuModeration;
use crate::middleware::request_log::{LogBuffer, LogRing};

/// Shared application state passed to every handler via `State<AppState>`.
#[derive(Clone)]
pub struct AppState {
    pub settings: Arc<Settings>,
    pub db: DatabaseConnection,
    pub cos_client: Option<CosClient>,
    pub moderation: Option<QiniuModeration>,
    pub log_buffer: Option<LogBuffer>,
}

impl AppState {
    pub fn new(settings: Arc<Settings>, db: DatabaseConnection) -> Self {
        let cos_client = CosClient::from_settings(&settings);
        let moderation =
            if settings.qiniu_access_key.is_some() && settings.qiniu_secret_key.is_some() {
                Some(QiniuModeration::new(
                    settings.qiniu_access_key.clone(),
                    settings.qiniu_secret_key.clone(),
                ))
            } else {
                None
            };

        // Log ring buffer for dev mode (500 entries)
        let log_buffer = if matches!(settings.env.as_str(), "development" | "dev") {
            Some(Arc::new(Mutex::new(LogRing::new(500))))
        } else {
            None
        };

        Self {
            settings,
            db,
            cos_client,
            moderation,
            log_buffer,
        }
    }
}
