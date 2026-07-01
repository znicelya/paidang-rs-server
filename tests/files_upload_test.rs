use std::sync::Arc;

use sea_orm::DatabaseConnection;

use paidang_rs_server::app_state::AppState;
use paidang_rs_server::config;
use paidang_rs_server::domain::files::dto::{ModerateUploadRequest, UploadPolicyRequest};
use paidang_rs_server::domain::files::service;
use paidang_rs_server::error::AppError;

#[tokio::test]
async fn image_upload_requires_moderation_before_cos_upload() {
    let state = test_state_with_cos_without_moderation();

    let err = service::upload(
        &state,
        b"fake-image".to_vec(),
        "avatar.jpg",
        "image/jpeg",
        "avatars/",
    )
    .await
    .expect_err("upload must fail before COS when moderation is unavailable");

    assert_eq!(err.to_string(), "图片审核服务未配置，禁止上传");
}

#[test]
fn direct_cos_upload_policy_is_disabled() {
    let state = test_state_with_cos_without_moderation();

    let err = service::upload_policy(
        &state,
        1,
        UploadPolicyRequest {
            prefix: Some("gallery".into()),
            file_name: "sample.jpg".into(),
            content_type: Some("image/jpeg".into()),
        },
    )
    .expect_err("direct COS policy must be disabled");

    assert!(matches!(err, AppError::InputValidation(_)));
    assert_eq!(
        err.to_string(),
        "直传 COS 已禁用，请通过 /files 上传并完成审核"
    );
}

#[tokio::test]
async fn post_upload_moderation_is_disabled() {
    let state = test_state_with_cos_without_moderation();

    let err = service::moderate_uploaded_object(
        &state,
        ModerateUploadRequest {
            key: "gallery/sample.jpg".into(),
            content_type: Some("image/jpeg".into()),
        },
    )
    .await
    .expect_err("post-upload moderation must be disabled");

    assert!(matches!(err, AppError::InputValidation(_)));
    assert_eq!(
        err.to_string(),
        "上传后审核接口已禁用，请通过 /files 上传并完成审核"
    );
}

fn test_state_with_cos_without_moderation() -> AppState {
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
        jwt_secret: Some("test-secret".into()),
        wechat_appid: None,
        wechat_secret: None,
        qiniu_access_key: None,
        qiniu_secret_key: None,
        cos_secret_id: Some("cos-secret-id".into()),
        cos_secret_key: Some("cos-secret-key".into()),
        cos_bucket: Some("paidang-test".into()),
        cos_region: Some("ap-guangzhou".into()),
    };
    AppState::new(Arc::new(settings), DatabaseConnection::Disconnected)
}
