//! rwk HTTP API binary.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use clap::Parser;
use rwk_core::AppState;
use rwk_core::config::Config;
use rwk_core::db;
use rwk_core::jobs::{self, Worker};
use rwk_core::mail::{Mailer, SmtpMailer};
use rwk_core::telemetry;

#[derive(Parser, Debug)]
#[command(name = "rwk-api", version, about = "rwk HTTP API")]
struct Args {
    /// Write the HTTP API spec to PATH and exit (does not load config or touch the database).
    #[arg(long, value_name = "PATH")]
    export_openapi: Option<PathBuf>,
    /// Serve HTTP only (do not run the job worker).
    #[arg(long, conflicts_with = "worker_only")]
    api_only: bool,
    /// Run the job worker only (do not serve HTTP).
    #[arg(long, conflicts_with = "api_only")]
    worker_only: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    if let Some(path) = args.export_openapi {
        return export_openapi(&path);
    }

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("tokio runtime")?
        .block_on(run(args))
}

fn export_openapi(path: &std::path::Path) -> anyhow::Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create directory {}", parent.display()))?;
    }
    let spec = rwk_api::openapi();
    let json = serde_json::to_string_pretty(&spec).context("serialize API spec")?;
    std::fs::write(path, format!("{json}\n"))
        .with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

async fn run(args: Args) -> anyhow::Result<()> {
    let config = Config::from_env().context("load config")?;
    telemetry::init(&config).context("init tracing")?;

    tracing::info!(version = rwk_core::VERSION, "starting rwk-api");

    let pool = db::connect(&config.database)
        .await
        .context("connect database")?;
    if config.run_migrations {
        db::run_migrations(&pool).await.context("run migrations")?;
    }

    let mailer: Arc<dyn Mailer> =
        Arc::new(SmtpMailer::from_config(&config.mail).context("smtp mailer")?);
    let state = AppState::new(config, pool.clone(), Arc::clone(&mailer));

    if args.worker_only {
        tracing::info!("running worker only");
        return jobs::run(Worker::new(pool, mailer)).await;
    }

    let worker = if args.api_only {
        None
    } else {
        tracing::info!("starting job worker");
        Some(tokio::spawn(jobs::run(Worker::new(pool, mailer))))
    };

    let result = serve_http(state).await;
    if let Some(handle) = worker {
        handle.abort();
    }
    result
}

async fn serve_http(state: AppState) -> anyhow::Result<()> {
    let addr = state
        .config()
        .listen_addr()
        .context("parse listen address")?;
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
