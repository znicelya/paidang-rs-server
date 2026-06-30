//! Tencent Cloud COS client via reqwest + COS XML API V5 signing.
//!
//! In production, prefer `qcos` crate; this fallback signs requests directly
//! for the COS XML API.

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
        format!("https://{}{}", self.host(), self.resource_path(key))
    }

    pub fn signed_get_url(&self, key: &str, ttl_secs: i64) -> String {
        let now = chrono::Utc::now().timestamp();
        self.signed_get_url_at(key, now, ttl_secs)
    }

    pub fn post_upload_policy(&self, key: &str, ttl_secs: i64) -> serde_json::Value {
        let now = chrono::Utc::now().timestamp();
        self.post_upload_policy_at(key, now, ttl_secs)
    }

    fn post_upload_policy_at(&self, key: &str, start: i64, ttl_secs: i64) -> serde_json::Value {
        let end = start + ttl_secs.max(1);
        let sign_time = format!("{start};{end}");
        let key_time = sign_time.clone();
        let expiration = chrono::DateTime::from_timestamp(end, 0)
            .unwrap_or_else(chrono::Utc::now)
            .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let policy_json = serde_json::json!({
            "expiration": expiration,
            "conditions": [
                { "q-sign-algorithm": "sha1" },
                { "q-ak": self.config.secret_id },
                { "q-sign-time": sign_time },
                { "q-key-time": key_time },
                { "key": key },
                { "success_action_status": "200" },
            ],
        });
        let policy_text = serde_json::to_string(&policy_json).expect("serialize COS policy");
        let policy = {
            use base64::Engine;
            base64::engine::general_purpose::STANDARD.encode(policy_text.as_bytes())
        };
        let string_to_sign = sha1_hex(policy.as_bytes());
        let sign_key = hmac_sha1_hex(self.config.secret_key.as_bytes(), &key_time);
        let signature = hmac_sha1_hex(sign_key.as_bytes(), &string_to_sign);
        let signed_url = self.signed_get_url_at(key, start, ttl_secs);

        serde_json::json!({
            "url": format!("https://{}/", self.host()),
            "key": key,
            "path": format!("/files/{key}"),
            "signed_url": signed_url,
            "form_data": {
                "key": key,
                "policy": policy,
                "q-sign-algorithm": "sha1",
                "q-ak": self.config.secret_id,
                "q-key-time": key_time,
                "q-sign-time": sign_time,
                "q-signature": signature,
                "success_action_status": "200",
            },
        })
    }

    fn signed_get_url_at(&self, key: &str, start: i64, ttl_secs: i64) -> String {
        let end = start + ttl_secs.max(1);
        let sign_time = format!("{start};{end}");
        let key_time = sign_time.clone();
        let host = self.host();
        let header_list = "host";
        let url_param_list = "";
        let http_string = format!(
            "get\n{}\n\nhost={}\n",
            self.resource_path(key),
            percent_encode(&host),
        );
        let string_to_sign = format!("sha1\n{sign_time}\n{}\n", sha1_hex(http_string.as_bytes()));
        let sign_key = hmac_sha1_hex(self.config.secret_key.as_bytes(), &key_time);
        let signature = hmac_sha1_hex(sign_key.as_bytes(), &string_to_sign);

        format!(
            "{}?q-sign-algorithm=sha1&q-ak={}&q-sign-time={}&q-key-time={}&q-header-list={}&q-url-param-list={}&q-signature={}",
            self.object_url(key),
            percent_encode(&self.config.secret_id),
            percent_encode(&sign_time),
            percent_encode(&key_time),
            header_list,
            url_param_list,
            signature,
        )
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
        let start = now.timestamp();
        let end = start + 3600;
        let date_str = now.format("%a, %d %b %Y %H:%M:%S GMT").to_string();
        let host = self.host();
        let authorization = self.authorization("put", key, content_type, &date_str, start, end);

        let resp = self
            .http
            .put(&url)
            .header("Host", &host)
            .header("Date", &date_str)
            .header("Content-Type", content_type)
            .header("Authorization", authorization)
            .body(body)
            .send()
            .await
            .map_err(|e| AppError::External(format!("COS put: {e}")))?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(AppError::External(format!("COS {status}: {text}")));
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

    /// HEAD object returns content-type and content-length.
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

    /// List objects by prefix (max 1000).
    pub async fn list_objects(&self, prefix: &str) -> Result<Vec<String>, AppError> {
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
        let mut keys = Vec::new();
        for cap in body.split("<Key>").skip(1) {
            if let Some(end) = cap.find("</Key>") {
                keys.push(cap[..end].to_string());
            }
        }
        Ok(keys)
    }

    fn resource_path(&self, key: &str) -> String {
        canonical_uri_path(key)
    }

    fn authorization(
        &self,
        method: &str,
        key: &str,
        content_type: &str,
        date_str: &str,
        start: i64,
        end: i64,
    ) -> String {
        let sign_time = format!("{start};{end}");
        let key_time = sign_time.clone();
        let host = self.host();
        let header_list = "content-type;date;host";
        let http_string = format!(
            "{}\n{}\n\ncontent-type={}&date={}&host={}\n",
            method.to_ascii_lowercase(),
            self.resource_path(key),
            percent_encode(content_type),
            percent_encode(date_str),
            percent_encode(&host),
        );
        let string_to_sign = format!("sha1\n{sign_time}\n{}\n", sha1_hex(http_string.as_bytes()));
        let sign_key = hmac_sha1_hex(self.config.secret_key.as_bytes(), &key_time);
        let signature = hmac_sha1_hex(sign_key.as_bytes(), &string_to_sign);

        format!(
            "q-sign-algorithm=sha1&q-ak={}&q-sign-time={}&q-key-time={}&q-header-list={}&q-url-param-list=&q-signature={}",
            self.config.secret_id, sign_time, key_time, header_list, signature
        )
    }

    fn sign(&self, input: &str) -> String {
        hmac_sha1_hex(self.config.secret_key.as_bytes(), input)
    }
}

fn canonical_uri_path(key: &str) -> String {
    let key = key.trim_start_matches('/');
    if key.is_empty() {
        return "/".to_string();
    }
    let path = key
        .split('/')
        .map(percent_encode)
        .collect::<Vec<_>>()
        .join("/");
    format!("/{path}")
}

fn percent_encode(input: &str) -> String {
    let mut encoded = String::with_capacity(input.len());
    for byte in input.as_bytes() {
        match *byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(*byte as char)
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}

fn sha1_hex(input: &[u8]) -> String {
    use sha1::{Digest, Sha1};

    let mut hasher = Sha1::new();
    hasher.update(input);
    to_hex(&hasher.finalize())
}

fn hmac_sha1_hex(key: &[u8], input: &str) -> String {
    use hmac::{Hmac, Mac};
    use sha1::Sha1;

    type HmacSha1 = Hmac<Sha1>;
    let mut mac = HmacSha1::new_from_slice(key).expect("HMAC-SHA1 key");
    mac.update(input.as_bytes());
    to_hex(&mac.finalize().into_bytes())
}

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::{CosClient, CosConfig};

    fn client() -> CosClient {
        CosClient::new(CosConfig {
            secret_id: "secret-id".into(),
            secret_key: "secret-key".into(),
            bucket: "bucket-123".into(),
            region: "ap-guangzhou".into(),
        })
    }

    #[test]
    fn cos_authorization_uses_time_ranges() {
        let authorization = client().authorization(
            "put",
            "avatars/1_1782384655469.jpeg",
            "image/jpeg",
            "Thu, 25 Jun 2026 10:50:55 GMT",
            1782384655,
            1782388255,
        );

        assert!(authorization.contains("q-sign-time=1782384655;1782388255"));
        assert!(authorization.contains("q-key-time=1782384655;1782388255"));
        assert!(authorization.contains("q-header-list=content-type;date;host"));
        let signature = authorization
            .rsplit_once("q-signature=")
            .map(|(_, signature)| signature)
            .unwrap();
        assert_eq!(signature.len(), 40);
        assert!(signature.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(signature.chars().all(|c| !c.is_ascii_uppercase()));
    }

    #[test]
    fn cos_signature_is_lowercase_hex_hmac_sha1() {
        let signature = client().sign("test-signing-input");

        assert_eq!(signature.len(), 40);
        assert!(signature.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(signature.chars().all(|c| !c.is_ascii_uppercase()));
    }

    #[test]
    fn signed_get_url_uses_cos_query_signature() {
        let url = client().signed_get_url_at("gallery/a b.jpg", 100, 60);

        assert!(
            url.starts_with("https://bucket-123.cos.ap-guangzhou.myqcloud.com/gallery/a%20b.jpg?")
        );
        assert!(url.contains("q-sign-algorithm=sha1"));
        assert!(url.contains("q-sign-time=100%3B160"));
        assert!(url.contains("q-key-time=100%3B160"));
        assert!(url.contains("q-header-list=host"));
        assert!(url.contains("q-url-param-list="));

        let signature = url
            .rsplit_once("q-signature=")
            .map(|(_, signature)| signature)
            .unwrap();
        assert_eq!(signature.len(), 40);
        assert!(signature.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(signature.chars().all(|c| !c.is_ascii_uppercase()));
    }

    #[test]
    fn post_upload_policy_contains_cos_form_fields() {
        let policy = client().post_upload_policy_at("gallery/a b.jpg", 100, 60);
        let form = policy.get("form_data").unwrap();

        assert_eq!(
            policy.get("url").and_then(|v| v.as_str()).unwrap(),
            "https://bucket-123.cos.ap-guangzhou.myqcloud.com/"
        );
        assert_eq!(
            policy.get("key").and_then(|v| v.as_str()).unwrap(),
            "gallery/a b.jpg"
        );
        assert_eq!(
            policy.get("path").and_then(|v| v.as_str()).unwrap(),
            "/files/gallery/a b.jpg"
        );
        assert_eq!(
            form.get("key").and_then(|v| v.as_str()).unwrap(),
            "gallery/a b.jpg"
        );
        assert_eq!(
            form.get("q-sign-algorithm")
                .and_then(|v| v.as_str())
                .unwrap(),
            "sha1"
        );
        assert_eq!(
            form.get("q-ak").and_then(|v| v.as_str()).unwrap(),
            "secret-id"
        );
        assert_eq!(
            form.get("q-key-time").and_then(|v| v.as_str()).unwrap(),
            "100;160"
        );
        assert_eq!(
            form.get("q-sign-time").and_then(|v| v.as_str()).unwrap(),
            "100;160"
        );
        assert_eq!(
            form.get("success_action_status")
                .and_then(|v| v.as_str())
                .unwrap(),
            "200"
        );
        let signature = form.get("q-signature").and_then(|v| v.as_str()).unwrap();
        assert_eq!(signature.len(), 40);
        assert!(signature.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(signature.chars().all(|c| !c.is_ascii_uppercase()));

        let policy_text = String::from_utf8(
            base64::Engine::decode(
                &base64::engine::general_purpose::STANDARD,
                form.get("policy").and_then(|v| v.as_str()).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        assert!(!policy_text.contains("\"bucket\""));
    }
}
