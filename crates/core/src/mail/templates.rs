//! Askama HTML templates plus a plain-text fallback.

use askama::Template;
use serde::{Deserialize, Serialize};

use super::Email;

/// Which email to render. Stored in `SendEmail` job payloads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmailTemplate {
    VerifyEmail,
    PasswordReset,
}

#[derive(Template)]
#[template(path = "verify_email.html")]
struct VerifyEmail<'a> {
    url: &'a str,
}

#[derive(Template)]
#[template(path = "password_reset.html")]
struct PasswordReset<'a> {
    url: &'a str,
}

/// Render `template` with JSON `data` (must include `url`).
pub fn render(
    to: &str,
    template: EmailTemplate,
    data: &serde_json::Value,
) -> anyhow::Result<Email> {
    let url = data
        .get("url")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("email data missing url"))?;
    match template {
        EmailTemplate::VerifyEmail => Ok(Email {
            kind: "verify",
            to: to.to_owned(),
            subject: "Verify your email".into(),
            html: VerifyEmail { url }.render()?,
            text: format!("Verify your email by opening this link:\n{url}\n"),
            url: url.to_owned(),
        }),
        EmailTemplate::PasswordReset => Ok(Email {
            kind: "reset",
            to: to.to_owned(),
            subject: "Reset your password".into(),
            html: PasswordReset { url }.render()?,
            text: format!("Reset your password by opening this link:\n{url}\n"),
            url: url.to_owned(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_template_includes_url() {
        let url = "http://localhost:8080/verify-email?token=abc";
        let email = render(
            "a@example.com",
            EmailTemplate::VerifyEmail,
            &serde_json::json!({ "url": url }),
        )
        .unwrap();
        assert_eq!(email.kind, "verify");
        assert!(email.html.contains(url));
        assert!(email.text.contains(url));
        assert_eq!(email.url, url);
    }

    #[test]
    fn reset_template_includes_url() {
        let url = "http://localhost:8080/reset-password?token=xyz";
        let email = render(
            "a@example.com",
            EmailTemplate::PasswordReset,
            &serde_json::json!({ "url": url }),
        )
        .unwrap();
        assert_eq!(email.kind, "reset");
        assert!(email.html.contains(url));
        assert!(email.text.contains(url));
    }
}
