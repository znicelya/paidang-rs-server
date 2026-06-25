//! Gallery DTOs.

use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

// ── Gallery DTOs ─────────────────────────────────────

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateGalleryReq {
    pub group_id: Option<i32>,
    #[validate(length(min = 1))]
    pub title: String,
    pub subtitle: Option<String>,
    pub cover_image: Option<String>,
    pub image_url: Option<String>,
    pub image_list: Option<serde_json::Value>,
    pub video_url: Option<String>,
    pub media_type: Option<String>,
    pub tags: Option<String>,
    pub photographer_id: Option<i32>,
    pub photographer_name: Option<String>,
    pub shooting_location: Option<String>,
    pub shooting_date: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub file_size: Option<i32>,
    pub sort_order: Option<i32>,
    pub is_cover: Option<i8>,
    pub status: Option<i8>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams, ToSchema)]
pub struct UpdateGalleryReq {
    pub group_id: Option<i32>,
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub cover_image: Option<String>,
    pub image_url: Option<String>,
    pub image_list: Option<serde_json::Value>,
    pub video_url: Option<String>,
    pub media_type: Option<String>,
    pub tags: Option<String>,
    pub photographer_id: Option<i32>,
    pub photographer_name: Option<String>,
    pub shooting_location: Option<String>,
    pub shooting_date: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub file_size: Option<i32>,
    pub sort_order: Option<i32>,
    pub is_cover: Option<i8>,
    pub status: Option<i8>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams, ToSchema)]
pub struct GalleryListQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub group_id: Option<i32>,
    pub photographer_id: Option<i32>,
    pub status: Option<i8>,
}

// ── Gallery Tag DTOs ─────────────────────────────────

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateTagReq {
    #[validate(length(min = 1))]
    pub tag_name: String,
    pub tag_type: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams, ToSchema)]
pub struct UpdateTagReq {
    pub tag_name: Option<String>,
    pub tag_type: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams, ToSchema)]
pub struct TagListQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub tag_type: Option<String>,
}
