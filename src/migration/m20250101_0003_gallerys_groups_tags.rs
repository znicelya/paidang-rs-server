//! Migration 0003 — gallery groups + gallery + tags.
//!
//! Ported from `paidang-worker-server/migrations/0003_gallerys_groups_tags_tables.sql`.
//! `image_list` is JSON. No FK (group_id relation is app-managed).

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

const STATEMENTS: &[&str] = &[
    // gallery_group
    r#"CREATE TABLE IF NOT EXISTS gallery_group (
        group_id     INT AUTO_INCREMENT PRIMARY KEY,
        name         VARCHAR(255) NOT NULL,
        cover_image  VARCHAR(1024),
        description  TEXT,
        sort_order   INT DEFAULT 0,
        is_visible   TINYINT DEFAULT 1,
        status       TINYINT DEFAULT 1,
        create_time  DATETIME DEFAULT CURRENT_TIMESTAMP,
        update_time  DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
        create_by    INT,
        update_by    INT
    ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4"#,
    r#"CREATE INDEX idx_gallery_group_sort ON gallery_group(sort_order)"#,
    r#"CREATE INDEX idx_gallery_group_status ON gallery_group(status)"#,
    // gallery
    r#"CREATE TABLE IF NOT EXISTS gallery (
        gallery_id         INT AUTO_INCREMENT PRIMARY KEY,
        group_id           INT,
        title              VARCHAR(255) NOT NULL,
        subtitle           VARCHAR(255),
        cover_image        VARCHAR(1024),
        image_url          VARCHAR(1024),
        image_list         JSON,
        video_url          VARCHAR(1024),
        media_type         VARCHAR(16) DEFAULT 'image',
        tags               VARCHAR(255),
        photographer_id    INT,
        photographer_name  VARCHAR(128),
        shooting_location  VARCHAR(128),
        shooting_date      VARCHAR(16),
        width              INT,
        height            INT,
        file_size         INT,
        view_count        INT DEFAULT 0,
        like_count        INT DEFAULT 0,
        sort_order        INT DEFAULT 0,
        is_cover          TINYINT DEFAULT 0,
        status            TINYINT DEFAULT 1,
        create_time       DATETIME DEFAULT CURRENT_TIMESTAMP,
        update_time       DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
        create_by         INT,
        update_by         INT
    ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4"#,
    r#"CREATE INDEX idx_gallery_group_id ON gallery(group_id)"#,
    r#"CREATE INDEX idx_gallery_status ON gallery(status)"#,
    r#"CREATE INDEX idx_gallery_photographer ON gallery(photographer_id)"#,
    r#"CREATE INDEX idx_gallery_view_count ON gallery(view_count)"#,
    r#"CREATE INDEX idx_gallery_like_count ON gallery(like_count)"#,
    r#"CREATE INDEX idx_gallery_create_time ON gallery(create_time)"#,
    // gallery_tag
    r#"CREATE TABLE IF NOT EXISTS gallery_tag (
        tag_id      INT AUTO_INCREMENT PRIMARY KEY,
        tag_name    VARCHAR(64) NOT NULL UNIQUE,
        tag_type    VARCHAR(32) DEFAULT 'style',
        use_count   INT DEFAULT 0,
        sort_order  INT DEFAULT 0,
        create_time DATETIME DEFAULT CURRENT_TIMESTAMP
    ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4"#,
    r#"CREATE INDEX idx_gallery_tag_type ON gallery_tag(tag_type)"#,
    r#"CREATE INDEX idx_gallery_tag_use_count ON gallery_tag(use_count)"#,
];

const DOWN_STATEMENTS: &[&str] = &[
    "DROP TABLE IF EXISTS gallery_tag",
    "DROP TABLE IF EXISTS gallery",
    "DROP TABLE IF EXISTS gallery_group",
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
