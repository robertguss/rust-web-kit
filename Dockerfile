# syntax=docker/dockerfile:1

# Multi-stage: pnpm SPA → cargo-chef / cargo build (BuildKit caches) → debian-slim.
# exe.dev HTTPS proxy prefers EXPOSE 80, else the smallest exposed TCP port >= 1024.

FROM node:lts-bookworm AS web
WORKDIR /web
RUN corepack enable && corepack prepare pnpm@12.4.1 --activate
COPY apps/web/package.json apps/web/pnpm-lock.yaml ./
RUN --mount=type=cache,target=/root/.local/share/pnpm/store \
    pnpm install --frozen-lockfile
COPY apps/web ./
RUN pnpm build

FROM rust:bookworm AS chef
WORKDIR /src
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo install cargo-chef --locked --version 0.1.78

FROM chef AS planner
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates ./crates
COPY migrations ./migrations
COPY .sqlx ./.sqlx
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /src/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/src/target \
    cargo chef cook --release --recipe-path recipe.json
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates ./crates
COPY migrations ./migrations
COPY .sqlx ./.sqlx
ENV SQLX_OFFLINE=true
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/src/target \
    cargo build --release -p rwk-api \
    && cp /src/target/release/rwk-api /usr/local/bin/rwk-api

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --system --gid 1000 rwk \
    && useradd --system --uid 1000 --gid rwk --home-dir /app --create-home rwk
COPY --from=builder /usr/local/bin/rwk-api /usr/local/bin/rwk-api
COPY --from=web /web/dist /app/www
USER rwk
WORKDIR /app
ENV RWK_SERVER__HOST=0.0.0.0 \
    RWK_SERVER__PORT=8080 \
    RWK_SERVER__STATIC_DIR=/app/www \
    RWK_RUN_MIGRATIONS=true
EXPOSE 8080
HEALTHCHECK --interval=10s --timeout=3s --start-period=15s --retries=5 \
    CMD curl -fsS http://127.0.0.1:8080/api/health || exit 1
CMD ["rwk-api"]
