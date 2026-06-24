# paidang-rs-server

Self-hosted Rust rewrite of `paidang-worker-server` (Cloudflare Worker / TypeScript).
Serves the same API surface to the `paidang-mini` WeChat mini-program, replacing
Cloudflare's D1/R2/KV with self-hosted **MySQL** + **Tencent Cloud COS**.

- Web framework: **axum**
- ORM: **SeaORM** (MySQL)
- Auth: **JWT** (replaces the `X-User-Id` header)
- Object storage: **Tencent COS** (`qcos`)
- Image moderation: **Qiniu** (HMAC-SHA1 signed)
- Docs: `docs/superpowers/specs/2026-06-24-paidang-rs-server-design.md`

## Configuration

Secrets and connection strings come from environment variables (`.env` in dev).
Non-sensitive defaults live in `config/default.toml` (+ `config/production.toml`,
selected by `RUN_ENV`). See `.env.example` for all variables.

## Run

```bash
cp .env.example .env   # fill in secrets
cargo run
```

Health check: `GET /` → `ok`.

## Development status

See the implementation plan. Milestones:

1. Scaffold + config + DB schema + qcos smoke
2. Auth axis (JWT + WeChat login)
3. Core business domains (bookings + slots)
4. Content domains (packages + gallery)
5. Files + external integrations
6. Logging/monitor + deploy
7. Integration tests + cutover
