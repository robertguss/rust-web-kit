//! [`AppError`] mapped to RFC 9457 Problem Details.

use std::collections::BTreeMap;

use axum::Json;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::Serialize;

/// Application error. Internal details are logged, never sent to clients.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("not found")]
    NotFound,
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("validation failed")]
    Validation(garde::Report),
    #[error("conflict")]
    Conflict,
    #[error("{0}")]
    BadRequest(String),
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

/// RFC 9457 problem+json body.
#[derive(Debug, Serialize)]
pub struct Problem {
    #[serde(rename = "type")]
    pub type_uri: &'static str,
    pub title: &'static str,
    pub status: u16,
    pub detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<BTreeMap<String, Vec<String>>>,
}

impl AppError {
    fn problem(&self) -> (StatusCode, Problem) {
        match self {
            Self::NotFound => (
                StatusCode::NOT_FOUND,
                Problem {
                    type_uri: "about:blank",
                    title: "Not Found",
                    status: 404,
                    detail: "The requested resource was not found.".into(),
                    instance: None,
                    errors: None,
                },
            ),
            Self::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                Problem {
                    type_uri: "about:blank",
                    title: "Unauthorized",
                    status: 401,
                    detail: "Authentication is required.".into(),
                    instance: None,
                    errors: None,
                },
            ),
            Self::Forbidden => (
                StatusCode::FORBIDDEN,
                Problem {
                    type_uri: "about:blank",
                    title: "Forbidden",
                    status: 403,
                    detail: "You are not allowed to perform this action.".into(),
                    instance: None,
                    errors: None,
                },
            ),
            Self::Validation(report) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                Problem {
                    type_uri: "about:blank",
                    title: "Unprocessable Entity",
                    status: 422,
                    detail: "One or more fields failed validation.".into(),
                    instance: None,
                    errors: Some(field_errors(report)),
                },
            ),
            Self::Conflict => (
                StatusCode::CONFLICT,
                Problem {
                    type_uri: "about:blank",
                    title: "Conflict",
                    status: 409,
                    detail: "The request conflicts with the current state.".into(),
                    instance: None,
                    errors: None,
                },
            ),
            Self::BadRequest(detail) => (
                StatusCode::BAD_REQUEST,
                Problem {
                    type_uri: "about:blank",
                    title: "Bad Request",
                    status: 400,
                    detail: detail.clone(),
                    instance: None,
                    errors: None,
                },
            ),
            Self::Internal(err) => {
                tracing::error!(error = %err, "internal error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Problem {
                        type_uri: "about:blank",
                        title: "Internal Server Error",
                        status: 500,
                        detail: "An internal error occurred.".into(),
                        instance: None,
                        errors: None,
                    },
                )
            }
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, problem) = self.problem();
        let mut response = (status, Json(problem)).into_response();
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/problem+json"),
        );
        response
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match &err {
            sqlx::Error::RowNotFound => Self::NotFound,
            sqlx::Error::Database(db) if db.code().as_deref() == Some("23505") => Self::Conflict,
            _ => Self::Internal(err.into()),
        }
    }
}

impl From<garde::Report> for AppError {
    fn from(report: garde::Report) -> Self {
        Self::Validation(report)
    }
}

fn field_errors(report: &garde::Report) -> BTreeMap<String, Vec<String>> {
    let mut errors: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (path, error) in report.iter() {
        errors
            .entry(path.to_string())
            .or_default()
            .push(error.message().to_string());
    }
    errors
}

#[cfg(test)]
mod tests {
    use axum::body::to_bytes;
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use garde::Validate;
    use serde_json::Value;

    use super::*;

    #[derive(Validate)]
    struct Sample {
        #[garde(length(min = 8))]
        password: String,
    }

    async fn problem_json(error: AppError) -> (StatusCode, String, Value) {
        let response = error.into_response();
        let status = response.status();
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .expect("content-type")
            .to_str()
            .unwrap()
            .to_owned();
        let bytes = to_bytes(response.into_body(), 64 * 1024).await.unwrap();
        let body: Value = serde_json::from_slice(&bytes).unwrap();
        (status, content_type, body)
    }

    #[tokio::test]
    async fn not_found_is_404_problem_json() {
        let (status, ct, body) = problem_json(AppError::NotFound).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(ct, "application/problem+json");
        assert_eq!(body["status"], 404);
        assert_eq!(body["title"], "Not Found");
        assert!(body.get("errors").is_none());
    }

    #[tokio::test]
    async fn unauthorized_forbidden_conflict_bad_request_codes() {
        for (error, code) in [
            (AppError::Unauthorized, StatusCode::UNAUTHORIZED),
            (AppError::Forbidden, StatusCode::FORBIDDEN),
            (AppError::Conflict, StatusCode::CONFLICT),
            (AppError::BadRequest("nope".into()), StatusCode::BAD_REQUEST),
        ] {
            let (status, ct, body) = problem_json(error).await;
            assert_eq!(status, code);
            assert_eq!(ct, "application/problem+json");
            assert_eq!(body["status"], i64::from(code.as_u16()));
        }
    }

    #[tokio::test]
    async fn validation_is_422_with_field_errors() {
        let report = Sample {
            password: "x".into(),
        }
        .validate()
        .unwrap_err();
        let (status, _, body) = problem_json(AppError::from(report)).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(body["status"], 422);
        let errors = body["errors"]["password"]
            .as_array()
            .expect("password errors");
        assert!(!errors.is_empty());
    }

    #[tokio::test]
    async fn internal_is_500_and_does_not_leak() {
        let (status, _, body) =
            problem_json(AppError::Internal(anyhow::anyhow!("secret sauce"))).await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(body["detail"], "An internal error occurred.");
        let dumped = body.to_string();
        assert!(!dumped.contains("secret sauce"));
    }

    #[tokio::test]
    async fn sqlx_row_not_found_maps_to_404() {
        let (status, _, _) = problem_json(AppError::from(sqlx::Error::RowNotFound)).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }
}
