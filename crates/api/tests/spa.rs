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
    std::fs::create_dir_all(dir.join("assets")).unwrap();
    std::fs::write(dir.join("index.html"), b"<html><body>rwk-spa</body></html>").unwrap();
    std::fs::write(dir.join("ok.txt"), b"asset-ok").unwrap();
    std::fs::write(dir.join("assets").join("app.js"), b"console.log(1)").unwrap();

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

    let hashed = client.get("/assets/app.js").await;
    assert_eq!(hashed.status, StatusCode::OK);
    assert_eq!(hashed.text(), "console.log(1)");

    let missing_asset = client.get("/assets/missing.js").await;
    assert_eq!(missing_asset.status, StatusCode::NOT_FOUND);
    assert!(
        !missing_asset.text().contains("rwk-spa"),
        "hashed assets must not fall back to index.html: {}",
        missing_asset.text()
    );

    let health = client.get("/api/health").await;
    assert_eq!(health.status, StatusCode::OK);
    assert_eq!(health.json()["status"], "ok");

    let _ = std::fs::remove_dir_all(&dir);
}
