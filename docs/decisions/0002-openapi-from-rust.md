# 0002. OpenAPI from Rust

## Context

The frontend needs a typed client that cannot drift from the HTTP API. Hand-written TS types and a separate spec file both rot.

## Decision

Annotate Axum handlers with utoipa (`#[utoipa::path]`, `utoipa-axum` typed router). Export with `rwk-api --export-openapi`. `just gen` writes `apps/web/openapi.json` and runs `@hey-api/openapi-ts` (TanStack Query plugin) into `apps/web/src/api/generated/`. `just check` fails if regenerating changes those files.

Spec paths are unprefixed; `servers[0].url` is `/api` so the client uses `baseUrl: '/api'`.

## Consequences

The Rust types are the source of truth. Generated TS is committed and never edited by hand. Adding an endpoint is incomplete until `just gen` has been run.
