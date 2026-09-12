# Recipes are the agent-facing command vocabulary; CLAUDE.md documents them.
set dotenv-load := false
set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

export DATABASE_URL := env("DATABASE_URL", "postgres://rwk:rwk@localhost:5432/rwk")

# List recipes.
default:
    @just --list

# Install tools, JS deps, and git hooks.
setup:
    mise install
    pnpm --dir apps/web install
    hk install

# Start API + web together (Phase 6).
dev:
    @echo "TODO: just dev (Phase 6)"

# Run the API on :8080.
dev-api:
    cargo run -p rwk-api

# Run the Vite dev server on :5173.
dev-web:
    pnpm --dir apps/web dev

# Start Postgres and Mailpit.
db-up:
    docker compose up -d
    @for i in $(seq 1 30); do \
      if docker compose exec -T postgres pg_isready -U rwk -d rwk >/dev/null 2>&1; then \
        docker compose exec -T postgres pg_isready -U rwk -d rwk; \
        exit 0; \
      fi; \
      sleep 1; \
    done; \
    echo "postgres did not become ready"; \
    exit 1

# Stop local data services.
db-down:
    docker compose down

# Apply SQLx migrations.
migrate:
    sqlx migrate run --source migrations

# Create a new SQLx migration.
migrate-new NAME:
    sqlx migrate add --source migrations --simple {{NAME}}

# Refresh the committed .sqlx offline cache.
sqlx-prepare:
    cargo sqlx prepare --workspace -- --all-targets

# Export OpenAPI and generate the TS client (Phase 4).
gen:
    @echo "TODO: just gen (Phase 4)"

# Format/lint/typecheck everything that exists in this phase.
check:
    cargo fmt --all -- --check
    cargo clippy --all-targets -- -D warnings
    pnpm --dir apps/web typecheck
    pnpm --dir apps/web lint
    cargo sqlx prepare --check --workspace -- --all-targets

# Run API and web tests.
test: test-api test-web

# Rust tests.
test-api:
    cargo nextest run --workspace

# Frontend unit tests (Phase 6).
test-web:
    @echo "TODO: just test-web (Phase 6)"

# Playwright smoke (Phase 7).
e2e:
    @echo "TODO: just e2e (Phase 7)"

# Full CI pipeline (Phase 7).
ci:
    @echo "TODO: just ci (Phase 7)"

# Release-ish local build.
build:
    cargo build --release -p rwk-api
    pnpm --dir apps/web build

# Build the production image (Phase 7).
docker-build:
    @echo "TODO: just docker-build (Phase 7)"

# Deploy the image to a VM over SSH (Phase 7).
deploy HOST:
    @echo "TODO: just deploy {{HOST}} (Phase 7)"

# Rename the placeholder `rwk` project (Phase 8).
init NAME:
    @echo "TODO: just init {{NAME}} (Phase 8)"

# Remove build artifacts.
clean:
    cargo clean
    rm -rf apps/web/dist apps/web/node_modules
