//! Project persistence. All queries are owner-scoped.

use sqlx::PgPool;
use uuid::Uuid;

use super::model::Project;

/// Insert a project owned by `owner_id`.
pub async fn create(
    pool: &PgPool,
    owner_id: Uuid,
    name: &str,
    description: Option<&str>,
) -> Result<Project, sqlx::Error> {
    let id = Uuid::now_v7();
    sqlx::query_as!(
        Project,
        r#"
        INSERT INTO projects (id, owner_id, name, description)
        VALUES ($1, $2, $3, $4)
        RETURNING id, owner_id, name, description, created_at, updated_at
        "#,
        id,
        owner_id,
        name,
        description,
    )
    .fetch_one(pool)
    .await
}

/// Fetch one project if it belongs to `owner_id`.
pub async fn find_owned(
    pool: &PgPool,
    owner_id: Uuid,
    id: Uuid,
) -> Result<Option<Project>, sqlx::Error> {
    sqlx::query_as!(
        Project,
        r#"
        SELECT id, owner_id, name, description, created_at, updated_at
        FROM projects
        WHERE id = $1 AND owner_id = $2
        "#,
        id,
        owner_id,
    )
    .fetch_optional(pool)
    .await
}

/// Count projects owned by `owner_id`.
pub async fn count_owned(pool: &PgPool, owner_id: Uuid) -> Result<i64, sqlx::Error> {
    let total = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) AS "count!"
        FROM projects
        WHERE owner_id = $1
        "#,
        owner_id,
    )
    .fetch_one(pool)
    .await?;
    Ok(total)
}

/// Page of projects owned by `owner_id`, newest first.
pub async fn list_owned(
    pool: &PgPool,
    owner_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<Project>, sqlx::Error> {
    sqlx::query_as!(
        Project,
        r#"
        SELECT id, owner_id, name, description, created_at, updated_at
        FROM projects
        WHERE owner_id = $1
        ORDER BY created_at DESC, id DESC
        LIMIT $2 OFFSET $3
        "#,
        owner_id,
        limit,
        offset,
    )
    .fetch_all(pool)
    .await
}

/// Update name and/or description for an owned project.
pub async fn update_owned(
    pool: &PgPool,
    owner_id: Uuid,
    id: Uuid,
    name: Option<&str>,
    description: Option<&str>,
) -> Result<Option<Project>, sqlx::Error> {
    sqlx::query_as!(
        Project,
        r#"
        UPDATE projects
        SET
            name = COALESCE($3, name),
            description = COALESCE($4, description),
            updated_at = now()
        WHERE id = $1 AND owner_id = $2
        RETURNING id, owner_id, name, description, created_at, updated_at
        "#,
        id,
        owner_id,
        name,
        description,
    )
    .fetch_optional(pool)
    .await
}

/// Delete an owned project. Returns whether a row was removed.
pub async fn delete_owned(pool: &PgPool, owner_id: Uuid, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        DELETE FROM projects
        WHERE id = $1 AND owner_id = $2
        "#,
        id,
        owner_id,
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}
