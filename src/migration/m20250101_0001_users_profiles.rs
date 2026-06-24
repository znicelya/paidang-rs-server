//! Migration 0001 — users + profiles.
//!
//! Ported from `paidang-worker-server/migrations/0001_users_profiles_tables.sql`.
//! Changes for MySQL (spec §4.1): `AUTO_INCREMENT`, `CURRENT_TIMESTAMP` /
//! `ON UPDATE CURRENT_TIMESTAMP` (drops the `update_*_trigger` SQLite triggers),
//! `TINYINT` for boolean-ish flags, **no foreign keys** (decision #10).

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

const STATEMENTS: &[&str] = &[
    // user — primary account table
    r#"CREATE TABLE IF NOT EXISTS `user` (
        user_id          INT AUTO_INCREMENT PRIMARY KEY,
        openid           VARCHAR(64) NOT NULL UNIQUE,
        unionid          VARCHAR(64),
        session_key      VARCHAR(128),
        role             TINYINT NOT NULL DEFAULT 0,
        phone            VARCHAR(20),
        status           TINYINT NOT NULL DEFAULT 1,
        last_login_time  DATETIME,
        create_time      DATETIME DEFAULT CURRENT_TIMESTAMP,
        update_time      DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
    ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4"#,
    r#"CREATE INDEX idx_user_phone ON `user`(phone)"#,
    // user_profile — one-to-one profile (UNIQUE on user_id; no FK)
    r#"CREATE TABLE IF NOT EXISTS user_profile (
        profile_id        INT AUTO_INCREMENT PRIMARY KEY,
        user_id           INT NOT NULL UNIQUE,
        nickname          VARCHAR(128),
        avatar_url        VARCHAR(1024),
        background_image  VARCHAR(1024),
        gender            TINYINT DEFAULT 0,
        country           VARCHAR(64),
        province          VARCHAR(64),
        city              VARCHAR(64),
        birthday          VARCHAR(16),
        bio               TEXT,
        create_time       DATETIME DEFAULT CURRENT_TIMESTAMP,
        update_time       DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
    ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4"#,
];

const DOWN_STATEMENTS: &[&str] = &[
    "DROP TABLE IF EXISTS user_profile",
    "DROP TABLE IF EXISTS `user`",
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
