//! Booking logs DTOs.

use serde::Deserialize;

/// Booking log list query params.
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct ListQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub booking_id: Option<i32>,
}
