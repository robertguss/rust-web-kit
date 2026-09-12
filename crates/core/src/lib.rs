//! Shared library for rwk: config, database, errors, telemetry, and app state.

pub mod config;
pub mod db;
pub mod error;
pub mod state;
pub mod telemetry;

pub use config::Config;
pub use error::AppError;
pub use state::AppState;

/// Crate version from `Cargo.toml`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
