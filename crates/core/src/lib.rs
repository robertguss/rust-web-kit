//! Shared library for rwk: config, database, errors, auth, users, projects, jobs, mail, telemetry.

pub mod auth;
pub mod config;
pub mod db;
pub mod error;
pub mod jobs;
pub mod mail;
pub mod projects;
pub mod state;
pub mod telemetry;
pub mod users;

pub use config::Config;
pub use error::AppError;
pub use state::AppState;

/// Crate version from `Cargo.toml`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
