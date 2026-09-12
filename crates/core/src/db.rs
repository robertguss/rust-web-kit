//! Postgres pool and migrations.

use std::time::Duration;

use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

use crate::config::DatabaseConfig;

/// Migrator for the workspace `migrations/` directory.
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../../migrations");

/// Connect with conservative pool settings.
pub async fn connect(database: &DatabaseConfig) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .min_connections(1)
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(600))
        .connect(&database.url)
        .await
}

/// Apply all pending migrations.
pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    MIGRATOR.run(pool).await
}

/// `SELECT 1` used by `/api/health`.
pub async fn ping(pool: &PgPool) -> Result<(), sqlx::Error> {
    let _: i32 = sqlx::query_scalar!(r#"SELECT 1 AS "one!""#)
        .fetch_one(pool)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn migrations_apply(pool: PgPool) {
        let tables: i64 = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) AS "count!"
            FROM information_schema.tables
            WHERE table_schema = 'public'
              AND table_name IN ('users', 'oauth_accounts', 'auth_tokens', 'projects')
            "#
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(tables, 4);

        let citext: Option<String> =
            sqlx::query_scalar!("SELECT extname FROM pg_extension WHERE extname = 'citext'")
                .fetch_optional(&pool)
                .await
                .unwrap();
        assert_eq!(citext.as_deref(), Some("citext"));

        ping(&pool).await.unwrap();
    }
}
