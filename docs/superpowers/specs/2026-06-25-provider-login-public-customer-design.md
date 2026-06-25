# paidang-rs-server provider login and public customer design

Date: 2026-06-25
Status: approved design, pending implementation plan

## Background

`paidang-rs-server` currently keeps a `user.role` column and also places `role`
inside JWT claims. Several handlers use `role >= 2` as an admin bypass and
`role < 2` as a photographer-owner filter.

The current business does not use roles in this Rust service. There are only two
runtime actors:

- logged-in provider: the service operator / photographer
- public customer: an unauthenticated customer using public booking and browsing
  flows

The database may keep its `role` column for schema compatibility, but this
project must not use it for authorization decisions.

## Decision

Use a single authorization model:

- JWT means "logged-in provider".
- No JWT means "public customer".
- `user.role` remains stored and may be returned for backward compatibility, but
  it must not be read for access control.
- JWT claims should carry identity only: `sub`, `openid`, and `exp`.
- `AuthUser` should carry identity only: `user_id` and `openid`.

This is scheme A from the design discussion and is intentionally stricter than
the current role model.

## Public Customer API

These endpoints must be callable without `Authorization`:

- `POST /bookings`
- `GET /date-slots/day`
- `GET /date-slots/monthly`
- `GET /date-settings/check`
- existing public read endpoints for packages, package items/gallery, gallery,
  gallery tags, and gallery groups
- file download remains public as it is today

Public booking creation records no authenticated operator:

- `operator_id = None`
- `operator_type = "customer"` for the booking creation log

The request body still supplies the booking customer information and target
`photographer_id`.

## Logged-In Provider API

All non-public management operations require `AuthUser`. A valid token is enough;
there is no admin/photographer distinction.

Examples:

- user profile read/update/avatar upload
- packages, package items, and package gallery writes
- gallery, gallery tag, and gallery group writes
- file upload/list/delete
- time slot template CRUD
- date slot CRUD and list
- date setting CRUD and list
- booking list/read/update/delete/stats/today
- booking log list/read

## Ownership Rules

Logged-in provider resources are scoped by `photographer_id`.

- A provider can only read or mutate records whose `photographer_id` equals
  `AuthUser.user_id`.
- There is no admin bypass.
- List/stat/today endpoints force the photographer filter to `AuthUser.user_id`.
  Query parameters asking for another `photographer_id` are ignored for scoped
  provider views.
- Create endpoints for provider-owned resources must require
  `body.photographer_id == AuthUser.user_id`, or derive the owner from
  `AuthUser.user_id` when the existing DTO already supports that flow.

## Code Changes

Primary changes:

- `src/middleware/auth.rs`
  - remove `role` from `Claims`
  - remove `role` from `AuthUser`
  - keep JWT signing and verification centralized
- `src/domain/auth/service.rs`
  - stop placing database `role` into JWT claims
  - keep `role` in login response only if needed for response compatibility
- `src/util.rs`
  - replace `require_admin` with a provider-login helper if needed, or remove
    calls where `AuthUser` extraction already proves login
  - change owner checks to strict `auth.user_id == photographer_id`
- `src/middleware/role_guard.rs`
  - remove the module or stop exporting it, because no handler should depend on
    role-based extractors
- `src/domain/bookings/service.rs`
  - remove `auth_role` from list filtering
  - make list filtering accept the effective provider id directly
  - allow public booking creation to pass `operator_id = None` and log as
    `customer`
- handlers in `bookings`, `date_slots`, and `date_settings`
  - make customer query/create endpoints public
  - force provider management endpoints to the logged-in provider id
- handlers in `packages`, `gallery`, `gallery_groups`, and `files`
  - remove `require_admin` checks; `AuthUser` extraction is the permission check

Compatibility-only places:

- `src/entity/user.rs` and migrations keep `role`.
- `src/domain/auth/dto.rs`, `src/domain/user/dto.rs`, and profile/login service
  may continue returning `role`, but comments and tests must make clear it is not
  authoritative.

## Tests

Update or add tests for:

- JWT sign/verify without a `role` claim.
- A token remains valid only as provider identity.
- Public customer can create a booking without a token.
- Public customer can call date slot day/monthly and date setting check without a
  token.
- Provider list/stat/today endpoints are always scoped to `AuthUser.user_id`.
- Provider cannot access another provider's booking/date-slot/date-setting even
  if the database `role` value is `2`.
- Package/gallery/group/file write endpoints require login but do not inspect
  `role`.

## Non-Goals

- Do not remove the database `role` column.
- Do not add customer login.
- Do not add multi-provider admin management.
- Do not change mini-program UI behavior beyond what is required by API auth
  compatibility.

## Self Review

- No placeholder requirements remain.
- The design consistently treats JWT as provider identity only.
- Public customer and logged-in provider surfaces are explicitly separated.
- The database `role` field is retained only for compatibility and never used for
  authorization.
