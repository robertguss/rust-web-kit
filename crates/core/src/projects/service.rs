//! Owner-scoped project operations.

use sqlx::PgPool;
use uuid::Uuid;

use crate::AppError;

use super::model::{Project, ProjectPage};
use super::repo;

/// Create a project for `owner_id`.
pub async fn create(
    pool: &PgPool,
    owner_id: Uuid,
    name: &str,
    description: Option<&str>,
) -> Result<Project, AppError> {
    Ok(repo::create(pool, owner_id, name, description).await?)
}

/// Get one owned project, or 404.
pub async fn get(pool: &PgPool, owner_id: Uuid, id: Uuid) -> Result<Project, AppError> {
    repo::find_owned(pool, owner_id, id)
        .await?
        .ok_or(AppError::NotFound)
}

/// Paginated list for `owner_id`.
pub async fn list(
    pool: &PgPool,
    owner_id: Uuid,
    page: i64,
    per_page: i64,
) -> Result<ProjectPage, AppError> {
    let total = repo::count_owned(pool, owner_id).await?;
    let offset = (page - 1) * per_page;
    let items = repo::list_owned(pool, owner_id, per_page, offset).await?;
    Ok(ProjectPage {
        items,
        total,
        page,
        per_page,
    })
}

/// Patch an owned project, or 404.
pub async fn update(
    pool: &PgPool,
    owner_id: Uuid,
    id: Uuid,
    name: Option<&str>,
    description: Option<&str>,
) -> Result<Project, AppError> {
    repo::update_owned(pool, owner_id, id, name, description)
        .await?
        .ok_or(AppError::NotFound)
}

/// Delete an owned project, or 404.
pub async fn delete(pool: &PgPool, owner_id: Uuid, id: Uuid) -> Result<(), AppError> {
    if repo::delete_owned(pool, owner_id, id).await? {
        Ok(())
    } else {
        Err(AppError::NotFound)
    }
}
