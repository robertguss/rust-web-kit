//! Register, login, verify-email, and password-reset flows.

use chrono::Utc;
use sqlx::PgPool;
use tower_sessions::Session;

use crate::AppError;
use crate::mail::Mailer;
use crate::users::model::User;
use crate::users::{repo, service as users};

use super::extract::CurrentUser;
use super::password::{DUMMY_PASSWORD_HASH, hash_password, verify_password};
use super::tokens::{self, AuthTokenKind};

/// Register a password user, issue a verify token, and start a session.
pub async fn register(
    pool: &PgPool,
    session: &Session,
    mailer: &dyn Mailer,
    app_url: &str,
    email: &str,
    password: &str,
) -> Result<User, AppError> {
    if users::find_by_email(pool, email).await?.is_some() {
        return Err(AppError::Conflict);
    }
    let password_hash = hash_password(password)?;
    let user = users::create(pool, email, &password_hash).await?;
    let token = tokens::issue(
        pool,
        user.id,
        AuthTokenKind::EmailVerify,
        tokens::verify_ttl(),
    )
    .await?;
    let url = format!("{app_url}/verify-email?token={}", token.plaintext);
    mailer
        .send_verify_email(&user.email, &url)
        .await
        .map_err(AppError::Internal)?;
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

/// Always succeeds. Sends a reset link only when the email exists.
pub async fn forgot_password(
    pool: &PgPool,
    mailer: &dyn Mailer,
    app_url: &str,
    email: &str,
) -> Result<(), AppError> {
    let Some(user) = users::find_by_email(pool, email).await? else {
        return Ok(());
    };
    let token = tokens::issue(
        pool,
        user.id,
        AuthTokenKind::PasswordReset,
        tokens::reset_ttl(),
    )
    .await?;
    let url = format!("{app_url}/reset-password?token={}", token.plaintext);
    mailer
        .send_password_reset(&user.email, &url)
        .await
        .map_err(AppError::Internal)?;
    Ok(())
}

/// Consume a reset token and set a new password.
pub async fn reset_password(pool: &PgPool, token: &str, password: &str) -> Result<(), AppError> {
    let user_id = tokens::consume(pool, token, AuthTokenKind::PasswordReset).await?;
    let password_hash = hash_password(password)?;
    repo::update_password(pool, user_id, &password_hash).await?;
    Ok(())
}
