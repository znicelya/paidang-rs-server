//! Date settings DTOs.

use serde::Deserialize;
use utoipa::ToSchema;
use validator::{Validate, ValidationError};

/// Validate YYYY-MM-DD date format.
fn valid_date_format(s: &str) -> Result<(), ValidationError> {
    let re = regex::Regex::new(r"^\d{4}-\d{2}-\d{2}$")
        .map_err(|_| ValidationError::new("regex"))?;
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
    pub is_available: Option<i8>,
    pub use_template_id: Option<i32>,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams, ToSchema)]
pub struct UpdateReq {
    pub target_date: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
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
    pub target_date: String,
}
