//! Qiniu content moderation (image censorship).
//!
//! Ports `qiniuAuth.ts` + `moderation.ts` logic.
//! Calls POST `https://ai.qiniuapi.com/v3/image/censor` with HMAC-SHA1 auth.
//!
//! Missing Qiniu credentials → moderation returns `unknown`; upload callers must reject unknown results.

use crate::error::AppError;
use reqwest::Client;
use tracing::warn;

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
        mime.starts_with("image/") && !mime.contains("svg") && !mime.contains("gif")
    }

    /// Submit a base64-encoded image for moderation.
    /// Qiniu requires data URI payloads to use the application/octet-stream prefix.
    pub async fn moderate(
        &self,
        base64_data: &str,
        _mime_type: &str,
    ) -> Result<ModerationResult, AppError> {
        self.moderate_uri(&format!(
            "data:application/octet-stream;base64,{}",
            base64_data
        ))
        .await
    }

    /// Submit an image URL or data URI for moderation.
    pub async fn moderate_uri(&self, uri: &str) -> Result<ModerationResult, AppError> {
        let (access_key, secret_key) = match (&self.access_key, &self.secret_key) {
            (Some(ak), Some(sk)) => (ak, sk),
            _ => {
                warn!("qiniu moderation unknown: credentials are not configured");
                return Ok(ModerationResult::Unknown);
            }
        };

        let url = "https://ai.qiniuapi.com/v3/image/censor";
        let body = serde_json::json!({
            "data": {
                "uri": uri,
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
            .map_err(|err| {
                warn!(error = %err, "qiniu moderation unknown: request failed");
                ModerationResult::Unknown
            });

        let resp = match resp {
            Ok(resp) => resp,
            Err(result) => return Ok(result),
        };

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            warn!(
                status = %status,
                body = %truncate_log(&body, 500),
                "qiniu moderation unknown: non-success response"
            );
            return Ok(ModerationResult::Unknown);
        }

        let text = resp.text().await.unwrap_or_default();
        let result = match serde_json::from_str::<serde_json::Value>(&text) {
            Ok(result) => result,
            Err(err) => {
                warn!(
                    error = %err,
                    body = %truncate_log(&text, 500),
                    "qiniu moderation unknown: invalid json response"
                );
                return Ok(ModerationResult::Unknown);
            }
        };

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
            if suggestion == "pass" {
                return Ok(ModerationResult::Pass);
            }
            warn!(
                suggestion = %suggestion,
                body = %truncate_log(&text, 500),
                "qiniu moderation unknown: review suggestion"
            );
            return Ok(ModerationResult::Unknown);
        }

        warn!(
            body = %truncate_log(&text, 500),
            "qiniu moderation unknown: missing result suggestion"
        );
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

        let content_type = "application/json";
        let signing_str = qiniu_signing_data("POST", url, content_type, body);

        type HmacSha1 = Hmac<Sha1>;
        let mut mac = HmacSha1::new_from_slice(secret_key.as_bytes()).expect("HMAC-SHA1 key");
        mac.update(signing_str.as_bytes());
        let sig = base64::engine::general_purpose::URL_SAFE.encode(mac.finalize().into_bytes());

        format!("{access_key}:{sig}")
    }
}

fn qiniu_signing_path(url: &str) -> String {
    let without_scheme = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);
    let path_start = without_scheme.find('/').unwrap_or(0);
    without_scheme[path_start..].to_string()
}

fn qiniu_signing_host(url: &str) -> String {
    let without_scheme = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);
    let host_end = without_scheme.find('/').unwrap_or(without_scheme.len());
    without_scheme[..host_end].to_string()
}

fn qiniu_signing_data(method: &str, url: &str, content_type: &str, body: &str) -> String {
    let mut data = format!(
        "{} {}\nHost: {}",
        method,
        qiniu_signing_path(url),
        qiniu_signing_host(url)
    );

    if !content_type.is_empty() {
        data.push_str(&format!("\nContent-Type: {content_type}"));
    }

    data.push_str("\n\n");

    if !body.is_empty() && !content_type.is_empty() && content_type != "application/octet-stream" {
        data.push_str(body);
    }

    data
}

fn truncate_log(value: &str, max_len: usize) -> String {
    let mut chars = value.chars();
    let truncated = chars.by_ref().take(max_len).collect::<String>();
    if chars.next().is_none() {
        value.to_string()
    } else {
        format!("{truncated}...")
    }
}

#[cfg(test)]
mod tests {
    use super::qiniu_signing_data;

    #[test]
    fn qiniu_signing_data_matches_documented_format() {
        let data = qiniu_signing_data(
            "POST",
            "https://ai.qiniuapi.com/v3/image/censor",
            "application/json",
            r#"{"data":{"uri":"x"}}"#,
        );

        assert_eq!(
            data,
            "POST /v3/image/censor\nHost: ai.qiniuapi.com\nContent-Type: application/json\n\n{\"data\":{\"uri\":\"x\"}}"
        );
    }
}
