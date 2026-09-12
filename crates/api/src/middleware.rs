//! Origin check for mutating requests.

use axum::extract::{Request, State};
use axum::http::{Method, header};
use axum::middleware::Next;
use axum::response::Response;
use rwk_core::AppError;
use rwk_core::AppState;
use rwk_core::config::Environment;

/// Reject mutating requests whose `Origin` does not match `app_url`.
/// Missing Origin is allowed (same-site navigations, tests).
pub async fn origin_check(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    if matches!(
        *request.method(),
        Method::GET | Method::HEAD | Method::OPTIONS
    ) || state.config().env == Environment::Test
    {
        return Ok(next.run(request).await);
    }

    let Some(origin) = request.headers().get(header::ORIGIN) else {
        return Ok(next.run(request).await);
    };
    let origin = origin.to_str().unwrap_or_default();
    let expected = state.config().app_url.trim_end_matches('/');
    if origin.trim_end_matches('/') != expected {
        return Err(AppError::Forbidden);
    }
    Ok(next.run(request).await)
}
