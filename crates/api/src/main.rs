//! rwk HTTP API binary.

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context;
use rwk_core::AppState;
use rwk_core::config::Config;
use rwk_core::db;
use rwk_core::mail::LogMailer;
use rwk_core::telemetry;

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
    let state = AppState::new(config, pool, Arc::new(LogMailer));
    let app = rwk_api::app(state);

    tracing::info!("listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("bind {addr}"))?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .context("server")?;
    Ok(())
}
