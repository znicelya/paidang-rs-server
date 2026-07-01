//! Files DTOs.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UploadPolicyRequest {
    pub prefix: Option<String>,
    pub file_name: String,
    pub content_type: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ModerateUploadRequest {
    pub key: String,
    pub content_type: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct Base64UploadRequest {
    pub file_name: Option<String>,
    pub content_type: Option<String>,
    pub prefix: Option<String>,
    pub folder: Option<String>,
    pub data_base64: String,
}

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
