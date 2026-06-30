//! User profile service — read/update operations.
//! Identity is always derived from the JWT (spec §6.4); client-supplied
//! `user_id` params are removed from the API.

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

use crate::app_state::AppState;
use crate::entity::{user, user_profile};
use crate::error::AppError;

use super::dto::{ProfileData, UpdateProfileRequest};

/// Read the authenticated user's combined profile (user + user_profile).
pub async fn get_profile(state: &AppState, user_id: i32) -> Result<ProfileData, AppError> {
    let u = user::Entity::find_by_id(user_id)
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB: {e}")))?;

    let u = u.ok_or(AppError::NotFound("用户不存在".into()))?;

    let p = user_profile::Entity::find()
        .filter(user_profile::Column::UserId.eq(user_id))
        .one(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("DB: {e}")))?;

    Ok(ProfileData {
        user_id: u.user_id,
        openid: u.openid,
        role: u.role,
        phone: u.phone,
        status: u.status,
        nickname: p.as_ref().and_then(|r| r.nickname.clone()),
        avatar_url: p.as_ref().and_then(|r| r.avatar_url.clone()),
        background_image: p.as_ref().and_then(|r| r.background_image.clone()),
        gender: p.as_ref().and_then(|r| r.gender),
        country: p.as_ref().and_then(|r| r.country.clone()),
        province: p.as_ref().and_then(|r| r.province.clone()),
        city: p.as_ref().and_then(|r| r.city.clone()),
        birthday: p.as_ref().and_then(|r| r.birthday.clone()),
        bio: p.as_ref().and_then(|r| r.bio.clone()),
        create_time: u
            .create_time
            .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
    })
}

/// Update the authenticated user's profile. Phone goes to user, everything
/// else to user_profile (inserts profile record if one doesn't exist yet).
pub async fn update_profile(
    state: &AppState,
    user_id: i32,
    req: &UpdateProfileRequest,
) -> Result<ProfileData, AppError> {
    // -- user table: phone --
    if let Some(ref phone) = req.phone {
        let mut active: user::ActiveModel = user::Entity::find_by_id(user_id)
            .one(&state.db)
            .await
            .map_err(|e| AppError::Internal(format!("DB: {e}")))?
            .ok_or(AppError::NotFound("用户不存在".into()))?
            .into();
        active.phone = Set(Some(phone.clone()));
        active
            .update(&state.db)
            .await
            .map_err(|e| AppError::Internal(format!("DB: {e}")))?;
    }

    // -- user_profile: upsert --
    let has_profile_fields = req.nickname.is_some()
        || req.avatar_url.is_some()
        || req.background_image.is_some()
        || req.gender.is_some()
        || req.birthday.is_some()
        || req.province.is_some()
        || req.city.is_some()
        || req.bio.is_some();

    if has_profile_fields {
        let existing = user_profile::Entity::find()
            .filter(user_profile::Column::UserId.eq(user_id))
            .one(&state.db)
            .await
            .map_err(|e| AppError::Internal(format!("DB: {e}")))?;

        if let Some(rec) = existing {
            let mut active: user_profile::ActiveModel = rec.into();
            set_profile_fields(&mut active, req);
            active
                .update(&state.db)
                .await
                .map_err(|e| AppError::Internal(format!("DB: {e}")))?;
        } else {
            let mut model = user_profile::ActiveModel {
                user_id: Set(user_id),
                ..Default::default()
            };
            set_profile_fields(&mut model, req);
            model
                .insert(&state.db)
                .await
                .map_err(|e| AppError::Internal(format!("DB: {e}")))?;
        }
    }

    get_profile(state, user_id).await
}

fn set_profile_fields(model: &mut user_profile::ActiveModel, req: &UpdateProfileRequest) {
    if let Some(ref v) = req.nickname {
        model.nickname = Set(Some(v.clone()));
    }
    if let Some(ref v) = req.avatar_url {
        model.avatar_url = Set(Some(v.clone()));
    }
    if let Some(ref v) = req.background_image {
        model.background_image = Set(Some(v.clone()));
    }
    if let Some(v) = req.gender {
        model.gender = Set(Some(v));
    }
    if let Some(ref v) = req.birthday {
        model.birthday = Set(Some(v.clone()));
    }
    if let Some(ref v) = req.province {
        model.province = Set(Some(v.clone()));
    }
    if let Some(ref v) = req.city {
        model.city = Set(Some(v.clone()));
    }
    if let Some(ref v) = req.bio {
        model.bio = Set(Some(v.clone()));
    }
}
