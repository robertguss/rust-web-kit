//! Vite production build served with an SPA fallback.

use std::path::Path;

use axum::Router;
use tower_http::services::{ServeDir, ServeFile};

/// Attach `ServeDir` with `index.html` fallback for unknown paths.
///
/// Mount this as the router fallback so `/api` and `/docs` stay unmatched.
pub fn with_spa(router: Router, static_dir: &str) -> Router {
    let dir = Path::new(static_dir);
    let index = dir.join("index.html");
    // `fallback` keeps ServeFile's 200. `not_found_service` would force 404.
    let files = ServeDir::new(dir).fallback(ServeFile::new(index));
    router.fallback_service(files)
}
