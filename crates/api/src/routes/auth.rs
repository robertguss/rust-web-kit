//! `/api/auth/*` password-auth handlers.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use garde::Validate;
use rwk_core::AppError;
use rwk_core::AppState;
use rwk_core::auth::RequireAuth;
use rwk_core::auth::service;
use rwk_core::error::Problem;
use rwk_core::users::UserResponse;
use tower_sessions::Session;

use crate::dto::{Credentials, EmailBody, ResetPasswordBody, TokenBody};

/// Register and start a session.
#[utoipa::path(
    post,
    path = "/register",
    tag = "auth",
    request_body = Credentials,
    responses(
        (status = 201, description = "Registered", body = UserResponse),
        (status = 409, description = "Email already registered", body = Problem),
        (status = 422, description = "Validation failed", body = Problem),
    )
)]
pub async fn register(
    State(state): State<AppState>,
    session: Session,
    Json(body): Json<Credentials>,
) -> Result<(StatusCode, Json<UserResponse>), AppError> {
    body.validate()?;
    let user = service::register(
        state.db(),
        &session,
        state.mailer(),
        &state.config().app_url,
        &body.email,
        &body.password,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(UserResponse::from(user))))
}

/// Log in and start a session.
#[utoipa::path(
    post,
    path = "/login",
    tag = "auth",
    request_body = Credentials,
    responses(
        (status = 200, description = "Logged in", body = UserResponse),
        (status = 401, description = "Invalid credentials", body = Problem),
        (status = 422, description = "Validation failed", body = Problem),
    )
)]
pub async fn login(
    State(state): State<AppState>,
    session: Session,
    Json(body): Json<Credentials>,
) -> Result<Json<UserResponse>, AppError> {
    body.validate()?;
    let user = service::login(state.db(), &session, &body.email, &body.password).await?;
    Ok(Json(UserResponse::from(user)))
}

/// End the session.
#[utoipa::path(
    post,
    path = "/logout",
    tag = "auth",
    responses((status = 204, description = "Logged out"))
)]
pub async fn logout(session: Session) -> Result<StatusCode, AppError> {
    service::logout(&session).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Current user from the session cookie.
#[utoipa::path(
    get,
    path = "/me",
    tag = "auth",
    responses(
        (status = 200, description = "Current user", body = UserResponse),
        (status = 401, description = "Not authenticated", body = Problem),
    )
)]
pub async fn me(RequireAuth(user): RequireAuth) -> Json<UserResponse> {
    Json(UserResponse::from(user))
}

/// Confirm email with a token from the verify link.
#[utoipa::path(
    post,
    path = "/verify-email",
    tag = "auth",
    request_body = TokenBody,
    responses(
        (status = 204, description = "Email verified"),
        (status = 400, description = "Invalid or expired token", body = Problem),
    )
)]
pub async fn verify_email(
    State(state): State<AppState>,
    Json(body): Json<TokenBody>,
) -> Result<StatusCode, AppError> {
    body.validate()?;
    service::verify_email(state.db(), &body.token).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Always 204. Sends a reset link when the email exists.
#[utoipa::path(
    post,
    path = "/forgot-password",
    tag = "auth",
    request_body = EmailBody,
    responses((status = 204, description = "Accepted"))
)]
pub async fn forgot_password(
    State(state): State<AppState>,
    Json(body): Json<EmailBody>,
) -> Result<StatusCode, AppError> {
    body.validate()?;
    service::forgot_password(
        state.db(),
        state.mailer(),
        &state.config().app_url,
        &body.email,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Set a new password using a reset token.
#[utoipa::path(
    post,
    path = "/reset-password",
    tag = "auth",
    request_body = ResetPasswordBody,
    responses(
        (status = 204, description = "Password updated"),
        (status = 400, description = "Invalid or expired token", body = Problem),
        (status = 422, description = "Validation failed", body = Problem),
    )
)]
pub async fn reset_password(
    State(state): State<AppState>,
    Json(body): Json<ResetPasswordBody>,
) -> Result<StatusCode, AppError> {
    body.validate()?;
    service::reset_password(state.db(), &body.token, &body.password).await?;
    Ok(StatusCode::NO_CONTENT)
}
