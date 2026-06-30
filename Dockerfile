# ── Stage 1: Builder ──────────────────────────────────
FROM lukemathwalker/cargo-chef:latest-rust-1.91-bookworm AS chef
WORKDIR /app

# ── Stage 2: Plan (cargo-chef caches dependency layer) ──
FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
COPY src/migration/ src/migration/
RUN cargo chef prepare --recipe-path recipe.json

# ── Stage 3: Build dependencies ──
FROM chef AS builder
# 使用 HTTP 清华源，避免证书问题
RUN echo "deb http://mirrors.tuna.tsinghua.edu.cn/debian/ bookworm main contrib non-free non-free-firmware" > /etc/apt/sources.list && \
    echo "deb http://mirrors.tuna.tsinghua.edu.cn/debian/ bookworm-updates main contrib non-free non-free-firmware" >> /etc/apt/sources.list && \
    echo "deb http://mirrors.tuna.tsinghua.edu.cn/debian/ bookworm-backports main contrib non-free non-free-firmware" >> /etc/apt/sources.list && \
    echo "deb http://mirrors.tuna.tsinghua.edu.cn/debian-security bookworm-security main contrib non-free non-free-firmware" >> /etc/apt/sources.list && \
    apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# ── Stage 4: Build application ──
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
COPY src/migration/ src/migration/
COPY config/ config/
RUN cargo build --release --bin paidang-rs-server

# ── Stage 5: Runtime ───────────────────────────────────
FROM debian:bookworm-slim AS runtime

# 同样使用 HTTP 清华源
RUN echo "deb http://mirrors.tuna.tsinghua.edu.cn/debian/ bookworm main contrib non-free non-free-firmware" > /etc/apt/sources.list && \
    echo "deb http://mirrors.tuna.tsinghua.edu.cn/debian/ bookworm-updates main contrib non-free non-free-firmware" >> /etc/apt/sources.list && \
    echo "deb http://mirrors.tuna.tsinghua.edu.cn/debian/ bookworm-backports main contrib non-free non-free-firmware" >> /etc/apt/sources.list && \
    echo "deb http://mirrors.tuna.tsinghua.edu.cn/debian-security bookworm-security main contrib non-free non-free-firmware" >> /etc/apt/sources.list && \
    apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/paidang-rs-server /app/paidang-rs-server
COPY config/ /app/config/

ENV RUN_ENV=production

EXPOSE 8787

ENTRYPOINT ["/app/paidang-rs-server"]
