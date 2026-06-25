# Provider Login And Public Customer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace role-based authorization with a provider-login model and make customer booking/availability endpoints public.

**Architecture:** JWT becomes identity-only and means "logged-in provider". Public customer endpoints remove `AuthUser` extraction entirely. Provider management endpoints keep `AuthUser` but scope provider-owned resources to `AuthUser.user_id` with no admin bypass.

**Tech Stack:** Rust 2024, axum extractors, SeaORM services, jsonwebtoken, utoipa, tokio integration tests.

---

## File Structure

- `src/middleware/auth.rs`: identity-only `Claims` and `AuthUser`.
- `src/domain/auth/service.rs`: sign JWT without role; keep response `role` only for compatibility.
- `src/util.rs`: strict owner guard only.
- `src/middleware/mod.rs` and `src/middleware/role_guard.rs`: remove exported role guard surface.
- `src/domain/bookings/service.rs`: provider-scoped list and customer/provider create log type.
- `src/domain/bookings/handler.rs`: public create; provider-scoped management.
- `src/domain/date_slots/handler.rs`: public `day` and `monthly`; provider-scoped management.
- `src/domain/date_settings/handler.rs`: public `check`; provider-scoped management.
- `src/domain/packages/handler.rs`, `src/domain/gallery/handler.rs`, `src/domain/gallery_groups/handler.rs`, `src/domain/files/handler.rs`: remove admin role checks.
- `tests/common.rs`, `tests/auth_test.rs`, `tests/bookings_test.rs`: update signatures and assertions.
- `tests/provider_auth_model_test.rs`: prove role values do not grant bypass.

---

### Task 1: Make JWT Claims Identity-Only

**Files:**
- Modify: `src/middleware/auth.rs`
- Modify: `src/domain/auth/service.rs`
- Modify: `tests/common.rs`
- Modify: `tests/auth_test.rs`

- [ ] **Step 1: Write failing auth tests**

In `tests/auth_test.rs`, remove all `role` fields from `auth::Claims` literals and remove `assert_eq!(decoded.role, 1)`. The main test body should be:

```rust
let claims = auth::Claims {
    sub: 42,
    openid: "test-openid-42".into(),
    exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as u64,
};
let token = auth::sign_jwt(claims.clone(), "test-secret").unwrap();
let decoded = auth::verify_jwt(&token, "test-secret").unwrap();
assert_eq!(decoded.sub, 42);
assert_eq!(decoded.openid, "test-openid-42");
```

- [ ] **Step 2: Verify the tests fail**

Run: `cargo test jwt_sign_and_verify jwt_verify_wrong_secret_fails jwt_expired_token_fails`

Expected: compile errors referencing the still-existing `role` field mismatch.

- [ ] **Step 3: Implement identity-only claims**

In `src/middleware/auth.rs`, define:

```rust
pub struct Claims {
    pub sub: i32,
    pub openid: String,
    pub exp: u64,
}

pub struct AuthUser {
    pub user_id: i32,
    pub openid: String,
}
```

Update extraction to:

```rust
Ok(AuthUser { user_id: claims.sub, openid: claims.openid })
```

- [ ] **Step 4: Stop signing role into JWT**

In `src/domain/auth/service.rs`, build claims as:

```rust
let claims = Claims { sub: user_id, openid: record.openid.clone(), exp };
```

Keep `LoginData { role: record.role, ... }` unchanged for response compatibility.

- [ ] **Step 5: Update token helpers**

In `tests/common.rs`, change helper signatures to:

```rust
pub fn test_jwt(user_id: i32, secret: &str) -> String
pub fn expired_jwt(user_id: i32, secret: &str) -> String
```

Both helpers must construct:

```rust
auth::Claims {
    sub: user_id,
    openid: format!("test-openid-{user_id}"),
    exp,
}
```

- [ ] **Step 6: Verify and commit**

Run: `cargo test jwt_`

Expected: all JWT tests pass.

Commit:

```bash
git add src/middleware/auth.rs src/domain/auth/service.rs tests/common.rs tests/auth_test.rs
git commit -m "refactor: make jwt provider identity only"
```

---

### Task 2: Replace Role Guards With Strict Provider Ownership

**Files:**
- Modify: `src/util.rs`
- Modify: `src/middleware/mod.rs`
- Delete: `src/middleware/role_guard.rs`
- Add: `tests/provider_auth_model_test.rs`

- [ ] **Step 1: Write failing ownership tests**

Create `tests/provider_auth_model_test.rs`:

```rust
use paidang_rs_server::error::AppError;
use paidang_rs_server::middleware::auth::AuthUser;
use paidang_rs_server::util::require_owner;

#[test]
fn provider_can_access_own_photographer_resource() {
    let auth = AuthUser { user_id: 10, openid: "provider-10".into() };
    assert!(require_owner(&auth, 10).is_ok());
}

#[test]
fn provider_cannot_access_another_photographer_resource() {
    let auth = AuthUser { user_id: 10, openid: "provider-10".into() };
    let err = require_owner(&auth, 99).unwrap_err();
    assert!(matches!(err, AppError::Forbidden(_)));
}
```

- [ ] **Step 2: Verify the tests fail**

Run: `cargo test --test provider_auth_model_test`

Expected: compile failure until `AuthUser` and `require_owner` are updated.

- [ ] **Step 3: Implement strict ownership**

Replace `src/util.rs` with:

```rust
//! Shared authorization guards.
//!
//! A valid JWT represents a logged-in provider. The database `user.role` column
//! is retained for compatibility but is never used for authorization.

use crate::error::AppError;
use crate::middleware::auth::AuthUser;

pub fn require_owner(auth: &AuthUser, photographer_id: i32) -> Result<(), AppError> {
    if auth.user_id == photographer_id {
        Ok(())
    } else {
        Err(AppError::Forbidden("无权操作此资源".into()))
    }
}
```

- [ ] **Step 4: Remove role guard export**

Change `src/middleware/mod.rs` to:

```rust
pub mod auth;
pub mod request_log;
```

Delete the unused role guard file:

```bash
git rm src/middleware/role_guard.rs
```

- [ ] **Step 5: Verify and commit**

Run: `cargo test --test provider_auth_model_test`

Expected: both tests pass.

Commit:

```bash
git add src/util.rs src/middleware/mod.rs tests/provider_auth_model_test.rs
git add -u src/middleware/role_guard.rs
git commit -m "refactor: remove role based authorization guards"
```

---

### Task 3: Make Booking Creation Public And Provider Lists Scoped

**Files:**
- Modify: `src/domain/bookings/service.rs`
- Modify: `src/domain/bookings/handler.rs`
- Modify: `tests/bookings_test.rs`

- [ ] **Step 1: Update booking tests for new service signatures**

In `tests/bookings_test.rs`, public customer creation uses:

```rust
let result = service::create(&ctx.state, &body, None, "customer").await.unwrap();
```

Provider setup creation uses:

```rust
service::create(&ctx.state, &body, Some(pid), "provider").await.unwrap();
```

List calls use:

```rust
let (rows, total) = service::list(&ctx.state, &query, pid).await.unwrap();
```

Add assertions after reading creation logs:

```rust
assert_eq!(logs[0].operator_id, None);
assert_eq!(logs[0].operator_type.as_deref(), Some("customer"));
```

- [ ] **Step 2: Add scoped list regression test**

Add a test that seeds two photographers, creates one booking for each, calls `service::list(&ctx.state, &query_with_other_photographer_id, first_pid)`, and asserts every returned row has `photographer_id == first_pid`.

Use this assertion block:

```rust
assert_eq!(total, 1);
assert_eq!(rows.len(), 1);
assert_eq!(rows[0].photographer_id, pid_a);
```

- [ ] **Step 3: Verify tests fail**

Run: `cargo test --test bookings_test create_booking_success list_bookings_pagination`

Expected: compile errors because service signatures still include old parameters.

- [ ] **Step 4: Update booking service**

Change list signature to:

```rust
pub async fn list(state: &AppState, query: &BookingListQuery, provider_id: i32) -> Result<(Vec<booking::Model>, u64), AppError>
```

Start the query with:

```rust
let mut select = booking::Entity::find()
    .filter(booking::Column::PhotographerId.eq(provider_id));
```

Change create signature to:

```rust
pub async fn create(state: &AppState, body: &CreateBookingRequest, operator_id: Option<i32>, operator_type: &str) -> Result<CreateBookingData, AppError>
```

Pass `operator_type` into the create log:

```rust
insert_log(&txn, inserted.booking_id, "created", None, Some(status), operator_id, operator_type, None).await?;
```

- [ ] **Step 5: Update booking handlers**

In `src/domain/bookings/handler.rs`:

- `create` removes `auth: AuthUser` and calls `service::create(&state, &body, None, "customer")`.
- `list` calls `service::list(&state, &q, auth.user_id)`.
- `stats` uses `service::stats(&state, Some(auth.user_id))`.
- `today` sets `photographer_id: Some(auth.user_id)` and calls `service::list(&state, &query, auth.user_id)`.
- `update` calls `service::update(&state, id, &body, Some(auth.user_id))`.

- [ ] **Step 6: Verify and commit**

Run: `cargo test --test bookings_test`

Expected: booking service tests pass.

Commit:

```bash
git add src/domain/bookings/service.rs src/domain/bookings/handler.rs tests/bookings_test.rs
git commit -m "refactor: make customer booking creation public"
```

---

### Task 4: Make Availability Queries Public And Keep Management Scoped

**Files:**
- Modify: `src/domain/date_slots/handler.rs`
- Modify: `src/domain/date_settings/handler.rs`

- [ ] **Step 1: Update date slot management scoping**

In `src/domain/date_slots/handler.rs`, list must ignore query `photographer_id` for providers:

```rust
let (rows, total) = service::list(&state, &q, Some(auth.user_id)).await?;
```

Keep `require_owner(&auth, body.photographer_id)?` for create and strict owner checks for read/update/delete.

- [ ] **Step 2: Make date slot customer queries public**

Change `day` signature to remove `auth: AuthUser` and remove `require_owner`:

```rust
pub async fn day(
    State(state): State<AppState>,
    Query(q): Query<DayQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let rows = service::day(&state, q.photographer_id, &q.slot_date).await?;
```

Change `monthly` the same way, calling `service::monthly(&state, q.photographer_id, &q.year_month)`.

- [ ] **Step 3: Update date setting management scoping**

In `src/domain/date_settings/handler.rs`, list must ignore query `photographer_id` for providers:

```rust
let (rows, total) = service::list(&state, &q, Some(auth.user_id)).await?;
```

Keep strict owner checks for read/create/update/delete.

- [ ] **Step 4: Make date setting check public**

Change `check` signature to remove `auth: AuthUser` and remove `require_owner`:

```rust
pub async fn check(
    State(state): State<AppState>,
    Query(q): Query<CheckQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let v = service::check(&state, q.photographer_id, &q.target_date).await?;
    Ok(Json(ApiResponse::ok(v)))
}
```

- [ ] **Step 5: Verify and commit**

Run: `cargo check`

Expected: no compile errors from public date handler signatures.

Commit:

```bash
git add src/domain/date_slots/handler.rs src/domain/date_settings/handler.rs
git commit -m "refactor: make availability queries public"
```

---

### Task 5: Remove Admin Checks From Content And File Writes

**Files:**
- Modify: `src/domain/packages/handler.rs`
- Modify: `src/domain/gallery/handler.rs`
- Modify: `src/domain/gallery_groups/handler.rs`
- Modify: `src/domain/files/handler.rs`

- [ ] **Step 1: Remove admin imports and checks**

In each handler file, delete:

```rust
use crate::util::require_admin;
```

Delete every line:

```rust
require_admin(&auth)?;
```

Keep `auth: AuthUser` parameters for write handlers. If a handler only used auth for the deleted guard, rename the parameter to `_auth: AuthUser`.

- [ ] **Step 2: Preserve creator/updater identity where already used**

Keep calls such as:

```rust
service::create_package(&state, &body, auth.user_id).await?;
service::update(&state, id, body, auth.user_id).await?;
```

Do not replace those `auth` variables with `_auth`.

- [ ] **Step 3: Verify and commit**

Run: `cargo check`

Expected: no unresolved `require_admin` imports and no unused `auth` warnings that block compilation.

Commit:

```bash
git add src/domain/packages/handler.rs src/domain/gallery/handler.rs src/domain/gallery_groups/handler.rs src/domain/files/handler.rs
git commit -m "refactor: allow logged in providers to manage content"
```

---

### Task 6: Final Role Reference Cleanup And Verification

**Files:**
- Modify comments in touched files as needed
- Modify: `src/openapi.rs` only if descriptions still say admin-only
- Modify: `README.md` or existing docs only if they say role-based guards are active

- [ ] **Step 1: Search for remaining runtime role authorization**

Run:

```bash
rg -n "auth\.role|require_admin|role >=|role <|role_guard|Admin|Photographer" src tests
```

Expected: no runtime authorization references remain. Allowed matches are database fields, response DTO fields, migration columns, and tests intentionally setting `role` to prove it is ignored.

- [ ] **Step 2: Update stale API docs and comments**

Replace phrases like `write admin` and `admin only` in handler comments/OpenAPI descriptions with `write provider` or `login required`.

Example replacement:

```rust
/// POST /packages — create a new package (provider login required).
```

- [ ] **Step 3: Run full verification**

Run:

```bash
cargo fmt --check
cargo check
cargo test jwt_
cargo test --test provider_auth_model_test
cargo test --test bookings_test
```

Expected: all commands pass. If `cargo test --test bookings_test` cannot run because Docker is unavailable, record the exact Docker/testcontainers error and still run `cargo check`.

- [ ] **Step 4: Commit final cleanup**

Run:

```bash
git add README.md src/openapi.rs src tests
git commit -m "docs: align auth wording with provider model"
```

If Step 2 made no file changes, skip this commit and record that no cleanup commit was needed.

---

## Self Review

Spec coverage:

- Identity-only JWT: Task 1.
- Database role retained but unused for authorization: Tasks 1, 2, and 6.
- Public booking creation: Task 3.
- Public date slot day/monthly and date setting check: Task 4.
- Provider-scoped management without admin bypass: Tasks 2, 3, 4, and 5.
- Tests for JWT and ownership behavior: Tasks 1, 2, 3, and 6.

Placeholder scan:

- The plan contains no unresolved markers, deferred implementation placeholders, or undefined task references.

Type consistency:

- `Claims` consistently uses `sub`, `openid`, `exp`.
- `AuthUser` consistently uses `user_id`, `openid`.
- `bookings::service::list` consistently accepts `(state, query, provider_id)`.
- `bookings::service::create` consistently accepts `(state, body, operator_id, operator_type)`.

