//! Gallery groups DTOs.

use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateReq {
    #[validate(length(min = 1))]
    pub name: String,
    pub cover_image: Option<String>,
    pub description: Option<String>,
    pub sort_order: Option<i32>,
    pub is_visible: Option<i8>,
    pub status: Option<i8>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams, ToSchema)]
pub struct UpdateReq {
    pub name: Option<String>,
    pub cover_image: Option<String>,
    pub description: Option<String>,
    pub sort_order: Option<i32>,
    pub is_visible: Option<i8>,
    pub status: Option<i8>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams, ToSchema)]
pub struct ListQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}
