//! `GET /api/health` — liveness plus a `SELECT 1` database check.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use rwk_core::AppState;
use rwk_core::db;
use serde::Serialize;
use utoipa::ToSchema;

/// Health payload: `status` is overall, `db` is the ping result.
#[derive(Serialize, ToSchema)]
pub struct Health {
    pub status: &'static str,
    pub db: &'static str,
}

/// 200 when the db ping succeeds; 503 when it fails. JSON body is always returned.
#[utoipa::path(
    get,
    path = "/api/health",
    tag = "health",
    responses(
        (status = 200, description = "Healthy", body = Health),
        (status = 503, description = "Database unreachable", body = Health),
    )
)]
pub async fn health(State(state): State<AppState>) -> (StatusCode, Json<Health>) {
    match db::ping(state.db()).await {
        Ok(()) => (
            StatusCode::OK,
            Json(Health {
                status: "ok",
                db: "ok",
            }),
        ),
        Err(error) => {
            tracing::error!(%error, "health db check failed");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(Health {
                    status: "degraded",
                    db: "error",
                }),
            )
        }
    }
}
