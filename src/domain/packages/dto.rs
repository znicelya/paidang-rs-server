//! Packages domain DTOs.

use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

/// POST /packages
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreatePackageReq {
    #[validate(length(min = 1))]
    pub name: String,
    pub subtitle: Option<String>,
    pub category: Option<String>,
    #[validate(range(min = 0))]
    pub price: i32,
    pub original_price: Option<i32>,
    pub deposit: Option<i32>,
    pub cover_image: Option<String>,
    pub description: Option<String>,
    pub service_items: Option<serde_json::Value>,
    pub suitable_people: Option<String>,
    pub shooting_location: Option<String>,
    pub validity_days: Option<i32>,
    pub sort_order: Option<i32>,
    pub is_hot: Option<i8>,
    pub is_recommend: Option<i8>,
    pub status: Option<i8>,
}

/// PUT /packages/:id
#[derive(Debug, Default, Deserialize, Validate, ToSchema)]
pub struct UpdatePackageReq {
    pub name: Option<String>,
    pub subtitle: Option<String>,
    pub category: Option<String>,
    pub price: Option<i32>,
    pub original_price: Option<i32>,
    pub deposit: Option<i32>,
    pub cover_image: Option<String>,
    pub description: Option<String>,
    pub service_items: Option<serde_json::Value>,
    pub suitable_people: Option<String>,
    pub shooting_location: Option<String>,
    pub validity_days: Option<i32>,
    pub sort_order: Option<i32>,
    pub is_hot: Option<i8>,
    pub is_recommend: Option<i8>,
    pub status: Option<i8>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams, ToSchema)]
pub struct ListQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub category: Option<String>,
    pub status: Option<i8>,
}

// ── Package Items ──────────────────────────────────

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateItemReq {
    #[validate(range(min = 1))]
    pub package_id: i32,
    #[validate(length(min = 1))]
    pub item_type: String,
    #[validate(length(min = 1))]
    pub item_name: String,
    pub quantity: Option<i32>,
    pub unit: Option<String>,
    pub item_value: Option<String>,
    pub sort_order: Option<i32>,
    pub is_default: Option<i8>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams, ToSchema)]
pub struct UpdateItemReq {
    pub item_type: Option<String>,
    pub item_name: Option<String>,
    pub quantity: Option<i32>,
    pub unit: Option<String>,
    pub item_value: Option<String>,
    pub sort_order: Option<i32>,
    pub is_default: Option<i8>,
}

// ── Package Gallery ───────────────────────────────

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateGalleryReq {
    #[validate(range(min = 1))]
    pub package_id: i32,
    #[validate(length(min = 1))]
    pub image_url: String,
    pub image_type: Option<String>,
    pub caption: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams, ToSchema)]
pub struct UpdateGalleryReq {
    pub image_url: Option<String>,
    pub image_type: Option<String>,
    pub caption: Option<String>,
    pub sort_order: Option<i32>,
}
