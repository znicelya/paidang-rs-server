//! Logs DTOs.

use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, utoipa::IntoParams, ToSchema)]
pub struct SinceQuery {
    pub since: Option<usize>,
}
