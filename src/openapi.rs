//! OpenAPI 3.0 documentation for paidang-rs-server.
//!
//! Defines the top-level `ApiDoc` struct used by `utoipa` to generate
//! the `GET /` OpenAPI spec and the Swagger UI at `/swagger-ui`.

use utoipa::OpenApi;

use crate::domain::auth::dto;
use crate::domain::bookings::dto as bookings_dto;
use crate::domain::packages::dto as packages_dto;
use crate::response;
use crate::error;

/// Top-level OpenAPI document.
///
/// Routes are registered in `main.rs` via `.merge()` and `utoipa_axum::router::OpenApiRouter`.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "paidang-rs-server",
        description = "Self-hosted Rust rewrite of paidang-worker-server — axum + SeaORM + MySQL + Tencent COS",
        version = "0.1.0",
    ),
    components(
        // Auth DTOs
        schemas(dto::LoginReq, dto::LoginResp, dto::PhoneCodeReq),
        // Booking DTOs
        schemas(bookings_dto::CreateBookingRequest, bookings_dto::UpdateBookingRequest, bookings_dto::CreateBookingData, bookings_dto::StatsData),
        // Package DTOs
        schemas(packages_dto::CreatePackageReq, packages_dto::UpdatePackageReq, packages_dto::CreateItemReq, packages_dto::CreateGalleryReq),
        // Response envelope
        schemas(response::ApiResponse<serde_json::Value>, response::PaginatedData<serde_json::Value>),
        // Error
        schemas(error::ErrorBody),
    ),
    tags(
        (name = "auth", description = "Authentication (WeChat login + JWT)"),
        (name = "user", description = "User profile read/update"),
        (name = "packages", description = "Photography packages (read public, write admin)"),
        (name = "gallery-groups", description = "Gallery groups (read public, write admin)"),
        (name = "gallery", description = "Gallery images/media (read public, write admin)"),
        (name = "time-slot-templates", description = "Time slot templates (photographer-owned)"),
        (name = "date-slots", description = "Date slot instances (photographer-owned)"),
        (name = "date-settings", description = "Date availability settings (photographer-owned)"),
        (name = "bookings", description = "Bookings (JWT-protected, ownership scoped)"),
        (name = "booking-logs", description = "Booking audit logs (read-only)"),
        (name = "files", description = "File upload/download/delete via COS"),
        (name = "logs", description = "Realtime dev-mode log viewer"),
    )
)]
pub struct ApiDoc;
