# 0005. Single image

## Context

The target deploy is one VM (including exe.dev): one thing to build, push, and run. Splitting a static host from the API doubles compose and TLS.

## Decision

Multi-stage Dockerfile: pnpm SPA build → cargo release binary → debian-slim runtime. Axum serves `/api`, `/docs`, and the SPA (`ServeDir` + `index.html` fallback). `RWK_RUN_MIGRATIONS=true` in the image. `docker-compose.prod.yml` is app + Postgres.

## Consequences

Frontend deploys with the API; there is no separate CDN in the default path. SPA fallback must not swallow `/api` or hashed `/assets` 404s. Image size includes the Rust runtime and `www/`.
