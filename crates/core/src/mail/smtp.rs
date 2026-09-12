//! Lettre SMTP transport built from [`crate::config::MailConfig`].

use async_trait::async_trait;
use lettre::message::{Mailbox, MultiPart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

use crate::config::MailConfig;

use super::{Email, Mailer};

/// Sends mail over SMTP (Mailpit locally, real SMTP when credentials are set).
#[derive(Clone)]
pub struct SmtpMailer {
    from: String,
    transport: AsyncSmtpTransport<Tokio1Executor>,
}

impl SmtpMailer {
    /// Unauthenticated host (Mailpit) uses no TLS. Username+password uses STARTTLS.
    pub fn from_config(mail: &MailConfig) -> anyhow::Result<Self> {
        let transport = if mail.smtp_username.is_some() {
            let mut builder =
                AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&mail.smtp_host)?;
            builder = builder.port(mail.smtp_port);
            if let (Some(user), Some(password)) = (&mail.smtp_username, &mail.smtp_password) {
                builder = builder.credentials(Credentials::new(user.clone(), password.clone()));
            }
            builder.build()
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&mail.smtp_host)
                .port(mail.smtp_port)
                .build()
        };
        Ok(Self {
            from: mail.from.clone(),
            transport,
        })
    }
}

#[async_trait]
impl Mailer for SmtpMailer {
    async fn send(&self, email: &Email) -> anyhow::Result<()> {
        let from: Mailbox = self.from.parse()?;
        let to: Mailbox = email.to.parse()?;
        let message = Message::builder()
            .from(from)
            .to(to)
            .subject(&email.subject)
            .multipart(MultiPart::alternative_plain_html(
                email.text.clone(),
                email.html.clone(),
            ))?;
        self.transport.send(message).await?;
        Ok(())
    }
}
