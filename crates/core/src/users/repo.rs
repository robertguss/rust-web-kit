//! User persistence.

use chrono::{DateTime, Utc};
use sqlx::{Executor, PgPool, Postgres};
use uuid::Uuid;

use super::model::User;

/// Insert a password user.
pub async fn create<'e, E>(
    executor: E,
    email: &str,
    password_hash: &str,
) -> Result<User, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    let id = Uuid::now_v7();
    sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (id, email, password_hash)
        VALUES ($1, $2, $3)
        RETURNING id, email, password_hash, email_verified_at, created_at, updated_at
        "#,
        id,
        email,
        password_hash,
    )
    .fetch_one(executor)
    .await
}

/// Insert an OAuth-only user with a verified email and no password.
pub async fn create_oauth<'e, E>(
    executor: E,
    email: &str,
    verified_at: DateTime<Utc>,
) -> Result<User, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    let id = Uuid::now_v7();
    sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (id, email, password_hash, email_verified_at)
        VALUES ($1, $2, NULL, $3)
        RETURNING id, email, password_hash, email_verified_at, created_at, updated_at
        "#,
        id,
        email,
        verified_at,
    )
    .fetch_one(executor)
    .await
}

/// Look up by citext email.
pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as!(
        User,
        r#"
        SELECT id, email, password_hash, email_verified_at, created_at, updated_at
        FROM users
        WHERE email = $1
        "#,
        email,
    )
    .fetch_optional(pool)
    .await
}

/// Look up by id.
pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as!(
        User,
        r#"
        SELECT id, email, password_hash, email_verified_at, created_at, updated_at
        FROM users
        WHERE id = $1
        "#,
        id,
    )
    .fetch_optional(pool)
    .await
}

/// Mark email verified.
pub async fn set_verified(
    pool: &PgPool,
    id: Uuid,
    verified_at: DateTime<Utc>,
) -> Result<User, sqlx::Error> {
    sqlx::query_as!(
        User,
        r#"
        UPDATE users
        SET email_verified_at = $2, updated_at = now()
        WHERE id = $1
        RETURNING id, email, password_hash, email_verified_at, created_at, updated_at
        "#,
        id,
        verified_at,
    )
    .fetch_one(pool)
    .await
}

/// Replace the password hash.
pub async fn update_password(
    pool: &PgPool,
    id: Uuid,
    password_hash: &str,
) -> Result<User, sqlx::Error> {
    sqlx::query_as!(
        User,
        r#"
        UPDATE users
        SET password_hash = $2, updated_at = now()
        WHERE id = $1
        RETURNING id, email, password_hash, email_verified_at, created_at, updated_at
        "#,
        id,
        password_hash,
    )
    .fetch_one(pool)
    .await
}
