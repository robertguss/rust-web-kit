---
name: add-migration
description: Add a SQLx Postgres migration and refresh the offline query cache. Use when changing schema, adding a table, index, or enum, or when sqlx prepare is stale.
---

# Add a migration

## Files to touch

- `migrations/` — new `N_name.sql` from `just migrate-new NAME`
- `crates/core/src/<feature>/repo.rs` — queries that use the new columns
- `.sqlx/` — only via `just sqlx-prepare`, never by hand

## Steps

1. `just migrate-new NAME` (creates an empty `--simple` file under `migrations/`).
2. Write SQL. Prefer additive changes. Use `IF NOT EXISTS` only when re-runnable by design; SQLx migrations run once.
3. `just migrate` (needs `DATABASE_URL`, default `postgres://rwk:rwk@localhost:5432/rwk`).
4. Update repo functions. Keep `query!` / `query_as!` in the repo, not the handler.
5. `just sqlx-prepare`.
6. Tests that prove the schema (existing `#[sqlx::test]` or a new one).

Editing a migration that has already been applied locally requires `sqlx database reset -y --source migrations` then `just migrate`. Do not rewrite applied files that other clones have already run; add a new migration instead.

## Checklist

- [ ] File is plain SQL in `migrations/`
- [ ] Indexes for FK / lookup columns
- [ ] Repo compiles against the new shape
- [ ] `just sqlx-prepare` refreshed `.sqlx/`
- [ ] `just check` (includes `cargo sqlx prepare --check`)
