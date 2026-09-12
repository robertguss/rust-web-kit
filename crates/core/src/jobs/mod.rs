//! Postgres job queue (in-tree: apalis-postgres still depends on sqlx 0.8).

mod cleanup;
mod queue;
mod worker;

#[cfg(test)]
mod tests;

use std::time::Duration;

use serde::{Deserialize, Serialize};
use sqlx::{Executor, PgPool, Postgres};
use uuid::Uuid;

use crate::AppError;
use crate::mail::EmailTemplate;

pub use worker::{Worker, run};

/// Default attempts before a job is marked complete (dead).
pub const DEFAULT_MAX_ATTEMPTS: i32 = 5;

/// Hourly cleanup tick.
pub const CLEANUP_INTERVAL: Duration = Duration::from_hours(1);

/// Work item stored as JSON in `jobs.payload`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Job {
    SendEmail {
        to: String,
        template: EmailTemplate,
        data: serde_json::Value,
    },
    CleanupExpiredTokens,
}

impl Job {
    pub fn kind_name(&self) -> &'static str {
        match self {
            Self::SendEmail { .. } => "send_email",
            Self::CleanupExpiredTokens => "cleanup_expired_tokens",
        }
    }

    pub fn send_email(
        to: impl Into<String>,
        template: EmailTemplate,
        url: impl Into<String>,
    ) -> Self {
        Self::SendEmail {
            to: to.into(),
            template,
            data: serde_json::json!({ "url": url.into() }),
        }
    }
}

/// A cron entry the worker ticks.
#[derive(Debug, Clone)]
pub struct CronSpec {
    pub name: &'static str,
    pub interval: Duration,
    pub job: Job,
}

/// Registered periodic jobs. The worker enqueues each on its interval.
pub fn cron_jobs() -> [CronSpec; 1] {
    [CronSpec {
        name: "cleanup_expired_tokens",
        interval: CLEANUP_INTERVAL,
        job: Job::CleanupExpiredTokens,
    }]
}

/// Insert `job` using any executor so callers can join an open transaction.
pub async fn enqueue<'e, E>(executor: E, job: Job) -> Result<Uuid, AppError>
where
    E: Executor<'e, Database = Postgres>,
{
    let id = Uuid::now_v7();
    let payload = serde_json::to_value(&job).map_err(|error| AppError::Internal(error.into()))?;
    sqlx::query!(
        r#"
        INSERT INTO jobs (id, kind, payload, run_at, max_attempts)
        VALUES ($1, $2, $3, now(), $4)
        "#,
        id,
        job.kind_name(),
        payload,
        DEFAULT_MAX_ATTEMPTS,
    )
    .execute(executor)
    .await?;
    Ok(id)
}

/// Enqueue a cron job. A pending row of the same kind is left unchanged.
pub async fn enqueue_cron<'e, E>(executor: E, job: Job) -> Result<(), AppError>
where
    E: Executor<'e, Database = Postgres>,
{
    let id = Uuid::now_v7();
    let payload = serde_json::to_value(&job).map_err(|error| AppError::Internal(error.into()))?;
    sqlx::query!(
        r#"
        INSERT INTO jobs (id, kind, payload, run_at, max_attempts)
        VALUES ($1, $2, $3, now(), $4)
        ON CONFLICT (kind) WHERE completed_at IS NULL AND kind = 'cleanup_expired_tokens'
        DO NOTHING
        "#,
        id,
        job.kind_name(),
        payload,
        DEFAULT_MAX_ATTEMPTS,
    )
    .execute(executor)
    .await?;
    Ok(())
}

/// Pool-backed helper stored on [`crate::AppState`].
#[derive(Debug, Clone)]
pub struct JobQueue {
    pool: PgPool,
}

impl JobQueue {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn enqueue(&self, job: Job) -> Result<Uuid, AppError> {
        enqueue(&self.pool, job).await
    }
}
