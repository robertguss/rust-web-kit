//! Request bodies for auth routes.

use garde::Validate;
use serde::Deserialize;
use utoipa::ToSchema;

/// Register / login credentials.
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct Credentials {
    #[garde(email)]
    pub email: String,
    #[garde(length(min = 8, max = 128))]
    pub password: String,
}

/// Email-only body (forgot-password).
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct EmailBody {
    #[garde(email)]
    pub email: String,
}

/// Opaque token from an email link.
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct TokenBody {
    #[garde(length(min = 1))]
    pub token: String,
}

/// Reset password with a token.
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ResetPasswordBody {
    #[garde(length(min = 1))]
    pub token: String,
    #[garde(length(min = 8, max = 128))]
    pub password: String,
}
