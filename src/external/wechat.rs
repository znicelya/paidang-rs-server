//! WeChat mini-program APIs: code2session, access_token, getuserphonenumber.
//! Ported from `paidang-worker-server/src/endpoints/auth/login.ts`.
//! Trait-based for testability (mock in integration tests).

use serde::Deserialize;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::error::AppError;

// ── Response types ──────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct Code2SessionResp {
    pub openid: String,
    pub session_key: String,
    pub unionid: Option<String>,
    pub errcode: Option<i64>,
    pub errmsg: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AccessTokenResp {
    pub access_token: String,
    pub expires_in: u64,
    pub errcode: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct PhoneNumberResp {
    pub errcode: i64,
    pub errmsg: Option<String>,
    pub phone_info: Option<PhoneInfo>,
}

#[derive(Debug, Deserialize)]
pub struct PhoneInfo {
    pub pure_phone_number: String,
}

// ── Trait (mockable) ───────────────────────────────────────

#[async_trait::async_trait]
pub trait WechatApi: Send + Sync {
    async fn code2session(&self, code: &str) -> Result<Code2SessionResp, AppError>;
    async fn get_user_phone(&self, phone_code: &str) -> Result<String, AppError>;
}

// ── Real implementation via reqwest ────────────────────────

pub struct ReqwestWechat {
    appid: String,
    secret: String,
    http: reqwest::Client,
    /// In-memory access_token cache (single-instance, matches TS original).
    token_cache: Mutex<Option<CachedToken>>,
}

struct CachedToken {
    token: String,
    expires_at: Instant,
}

impl ReqwestWechat {
    pub fn new(appid: String, secret: String) -> Self {
        Self {
            appid,
            secret,
            http: reqwest::Client::new(),
            token_cache: Mutex::new(None),
        }
    }

    /// Fetch a fresh access_token, using cache when valid (5 min buffer).
    async fn get_access_token(&self) -> Result<String, AppError> {
        // Check cache first
        {
            let guard = self.token_cache.lock().unwrap();
            if let Some(cached) = guard.as_ref() {
                if Instant::now() < cached.expires_at {
                    return Ok(cached.token.clone());
                }
            }
        }

        let url = format!(
            "https://api.weixin.qq.com/cgi-bin/token?grant_type=client_credential&appid={}&secret={}",
            self.appid, self.secret,
        );
        let resp: AccessTokenResp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::External(format!("WeChat token request failed: {e}")))?
            .json()
            .await
            .map_err(|e| AppError::External(format!("WeChat token parse failed: {e}")))?;

        if let Some(code) = resp.errcode {
            if code != 0 {
                return Err(AppError::External(format!(
                    "WeChat token error: code={code}"
                )));
            }
        }

        let expires_at = Instant::now() + Duration::from_secs(resp.expires_in.saturating_sub(300));
        let token = resp.access_token.clone();

        let mut guard = self.token_cache.lock().unwrap();
        *guard = Some(CachedToken {
            token: resp.access_token,
            expires_at,
        });

        Ok(token)
    }
}

#[async_trait::async_trait]
impl WechatApi for ReqwestWechat {
    async fn code2session(&self, code: &str) -> Result<Code2SessionResp, AppError> {
        let url = format!(
            "https://api.weixin.qq.com/sns/jscode2session?appid={}&secret={}&js_code={}&grant_type=authorization_code",
            self.appid, self.secret, code,
        );
        let resp: Code2SessionResp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("WeChat code2session failed: {:#?}", e);
                AppError::External(format!("WeChat code2session failed: {e}"))
            })?
            .json()
            .await
            .map_err(|e| {
                tracing::error!("WeChat code2session parse failed: {:#?}", e);
                AppError::External(format!("WeChat code2session parse failed: {e}"))
            })?;

        if let Some(code) = resp.errcode {
            if code != 0 {
                return Err(AppError::InputValidation(format!(
                    "WeChat API error: {}",
                    resp.errmsg.as_deref().unwrap_or("unknown")
                )));
            }
        }
        Ok(resp)
    }

    async fn get_user_phone(&self, phone_code: &str) -> Result<String, AppError> {
        let token = self.get_access_token().await?;
        let url = "https://api.weixin.qq.com/wxa/business/getuserphonenumber?access_token="
            .to_string()
            + &token;
        let body = serde_json::json!({ "code": phone_code });

        let resp: PhoneNumberResp = self
            .http
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::External(format!("WeChat getphone failed: {e}")))?
            .json()
            .await
            .map_err(|e| AppError::External(format!("WeChat getphone parse failed: {e}")))?;

        if resp.errcode != 0 {
            return Err(AppError::External(format!(
                "WeChat getphone error: {}",
                resp.errmsg.as_deref().unwrap_or("unknown")
            )));
        }

        resp.phone_info
            .map(|i| i.pure_phone_number)
            .ok_or_else(|| AppError::External("WeChat getphone: no phone_info".into()))
    }
}
