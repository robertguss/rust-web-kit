# rust-web-kit

AI-first full-stack starter: Rust (Axum) API + React (Vite) frontend, shipped as a single Docker image.

The placeholder project name is `rwk` (crates, npm package, database, compose project). Rename it with `just init NAME` once Phase 8 lands.

## Quickstart

```bash
mise install
just setup
just db-up
just dev-api
```

Health check: `curl localhost:8080/api/health` → `{"status":"ok"}`.

Vite (later): `just dev-web` on port 5173.

See `docs/PLAN.md` for the full stack and phase plan.
