//! Qiniu content moderation (image censorship).
//!
//! Ports `qiniuAuth.ts` + `moderation.ts` logic.
//! Calls POST `https://ai.qiniuapi.com/v3/image/censor` with HMAC-SHA1 auth.
//!
//! Missing Qiniu credentials → moderation returns `unknown` (graceful degradation per spec §7.2).

use crate::error::AppError;
use reqwest::Client;

/// Result of a single moderation scan.
#[derive(Debug, Clone)]
pub enum ModerationResult {
    /// Image passed all checks.
    Pass,
    /// Image was blocked (reason: e.g. "pulp", "terror", "politician").
    Block(String),
    /// Moderation skipped — missing credentials or non-image content.
    Unknown,
}

/// Moderation client. Requires Qiniu access/secret key.
#[derive(Clone)]
pub struct QiniuModeration {
    http: Client,
    access_key: Option<String>,
    secret_key: Option<String>,
}

impl QiniuModeration {
    pub fn new(access_key: Option<String>, secret_key: Option<String>) -> Self {
        Self {
            http: Client::new(),
            access_key,
            secret_key,
        }
    }

    /// Returns true if credentials are available and moderation can run.
    pub fn is_configured(&self) -> bool {
        self.access_key.is_some() && self.secret_key.is_some()
    }

    /// Check if a MIME type should be moderated (images only).
    pub fn should_moderate(mime: &str) -> bool {
        mime.starts_with("image/")
            && !mime.contains("svg")
            && !mime.contains("gif")
    }

    /// Submit a base64-encoded image for moderation.
    /// `mime_type` e.g. `image/jpeg` — used as `application/octet-stream` prefix per existing fix.
    pub async fn moderate(
        &self,
        base64_data: &str,
        _mime_type: &str,
    ) -> Result<ModerationResult, AppError> {
        let (access_key, secret_key) = match (&self.access_key, &self.secret_key) {
            (Some(ak), Some(sk)) => (ak, sk),
            _ => return Ok(ModerationResult::Unknown),
        };

        let url = "https://ai.qiniuapi.com/v3/image/censor";
        let body = serde_json::json!({
            "data": {
                "uri": format!("data:application/octet-stream;base64,{}", base64_data),
            },
            "params": {
                "scenes": ["pulp", "terror", "politician"],
            },
        });
        let body_str = body.to_string();

        let token = self.qiniu_auth_token(access_key, secret_key, url, &body_str);

        let resp = self
            .http
            .post(url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Qiniu {token}"))
            .body(body_str)
            .send()
            .await
            .map_err(|e| AppError::External(format!("Qiniu moderation: {e}")))?;

        let status = resp.status();
        if !status.is_success() {
            return Ok(ModerationResult::Unknown);
        }

        let result: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| AppError::External(format!("Qiniu parse: {e}")))?;

        // Check the result. `result.scenes` is an OBJECT keyed by scene name
        // (`pulp` / `terror` / `politician`), each carrying its own `suggestion`.
        if let Some(suggestion) = result
            .pointer("/result/suggestion")
            .and_then(|v| v.as_str())
        {
            if suggestion == "block" {
                let reason = result
                    .pointer("/result/scenes")
                    .and_then(|v| v.as_object())
                    .map(|scenes| {
                        scenes
                            .iter()
                            .filter(|(_, sc)| {
                                sc.get("suggestion").and_then(|s| s.as_str()) == Some("block")
                            })
                            .map(|(name, _)| name.clone())
                            .collect::<Vec<_>>()
                            .join(",")
                    })
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| "unknown".to_string());
                return Ok(ModerationResult::Block(reason));
            }
            return Ok(ModerationResult::Pass);
        }

        Ok(ModerationResult::Unknown)
    }

    fn qiniu_auth_token(
        &self,
        access_key: &str,
        secret_key: &str,
        url: &str,
        body: &str,
    ) -> String {
        use base64::Engine;
        use hmac::{Hmac, Mac};
        use sha1::Sha1;

        let path = url.trim_start_matches("https://ai.qiniuapi.com");
        let host = "ai.qiniuapi.com";
        let content_type = "application/json";

        let signing_str = format!("POST {path}\nHost: {host}\nContent-Type: {content_type}\n\n{body}");

        type HmacSha1 = Hmac<Sha1>;
        let mut mac = HmacSha1::new_from_slice(secret_key.as_bytes())
            .expect("HMAC-SHA1 key");
        mac.update(signing_str.as_bytes());
        let sig = base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes());

        format!("{access_key}:{sig}")
    }
}
