// Scaffolding: foundation types are built ahead of their first consumers.
// TODO(M7): remove this `allow` and enforce `cargo clippy -- -D warnings`.
#![allow(dead_code)]

pub mod app_state;
pub mod config;
pub mod domain;
pub mod entity;
pub mod error;
pub mod external;
pub mod migration;
pub mod middleware;
pub mod response;