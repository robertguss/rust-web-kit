//! SPA fallback serves `index.html` for non-API, non-file paths.

mod common;

use axum::http::StatusCode;
use common::TestApp;
use rwk_core::config::Config;
use sqlx::PgPool;
use uuid::Uuid;

#[sqlx::test(migrations = "../../migrations")]
async fn spa_fallback_and_assets(pool: PgPool) {
    let dir = std::env::temp_dir().join(format!("rwk-spa-{}", Uuid::now_v7()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("index.html"), b"<html><body>rwk-spa</body></html>").unwrap();
    std::fs::write(dir.join("ok.txt"), b"asset-ok").unwrap();

    let mut config = Config::for_tests();
    config.server.static_dir = dir.to_string_lossy().into_owned();
    let app = TestApp::with_config(pool, config);
    let mut client = app.client();

    let index = client.get("/").await;
    assert_eq!(index.status, StatusCode::OK);
    assert!(index.text().contains("rwk-spa"), "{}", index.text());

    let nested = client.get("/app/projects/abc").await;
    assert_eq!(nested.status, StatusCode::OK);
    assert!(nested.text().contains("rwk-spa"), "{}", nested.text());

    let asset = client.get("/ok.txt").await;
    assert_eq!(asset.status, StatusCode::OK);
    assert_eq!(asset.text(), "asset-ok");

    let health = client.get("/api/health").await;
    assert_eq!(health.status, StatusCode::OK);
    assert_eq!(health.json()["status"], "ok");

    let _ = std::fs::remove_dir_all(&dir);
}
