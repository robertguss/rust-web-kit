//! `/api/auth/oauth/{provider}` authorize and callback.

use axum::extract::{Path, Query, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use rwk_core::AppError;
use rwk_core::AppState;
use rwk_core::auth::CurrentUser;
use rwk_core::auth::oauth;
use rwk_core::error::Problem;
use serde::Deserialize;
use tower_sessions::Session;

#[derive(Debug, Deserialize)]
pub struct CallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
}

fn found(location: &str) -> Response {
    (StatusCode::FOUND, [(header::LOCATION, location.to_owned())]).into_response()
}

fn login_error(code: &str) -> Response {
    found(&format!("/login?error={code}"))
}

fn map_callback_failure(error: AppError) -> Result<Response, AppError> {
    match error {
        AppError::NotFound => Err(AppError::NotFound),
        AppError::Conflict => Ok(login_error("conflict")),
        AppError::BadRequest(message)
            if message.contains("state") || message.contains("mismatch") =>
        {
            Ok(login_error("state"))
        }
        AppError::BadRequest(_) | AppError::Internal(_) => Ok(login_error("exchange")),
        other => Err(other),
    }
}

/// Redirect to the provider authorize URL (PKCE + state in the session).
#[utoipa::path(
    get,
    path = "/oauth/{provider}",
    tag = "auth",
    params(("provider" = String, Path, description = "google or github")),
    responses(
        (status = 302, description = "Redirect to the provider"),
        (status = 404, description = "Unknown or unconfigured provider", body = Problem),
    )
)]
pub async fn start_oauth(
    State(state): State<AppState>,
    session: Session,
    Path(provider): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let provider = provider.parse().map_err(|()| AppError::NotFound)?;
    let url = oauth::start(state.config(), &session, provider).await?;
    Ok((StatusCode::FOUND, [(header::LOCATION, url)]))
}

/// Exchange the authorization code, log the user in, redirect into the app.
#[utoipa::path(
    get,
    path = "/oauth/{provider}/callback",
    tag = "auth",
    params(
        ("provider" = String, Path, description = "google or github"),
        ("code" = Option<String>, Query, description = "Authorization code"),
        ("state" = Option<String>, Query, description = "CSRF state"),
        ("error" = Option<String>, Query, description = "Provider error"),
    ),
    responses(
        (status = 302, description = "Redirect to /app/projects or /login?error="),
        (status = 404, description = "Unknown or unconfigured provider", body = Problem),
    )
)]
pub async fn oauth_callback(
    State(state): State<AppState>,
    session: Session,
    Path(provider): Path<String>,
    Query(query): Query<CallbackQuery>,
) -> Result<Response, AppError> {
    let provider = provider.parse().map_err(|()| AppError::NotFound)?;
    oauth::ensure_configured(state.config(), provider)?;
    if query
        .error
        .as_deref()
        .is_some_and(|value| !value.is_empty())
    {
        return Ok(login_error("provider"));
    }
    let Some(code) = query.code.filter(|value| !value.is_empty()) else {
        return Ok(login_error("provider"));
    };
    let Some(state_param) = query.state.filter(|value| !value.is_empty()) else {
        return Ok(login_error("state"));
    };
    let user = match oauth::finish(
        state.config(),
        state.db(),
        &session,
        provider,
        &code,
        &state_param,
    )
    .await
    {
        Ok(user) => user,
        Err(error) => return map_callback_failure(error),
    };
    if let Err(error) = CurrentUser::login(&session, user.id).await {
        return map_callback_failure(error);
    }
    Ok(found("/app/projects"))
}
