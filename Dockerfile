# ── Builder ──────────────────────────────────────────────
FROM rust:1.91-slim-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Cache deps
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Build
COPY src/ src/
COPY config/ config/
RUN cargo build --release

# ── Runtime ──────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/paidang-rs-server /app/paidang-rs-server
COPY config/ /app/config/

ENV RUN_ENV=production

EXPOSE 8787

ENTRYPOINT ["/app/paidang-rs-server"]
