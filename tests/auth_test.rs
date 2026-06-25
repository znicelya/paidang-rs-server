//! Auth integration tests.
//!
//! Requires Docker (testcontainers-rs).

mod common;

use sea_orm::{ActiveModelTrait, EntityTrait, Set};

use common::setup;
use paidang_rs_server::entity::user;
use paidang_rs_server::middleware::auth;

#[tokio::test]
async fn jwt_sign_and_verify() {
    let claims = auth::Claims {
        sub: 42,
        openid: "test-openid-42".into(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as u64,
    };

    let token = auth::sign_jwt(claims.clone(), "test-secret").unwrap();
    let decoded = auth::verify_jwt(&token, "test-secret").unwrap();

    assert_eq!(decoded.sub, 42);
    assert_eq!(decoded.openid, "test-openid-42");
}

#[tokio::test]
async fn jwt_verify_invalid_token_fails() {
    let result = auth::verify_jwt("garbage-token", "test-secret");
    assert!(result.is_err());
}

#[tokio::test]
async fn jwt_verify_wrong_secret_fails() {
    let claims = auth::Claims {
        sub: 1,
        openid: "o".into(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as u64,
    };
    let token = auth::sign_jwt(claims, "secret-a").unwrap();
    let result = auth::verify_jwt(&token, "secret-b");
    assert!(result.is_err());
}

#[tokio::test]
async fn jwt_expired_token_fails() {
    let claims = auth::Claims {
        sub: 1,
        openid: "o".into(),
        exp: (chrono::Utc::now() - chrono::Duration::hours(1)).timestamp() as u64,
    };
    let token = auth::sign_jwt(claims, "test-secret").unwrap();
    let result = auth::verify_jwt(&token, "test-secret");
    assert!(result.is_err());
}

// 鈹€鈹€ DB-backed tests 鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€

#[tokio::test]
async fn user_insert_and_find() {
    let ctx = setup().await;

    let m = user::ActiveModel {
        openid: Set("wx-test-openid".into()),
        role: Set(0),
        status: Set(1),
        ..Default::default()
    }
    .insert(&ctx.state.db)
    .await
    .unwrap();

    assert!(m.user_id > 0);
    assert_eq!(m.openid, "wx-test-openid");

    let found = user::Entity::find_by_id(m.user_id)
        .one(&ctx.state.db)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(found.openid, "wx-test-openid");
}
