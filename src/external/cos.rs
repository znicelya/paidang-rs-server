//! Tencent Cloud COS client via reqwest + CAM V4 signing.
//!
//! Uses HMAC-SHA1 signing (compatible with COS XML API).
//! In production, prefer `qcos` crate — this is the fallback per spec §7.3.

use crate::error::AppError;
use reqwest::Client;
use std::sync::Arc;

/// COS client configuration.
pub struct CosConfig {
    pub secret_id: String,
    pub secret_key: String,
    pub bucket: String,
    pub region: String,
}

/// Client for Tencent Cloud Object Storage XML API.
#[derive(Clone)]
pub struct CosClient {
    pub config: Arc<CosConfig>,
    http: Client,
}

impl CosClient {
    pub fn new(config: CosConfig) -> Self {
        let http = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("reqwest::Client::new");
        Self {
            config: Arc::new(config),
            http,
        }
    }

    /// Build from app settings. Returns `None` when COS is not configured.
    pub fn from_settings(settings: &crate::config::Settings) -> Option<Self> {
        let secret_id = settings.cos_secret_id.as_ref()?;
        let secret_key = settings.cos_secret_key.as_ref()?;
        let bucket = settings.cos_bucket.as_ref()?;
        let region = settings.cos_region.as_ref()?;
        Some(Self::new(CosConfig {
            secret_id: secret_id.clone(),
            secret_key: secret_key.clone(),
            bucket: bucket.clone(),
            region: region.clone(),
        }))
    }

    fn host(&self) -> String {
        format!(
            "{}.cos.{}.myqcloud.com",
            self.config.bucket, self.config.region
        )
    }

    fn object_url(&self, key: &str) -> String {
        format!("https://{}/{}", self.host(), key.trim_start_matches('/'))
    }

    /// PUT an object. `content_type` should be inferred from the file.
    /// Returns the public URL on success.
    pub async fn put_object(
        &self,
        key: &str,
        body: Vec<u8>,
        content_type: &str,
    ) -> Result<String, AppError> {
        let url = self.object_url(key);
        let now = chrono::Utc::now();
        let date_str = now.format("%a, %d %b %Y %H:%M:%S GMT").to_string();

        // Build signature
        let string_to_sign = format!(
            "put\n\n{}\n{}\n/{}",
            content_type,
            date_str,
            self.resource_path(key)
        );
        let signature = self.sign(&string_to_sign);

        let resp = self
            .http
            .put(&url)
            .header("Host", self.host())
            .header("Date", &date_str)
            .header("Content-Type", content_type)
            .header(
                "Authorization",
                format!(
                    "q-sign-algorithm=sha1&q-ak={}&q-sign-time={}&q-key-time={}&q-header-list=host;date;content-type&q-url-param-list=&q-signature={}",
                    self.config.secret_id,
                    now.timestamp(),
                    now.timestamp() + 3600,
                    signature,
                ),
            )
            .body(body)
            .send()
            .await
            .map_err(|e| AppError::External(format!("COS put: {e}")))?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(AppError::External(format!(
                "COS {status}: {text}"
            )));
        }
        Ok(url)
    }

    /// GET an object's bytes.
    pub async fn get_object(&self, key: &str) -> Result<(Vec<u8>, String), AppError> {
        let url = self.object_url(key);
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::External(format!("COS get: {e}")))?;
        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(AppError::External(format!("COS {status}: {text}")));
        }
        let ct = resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_string();
        let body = resp
            .bytes()
            .await
            .map_err(|e| AppError::External(format!("COS read: {e}")))?
            .to_vec();
        Ok((body, ct))
    }

    /// HEAD object — returns content-type and content-length.
    pub async fn head_object(&self, key: &str) -> Result<(String, u64), AppError> {
        let url = self.object_url(key);
        let resp = self
            .http
            .head(&url)
            .send()
            .await
            .map_err(|e| AppError::External(format!("COS head: {e}")))?;
        let status = resp.status();
        if !status.is_success() {
            return Err(AppError::External(format!("COS head {status}")));
        }
        let ct = resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_string();
        let len = resp
            .headers()
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        Ok((ct, len))
    }

    /// DELETE an object.
    pub async fn delete_object(&self, key: &str) -> Result<(), AppError> {
        let url = self.object_url(key);
        let resp = self
            .http
            .delete(&url)
            .send()
            .await
            .map_err(|e| AppError::External(format!("COS delete: {e}")))?;
        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(AppError::External(format!("COS {status}: {text}")));
        }
        Ok(())
    }

    /// List objects with a prefix (max 1000).
    pub async fn list_objects(
        &self,
        prefix: &str,
    ) -> Result<Vec<String>, AppError> {
        let url = format!(
            "https://{}/?prefix={}&max-keys=1000",
            self.host(),
            prefix.trim_start_matches('/')
        );
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::External(format!("COS list: {e}")))?;
        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(AppError::External(format!("COS {status}: {text}")));
        }
        let body = resp
            .text()
            .await
            .map_err(|e| AppError::External(format!("COS list body: {e}")))?;
        // Simple XML parse — just extract <Key> elements
        let mut keys = Vec::new();
        for cap in body.split("<Key>").skip(1) {
            if let Some(end) = cap.find("</Key>") {
                keys.push(cap[..end].to_string());
            }
        }
        Ok(keys)
    }

    // ── private helpers ─────────────────────────────────

    fn resource_path(&self, key: &str) -> String {
        format!("/{}", key.trim_start_matches('/'))
    }

    fn sign(&self, input: &str) -> String {
        use base64::Engine;
        use hmac::{Hmac, Mac};
        use sha1::Sha1;
        type HmacSha1 = Hmac<Sha1>;
        let mut mac =
            HmacSha1::new_from_slice(self.config.secret_key.as_bytes())
                .expect("HMAC-SHA1 key");
        mac.update(input.as_bytes());
        let result = mac.finalize();
        base64::engine::general_purpose::STANDARD.encode(result.into_bytes())
    }
}
