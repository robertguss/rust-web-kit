//! User-facing helpers on top of the repo.

use sqlx::PgPool;

use crate::AppError;

use super::model::User;
use super::repo;

/// Create a user; unique email becomes [`AppError::Conflict`].
pub async fn create(pool: &PgPool, email: &str, password_hash: &str) -> Result<User, AppError> {
    Ok(repo::create(pool, email, password_hash).await?)
}

/// Fetch by email or `None`.
pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, AppError> {
    Ok(repo::find_by_email(pool, email).await?)
}

/// Fetch by id or `None`.
pub async fn find_by_id(pool: &PgPool, id: uuid::Uuid) -> Result<Option<User>, AppError> {
    Ok(repo::find_by_id(pool, id).await?)
}
