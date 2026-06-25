//! Time slot templates DTOs.

use serde::Deserialize;
use utoipa::ToSchema;
use validator::{Validate, ValidationError};

/// Validate HH:MM time format.
fn valid_time_format(s: &str) -> Result<(), ValidationError> {
    let re = regex::Regex::new(r"^\d{2}:\d{2}$")
        .map_err(|_| ValidationError::new("regex"))?;
    if re.is_match(s) { Ok(()) } else { Err(ValidationError::new("invalid_time_format")) }
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateReq {
    #[validate(length(min = 1))]
    pub slot_name: String,
    #[validate(custom(function = "valid_time_format"))]
    pub start_time: String,
    #[validate(custom(function = "valid_time_format"))]
    pub end_time: String,
    pub sort_order: Option<i32>,
    pub is_default: Option<i8>,
    pub status: Option<i8>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateReq {
    pub slot_name: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub sort_order: Option<i32>,
    pub is_default: Option<i8>,
    pub status: Option<i8>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams, ToSchema)]
pub struct ListQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}
