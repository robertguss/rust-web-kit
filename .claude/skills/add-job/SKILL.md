---
name: add-job
description: Add a background job to the in-tree Postgres queue. Use when enqueueing email or other async work, or adding a cron job.
---

# Add a job

Queue is in-tree (`crates/core/src/jobs/`), not apalis. Enqueue on the same SQLx executor as the write.

## Files to touch

- `crates/core/src/jobs/mod.rs` — `Job` variant + `kind_name`
- `crates/core/src/jobs/worker.rs` — dispatch in `process_one`
- New handler module under `crates/core/src/jobs/` if the work is more than a few lines (see `cleanup.rs`)
- `crates/core/src/mail/` + `crates/core/templates/` — only for email
- Caller (e.g. `crates/core/src/auth/service.rs`) — `jobs::enqueue(&mut *tx, job)` before `commit`
- `crates/core/src/jobs/tests.rs` — rollback / worker assertions
- Cron only: `cron_jobs()` and possibly a unique partial index (see `jobs_cleanup_pending_idx`)

## Enqueue (transactional)

```rust
let mut tx = pool.begin().await?;
// … business insert …
jobs::enqueue(&mut *tx, Job::send_email(to, EmailTemplate::VerifyEmail, url)).await?;
tx.commit().await?;
```

`JobQueue` on `AppState` is for enqueue outside a transaction. Prefer the executor form.

Cron: add a `CronSpec` in `cron_jobs()`. Use `enqueue_cron` so a pending row of that kind is not duplicated. A new unique kind needs its own partial unique index if you want that guarantee.

## Checklist

- [ ] `Job` is serde-tagged (`type` / snake_case) so old rows still decode
- [ ] Worker handles the new variant
- [ ] Failures retry (`max_attempts`, backoff already in the worker)
- [ ] Test: enqueue in a rolled-back transaction leaves no row
- [ ] Test: worker + `LogMailer` / fake side effect
- [ ] `just sqlx-prepare` if SQL changed
- [ ] `just check`
