//! Job queue, worker, and cleanup tests against a real Postgres.

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{Duration, Utc};
use sqlx::PgPool;

use crate::auth::tokens::{self, AuthTokenKind};
use crate::db::MIGRATOR;
use crate::mail::{Email, EmailTemplate, LogMailer, Mailer};
use crate::users::repo as users;

use super::{Job, Worker, cron_jobs, enqueue, enqueue_cron};

#[sqlx::test(migrator = "MIGRATOR")]
async fn enqueue_in_rolled_back_transaction_leaves_no_job(pool: PgPool) {
    let mut tx = pool.begin().await.unwrap();
    enqueue(&mut *tx, Job::CleanupExpiredTokens).await.unwrap();
    let in_tx: i64 = sqlx::query_scalar!(r#"SELECT COUNT(*) AS "count!" FROM jobs"#)
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    assert_eq!(in_tx, 1);
    tx.rollback().await.unwrap();
    let after: i64 = sqlx::query_scalar!(r#"SELECT COUNT(*) AS "count!" FROM jobs"#)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(after, 0);
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn worker_processes_send_email_with_log_mailer(pool: PgPool) {
    let mailer = LogMailer::new();
    let url = "http://localhost:8080/verify-email?token=rendered-token";
    enqueue(
        &pool,
        Job::send_email("user@example.com", EmailTemplate::VerifyEmail, url),
    )
    .await
    .unwrap();

    let worker = Worker::new(pool.clone(), Arc::new(mailer.clone()));
    assert!(worker.process_one().await.unwrap());

    let sent = mailer.messages();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].kind, "verify");
    assert_eq!(sent[0].to, "user@example.com");
    assert!(sent[0].html.contains(url), "html: {}", sent[0].html);
    assert!(sent[0].text.contains(url), "text: {}", sent[0].text);
    assert!(sent[0].html.contains("Welcome"));
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn failed_job_is_retried_with_backoff(pool: PgPool) {
    enqueue(
        &pool,
        Job::send_email(
            "user@example.com",
            EmailTemplate::VerifyEmail,
            "http://localhost:8080/verify-email?token=x",
        ),
    )
    .await
    .unwrap();

    let worker = Worker::new(pool.clone(), Arc::new(Boom));
    assert!(worker.process_one().await.unwrap());

    let row = sqlx::query!(
        r#"
        SELECT attempts, completed_at, last_error, run_at
        FROM jobs
        "#
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(row.attempts, 1);
    assert!(row.completed_at.is_none());
    assert!(
        row.last_error
            .as_deref()
            .unwrap_or("")
            .contains("smtp down")
    );
    assert!(row.run_at > Utc::now());
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn cleanup_deletes_expired_tokens_and_sessions(pool: PgPool) {
    let password_hash = crate::auth::password::hash_password("password12").unwrap();
    let user = users::create(&pool, "clean@example.com", &password_hash)
        .await
        .unwrap();

    tokens::insert_expired(&pool, user.id, AuthTokenKind::EmailVerify, "expired-plain")
        .await
        .unwrap();
    tokens::issue(
        &pool,
        user.id,
        AuthTokenKind::EmailVerify,
        tokens::verify_ttl(),
    )
    .await
    .unwrap();

    sqlx::query!(
        r#"
        INSERT INTO sessions (id, data, expiry_date)
        VALUES ($1, $2, $3)
        "#,
        "expired-session",
        vec![0_u8],
        Utc::now() - Duration::hours(1),
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query!(
        r#"
        INSERT INTO sessions (id, data, expiry_date)
        VALUES ($1, $2, $3)
        "#,
        "live-session",
        vec![1_u8],
        Utc::now() + Duration::hours(1),
    )
    .execute(&pool)
    .await
    .unwrap();

    let old_job = uuid::Uuid::now_v7();
    let recent_job = uuid::Uuid::now_v7();
    let empty = serde_json::json!({});
    sqlx::query!(
        r#"
        INSERT INTO jobs (id, kind, payload, run_at, completed_at)
        VALUES ($1, 'send_email', $2, now(), $3)
        "#,
        old_job,
        empty.clone(),
        Utc::now() - Duration::days(8),
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query!(
        r#"
        INSERT INTO jobs (id, kind, payload, run_at, completed_at)
        VALUES ($1, 'send_email', $2, now(), $3)
        "#,
        recent_job,
        empty,
        Utc::now() - Duration::days(1),
    )
    .execute(&pool)
    .await
    .unwrap();

    enqueue_cron(&pool, Job::CleanupExpiredTokens)
        .await
        .unwrap();
    let worker = Worker::new(pool.clone(), Arc::new(LogMailer::new()));
    assert!(worker.process_one().await.unwrap());

    let tokens: i64 = sqlx::query_scalar!(r#"SELECT COUNT(*) AS "count!" FROM auth_tokens"#)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(tokens, 1);

    let sessions: Vec<String> = sqlx::query_scalar!("SELECT id FROM sessions ORDER BY id")
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(sessions, vec!["live-session".to_owned()]);

    let leftover: Vec<uuid::Uuid> = sqlx::query_scalar!(
        r#"
        SELECT id
        FROM jobs
        WHERE kind = 'send_email'
        ORDER BY id
        "#
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(leftover, vec![recent_job]);
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn enqueue_cron_does_not_duplicate_pending_cleanup(pool: PgPool) {
    enqueue_cron(&pool, Job::CleanupExpiredTokens)
        .await
        .unwrap();
    enqueue_cron(&pool, Job::CleanupExpiredTokens)
        .await
        .unwrap();
    let pending: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) AS "count!"
        FROM jobs
        WHERE completed_at IS NULL AND kind = 'cleanup_expired_tokens'
        "#
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(pending, 1);
}

#[test]
fn cleanup_cron_is_registered_hourly() {
    let jobs = cron_jobs();
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].name, "cleanup_expired_tokens");
    assert_eq!(jobs[0].interval, super::CLEANUP_INTERVAL);
    assert_eq!(jobs[0].job, Job::CleanupExpiredTokens);
}

struct Boom;

#[async_trait]
impl Mailer for Boom {
    async fn send(&self, _: &Email) -> anyhow::Result<()> {
        anyhow::bail!("smtp down");
    }
}
