//! Random 32-byte auth tokens stored as SHA-256 hashes.
//!
//! TODO(phase-5): `CleanupExpiredTokens` must also `DELETE FROM sessions WHERE expiry_date < now()`.

use chrono::{Duration, Utc};
use rand::Rng;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

use crate::AppError;

/// Postgres `auth_token_kind`.
#[derive(Debug, Clone, Copy, sqlx::Type)]
#[sqlx(type_name = "auth_token_kind", rename_all = "snake_case")]
pub enum AuthTokenKind {
    EmailVerify,
    PasswordReset,
}

/// Newly issued token: plaintext is emailed; only `hash` is stored.
pub struct IssuedToken {
    pub plaintext: String,
    pub hash: String,
}

/// Persist a token and return the plaintext to put in a link.
pub async fn issue(
    pool: &PgPool,
    user_id: Uuid,
    kind: AuthTokenKind,
    ttl: Duration,
) -> Result<IssuedToken, AppError> {
    let issued = generate();
    let id = Uuid::now_v7();
    let expires_at = Utc::now() + ttl;
    sqlx::query!(
        r#"
        INSERT INTO auth_tokens (id, user_id, kind, token_hash, expires_at)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        id,
        user_id,
        kind as AuthTokenKind,
        issued.hash,
        expires_at,
    )
    .execute(pool)
    .await?;
    Ok(issued)
}

/// Consume a still-valid unused token in one atomic update.
pub async fn consume(
    pool: &PgPool,
    plaintext: &str,
    kind: AuthTokenKind,
) -> Result<Uuid, AppError> {
    let hash = hash_token(plaintext);
    let consumed = sqlx::query!(
        r#"
        UPDATE auth_tokens
        SET used_at = now()
        WHERE token_hash = $1
          AND kind = $2
          AND used_at IS NULL
          AND expires_at > now()
        RETURNING user_id
        "#,
        hash,
        kind as AuthTokenKind,
    )
    .fetch_optional(pool)
    .await?;
    if let Some(row) = consumed {
        return Ok(row.user_id);
    }

    let existing = sqlx::query!(
        r#"
        SELECT used_at, expires_at
        FROM auth_tokens
        WHERE token_hash = $1 AND kind = $2
        "#,
        hash,
        kind as AuthTokenKind,
    )
    .fetch_optional(pool)
    .await?;
    match existing {
        None => Err(AppError::BadRequest("invalid token".into())),
        Some(row) if row.used_at.is_some() => {
            Err(AppError::BadRequest("token already used".into()))
        }
        Some(row) if row.expires_at <= Utc::now() => {
            Err(AppError::BadRequest("token expired".into()))
        }
        Some(_) => Err(AppError::BadRequest("token already used".into())),
    }
}

fn generate() -> IssuedToken {
    let mut bytes = [0_u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    let plaintext =
        base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, bytes);
    let hash = hash_token(&plaintext);
    IssuedToken { plaintext, hash }
}

fn hash_token(plaintext: &str) -> String {
    let digest = Sha256::digest(plaintext.as_bytes());
    hex::encode(digest)
}

/// Token TTL helpers.
pub fn verify_ttl() -> Duration {
    Duration::hours(24)
}

/// Password-reset TTL.
pub fn reset_ttl() -> Duration {
    Duration::hours(1)
}

/// Used by tests that need an already-expired row.
pub async fn insert_expired(
    pool: &PgPool,
    user_id: Uuid,
    kind: AuthTokenKind,
    plaintext: &str,
) -> Result<(), AppError> {
    let hash = hash_token(plaintext);
    sqlx::query!(
        r#"
        INSERT INTO auth_tokens (id, user_id, kind, token_hash, expires_at)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        Uuid::now_v7(),
        user_id,
        kind as AuthTokenKind,
        hash,
        Utc::now() - Duration::hours(1),
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub fn hash_for_tests(plaintext: &str) -> String {
    hash_token(plaintext)
}
