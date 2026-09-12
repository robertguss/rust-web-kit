//! Session extractors.

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use tower_sessions::Session;
use uuid::Uuid;

use crate::AppError;
use crate::state::AppState;
use crate::users::model::User;
use crate::users::repo;

const USER_ID_KEY: &str = "user_id";

/// Logged-in user, or `None`.
pub struct CurrentUser(pub Option<User>);

/// Logged-in user; missing session is 401.
pub struct RequireAuth(pub User);

/// Logged-in user whose email is verified; unverified is 403.
///
/// Opt-in per route. Do not swap this in for [`RequireAuth`] on existing
/// handlers — login and session stay available without a verified address.
pub struct RequireVerified(pub User);

impl CurrentUser {
    /// Persist `user_id` on the session after rotating the session id.
    pub async fn login(session: &Session, user_id: Uuid) -> Result<(), AppError> {
        session
            .cycle_id()
            .await
            .map_err(|error| AppError::Internal(error.into()))?;
        session
            .insert(USER_ID_KEY, user_id)
            .await
            .map_err(|error| AppError::Internal(error.into()))
    }

    /// Drop session data (logout).
    pub async fn logout(session: &Session) -> Result<(), AppError> {
        session
            .flush()
            .await
            .map_err(|error| AppError::Internal(error.into()))
    }
}

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::Unauthorized)?;
        let user_id: Option<Uuid> = session
            .get(USER_ID_KEY)
            .await
            .map_err(|error| AppError::Internal(error.into()))?;
        let Some(user_id) = user_id else {
            return Ok(Self(None));
        };
        let user = repo::find_by_id(state.db(), user_id).await?;
        Ok(Self(user))
    }
}

impl FromRequestParts<AppState> for RequireAuth {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        match CurrentUser::from_request_parts(parts, state).await? {
            CurrentUser(Some(user)) => Ok(Self(user)),
            CurrentUser(None) => Err(AppError::Unauthorized),
        }
    }
}

impl FromRequestParts<AppState> for RequireVerified {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let RequireAuth(user) = RequireAuth::from_request_parts(parts, state).await?;
        if user.email_verified_at.is_none() {
            return Err(AppError::EmailNotVerified);
        }
        Ok(Self(user))
    }
}
