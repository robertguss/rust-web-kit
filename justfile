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

# Start API + Vite together.
dev:
    #!/usr/bin/env bash
    set -euo pipefail
    trap 'kill 0' EXIT
    just dev-api &
    just dev-web &
    wait

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

# Export OpenAPI and generate the TS client.
gen:
    cargo run -p rwk-api -- --export-openapi apps/web/openapi.json
    pnpm --dir apps/web exec openapi-ts

# Fail if `just gen` would change the committed-on-disk client (git-independent).
gen-check:
    #!/usr/bin/env bash
    set -euo pipefail
    hash_client() {
      find apps/web/openapi.json apps/web/src/api/generated -type f | sort | xargs sha256sum | sha256sum
    }
    before=$(hash_client)
    just gen
    after=$(hash_client)
    if [ "$before" != "$after" ]; then
      echo "generated OpenAPI client is stale." >&2
      echo "apps/web/openapi.json and apps/web/src/api/generated changed after 'just gen'." >&2
      echo "Run 'just gen' and include those files." >&2
      exit 1
    fi

# Format/lint/typecheck everything that exists in this phase.
check:
    cargo fmt --all -- --check
    cargo clippy --all-targets -- -D warnings
    pnpm --dir apps/web typecheck
    pnpm --dir apps/web lint
    cargo sqlx prepare --check --workspace -- --all-targets
    just gen-check

# Run API and web tests.
test: test-api test-web

# Rust tests.
test-api:
    cargo nextest run --workspace

# Frontend unit tests.
test-web:
    pnpm --dir apps/web test

# Playwright smoke against the API + built SPA (Postgres + Mailpit required).
e2e:
    #!/usr/bin/env bash
    set -euo pipefail
    if ! curl -fsS --max-time 1 http://127.0.0.1:8025/api/v1/info >/dev/null 2>&1; then
      just db-up
    fi
    pnpm --dir apps/web build
    if [ -n "${CI:-}" ]; then
      pnpm --dir apps/web exec playwright install --with-deps chromium
    else
      pnpm --dir apps/web exec playwright install chromium
    fi
    cargo build -p rwk-api
    started_api=0
    if ! curl -fsS --max-time 1 http://127.0.0.1:8080/api/health >/dev/null 2>&1; then
      ./target/debug/rwk-api &
      api_pid=$!
      started_api=1
      trap 'if [ "$started_api" = 1 ]; then kill "$api_pid" 2>/dev/null || true; fi' EXIT
      for _ in $(seq 1 60); do
        if curl -fsS --max-time 1 http://127.0.0.1:8080/api/health >/dev/null 2>&1; then
          break
        fi
        if ! kill -0 "$api_pid" 2>/dev/null; then
          echo "rwk-api exited before becoming healthy"
          exit 1
        fi
        sleep 0.5
      done
      curl -fsS --max-time 1 http://127.0.0.1:8080/api/health >/dev/null
    fi
    pnpm --dir apps/web exec playwright test

# Full CI pipeline.
ci:
    just check
    just test
    just e2e
    just docker-build

# Release-ish local build.
build:
    cargo build --release -p rwk-api
    pnpm --dir apps/web build

# Build the production image for the local architecture.
docker-build:
    DOCKER_BUILDKIT=1 docker build -t rwk:latest .

# Deploy the image to a VM over SSH. PLATFORM must match the VM's
# architecture, not your laptop's (Apple Silicon builds arm64 by default).
deploy HOST PLATFORM="linux/amd64":
    #!/usr/bin/env bash
    set -euo pipefail
    DOCKER_BUILDKIT=1 docker build --platform {{PLATFORM}} -t rwk:latest .
    docker save rwk:latest | ssh {{HOST}} docker load
    ssh {{HOST}} mkdir -p rwk
    scp docker-compose.prod.yml fnox.toml {{HOST}}:rwk/
    ssh {{HOST}} 'cd rwk && fnox exec -- docker compose -f docker-compose.prod.yml up -d'

# Rename the placeholder `rwk` project. Deletes this recipe.
init NAME:
    ./scripts/init.sh {{NAME}}

# Remove build artifacts.
clean:
    cargo clean
    rm -rf apps/web/dist apps/web/node_modules
