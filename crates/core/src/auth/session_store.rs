//! Postgres [`SessionStore`] compatible with tower-sessions 0.15 and sqlx 0.9.
//!
//! The published `tower-sessions-sqlx-store` crate is still on sqlx 0.8 /
//! tower-sessions 0.14, so this crate owns the store.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use time::OffsetDateTime;
use tower_sessions::SessionStore;
use tower_sessions::session::{Id, Record};
use tower_sessions::session_store::{Error, Result as StoreResult};

/// Session rows in `sessions`.
#[derive(Debug, Clone)]
pub struct PostgresSessionStore {
    pool: PgPool,
}

impl PostgresSessionStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SessionStore for PostgresSessionStore {
    async fn create(&self, record: &mut Record) -> StoreResult<()> {
        loop {
            let data = encode(record)?;
            let result = sqlx::query!(
                r#"
                INSERT INTO sessions (id, data, expiry_date)
                VALUES ($1, $2, $3)
                "#,
                record.id.to_string(),
                data,
                to_chrono(record.expiry_date),
            )
            .execute(&self.pool)
            .await;

            match result {
                Ok(_) => return Ok(()),
                Err(sqlx::Error::Database(db)) if db.code().as_deref() == Some("23505") => {
                    record.id = Id::default();
                }
                Err(error) => return Err(Error::Backend(error.to_string())),
            }
        }
    }

    async fn save(&self, record: &Record) -> StoreResult<()> {
        let data = encode(record)?;
        sqlx::query!(
            r#"
            INSERT INTO sessions (id, data, expiry_date)
            VALUES ($1, $2, $3)
            ON CONFLICT (id) DO UPDATE
            SET data = EXCLUDED.data, expiry_date = EXCLUDED.expiry_date
            "#,
            record.id.to_string(),
            data,
            to_chrono(record.expiry_date),
        )
        .execute(&self.pool)
        .await
        .map_err(|error| Error::Backend(error.to_string()))?;
        Ok(())
    }

    async fn load(&self, session_id: &Id) -> StoreResult<Option<Record>> {
        let row = sqlx::query!(
            r#"
            SELECT data, expiry_date
            FROM sessions
            WHERE id = $1 AND expiry_date > $2
            "#,
            session_id.to_string(),
            Utc::now(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| Error::Backend(error.to_string()))?;

        row.map(|row| decode(&row.data)).transpose()
    }

    async fn delete(&self, session_id: &Id) -> StoreResult<()> {
        sqlx::query!(
            r#"
            DELETE FROM sessions WHERE id = $1
            "#,
            session_id.to_string(),
        )
        .execute(&self.pool)
        .await
        .map_err(|error| Error::Backend(error.to_string()))?;
        Ok(())
    }
}

fn to_chrono(expiry: OffsetDateTime) -> DateTime<Utc> {
    DateTime::from_timestamp(expiry.unix_timestamp(), expiry.nanosecond())
        .unwrap_or(DateTime::<Utc>::UNIX_EPOCH)
}

fn encode(record: &Record) -> StoreResult<Vec<u8>> {
    serde_json::to_vec(record).map_err(|error| Error::Encode(error.to_string()))
}

fn decode(data: &[u8]) -> StoreResult<Record> {
    serde_json::from_slice(data).map_err(|error| Error::Decode(error.to_string()))
}
