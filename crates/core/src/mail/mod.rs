//! Outbound email: SMTP in production, log/record in tests.

mod smtp;
mod templates;

use std::sync::{Arc, Mutex};

use async_trait::async_trait;

pub use smtp::SmtpMailer;
pub use templates::{EmailTemplate, render};

/// A fully rendered outbound message.
#[derive(Debug, Clone)]
pub struct Email {
    pub kind: &'static str,
    pub to: String,
    pub subject: String,
    pub html: String,
    pub text: String,
    pub url: String,
}

/// Sends rendered email. Implementations must be cheap to clone via [`Arc`].
#[async_trait]
pub trait Mailer: Send + Sync {
    async fn send(&self, email: &Email) -> anyhow::Result<()>;
}

/// Logs the rendered message. Used by worker tests.
#[derive(Debug, Clone, Default)]
pub struct LogMailer {
    messages: Arc<Mutex<Vec<Email>>>,
}

impl LogMailer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn messages(&self) -> Vec<Email> {
        self.messages.lock().expect("mail lock").clone()
    }
}

#[async_trait]
impl Mailer for LogMailer {
    async fn send(&self, email: &Email) -> anyhow::Result<()> {
        tracing::info!(
            to = %email.to,
            subject = %email.subject,
            url = %email.url,
            "outbound email (log mailer)"
        );
        self.messages.lock().expect("mail lock").push(email.clone());
        Ok(())
    }
}

/// Captures sent mail for HTTP integration tests.
#[derive(Debug, Clone, Default)]
pub struct RecordingMailer {
    messages: Arc<Mutex<Vec<Email>>>,
}

impl RecordingMailer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn messages(&self) -> Vec<Email> {
        self.messages.lock().expect("mail lock").clone()
    }
}

#[async_trait]
impl Mailer for RecordingMailer {
    async fn send(&self, email: &Email) -> anyhow::Result<()> {
        self.messages.lock().expect("mail lock").push(email.clone());
        Ok(())
    }
}
