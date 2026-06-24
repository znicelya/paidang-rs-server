//! Migration 0004 — slot templates / date slots / date settings / bookings / logs.
//!
//! Ported from `paidang-worker-server/migrations/0004_bookings_tables.sql`.
//!
//! **Business triggers removed** (decision #8): `lock_slot_on_booking` and
//! `release_slot_on_booking_status` are NOT recreated in MySQL — their logic
//! moves into the bookings service layer (transactions + `SELECT ... FOR UPDATE`,
//! see `domain/bookings/service.rs`). The `update_*_trigger` auto-timestamp
//! triggers are also dropped (handled by `ON UPDATE CURRENT_TIMESTAMP`).
//! All foreign keys removed (decision #10).

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

const STATEMENTS: &[&str] = &[
    // time_slot_template
    r#"CREATE TABLE IF NOT EXISTS time_slot_template (
        template_id      INT AUTO_INCREMENT PRIMARY KEY,
        photographer_id  INT NOT NULL,
        slot_name        VARCHAR(64) NOT NULL,
        start_time       VARCHAR(8) NOT NULL,
        end_time         VARCHAR(8) NOT NULL,
        sort_order       INT DEFAULT 0,
        is_default       TINYINT DEFAULT 0,
        status           TINYINT DEFAULT 1,
        create_time      DATETIME DEFAULT CURRENT_TIMESTAMP,
        update_time      DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
        UNIQUE KEY uq_template_photographer_name (photographer_id, slot_name)
    ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4"#,
    r#"CREATE INDEX idx_template_photographer ON time_slot_template(photographer_id)"#,
    // date_slot
    r#"CREATE TABLE IF NOT EXISTS date_slot (
        slot_instance_id  INT AUTO_INCREMENT PRIMARY KEY,
        photographer_id   INT NOT NULL,
        template_id       INT,
        slot_date         VARCHAR(16) NOT NULL,
        slot_name         VARCHAR(64) NOT NULL,
        start_time        VARCHAR(8) NOT NULL,
        end_time          VARCHAR(8) NOT NULL,
        is_booked         TINYINT DEFAULT 0,
        booking_id        INT,
        is_special        TINYINT DEFAULT 0,
        status            TINYINT DEFAULT 1,
        price             INT,
        remark            VARCHAR(255),
        create_time       DATETIME DEFAULT CURRENT_TIMESTAMP,
        update_time       DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
        UNIQUE KEY uq_dateslot (photographer_id, slot_date, start_time, end_time)
    ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4"#,
    r#"CREATE INDEX idx_dateslot_query ON date_slot(photographer_id, slot_date, is_booked, status)"#,
    r#"CREATE INDEX idx_dateslot_booking ON date_slot(booking_id)"#,
    r#"CREATE INDEX idx_dateslot_date ON date_slot(slot_date)"#,
    // date_setting
    r#"CREATE TABLE IF NOT EXISTS date_setting (
        setting_id       INT AUTO_INCREMENT PRIMARY KEY,
        photographer_id  INT NOT NULL,
        target_date      VARCHAR(16) NOT NULL,
        start_time       VARCHAR(8),
        end_time         VARCHAR(8),
        is_available     TINYINT DEFAULT 1,
        use_template_id  INT,
        reason           VARCHAR(255),
        create_time      DATETIME DEFAULT CURRENT_TIMESTAMP,
        update_time      DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
        UNIQUE KEY uq_datesetting (photographer_id, target_date, start_time, end_time)
    ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4"#,
    r#"CREATE INDEX idx_datesetting_date ON date_setting(photographer_id, target_date)"#,
    // booking
    r#"CREATE TABLE IF NOT EXISTS booking (
        booking_id           INT AUTO_INCREMENT PRIMARY KEY,
        booking_no           VARCHAR(64) NOT NULL UNIQUE,
        user_id              INT,
        photographer_id      INT NOT NULL,
        slot_instance_id     INT,
        package_id           INT,
        booking_date         VARCHAR(16) NOT NULL,
        start_time           VARCHAR(8) NOT NULL,
        end_time             VARCHAR(8) NOT NULL,
        total_amount         INT DEFAULT 0,
        deposit_amount       INT DEFAULT 0,
        paid_amount          INT DEFAULT 0,
        status               VARCHAR(32) DEFAULT 'pending',
        cancel_reason        VARCHAR(255),
        cancel_time          DATETIME,
        customer_name        VARCHAR(64) NOT NULL,
        customer_phone       VARCHAR(32) NOT NULL,
        customer_remark      VARCHAR(255),
        photographer_remark  VARCHAR(255),
        reminder_sent        TINYINT DEFAULT 0,
        create_time          DATETIME DEFAULT CURRENT_TIMESTAMP,
        update_time          DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
    ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4"#,
    r#"CREATE INDEX idx_booking_user ON booking(user_id)"#,
    r#"CREATE INDEX idx_booking_photographer ON booking(photographer_id)"#,
    r#"CREATE INDEX idx_booking_date ON booking(booking_date)"#,
    r#"CREATE INDEX idx_booking_status ON booking(status)"#,
    r#"CREATE INDEX idx_booking_slot ON booking(slot_instance_id)"#,
    // booking_log
    r#"CREATE TABLE IF NOT EXISTS booking_log (
        log_id         INT AUTO_INCREMENT PRIMARY KEY,
        booking_id     INT NOT NULL,
        action         VARCHAR(64) NOT NULL,
        from_status    VARCHAR(32),
        to_status      VARCHAR(32),
        operator_id    INT,
        operator_type  VARCHAR(32),
        remark         VARCHAR(255),
        create_time    DATETIME DEFAULT CURRENT_TIMESTAMP
    ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4"#,
    r#"CREATE INDEX idx_booking_log_booking ON booking_log(booking_id)"#,
];

const DOWN_STATEMENTS: &[&str] = &[
    "DROP TABLE IF EXISTS booking_log",
    "DROP TABLE IF EXISTS booking",
    "DROP TABLE IF EXISTS date_setting",
    "DROP TABLE IF EXISTS date_slot",
    "DROP TABLE IF EXISTS time_slot_template",
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
