//! Request bodies and query params.

use garde::Validate;
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

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

const fn default_page() -> i64 {
    1
}

const fn default_per_page() -> i64 {
    20
}

/// Pagination query: `page` is 1-based, `per_page` is capped at 100.
#[derive(Debug, Deserialize, Validate, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct PageQuery {
    /// 1-based page index.
    #[garde(range(min = 1))]
    #[serde(default = "default_page")]
    pub page: i64,
    /// Page size (1–100).
    #[garde(range(min = 1, max = 100))]
    #[serde(default = "default_per_page")]
    pub per_page: i64,
}

/// Create a project.
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateProject {
    #[garde(length(min = 1, max = 100))]
    pub name: String,
    #[garde(inner(length(max = 2000)))]
    pub description: Option<String>,
}

/// Patch a project. Omitted fields are left unchanged.
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateProject {
    #[garde(inner(length(min = 1, max = 100)))]
    pub name: Option<String>,
    #[garde(inner(length(max = 2000)))]
    pub description: Option<String>,
}
