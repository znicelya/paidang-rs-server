use paidang_rs_server::error::AppError;
use paidang_rs_server::middleware::auth::AuthUser;
use paidang_rs_server::util::require_owner;

#[test]
fn provider_can_access_own_photographer_resource() {
    let auth = AuthUser {
        user_id: 10,
        openid: "provider-10".into(),
    };

    assert!(require_owner(&auth, 10).is_ok());
}

#[test]
fn provider_cannot_access_another_photographer_resource() {
    let auth = AuthUser {
        user_id: 10,
        openid: "provider-10".into(),
    };

    let err = require_owner(&auth, 99).unwrap_err();
    assert!(matches!(err, AppError::Forbidden(_)));
}
