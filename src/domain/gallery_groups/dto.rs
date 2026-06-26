//! Gallery groups DTOs.

use crate::util::deserialize_optional_i8;
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
    #[serde(default, deserialize_with = "deserialize_optional_i8")]
    pub is_visible: Option<i8>,
    #[serde(default, deserialize_with = "deserialize_optional_i8")]
    pub status: Option<i8>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams, ToSchema)]
pub struct UpdateReq {
    pub name: Option<String>,
    pub cover_image: Option<String>,
    pub description: Option<String>,
    pub sort_order: Option<i32>,
    #[serde(default, deserialize_with = "deserialize_optional_i8")]
    pub is_visible: Option<i8>,
    #[serde(default, deserialize_with = "deserialize_optional_i8")]
    pub status: Option<i8>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams, ToSchema)]
pub struct ListQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub create_by: Option<i32>,
    #[serde(default, deserialize_with = "deserialize_optional_i8")]
    pub status: Option<i8>,
    #[serde(default, deserialize_with = "deserialize_optional_i8")]
    pub is_visible: Option<i8>,
    pub keyword: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::{CreateReq, UpdateReq};

    #[test]
    fn gallery_group_reqs_accept_boolean_i8_flags() {
        let create_body = serde_json::json!({
            "name": "Wedding",
            "is_visible": true,
            "status": false
        });
        let create_req: CreateReq = serde_json::from_value(create_body).unwrap();
        assert_eq!(create_req.is_visible, Some(1));
        assert_eq!(create_req.status, Some(0));

        let update_body = serde_json::json!({ "is_visible": false, "status": true });
        let update_req: UpdateReq = serde_json::from_value(update_body).unwrap();
        assert_eq!(update_req.is_visible, Some(0));
        assert_eq!(update_req.status, Some(1));
    }
}
