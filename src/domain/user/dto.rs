//! DTOs for the user domain.

use crate::util::deserialize_optional_i8;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

/// GET /user/profile query.
#[derive(Debug, Deserialize, utoipa::IntoParams, ToSchema)]
pub struct ProfileQuery {
    pub user_id: Option<i32>,
}

/// GET /user/profile response data.
#[derive(Debug, Serialize, ToSchema)]
pub struct ProfileData {
    pub user_id: i32,
    pub openid: String,
    pub role: i8,
    pub phone: Option<String>,
    pub status: i8,
    pub nickname: Option<String>,
    pub avatar_url: Option<String>,
    pub background_image: Option<String>,
    pub gender: Option<i8>,
    pub country: Option<String>,
    pub province: Option<String>,
    pub city: Option<String>,
    pub birthday: Option<String>,
    pub bio: Option<String>,
    pub create_time: Option<String>,
}

/// PUT /user/profile request body — all fields optional.
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateProfileRequest {
    pub phone: Option<String>,
    pub nickname: Option<String>,
    pub avatar_url: Option<String>,
    pub background_image: Option<String>,
    #[validate(range(min = 0, max = 2))]
    #[serde(default, deserialize_with = "deserialize_optional_i8")]
    pub gender: Option<i8>,
    pub birthday: Option<String>,
    pub province: Option<String>,
    pub city: Option<String>,
    #[validate(length(max = 200))]
    pub bio: Option<String>,
}
