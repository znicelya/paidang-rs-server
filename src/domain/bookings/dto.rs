//! Bookings domain DTOs.

use serde::{Deserialize, Serialize};
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

/// Validate HH:MM time format.
fn valid_time_format(s: &str) -> Result<(), ValidationError> {
    let re =
        regex::Regex::new(r"^\d{2}:\d{2}$").map_err(|_| ValidationError::new("regex"))?;
    if re.is_match(s) {
        Ok(())
    } else {
        Err(ValidationError::new("invalid_time_format"))
    }
}

/// Validate date/time format for optional fields — validator passes &String for Option<String>.
fn valid_date_optional(s: &str) -> Result<(), ValidationError> {
    valid_date_format(s)
}

/// Validate optional time field — validator passes &String for Option<String>.
fn valid_time_optional(s: &str) -> Result<(), ValidationError> {
    valid_time_format(s)
}

/// POST /bookings request body (mirrors TS `bookingCreateSchema`).
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct CreateBookingRequest {
    #[validate(range(min = 1))]
    pub photographer_id: i32,
    pub user_id: Option<i32>,
    pub slot_instance_id: Option<i32>,
    pub package_id: Option<i32>,
    #[validate(custom(function = "valid_date_format"))]
    pub booking_date: String,
    #[validate(custom(function = "valid_time_format"))]
    pub start_time: String,
    #[validate(custom(function = "valid_time_format"))]
    pub end_time: String,
    pub total_amount: Option<i32>,
    pub deposit_amount: Option<i32>,
    pub paid_amount: Option<i32>,
    pub status: Option<String>,
    #[validate(length(min = 1))]
    pub customer_name: String,
    #[validate(length(min = 1))]
    pub customer_phone: String,
    pub customer_remark: Option<String>,
    pub photographer_remark: Option<String>,
}

/// PUT /bookings/:id request body.
#[derive(Debug, Default, Deserialize, Validate, ToSchema)]
pub struct UpdateBookingRequest {
    pub photographer_id: Option<i32>,
    pub slot_instance_id: Option<i32>,
    pub package_id: Option<i32>,
    #[validate(custom(function = "valid_date_optional"))]
    pub booking_date: Option<String>,
    #[validate(custom(function = "valid_time_optional"))]
    pub start_time: Option<String>,
    #[validate(custom(function = "valid_time_optional"))]
    pub end_time: Option<String>,
    pub total_amount: Option<i32>,
    pub deposit_amount: Option<i32>,
    pub paid_amount: Option<i32>,
    pub status: Option<String>,
    pub cancel_reason: Option<String>,
    pub customer_name: Option<String>,
    pub customer_phone: Option<String>,
    pub customer_remark: Option<String>,
    pub photographer_remark: Option<String>,
}

/// Booking list query params.
#[derive(Debug, Deserialize, utoipa::IntoParams, ToSchema)]
pub struct BookingListQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub photographer_id: Option<i32>,
    pub status: Option<String>,
    pub booking_date: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams, ToSchema)]
pub struct StatsQuery {
    pub photographer_id: Option<i32>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams, ToSchema)]
pub struct TodayQuery {
    pub photographer_id: Option<i32>,
}

/// Created booking response.
#[derive(Debug, Serialize, ToSchema)]
pub struct CreateBookingData {
    pub booking_id: i32,
    pub booking_no: String,
}

/// Stats response.
#[derive(Debug, Serialize, ToSchema)]
pub struct StatsData {
    pub pending: u64,
    pub today: u64,
    pub in_progress: u64,
}
