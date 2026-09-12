//! Poll loop plus hourly cron enqueue.

use std::sync::Arc;
use std::time::Duration;

use sqlx::PgPool;
use tokio::time::MissedTickBehavior;
use uuid::Uuid;

use crate::AppError;
use crate::mail::{Mailer, render};

use super::queue::{self, Claimed};
use super::{Job, cron_jobs, enqueue_cron};

/// Processes claimed jobs against a [`Mailer`].
#[derive(Clone)]
pub struct Worker {
    pool: PgPool,
    mailer: Arc<dyn Mailer>,
    id: String,
}

impl Worker {
    pub fn new(pool: PgPool, mailer: Arc<dyn Mailer>) -> Self {
        Self {
            pool,
            mailer,
            id: format!("rwk-{id}", id = Uuid::now_v7()),
        }
    }

    /// Claim and handle at most one due job. `Ok(false)` means the queue was empty.
    pub async fn process_one(&self) -> Result<bool, AppError> {
        let Some(claimed) = queue::claim(&self.pool, &self.id).await? else {
            return Ok(false);
        };
        match self.handle(&claimed).await {
            Ok(()) => queue::mark_done(&self.pool, claimed.id).await?,
            Err(error) => {
                tracing::error!(job_id = %claimed.id, error = %error, "job failed");
                queue::mark_failed(
                    &self.pool,
                    claimed.id,
                    claimed.attempts,
                    claimed.max_attempts,
                    &error.to_string(),
                )
                .await?;
            }
        }
        Ok(true)
    }

    /// Drain every currently due job.
    pub async fn process_available(&self) -> Result<usize, AppError> {
        let mut n = 0;
        while self.process_one().await? {
            n += 1;
        }
        Ok(n)
    }

    async fn handle(&self, claimed: &Claimed) -> anyhow::Result<()> {
        let job: Job = serde_json::from_value(claimed.payload.clone())?;
        match job {
            Job::SendEmail { to, template, data } => {
                let email = render(&to, template, &data)?;
                self.mailer.send(&email).await?;
            }
            Job::CleanupExpiredTokens => {
                super::cleanup::run(&self.pool).await?;
            }
        }
        Ok(())
    }
}

/// Run until the task is aborted: poll the queue and enqueue cron jobs on their intervals.
pub async fn run(worker: Worker) -> anyhow::Result<()> {
    let mut poll = tokio::time::interval(Duration::from_millis(400));
    poll.set_missed_tick_behavior(MissedTickBehavior::Delay);

    let cron = cron_jobs();
    let mut cleanup_tick = tokio::time::interval(cron[0].interval);
    cleanup_tick.set_missed_tick_behavior(MissedTickBehavior::Delay);

    loop {
        tokio::select! {
            _ = cleanup_tick.tick() => {
                for spec in cron_jobs() {
                    if let Err(error) = enqueue_cron(&worker.pool, spec.job).await {
                        tracing::error!(cron = spec.name, error = %error, "enqueue cron job");
                    }
                }
            }
            _ = poll.tick() => {
                if let Err(error) = worker.process_available().await {
                    tracing::error!(error = %error, "job worker");
                }
            }
        }
    }
}
