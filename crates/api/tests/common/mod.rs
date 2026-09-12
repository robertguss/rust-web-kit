//! Shared integration-test helper: test app + cookie-aware client.
#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use rwk_core::AppState;
use rwk_core::config::Config;
use rwk_core::jobs::Worker;
use rwk_core::mail::RecordingMailer;
use sqlx::PgPool;
use tower::ServiceExt;

/// Live app plus captured mail.
pub struct TestApp {
    pub router: Router,
    pub mail: RecordingMailer,
    pub pool: PgPool,
}

impl TestApp {
    pub fn new(pool: PgPool) -> Self {
        let config = Config::for_tests();
        let mail = RecordingMailer::new();
        let state = AppState::new(config, pool.clone(), Arc::new(mail.clone()));
        Self {
            router: rwk_api::app(state),
            mail,
            pool,
        }
    }

    pub fn client(&self) -> TestClient {
        TestClient {
            router: self.router.clone(),
            cookies: HashMap::new(),
        }
    }

    pub async fn drain_jobs(&self) {
        let worker = Worker::new(self.pool.clone(), Arc::new(self.mail.clone()));
        worker.process_available().await.expect("drain jobs");
    }

    pub async fn last_token(&self, kind: &str) -> String {
        self.drain_jobs().await;
        let messages = self.mail.messages();
        let url = messages
            .iter()
            .rev()
            .find(|message| message.kind == kind)
            .unwrap_or_else(|| panic!("missing {kind} mail"))
            .url
            .as_str();
        url.split("token=").nth(1).expect("token query").to_owned()
    }
}

/// Cookie-aware oneshot client.
pub struct TestClient {
    router: Router,
    cookies: HashMap<String, String>,
}

pub struct TestResponse {
    pub status: StatusCode,
    pub headers: axum::http::HeaderMap,
    pub body: Vec<u8>,
}

impl TestResponse {
    pub fn json(&self) -> serde_json::Value {
        serde_json::from_slice(&self.body).unwrap_or(serde_json::Value::Null)
    }

    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }
}

impl TestClient {
    pub async fn request(
        &mut self,
        method: &str,
        path: &str,
        body: Option<serde_json::Value>,
    ) -> TestResponse {
        let mut builder = Request::builder().method(method).uri(path);
        if !self.cookies.is_empty() {
            let cookie = self
                .cookies
                .iter()
                .map(|(name, value)| format!("{name}={value}"))
                .collect::<Vec<_>>()
                .join("; ");
            builder = builder.header(header::COOKIE, cookie);
        }
        let request = if let Some(json) = body {
            builder
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json.to_string()))
                .unwrap()
        } else {
            builder.body(Body::empty()).unwrap()
        };

        let response = self.router.clone().oneshot(request).await.unwrap();
        self.capture_cookies(response.headers());
        let status = response.status();
        let headers = response.headers().clone();
        let body = to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap()
            .to_vec();
        TestResponse {
            status,
            headers,
            body,
        }
    }

    pub async fn post_json(&mut self, path: &str, body: serde_json::Value) -> TestResponse {
        self.request("POST", path, Some(body)).await
    }

    pub async fn get(&mut self, path: &str) -> TestResponse {
        self.request("GET", path, None).await
    }

    #[allow(dead_code)]
    pub async fn patch_json(&mut self, path: &str, body: serde_json::Value) -> TestResponse {
        self.request("PATCH", path, Some(body)).await
    }

    #[allow(dead_code)]
    pub async fn delete(&mut self, path: &str) -> TestResponse {
        self.request("DELETE", path, None).await
    }

    pub fn cookie(&self, name: &str) -> Option<&str> {
        self.cookies.get(name).map(String::as_str)
    }

    fn capture_cookies(&mut self, headers: &axum::http::HeaderMap) {
        for value in headers.get_all(header::SET_COOKIE) {
            let Ok(value) = value.to_str() else { continue };
            let Some(pair) = value.split(';').next() else {
                continue;
            };
            if let Some((name, cookie_value)) = pair.split_once('=') {
                self.cookies
                    .insert(name.trim().to_owned(), cookie_value.trim().to_owned());
            }
        }
    }
}
