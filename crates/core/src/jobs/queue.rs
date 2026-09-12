//! Claim / complete / retry using `FOR UPDATE SKIP LOCKED`.

use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

pub(crate) struct Claimed {
    pub id: Uuid,
    pub payload: serde_json::Value,
    pub attempts: i32,
    pub max_attempts: i32,
}

/// Take the next due job. Holds no long-lived row lock; stale locks expire after 5 minutes.
pub(crate) async fn claim(pool: &PgPool, worker_id: &str) -> Result<Option<Claimed>, sqlx::Error> {
    sqlx::query_as!(
        Claimed,
        r#"
        WITH next AS (
            SELECT id
            FROM jobs
            WHERE completed_at IS NULL
              AND attempts < max_attempts
              AND run_at <= now()
              AND (locked_at IS NULL OR locked_at < now() - interval '5 minutes')
            ORDER BY run_at ASC, id ASC
            FOR UPDATE SKIP LOCKED
            LIMIT 1
        )
        UPDATE jobs AS j
        SET locked_at = now(),
            locked_by = $1,
            attempts = j.attempts + 1
        FROM next
        WHERE j.id = next.id
        RETURNING j.id, j.payload, j.attempts, j.max_attempts
        "#,
        worker_id,
    )
    .fetch_optional(pool)
    .await
}

pub(crate) async fn mark_done(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE jobs
        SET completed_at = now(), locked_at = NULL, locked_by = NULL
        WHERE id = $1
        "#,
        id,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub(crate) fn backoff(attempts: i32) -> Duration {
    let shift = u32::try_from(attempts.clamp(1, 8)).unwrap_or(8);
    Duration::seconds(2_i64.pow(shift))
}

pub(crate) async fn mark_failed(
    pool: &PgPool,
    id: Uuid,
    attempts: i32,
    max_attempts: i32,
    error: &str,
) -> Result<(), sqlx::Error> {
    if attempts >= max_attempts {
        sqlx::query!(
            r#"
            UPDATE jobs
            SET completed_at = now(),
                locked_at = NULL,
                locked_by = NULL,
                last_error = $2
            WHERE id = $1
            "#,
            id,
            error,
        )
        .execute(pool)
        .await?;
    } else {
        let run_at = Utc::now() + backoff(attempts);
        sqlx::query!(
            r#"
            UPDATE jobs
            SET locked_at = NULL,
                locked_by = NULL,
                last_error = $2,
                run_at = $3
            WHERE id = $1
            "#,
            id,
            error,
            run_at,
        )
        .execute(pool)
        .await?;
    }
    Ok(())
}
