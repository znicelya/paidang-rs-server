//! Packages domain DTOs.

use crate::util::deserialize_optional_i8;
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
    #[serde(default, deserialize_with = "deserialize_optional_i8")]
    pub is_hot: Option<i8>,
    #[serde(default, deserialize_with = "deserialize_optional_i8")]
    pub is_recommend: Option<i8>,
    #[serde(default, deserialize_with = "deserialize_optional_i8")]
    pub status: Option<i8>,
    /// Optional inline line items; when present, inserted with the package.
    pub items: Option<Vec<PackageItemInput>>,
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
    #[serde(default, deserialize_with = "deserialize_optional_i8")]
    pub is_hot: Option<i8>,
    #[serde(default, deserialize_with = "deserialize_optional_i8")]
    pub is_recommend: Option<i8>,
    #[serde(default, deserialize_with = "deserialize_optional_i8")]
    pub status: Option<i8>,
    /// When present, fully replaces the package's line items.
    pub items: Option<Vec<PackageItemInput>>,
}

/// A package line item supplied inline when creating/updating a package.
#[derive(Debug, Deserialize, ToSchema)]
pub struct PackageItemInput {
    pub item_type: Option<String>,
    pub item_name: String,
    pub quantity: Option<i32>,
    pub unit: Option<String>,
    pub item_value: Option<String>,
    pub sort_order: Option<i32>,
    #[serde(default, deserialize_with = "deserialize_optional_i8")]
    pub is_default: Option<i8>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams, ToSchema)]
pub struct ListQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub category: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_i8")]
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
    #[serde(default, deserialize_with = "deserialize_optional_i8")]
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
    #[serde(default, deserialize_with = "deserialize_optional_i8")]
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

#[cfg(test)]
mod tests {
    use super::{CreateItemReq, CreatePackageReq, UpdateItemReq, UpdatePackageReq};

    #[test]
    fn package_reqs_accept_boolean_i8_flags() {
        let create_body = serde_json::json!({
            "name": "Portrait",
            "price": 100,
            "is_hot": true,
            "is_recommend": false,
            "status": true
        });
        let create_req: CreatePackageReq = serde_json::from_value(create_body).unwrap();
        assert_eq!(create_req.is_hot, Some(1));
        assert_eq!(create_req.is_recommend, Some(0));
        assert_eq!(create_req.status, Some(1));

        let update_body = serde_json::json!({
            "is_hot": false,
            "is_recommend": true,
            "status": false
        });
        let update_req: UpdatePackageReq = serde_json::from_value(update_body).unwrap();
        assert_eq!(update_req.is_hot, Some(0));
        assert_eq!(update_req.is_recommend, Some(1));
        assert_eq!(update_req.status, Some(0));
    }

    #[test]
    fn package_item_reqs_accept_boolean_is_default() {
        let create_body = serde_json::json!({
            "package_id": 1,
            "item_type": "photo",
            "item_name": "Retouch",
            "is_default": true
        });
        let create_req: CreateItemReq = serde_json::from_value(create_body).unwrap();
        assert_eq!(create_req.is_default, Some(1));

        let update_body = serde_json::json!({ "is_default": false });
        let update_req: UpdateItemReq = serde_json::from_value(update_body).unwrap();
        assert_eq!(update_req.is_default, Some(0));
    }
}
