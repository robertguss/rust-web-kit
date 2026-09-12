//! rwk HTTP API binary.

use anyhow::Context;
use axum::Router;
use axum::http::Request;
use axum::routing::get;
use rwk_core::AppState;
use rwk_core::config::Config;
use rwk_core::db;
use rwk_core::telemetry;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::TraceLayer;

mod health;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::from_env().context("load config")?;
    telemetry::init(&config).context("init tracing")?;

    tracing::info!(version = rwk_core::VERSION, "starting rwk-api");

    let pool = db::connect(&config.database)
        .await
        .context("connect database")?;
    if config.run_migrations {
        db::run_migrations(&pool).await.context("run migrations")?;
    }

    let addr = config.listen_addr().context("parse listen address")?;
    let state = AppState::new(config, pool);

    let app = Router::new()
        .route("/api/health", get(health::health))
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                let request_id = request
                    .headers()
                    .get("x-request-id")
                    .and_then(|value| value.to_str().ok())
                    .unwrap_or("");
                tracing::info_span!(
                    "request",
                    method = %request.method(),
                    uri = %request.uri(),
                    request_id,
                )
            }),
        )
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .with_state(state);

    tracing::info!("listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("bind {addr}"))?;
    axum::serve(listener, app).await.context("server")?;
    Ok(())
}
