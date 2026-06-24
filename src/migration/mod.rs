//! SeaORM migrations. Ported from the four SQLite migration files of
//! `paidang-worker-server`, adapted for MySQL (no foreign keys, no triggers,
//! UTC+8 via `CURRENT_TIMESTAMP`). Applied at startup against the configured
//! MySQL database (see `main.rs`).

pub use sea_orm_migration::MigratorTrait;

mod m20250101_0001_users_profiles;
mod m20250101_0002_packages_items;
mod m20250101_0003_gallerys_groups_tags;
mod m20250101_0004_bookings;

use sea_orm_migration::MigrationTrait;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250101_0001_users_profiles::Migration),
            Box::new(m20250101_0002_packages_items::Migration),
            Box::new(m20250101_0003_gallerys_groups_tags::Migration),
            Box::new(m20250101_0004_bookings::Migration),
        ]
    }
}
