//! Files DTOs.

use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, utoipa::IntoParams, ToSchema)]
pub struct ListQuery {
    pub prefix: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams, ToSchema)]
pub struct SignQuery {
    pub key: String,
}

#[derive(Debug, Deserialize, utoipa::IntoParams, ToSchema)]
pub struct DeleteQuery {
    pub key: String,
}
