//! Auth service — WeChat login + auto-register + JWT issuance.
//! Ported from `paidang-worker-server/src/endpoints/auth/login.ts`.

use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

use crate::app_state::AppState;
use crate::entity::{user, user_profile};
use crate::error::AppError;
use crate::external::wechat::WechatApi;
use crate::middleware::auth::{sign_jwt, Claims};

use super::dto::LoginData;

/// Main login/orchestration. Called by the login handler.
///
/// # Flow (spec §6.1)
/// 1. code2session → openid, session_key, unionid
/// 2. Resolve phone: plain `phone` first, fallback `phone_code` → get_user_phone
/// 3. User lookup by openid: UPDATE if found, INSERT if new (+ user_profile)
/// 4. Sign JWT, return user + token + is_new
pub async fn login(
    state: &AppState,
    wechat: &dyn WechatApi,
    code: &str,
    nickname: Option<&str>,
    avatar_url: Option<&str>,
    phone: Option<&str>,
    phone_code: Option<&str>,
) -> Result<LoginData, AppError> {
    // 1. Exchange code for WeChat session
    let session = wechat.code2session(code).await?;

    // 2. Get phone number — plain first, fallback phone_code (WeChat auth)
    let mut phone_number: Option<String> = phone.map(|s| s.to_owned());
    if phone_number.is_none() {
        if let Some(pc) = phone_code {
            match wechat.get_user_phone(pc).await {
                Ok(p) => phone_number = Some(p),
                Err(e) => tracing::warn!("Failed to get phone number via phone_code: {e}"),
            }
        }
    }

    // 3. Find existing user by openid
    let existing = user::Entity::find()
        .filter(user::Column::Openid.eq(&session.openid))
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB error: {e}")))?;

    let mut is_new = false;
    let user_id: i32;

    if let Some(record) = existing {
        user_id = record.user_id;

        // Update session_key, unionid, optional phone, last_login_time
        let mut active: user::ActiveModel = record.into();
        active.session_key = Set(Some(session.session_key));
        if let Some(ref uid) = session.unionid {
            active.unionid = Set(Some(uid.clone()));
        }
        active.last_login_time = Set(Some(Utc::now().naive_utc()));
        if let Some(ref pn) = phone_number {
            active.phone = Set(Some(pn.clone()));
        }
        active
            .update(&state.db)
            .await
            .map_err(|e| AppError::Internal(format!("DB update failed: {e}")))?;
    } else {
        is_new = true;

        // Insert user
        let new_user = user::ActiveModel {
            openid: Set(session.openid.clone()),
            unionid: Set(session.unionid.clone()),
            session_key: Set(Some(session.session_key.clone())),
            role: Set(0),
            phone: Set(phone_number.clone()),
            status: Set(1),
            last_login_time: Set(Some(Utc::now().naive_utc())),
            ..Default::default()
        };
        let inserted = new_user
            .insert(&state.db)
            .await
            .map_err(|e| AppError::Internal(format!("DB insert user failed: {e}")))?;
        user_id = inserted.user_id;

        // Insert empty profile
        let new_profile = user_profile::ActiveModel {
            user_id: Set(user_id),
            nickname: Set(nickname.map(|s| s.to_owned())),
            avatar_url: Set(avatar_url.map(|s| s.to_owned())),
            ..Default::default()
        };
        new_profile
            .insert(&state.db)
            .await
            .map_err(|e| AppError::Internal(format!("DB insert profile failed: {e}")))?;
    }

    // 4. Re-read user to return current state
    let record = user::Entity::find_by_id(user_id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB read failed: {e}")))?
        .ok_or(AppError::Internal("user created but not found".into()))?;

    // 5. Sign JWT
    let expires_secs = state.settings.jwt.expires_secs;
    let exp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + expires_secs;

    let jwt_secret = state
        .settings
        .jwt_secret
        .as_ref()
        .ok_or(AppError::Internal("JWT_SECRET not configured".into()))?;

    let claims = Claims {
        sub: user_id,
        openid: record.openid.clone(),
        role: record.role,
        exp,
    };
    let token = sign_jwt(claims, jwt_secret)?;

    Ok(LoginData {
        user_id,
        openid: record.openid,
        role: record.role,
        phone: record.phone,
        is_new,
        token,
    })
}