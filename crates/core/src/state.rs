//! Shared Axum state. Mailer and jobs are placeholders until later phases.

use std::sync::Arc;

use sqlx::PgPool;

use crate::config::Config;

/// Cloneable process state with `Arc` internals.
#[derive(Clone)]
pub struct AppState {
    inner: Arc<Inner>,
}

struct Inner {
    config: Config,
    db: PgPool,
    mailer: MailerPlaceholder,
    jobs: JobsPlaceholder,
}

/// Filled in by the mail phase.
#[derive(Debug, Default, Clone)]
pub struct MailerPlaceholder;

/// Filled in by the jobs phase.
#[derive(Debug, Default, Clone)]
pub struct JobsPlaceholder;

impl AppState {
    /// Build state from loaded config and a live pool.
    pub fn new(config: Config, db: PgPool) -> Self {
        Self {
            inner: Arc::new(Inner {
                config,
                db,
                mailer: MailerPlaceholder,
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

    pub fn mailer(&self) -> &MailerPlaceholder {
        &self.inner.mailer
    }

    pub fn jobs(&self) -> &JobsPlaceholder {
        &self.inner.jobs
    }
}
