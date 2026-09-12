//! Auth integration tests: register, login, me, tokens.

mod common;

use axum::http::StatusCode;
use common::TestApp;
use rwk_core::auth::tokens::{self, AuthTokenKind};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

fn creds(email: &str) -> serde_json::Value {
    json!({ "email": email, "password": "password12" })
}

#[sqlx::test(migrations = "../../migrations")]
async fn register_login_me_logout(pool: PgPool) {
    let app = TestApp::new(pool);
    let mut client = app.client();

    let register = client
        .post_json("/api/auth/register", creds("a@example.com"))
        .await;
    assert_eq!(register.status, StatusCode::CREATED);
    assert_eq!(register.json()["email"], "a@example.com");
    assert_eq!(register.json()["email_verified"], false);

    let me = client.get("/api/auth/me").await;
    assert_eq!(me.status, StatusCode::OK);
    assert_eq!(me.json()["email"], "a@example.com");

    let logout = client.post_json("/api/auth/logout", json!({})).await;
    assert_eq!(logout.status, StatusCode::NO_CONTENT);

    let me = client.get("/api/auth/me").await;
    assert_eq!(me.status, StatusCode::UNAUTHORIZED);

    let login = client
        .post_json("/api/auth/login", creds("a@example.com"))
        .await;
    assert_eq!(login.status, StatusCode::OK);
    let me = client.get("/api/auth/me").await;
    assert_eq!(me.status, StatusCode::OK);
}

#[sqlx::test(migrations = "../../migrations")]
async fn duplicate_email_is_409(pool: PgPool) {
    let app = TestApp::new(pool);
    let mut client = app.client();
    let first = client
        .post_json("/api/auth/register", creds("dup@example.com"))
        .await;
    assert_eq!(first.status, StatusCode::CREATED);
    let second = client
        .post_json("/api/auth/register", creds("dup@example.com"))
        .await;
    assert_eq!(second.status, StatusCode::CONFLICT);
    assert_eq!(
        second.headers[axum::http::header::CONTENT_TYPE],
        "application/problem+json"
    );
    assert_eq!(second.json()["status"], 409);
}

#[sqlx::test(migrations = "../../migrations")]
async fn bad_password_is_401(pool: PgPool) {
    let app = TestApp::new(pool);
    let mut client = app.client();
    client
        .post_json("/api/auth/register", creds("b@example.com"))
        .await;
    client.post_json("/api/auth/logout", json!({})).await;
    let login = client
        .post_json(
            "/api/auth/login",
            json!({ "email": "b@example.com", "password": "wrongpass" }),
        )
        .await;
    assert_eq!(login.status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "../../migrations")]
async fn validation_is_422(pool: PgPool) {
    let app = TestApp::new(pool);
    let mut client = app.client();
    let response = client
        .post_json(
            "/api/auth/register",
            json!({ "email": "not-an-email", "password": "short" }),
        )
        .await;
    assert_eq!(response.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(response.json()["status"], 422);
    assert!(
        response.json()["errors"].get("email").is_some()
            || response.json()["errors"].get("password").is_some()
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn verify_email_happy_path(pool: PgPool) {
    let app = TestApp::new(pool.clone());
    let mut client = app.client();
    client
        .post_json("/api/auth/register", creds("v@example.com"))
        .await;
    let token = app.last_token("verify");
    let response = client
        .post_json("/api/auth/verify-email", json!({ "token": token }))
        .await;
    assert_eq!(response.status, StatusCode::NO_CONTENT);
    let me = client.get("/api/auth/me").await;
    assert_eq!(me.json()["email_verified"], true);
}

#[sqlx::test(migrations = "../../migrations")]
async fn verify_email_expired(pool: PgPool) {
    let app = TestApp::new(pool.clone());
    let mut client = app.client();
    let register = client
        .post_json("/api/auth/register", creds("exp@example.com"))
        .await;
    let user_id: Uuid = serde_json::from_value(register.json()["id"].clone()).unwrap();
    let token = "expired-token-value-aaaaaaaa";
    tokens::insert_expired(&pool, user_id, AuthTokenKind::EmailVerify, token)
        .await
        .unwrap();
    let response = client
        .post_json("/api/auth/verify-email", json!({ "token": token }))
        .await;
    assert_eq!(response.status, StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "../../migrations")]
async fn reset_password_happy_and_expired(pool: PgPool) {
    let app = TestApp::new(pool.clone());
    let mut client = app.client();
    let register = client
        .post_json("/api/auth/register", creds("r@example.com"))
        .await;
    let user_id: Uuid = serde_json::from_value(register.json()["id"].clone()).unwrap();
    client.post_json("/api/auth/logout", json!({})).await;

    let forgot = client
        .post_json(
            "/api/auth/forgot-password",
            json!({ "email": "r@example.com" }),
        )
        .await;
    assert_eq!(forgot.status, StatusCode::NO_CONTENT);
    let unknown = client
        .post_json(
            "/api/auth/forgot-password",
            json!({ "email": "nobody@example.com" }),
        )
        .await;
    assert_eq!(unknown.status, StatusCode::NO_CONTENT);

    let token = app.last_token("reset");
    let reset = client
        .post_json(
            "/api/auth/reset-password",
            json!({ "token": token, "password": "newpassword1" }),
        )
        .await;
    assert_eq!(reset.status, StatusCode::NO_CONTENT);

    let old = client
        .post_json("/api/auth/login", creds("r@example.com"))
        .await;
    assert_eq!(old.status, StatusCode::UNAUTHORIZED);
    let new = client
        .post_json(
            "/api/auth/login",
            json!({ "email": "r@example.com", "password": "newpassword1" }),
        )
        .await;
    assert_eq!(new.status, StatusCode::OK);

    let expired = "expired-reset-token-bbbbbbbb";
    tokens::insert_expired(&pool, user_id, AuthTokenKind::PasswordReset, expired)
        .await
        .unwrap();
    let response = client
        .post_json(
            "/api/auth/reset-password",
            json!({ "token": expired, "password": "anotherpass1" }),
        )
        .await;
    assert_eq!(response.status, StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "../../migrations")]
async fn openapi_and_docs_exist(pool: PgPool) {
    let app = TestApp::new(pool);
    let mut client = app.client();
    let spec = client.get("/api/openapi.json").await;
    assert_eq!(spec.status, StatusCode::OK);
    assert!(
        spec.json()["paths"]["/api/auth/register"].is_object()
            || spec.json()["paths"]["/register"].is_object()
    );
    let docs = client.get("/docs").await;
    assert_eq!(docs.status, StatusCode::OK);
    assert!(
        docs.text().contains("scalar")
            || docs.text().contains("Scalar")
            || docs.text().contains("openapi")
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn responses_include_request_id(pool: PgPool) {
    let app = TestApp::new(pool);
    let mut client = app.client();
    let health = client.get("/api/health").await;
    let request_id = health
        .headers
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");
    assert!(!request_id.is_empty(), "missing x-request-id");
    let register = client
        .post_json("/api/auth/register", creds("rid@example.com"))
        .await;
    let request_id = register
        .headers
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");
    assert!(!request_id.is_empty(), "missing x-request-id on POST");
}

#[sqlx::test(migrations = "../../migrations")]
async fn login_cycles_session_cookie(pool: PgPool) {
    let app = TestApp::new(pool);
    let mut client = app.client();
    assert_eq!(
        client
            .post_json("/api/auth/register", creds("fix@example.com"))
            .await
            .status,
        StatusCode::CREATED
    );
    assert_eq!(
        client.post_json("/api/auth/logout", json!({})).await.status,
        StatusCode::NO_CONTENT
    );
    let me = client.get("/api/auth/me").await;
    assert_eq!(me.status, StatusCode::UNAUTHORIZED);
    let before = client
        .cookie("rwk_session")
        .expect("anonymous session cookie")
        .to_owned();
    let login = client
        .post_json("/api/auth/login", creds("fix@example.com"))
        .await;
    assert_eq!(login.status, StatusCode::OK);
    let after = client
        .cookie("rwk_session")
        .expect("logged-in session cookie");
    assert_ne!(before, after, "session id must rotate on login");
}
