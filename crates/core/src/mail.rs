//! Outbound email. Phase 5 adds SMTP; this phase logs or records messages.
//!
//! Phase 5's expired-token cleanup job must also delete expired `sessions` rows.

use std::sync::{Arc, Mutex};

use async_trait::async_trait;

/// Outbound email used by auth flows.
#[async_trait]
pub trait Mailer: Send + Sync {
    async fn send_verify_email(&self, to: &str, url: &str) -> anyhow::Result<()>;
    async fn send_password_reset(&self, to: &str, url: &str) -> anyhow::Result<()>;
}

/// Logs verify/reset links. Used in development until SMTP lands.
#[derive(Debug, Default, Clone, Copy)]
pub struct LogMailer;

#[async_trait]
impl Mailer for LogMailer {
    async fn send_verify_email(&self, to: &str, url: &str) -> anyhow::Result<()> {
        tracing::info!(to, url, "verify-email link (log mailer)");
        Ok(())
    }

    async fn send_password_reset(&self, to: &str, url: &str) -> anyhow::Result<()> {
        tracing::info!(to, url, "password-reset link (log mailer)");
        Ok(())
    }
}

/// Captures sent mail for tests.
#[derive(Debug, Clone, Default)]
pub struct RecordingMailer {
    messages: Arc<Mutex<Vec<MailMessage>>>,
}

/// One captured outbound message.
#[derive(Debug, Clone)]
pub struct MailMessage {
    pub kind: &'static str,
    pub to: String,
    pub url: String,
}

impl RecordingMailer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn messages(&self) -> Vec<MailMessage> {
        self.messages.lock().expect("mail lock").clone()
    }
}

#[async_trait]
impl Mailer for RecordingMailer {
    async fn send_verify_email(&self, to: &str, url: &str) -> anyhow::Result<()> {
        self.messages.lock().expect("mail lock").push(MailMessage {
            kind: "verify",
            to: to.to_owned(),
            url: url.to_owned(),
        });
        Ok(())
    }

    async fn send_password_reset(&self, to: &str, url: &str) -> anyhow::Result<()> {
        self.messages.lock().expect("mail lock").push(MailMessage {
            kind: "reset",
            to: to.to_owned(),
            url: url.to_owned(),
        });
        Ok(())
    }
}
