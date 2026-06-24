//! Migration 0002 — packages + items + gallery.
//!
//! Ported from `paidang-worker-server/migrations/0002_packages_items_tables.sql`.
//! Same MySQL/UTC+8/no-FK rules. `service_items` is JSON. Price fields are
//! INT (cents, per the original spec).

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

const STATEMENTS: &[&str] = &[
    // package
    r#"CREATE TABLE IF NOT EXISTS package (
        package_id        INT AUTO_INCREMENT PRIMARY KEY,
        name              VARCHAR(255) NOT NULL,
        subtitle          VARCHAR(255),
        category          VARCHAR(64) DEFAULT '一般',
        price             INT NOT NULL,
        original_price    INT,
        deposit           INT DEFAULT 0,
        cover_image       VARCHAR(1024),
        description       TEXT,
        service_items     JSON,
        suitable_people   VARCHAR(255),
        shooting_location VARCHAR(64),
        validity_days     INT DEFAULT 365,
        sort_order        INT DEFAULT 0,
        is_hot            TINYINT DEFAULT 0,
        is_recommend      TINYINT DEFAULT 0,
        status            TINYINT DEFAULT 1,
        view_count        INT DEFAULT 0,
        sale_count        INT DEFAULT 0,
        create_time       DATETIME DEFAULT CURRENT_TIMESTAMP,
        update_time       DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
        create_by         INT,
        update_by         INT
    ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4"#,
    r#"CREATE INDEX idx_package_category ON package(category)"#,
    r#"CREATE INDEX idx_package_status ON package(status)"#,
    r#"CREATE INDEX idx_package_is_hot ON package(is_hot)"#,
    r#"CREATE INDEX idx_package_sort ON package(sort_order)"#,
    // package_item
    r#"CREATE TABLE IF NOT EXISTS package_item (
        item_id     INT AUTO_INCREMENT PRIMARY KEY,
        package_id  INT NOT NULL,
        item_type   VARCHAR(64) NOT NULL,
        item_name   VARCHAR(255) NOT NULL,
        quantity    INT DEFAULT 1,
        unit        VARCHAR(16) DEFAULT '张',
        item_value  VARCHAR(255),
        sort_order  INT DEFAULT 0,
        is_default  TINYINT DEFAULT 0,
        create_time DATETIME DEFAULT CURRENT_TIMESTAMP,
        update_time DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
    ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4"#,
    r#"CREATE INDEX idx_package_item_package_id ON package_item(package_id)"#,
    r#"CREATE INDEX idx_package_item_type ON package_item(item_type)"#,
    // package_gallery
    r#"CREATE TABLE IF NOT EXISTS package_gallery (
        gallery_id  INT AUTO_INCREMENT PRIMARY KEY,
        package_id  INT NOT NULL,
        image_url   VARCHAR(1024) NOT NULL,
        image_type  VARCHAR(32) DEFAULT 'sample',
        caption     VARCHAR(255),
        sort_order  INT DEFAULT 0,
        create_time DATETIME DEFAULT CURRENT_TIMESTAMP
    ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4"#,
    r#"CREATE INDEX idx_package_gallery_package_id ON package_gallery(package_id)"#,
];

const DOWN_STATEMENTS: &[&str] = &[
    "DROP TABLE IF EXISTS package_gallery",
    "DROP TABLE IF EXISTS package_item",
    "DROP TABLE IF EXISTS package",
];

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for stmt in STATEMENTS {
            manager.get_connection().execute_unprepared(stmt).await?;
        }
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for stmt in DOWN_STATEMENTS {
            manager.get_connection().execute_unprepared(stmt).await?;
        }
        Ok(())
    }
}
