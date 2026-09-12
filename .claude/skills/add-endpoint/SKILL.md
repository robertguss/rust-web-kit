---
name: add-endpoint
description: Add an HTTP API endpoint (Axum + utoipa + AppError + just gen). Use when adding a route, handler, REST resource, or OpenAPI path.
---

# Add an endpoint

Follow `CLAUDE.md` conventions. Copy `docs/slices/projects.md` for owner-scoped CRUD.

## Files to touch

- `crates/core/src/<feature>/{mod.rs,model.rs,repo.rs,service.rs}` — create the module if new; `pub mod` in `crates/core/src/lib.rs`
- `crates/api/src/dto.rs` — garde request types
- `crates/api/src/routes/<feature>.rs` — handlers; `pub mod` in `routes/mod.rs`
- `crates/api/src/router.rs` — `routes!(…)`, nest, `ApiDoc` tags + `components(schemas(…))`
- `crates/api/tests/<feature>.rs` — oneshot via `crates/api/tests/common`
- Frontend only after `just gen`: routes/components that import `@/api/generated/…`

Do not edit `apps/web/src/api/generated/`.

## Handler shape

```rust
#[utoipa::path(post, path = "/", tag = "<feature>", request_body = CreateX, responses(
    (status = 201, description = "Created", body = X),
    (status = 401, description = "Not authenticated", body = Problem),
    (status = 422, description = "Validation failed", body = Problem),
))]
pub async fn create_x(
    State(state): State<AppState>,
    RequireAuth(user): RequireAuth,
    Json(body): Json<CreateX>,
) -> Result<(StatusCode, Json<X>), AppError> {
    body.validate()?;
    let row = service::create(state.db(), user.id, …).await?;
    Ok((StatusCode::CREATED, Json(row)))
}
```

utoipa `path` is relative to the nested router. Other users / missing rows → `AppError::NotFound`.

## Checklist

- [ ] DTO has garde; handler calls `validate()?` before the service
- [ ] Return type is `Result<_, AppError>`
- [ ] `#[utoipa::path]` lists real status bodies (`Problem` on errors)
- [ ] Schema types added to `ApiDoc`
- [ ] Router nest + `routes!` registered
- [ ] Tests: auth, happy path, validation 422, ownership 404 if applicable
- [ ] `just gen` and only then UI
- [ ] `just sqlx-prepare` if SQL changed
- [ ] `just check`
