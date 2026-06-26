//! Gallery DTOs.

use crate::util::deserialize_optional_i8;
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
    #[serde(default, deserialize_with = "deserialize_optional_i8")]
    pub is_cover: Option<i8>,
    #[serde(default, deserialize_with = "deserialize_optional_i8")]
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
    #[serde(default, deserialize_with = "deserialize_optional_i8")]
    pub is_cover: Option<i8>,
    #[serde(default, deserialize_with = "deserialize_optional_i8")]
    pub status: Option<i8>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams, ToSchema)]
pub struct GalleryListQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub group_id: Option<i32>,
    pub photographer_id: Option<i32>,
    #[serde(default, deserialize_with = "deserialize_optional_i8")]
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

#[cfg(test)]
mod tests {
    use super::{CreateGalleryReq, UpdateGalleryReq};

    #[test]
    fn gallery_reqs_accept_boolean_i8_flags() {
        let create_body = serde_json::json!({
            "title": "Cover",
            "is_cover": true,
            "status": false
        });
        let create_req: CreateGalleryReq = serde_json::from_value(create_body).unwrap();
        assert_eq!(create_req.is_cover, Some(1));
        assert_eq!(create_req.status, Some(0));

        let update_body = serde_json::json!({ "is_cover": false, "status": true });
        let update_req: UpdateGalleryReq = serde_json::from_value(update_body).unwrap();
        assert_eq!(update_req.is_cover, Some(0));
        assert_eq!(update_req.status, Some(1));
    }
}
