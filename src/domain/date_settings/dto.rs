//! Date settings DTOs.

use crate::util::deserialize_optional_i8;
use serde::Deserialize;
use utoipa::ToSchema;
use validator::{Validate, ValidationError};

/// Validate YYYY-MM-DD date format.
fn valid_date_format(s: &str) -> Result<(), ValidationError> {
    let re =
        regex::Regex::new(r"^\d{4}-\d{2}-\d{2}$").map_err(|_| ValidationError::new("regex"))?;
    if re.is_match(s) {
        Ok(())
    } else {
        Err(ValidationError::new("invalid_date_format"))
    }
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateReq {
    #[validate(range(min = 1))]
    pub photographer_id: i32,
    #[validate(custom(function = "valid_date_format"))]
    pub target_date: String,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_i8")]
    pub is_available: Option<i8>,
    pub use_template_id: Option<i32>,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams, ToSchema)]
pub struct UpdateReq {
    pub target_date: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_i8")]
    pub is_available: Option<i8>,
    pub use_template_id: Option<i32>,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams, ToSchema)]
pub struct ListQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub photographer_id: Option<i32>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams, ToSchema)]
pub struct CheckQuery {
    pub photographer_id: i32,
    #[serde(alias = "date")]
    pub target_date: String,
}

#[cfg(test)]
mod tests {
    use super::{CreateReq, UpdateReq};

    #[test]
    fn create_req_accepts_boolean_is_available() {
        let body = serde_json::json!({
            "photographer_id": 1,
            "target_date": "2026-08-01",
            "is_available": false
        });

        let req: CreateReq = serde_json::from_value(body).unwrap();

        assert_eq!(req.is_available, Some(0));
    }

    #[test]
    fn update_req_accepts_boolean_is_available() {
        let body = serde_json::json!({ "is_available": true });

        let req: UpdateReq = serde_json::from_value(body).unwrap();

        assert_eq!(req.is_available, Some(1));
    }

    #[test]
    fn check_query_accepts_legacy_date_alias() {
        let body = serde_json::json!({
            "photographer_id": 1,
            "date": "2026-08-01"
        });

        let req: super::CheckQuery = serde_json::from_value(body).unwrap();

        assert_eq!(req.target_date, "2026-08-01");
    }
}
