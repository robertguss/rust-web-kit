//! OAuth route tests: unconfigured providers 404; configured start is 302.

mod common;

use axum::http::{StatusCode, header};
use common::TestApp;
use rwk_core::config::Config;
use sqlx::PgPool;

#[sqlx::test(migrations = "../../migrations")]
async fn unconfigured_provider_is_404(pool: PgPool) {
    let app = TestApp::new(pool);
    let mut client = app.client();

    for path in [
        "/api/auth/oauth/google",
        "/api/auth/oauth/github",
        "/api/auth/oauth/gitlab",
        "/api/auth/oauth/google/callback?code=x&state=y",
    ] {
        let response = client.get(path).await;
        assert_eq!(response.status, StatusCode::NOT_FOUND, "{path}");
        assert_eq!(
            response.headers[header::CONTENT_TYPE],
            "application/problem+json"
        );
        assert_eq!(response.json()["status"], 404);
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn configured_google_start_redirects(pool: PgPool) {
    let mut config = Config::for_tests();
    config.oauth.google_client_id = Some("google-id".into());
    config.oauth.google_client_secret = Some("google-secret".into());
    let app = TestApp::with_config(pool, config);
    let mut client = app.client();

    let response = client.get("/api/auth/oauth/google").await;
    assert_eq!(response.status, StatusCode::FOUND);
    let location = response.headers[header::LOCATION].to_str().unwrap();
    assert!(
        location.starts_with("https://accounts.google.com/o/oauth2/v2/auth"),
        "{location}"
    );
    assert!(location.contains("code_challenge="), "{location}");
    assert!(location.contains("state="), "{location}");
    assert!(
        location.contains("redirect_uri=") && location.contains("google"),
        "{location}"
    );
    assert!(client.cookie("rwk_session").is_some());
}

#[sqlx::test(migrations = "../../migrations")]
async fn callback_rejects_bad_state(pool: PgPool) {
    let mut config = Config::for_tests();
    config.oauth.github_client_id = Some("gh-id".into());
    config.oauth.github_client_secret = Some("gh-secret".into());
    let app = TestApp::with_config(pool, config);
    let mut client = app.client();

    let start = client.get("/api/auth/oauth/github").await;
    assert_eq!(start.status, StatusCode::FOUND);

    let response = client
        .get("/api/auth/oauth/github/callback?code=abc&state=not-the-csrf")
        .await;
    assert_eq!(response.status, StatusCode::FOUND);
    assert_eq!(response.headers[header::LOCATION], "/login?error=state");
}

#[sqlx::test(migrations = "../../migrations")]
async fn callback_without_start_redirects_to_login(pool: PgPool) {
    let mut config = Config::for_tests();
    config.oauth.google_client_id = Some("google-id".into());
    config.oauth.google_client_secret = Some("google-secret".into());
    let app = TestApp::with_config(pool, config);
    let mut client = app.client();

    let response = client
        .get("/api/auth/oauth/google/callback?code=abc&state=xyz")
        .await;
    assert_eq!(response.status, StatusCode::FOUND);
    assert_eq!(response.headers[header::LOCATION], "/login?error=state");
}

#[sqlx::test(migrations = "../../migrations")]
async fn callback_provider_error_redirects_to_login(pool: PgPool) {
    let mut config = Config::for_tests();
    config.oauth.google_client_id = Some("google-id".into());
    config.oauth.google_client_secret = Some("google-secret".into());
    let app = TestApp::with_config(pool, config);
    let mut client = app.client();

    let response = client
        .get("/api/auth/oauth/google/callback?error=access_denied")
        .await;
    assert_eq!(response.status, StatusCode::FOUND);
    assert_eq!(response.headers[header::LOCATION], "/login?error=provider");
}

#[sqlx::test(migrations = "../../migrations")]
async fn only_client_id_is_still_404(pool: PgPool) {
    let mut config = Config::for_tests();
    config.oauth.google_client_id = Some("google-id".into());
    let app = TestApp::with_config(pool, config);
    let mut client = app.client();
    let response = client.get("/api/auth/oauth/google").await;
    assert_eq!(response.status, StatusCode::NOT_FOUND);
}
