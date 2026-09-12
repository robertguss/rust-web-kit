//! Project CRUD integration tests: ownership, pagination, validation.

mod common;

use axum::http::StatusCode;
use common::{TestApp, TestClient};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

fn creds(email: &str) -> serde_json::Value {
    json!({ "email": email, "password": "password12" })
}

async fn register(client: &mut TestClient, email: &str) {
    let response = client.post_json("/api/auth/register", creds(email)).await;
    assert_eq!(response.status, StatusCode::CREATED);
}

#[sqlx::test(migrations = "../../migrations")]
async fn unauthenticated_is_401(pool: PgPool) {
    let app = TestApp::new(pool);
    let mut client = app.client();
    let list = client.get("/api/projects").await;
    assert_eq!(list.status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        list.headers[axum::http::header::CONTENT_TYPE],
        "application/problem+json"
    );

    let create = client
        .post_json("/api/projects", json!({ "name": "Nope" }))
        .await;
    assert_eq!(create.status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "../../migrations")]
async fn full_crud(pool: PgPool) {
    let app = TestApp::new(pool);
    let mut client = app.client();
    register(&mut client, "owner@example.com").await;

    let created = client
        .post_json(
            "/api/projects",
            json!({ "name": "Alpha", "description": "first" }),
        )
        .await;
    assert_eq!(created.status, StatusCode::CREATED);
    let id = created.json()["id"].as_str().expect("id").to_owned();
    assert_eq!(created.json()["name"], "Alpha");
    assert_eq!(created.json()["description"], "first");

    let got = client.get(&format!("/api/projects/{id}")).await;
    assert_eq!(got.status, StatusCode::OK);
    assert_eq!(got.json()["name"], "Alpha");

    let patched = client
        .patch_json(&format!("/api/projects/{id}"), json!({ "name": "Beta" }))
        .await;
    assert_eq!(patched.status, StatusCode::OK);
    assert_eq!(patched.json()["name"], "Beta");
    assert_eq!(patched.json()["description"], "first");

    let listed = client.get("/api/projects").await;
    assert_eq!(listed.status, StatusCode::OK);
    assert_eq!(listed.json()["total"], 1);
    assert_eq!(listed.json()["page"], 1);
    assert_eq!(listed.json()["per_page"], 20);
    assert_eq!(listed.json()["items"][0]["name"], "Beta");

    let deleted = client.delete(&format!("/api/projects/{id}")).await;
    assert_eq!(deleted.status, StatusCode::NO_CONTENT);

    let missing = client.get(&format!("/api/projects/{id}")).await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "../../migrations")]
async fn other_user_sees_404(pool: PgPool) {
    let app = TestApp::new(pool);
    let mut alice = app.client();
    let mut bob = app.client();
    register(&mut alice, "alice@example.com").await;
    register(&mut bob, "bob@example.com").await;

    let created = alice
        .post_json("/api/projects", json!({ "name": "Secret" }))
        .await;
    assert_eq!(created.status, StatusCode::CREATED);
    let id = created.json()["id"].as_str().expect("id").to_owned();

    let got = bob.get(&format!("/api/projects/{id}")).await;
    assert_eq!(got.status, StatusCode::NOT_FOUND);

    let patched = bob
        .patch_json(&format!("/api/projects/{id}"), json!({ "name": "Stolen" }))
        .await;
    assert_eq!(patched.status, StatusCode::NOT_FOUND);

    let deleted = bob.delete(&format!("/api/projects/{id}")).await;
    assert_eq!(deleted.status, StatusCode::NOT_FOUND);

    let listed = bob.get("/api/projects").await;
    assert_eq!(listed.status, StatusCode::OK);
    assert_eq!(listed.json()["total"], 0);
    assert_eq!(listed.json()["items"].as_array().map(Vec::len), Some(0));

    let still = alice.get(&format!("/api/projects/{id}")).await;
    assert_eq!(still.status, StatusCode::OK);
    assert_eq!(still.json()["name"], "Secret");
}

#[sqlx::test(migrations = "../../migrations")]
async fn pagination(pool: PgPool) {
    let app = TestApp::new(pool);
    let mut client = app.client();
    register(&mut client, "pages@example.com").await;

    for name in ["one", "two", "three"] {
        let created = client
            .post_json("/api/projects", json!({ "name": name }))
            .await;
        assert_eq!(created.status, StatusCode::CREATED);
    }

    let page1 = client.get("/api/projects?page=1&per_page=2").await;
    assert_eq!(page1.status, StatusCode::OK);
    assert_eq!(page1.json()["total"], 3);
    assert_eq!(page1.json()["page"], 1);
    assert_eq!(page1.json()["per_page"], 2);
    assert_eq!(page1.json()["items"].as_array().map(Vec::len), Some(2));

    let page2 = client.get("/api/projects?page=2&per_page=2").await;
    assert_eq!(page2.status, StatusCode::OK);
    assert_eq!(page2.json()["total"], 3);
    assert_eq!(page2.json()["page"], 2);
    assert_eq!(page2.json()["items"].as_array().map(Vec::len), Some(1));

    let page1_json = page1.json();
    let page2_json = page2.json();
    let names: Vec<&str> = page1_json["items"]
        .as_array()
        .unwrap()
        .iter()
        .chain(page2_json["items"].as_array().unwrap())
        .map(|item| item["name"].as_str().unwrap())
        .collect();
    assert_eq!(names.len(), 3);
}

#[sqlx::test(migrations = "../../migrations")]
async fn validation_errors(pool: PgPool) {
    let app = TestApp::new(pool);
    let mut client = app.client();
    register(&mut client, "valid@example.com").await;

    let empty_name = client
        .post_json("/api/projects", json!({ "name": "" }))
        .await;
    assert_eq!(empty_name.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        empty_name.headers[axum::http::header::CONTENT_TYPE],
        "application/problem+json"
    );
    assert!(empty_name.json()["errors"]["name"].is_array());

    let page_zero = client.get("/api/projects?page=0").await;
    assert_eq!(page_zero.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(page_zero.json()["errors"]["page"].is_array());

    let too_big = client.get("/api/projects?per_page=101").await;
    assert_eq!(too_big.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(too_big.json()["errors"]["per_page"].is_array());

    let created = client
        .post_json("/api/projects", json!({ "name": "Keep" }))
        .await;
    let id = created.json()["id"].as_str().expect("id").to_owned();
    let empty_patch = client
        .patch_json(&format!("/api/projects/{id}"), json!({ "name": "" }))
        .await;
    assert_eq!(empty_patch.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(empty_patch.json()["errors"]["name"].is_array());
}

#[sqlx::test(migrations = "../../migrations")]
async fn missing_id_is_404(pool: PgPool) {
    let app = TestApp::new(pool);
    let mut client = app.client();
    register(&mut client, "gone@example.com").await;
    let id = Uuid::now_v7();
    let got = client.get(&format!("/api/projects/{id}")).await;
    assert_eq!(got.status, StatusCode::NOT_FOUND);
}

#[test]
fn openapi_export_includes_project_paths() {
    let spec = serde_json::to_value(rwk_api::openapi()).expect("serialize spec");
    let paths = spec["paths"].as_object().expect("paths");
    assert!(paths.contains_key("/health"), "missing /health");
    assert!(
        paths.contains_key("/auth/register"),
        "missing /auth/register"
    );
    assert!(paths.contains_key("/projects"), "missing /projects");
    assert!(
        paths.contains_key("/projects/{id}"),
        "missing /projects/{{id}}"
    );
    assert_eq!(spec["servers"][0]["url"], "/api");
}
