/// Time-slot-templates: CRUD wired inline (compact, JWT-protected, owner-scoped).
/// Uses the same pattern as bookings: router.rs = routes() + handlers + DTOs.
mod router;
pub use router::routes;