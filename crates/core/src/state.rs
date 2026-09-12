//! Shared Axum state. Jobs remain a placeholder until the jobs phase.

use std::sync::Arc;

use sqlx::PgPool;

use crate::config::Config;
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
    jobs: JobsPlaceholder,
}

/// Filled in by the jobs phase.
#[derive(Debug, Default, Clone)]
pub struct JobsPlaceholder;

impl AppState {
    /// Build state from loaded config, a live pool, and a mailer.
    pub fn new(config: Config, db: PgPool, mailer: Arc<dyn Mailer>) -> Self {
        Self {
            inner: Arc::new(Inner {
                config,
                db,
                mailer,
                jobs: JobsPlaceholder,
            }),
        }
    }

    pub fn config(&self) -> &Config {
        &self.inner.config
    }

    pub fn db(&self) -> &PgPool {
        &self.inner.db
    }

    pub fn mailer(&self) -> &dyn Mailer {
        self.inner.mailer.as_ref()
    }

    pub fn jobs(&self) -> &JobsPlaceholder {
        &self.inner.jobs
    }
}
