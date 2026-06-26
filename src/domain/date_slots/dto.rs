//! Date slots DTOs.

use crate::util::deserialize_optional_i8;
use serde::Deserialize;
use utoipa::ToSchema;
use validator::{Validate, ValidationError};

/// Validate YYYY-MM-DD date format.
pub fn valid_date_format(s: &str) -> Result<(), ValidationError> {
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
    pub template_id: Option<i32>,
    #[validate(custom(function = "valid_date_format"))]
    pub slot_date: String,
    #[validate(length(min = 1))]
    pub slot_name: String,
    pub start_time: String,
    pub end_time: String,
    #[serde(default, deserialize_with = "deserialize_optional_i8")]
    pub is_special: Option<i8>,
    #[serde(default, deserialize_with = "deserialize_optional_i8")]
    pub status: Option<i8>,
    pub price: Option<i32>,
    pub remark: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams, ToSchema)]
pub struct UpdateReq {
    pub slot_name: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_i8")]
    pub is_special: Option<i8>,
    #[serde(default, deserialize_with = "deserialize_optional_i8")]
    pub status: Option<i8>,
    pub price: Option<i32>,
    pub remark: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams, ToSchema)]
pub struct ListQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub photographer_id: Option<i32>,
    pub slot_date: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams, ToSchema)]
pub struct DayQuery {
    pub photographer_id: i32,
    pub slot_date: String,
}

#[derive(Debug, Deserialize, utoipa::IntoParams, ToSchema)]
pub struct MonthlyQuery {
    pub photographer_id: i32,
    pub year_month: String,
}

#[cfg(test)]
mod tests {
    use super::{CreateReq, UpdateReq};

    #[test]
    fn create_req_accepts_boolean_i8_flags() {
        let body = serde_json::json!({
            "photographer_id": 1,
            "slot_date": "2026-08-01",
            "slot_name": "Morning",
            "start_time": "09:00",
            "end_time": "10:00",
            "is_special": true,
            "status": false
        });

        let req: CreateReq = serde_json::from_value(body).unwrap();

        assert_eq!(req.is_special, Some(1));
        assert_eq!(req.status, Some(0));
    }

    #[test]
    fn update_req_accepts_boolean_i8_flags() {
        let body = serde_json::json!({ "is_special": false, "status": true });

        let req: UpdateReq = serde_json::from_value(body).unwrap();

        assert_eq!(req.is_special, Some(0));
        assert_eq!(req.status, Some(1));
    }
}
