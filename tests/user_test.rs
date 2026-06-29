//! User endpoint integration tests.

use axum::Extension;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tower::ServiceExt;

use paidang_rs_server::app_state::AppState;
use paidang_rs_server::config;
use paidang_rs_server::middleware::auth::JwtSecret;

#[tokio::test]
async fn get_profile_without_auth_reaches_public_handler() {
    let jwt_secret = "test-secret".to_string();
    let state = test_state(jwt_secret.clone());

    let (router, _) = paidang_rs_server::domain::user::router::routes().split_for_parts();
    let app = router
        .layer(Extension(JwtSecret(jwt_secret)))
        .with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/user/profile")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(status, StatusCode::BAD_REQUEST, "{json}");
    assert_eq!(json["success"], false);
    assert_eq!(json["errors"][0]["message"], "user_id is required");
}

#[tokio::test]
async fn avatar_upload_accepts_mini_program_avatar_field() {
    let jwt_secret = "test-secret".to_string();
    let state = test_state(jwt_secret.clone());
    let user_id = 42;
    let token = test_jwt(user_id, &jwt_secret);
    let boundary = "paidang-avatar-boundary";
    let body = format!(
        "--{boundary}\r\n\
         Content-Disposition: form-data; name=\"user_id\"\r\n\r\n\
         {}\r\n\
         --{boundary}\r\n\
         Content-Disposition: form-data; name=\"avatar\"; filename=\"avatar.jpg\"\r\n\
         Content-Type: image/jpeg\r\n\r\n\
         fake-image-bytes\r\n\
         --{boundary}--\r\n",
        user_id
    );

    let (router, _) = paidang_rs_server::domain::user::router::routes().split_for_parts();
    let app = router
        .layer(Extension(JwtSecret(jwt_secret)))
        .with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/user/avatar")
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .header(
                    header::CONTENT_TYPE,
                    format!("multipart/form-data; boundary={boundary}"),
                )
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR, "{json}");
    assert_eq!(json["success"], false);
    assert_eq!(json["errors"][0]["message"], "COS not configured");
}

fn test_state(jwt_secret: String) -> AppState {
    let settings = config::Settings {
        env: "test".into(),
        server: config::ServerSettings {
            host: "0.0.0.0".into(),
            port: 9999,
        },
        database: config::DatabaseSettings { pool_size: 5 },
        jwt: config::JwtSettings {
            expires_secs: 86400,
        },
        pagination: config::PaginationSettings {
            default_page: 1,
            default_page_size: 20,
            max_page_size: 100,
        },
        database_url: None,
        jwt_secret: Some(jwt_secret),
        wechat_appid: None,
        wechat_secret: None,
        qiniu_access_key: None,
        qiniu_secret_key: None,
        cos_secret_id: None,
        cos_secret_key: None,
        cos_bucket: None,
        cos_region: None,
    };
    AppState::new(Arc::new(settings), DatabaseConnection::Disconnected)
}

fn test_jwt(user_id: i32, secret: &str) -> String {
    use jsonwebtoken::{EncodingKey, Header};
    use paidang_rs_server::middleware::auth;

    let now = chrono::Utc::now();
    let claims = auth::Claims {
        sub: user_id,
        openid: format!("test-openid-{user_id}"),
        exp: (now + chrono::Duration::hours(1)).timestamp() as u64,
    };
    jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap()
}
