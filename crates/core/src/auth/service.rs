//! Register, login, verify-email (including resend), and password-reset flows.

use chrono::Utc;
use sqlx::PgPool;
use tower_sessions::Session;

use crate::AppError;
use crate::jobs::{self, Job};
use crate::mail::EmailTemplate;
use crate::users::model::User;
use crate::users::{repo, service as users};

use super::extract::CurrentUser;
use super::password::{DUMMY_PASSWORD_HASH, hash_password, verify_password};
use super::tokens::{self, AuthTokenKind};

/// Register a password user, issue a verify token, and start a session.
///
/// The user row, verify token, and `SendEmail` job are written in one transaction.
pub async fn register(
    pool: &PgPool,
    session: &Session,
    app_url: &str,
    email: &str,
    password: &str,
) -> Result<User, AppError> {
    if users::find_by_email(pool, email).await?.is_some() {
        return Err(AppError::Conflict);
    }
    let password_hash = hash_password(password)?;
    let mut tx = pool.begin().await?;
    let user = repo::create(&mut *tx, email, &password_hash).await?;
    let token = tokens::issue(
        &mut *tx,
        user.id,
        AuthTokenKind::EmailVerify,
        tokens::verify_ttl(),
    )
    .await?;
    let url = format!("{app_url}/verify-email?token={}", token.plaintext);
    jobs::enqueue(
        &mut *tx,
        Job::send_email(user.email.clone(), EmailTemplate::VerifyEmail, url),
    )
    .await?;
    tx.commit().await?;
    CurrentUser::login(session, user.id).await?;
    Ok(user)
}

/// Authenticate with email+password and start a session.
pub async fn login(
    pool: &PgPool,
    session: &Session,
    email: &str,
    password: &str,
) -> Result<User, AppError> {
    let user = users::find_by_email(pool, email).await?;
    let password_hash = user
        .as_ref()
        .and_then(|user| user.password_hash.as_deref())
        .unwrap_or(DUMMY_PASSWORD_HASH);
    let password_ok = verify_password(password, password_hash);
    let Some(user) = user.filter(|user| user.password_hash.is_some() && password_ok) else {
        return Err(AppError::Unauthorized);
    };
    CurrentUser::login(session, user.id).await?;
    Ok(user)
}

/// End the session.
pub async fn logout(session: &Session) -> Result<(), AppError> {
    CurrentUser::logout(session).await
}

/// Consume a verify-email token.
pub async fn verify_email(pool: &PgPool, token: &str) -> Result<(), AppError> {
    let user_id = tokens::consume(pool, token, AuthTokenKind::EmailVerify).await?;
    repo::set_verified(pool, user_id, Utc::now()).await?;
    Ok(())
}

/// Issue a fresh verify token and enqueue mail, or no-op if already verified.
///
/// Token insert and `SendEmail` share one transaction, matching [`register`].
pub async fn resend_verification(
    pool: &PgPool,
    app_url: &str,
    user: &User,
) -> Result<(), AppError> {
    if user.email_verified_at.is_some() {
        return Ok(());
    }
    let mut tx = pool.begin().await?;
    let token = tokens::issue(
        &mut *tx,
        user.id,
        AuthTokenKind::EmailVerify,
        tokens::verify_ttl(),
    )
    .await?;
    let url = format!("{app_url}/verify-email?token={}", token.plaintext);
    jobs::enqueue(
        &mut *tx,
        Job::send_email(user.email.clone(), EmailTemplate::VerifyEmail, url),
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

/// Always succeeds. Enqueues a reset email only when the email exists.
///
/// Token insert and `SendEmail` share one transaction.
pub async fn forgot_password(pool: &PgPool, app_url: &str, email: &str) -> Result<(), AppError> {
    let Some(user) = users::find_by_email(pool, email).await? else {
        return Ok(());
    };
    let mut tx = pool.begin().await?;
    let token = tokens::issue(
        &mut *tx,
        user.id,
        AuthTokenKind::PasswordReset,
        tokens::reset_ttl(),
    )
    .await?;
    let url = format!("{app_url}/reset-password?token={}", token.plaintext);
    jobs::enqueue(
        &mut *tx,
        Job::send_email(user.email.clone(), EmailTemplate::PasswordReset, url),
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

/// Consume a reset token and set a new password.
pub async fn reset_password(pool: &PgPool, token: &str, password: &str) -> Result<(), AppError> {
    let user_id = tokens::consume(pool, token, AuthTokenKind::PasswordReset).await?;
    let password_hash = hash_password(password)?;
    repo::update_password(pool, user_id, &password_hash).await?;
    Ok(())
}
