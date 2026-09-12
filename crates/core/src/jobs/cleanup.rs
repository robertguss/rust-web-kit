//! Hourly maintenance: drop expired auth tokens, sessions, and old job rows.

use sqlx::PgPool;

/// Delete expired `auth_tokens` and `sessions`, and jobs finished more than 7 days ago.
pub async fn run(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query!("DELETE FROM auth_tokens WHERE expires_at < now()")
        .execute(pool)
        .await?;
    sqlx::query!("DELETE FROM sessions WHERE expiry_date < now()")
        .execute(pool)
        .await?;
    sqlx::query!(
        r#"
        DELETE FROM jobs
        WHERE completed_at < now() - interval '7 days'
        "#
    )
    .execute(pool)
    .await?;
    Ok(())
}
