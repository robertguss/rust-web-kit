//! rwk HTTP API library (binary + integration tests).

pub mod dto;
pub mod middleware;
pub mod router;
pub mod routes;
pub mod static_files;

pub use router::{ApiDoc, app, openapi};
