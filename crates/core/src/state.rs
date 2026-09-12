//! Shared Axum state.

use std::sync::Arc;

use sqlx::PgPool;

use crate::config::Config;
use crate::jobs::JobQueue;
use crate::mail::Mailer;

/// Cloneable process state with `Arc` internals.
#[derive(Clone)]
pub struct AppState {
    inner: Arc<Inner>,
}

struct Inner {
    config: Config,
    db: PgPool,
    mailer: Arc<dyn Mailer>,
    jobs: JobQueue,
}

impl AppState {
    /// Build state from loaded config, a live pool, and a mailer.
    pub fn new(config: Config, db: PgPool, mailer: Arc<dyn Mailer>) -> Self {
        let jobs = JobQueue::new(db.clone());
        Self {
            inner: Arc::new(Inner {
                config,
                db,
                mailer,
                jobs,
            }),
        }
    }

    pub fn config(&self) -> &Config {
        &self.inner.config
    }

    pub fn db(&self) -> &PgPool {
        &self.inner.db
    }

    pub fn mailer(&self) -> Arc<dyn Mailer> {
        Arc::clone(&self.inner.mailer)
    }

    pub fn jobs(&self) -> &JobQueue {
        &self.inner.jobs
    }
}
