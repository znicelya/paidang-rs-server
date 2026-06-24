//! Integration test utilities.
//!
//! Provides a test harness that spins up a MySQL container via testcontainers,
//! runs SeaORM migrations, and returns an `AppState` ready for test use.

use std::sync::Arc;

use sea_orm::{ConnectOptions, Database};
use testcontainers::runners::AsyncRunner;
use testcontainers::ImageExt;
use testcontainers_modules::mysql::Mysql;

use paidang_rs_server::app_state::AppState;
use paidang_rs_server::config;
use paidang_rs_server::middleware::auth;
use paidang_rs_server::migration;
use paidang_rs_server::migration::MigratorTrait;

/// A test context holding the DB container and AppState.
/// The container is dropped when this struct is dropped.
pub struct TestContext {
    pub state: AppState,
    #[allow(dead_code)]
    container: Option<testcontainers::ContainerAsync<Mysql>>,
    #[allow(dead_code)]
    pub jwt_secret: String,
}

/// Spin up a MySQL container, run migrations, return a ready-to-use TestContext.
/// The JWT secret is fixed to "test-secret" for predictable test token generation.
pub async fn setup() -> TestContext {
    // Try to start MySQL container; if Docker is not available, skip.
    let container = Mysql::default()
        .with_tag("8.4")
        .start()
        .await;

    let (state, container) = match container {
        Ok(container) => {
            let port = container.get_host_port_ipv4(3306).await.unwrap();
            let db_url = format!("mysql://root:test@127.0.0.1:{port}/testdb");

            let root_url = format!("mysql://root:test@127.0.0.1:{port}");
            let root_db = Database::connect(ConnectOptions::new(root_url.clone()))
                .await
                .unwrap();
            sea_orm::ConnectionTrait::execute_unprepared(
                &root_db,
                "CREATE DATABASE testdb CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci",
            )
            .await
            .unwrap();
            let _ = root_db.close().await;

            let mut opt = ConnectOptions::new(db_url.clone());
            opt.max_connections(5);
            let db = Database::connect(opt).await.unwrap();
            migration::Migrator::up(&db, None).await.unwrap();

            let settings = config::Settings {
                env: "test".into(),
                server: config::ServerSettings { host: "0.0.0.0".into(), port: 9999 },
                database: config::DatabaseSettings { pool_size: 5 },
                jwt: config::JwtSettings { expires_secs: 86400 },
                pagination: config::PaginationSettings { default_page: 1, default_page_size: 20, max_page_size: 100 },
                database_url: Some(db_url),
                jwt_secret: Some("test-secret".into()),
                wechat_appid: None,
                wechat_secret: None,
                qiniu_access_key: None,
                qiniu_secret_key: None,
                cos_secret_id: None,
                cos_secret_key: None,
                cos_bucket: None,
                cos_region: None,
            };
            let state = AppState::new(Arc::new(settings), db);
            (Some(state), Some(container))
        }
        Err(e) => {
            // Docker not available — panic with helpful message
            panic!("MySQL container could not be started: {e}. Is Docker running?")
        }
    };

    let state = state.unwrap();

    TestContext {
        state,
        container,
        jwt_secret: "test-secret".into(),
    }
}

/// Generate a valid JWT for the given user_id and role.
#[allow(dead_code)]
pub fn test_jwt(user_id: i32, role: i8, secret: &str) -> String {
    use jsonwebtoken::{EncodingKey, Header};
    let now = chrono::Utc::now();
    let claims = auth::Claims {
        sub: user_id,
        openid: format!("test-openid-{user_id}"),
        role,
        exp: (now + chrono::Duration::hours(1)).timestamp() as u64,
    };
    jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap()
}

/// Generate an expired JWT.
#[allow(dead_code)]
pub fn expired_jwt(user_id: i32, role: i8, secret: &str) -> String {
    use jsonwebtoken::{EncodingKey, Header};
    let claims = auth::Claims {
        sub: user_id,
        openid: format!("test-openid-{user_id}"),
        role,
        exp: (chrono::Utc::now() - chrono::Duration::hours(1)).timestamp() as u64,
    };
    jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap()
}
