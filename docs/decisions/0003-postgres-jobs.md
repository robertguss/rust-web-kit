# 0003. Postgres jobs

## Context

Register and password-reset must send email atomically with the user/token insert. A second broker (Redis, SQS) is more moving parts than a single-VM starter wants. apalis-postgres still depends on sqlx 0.8, which cannot share this workspace's sqlx 0.9 `PgPool`.

## Decision

In-tree Postgres queue (`migrations/3_jobs.sql`). `jobs::enqueue` takes any SQLx executor so callers pass `&mut *tx`. Worker runs in the API process by default. Jobs: `SendEmail`, hourly `CleanupExpiredTokens` (unique pending row via partial index + `enqueue_cron`).

## Consequences

Enqueue and business writes commit together; a rolled-back transaction leaves no job. No extra infrastructure locally. When apalis (or similar) supports sqlx 0.9 on a shared pool, this module is the seam to replace.
