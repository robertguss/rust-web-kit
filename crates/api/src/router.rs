//! HTTP router, sessions, and security middleware.

use std::sync::Arc;
use std::time::Duration;

use axum::extract::Request;
use axum::http::{HeaderName, HeaderValue, Method, header};
use axum::routing::get;
use axum::{Json, Router};
use rwk_core::AppState;
use rwk_core::auth::PostgresSessionStore;
use rwk_core::config::Environment;
use rwk_core::error::Problem;
use rwk_core::users::UserResponse;
use time::Duration as TimeDuration;
use tower_governor::GovernorLayer;
use tower_governor::governor::GovernorConfigBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use tower_sessions::cookie::SameSite;
use tower_sessions::{Expiry, SessionManagerLayer};
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use utoipa_scalar::{Scalar, Servable};

use crate::dto::{Credentials, EmailBody, ResetPasswordBody, TokenBody};
use crate::middleware::origin_check;
use crate::routes::auth;
use crate::routes::health::{self, Health};

/// Combined API document assembled from annotated handlers.
#[derive(OpenApi)]
#[openapi(
    info(title = "rwk", version = "0.1.0"),
    tags(
        (name = "auth", description = "Password authentication"),
        (name = "health", description = "Liveness"),
    ),
    components(schemas(
        UserResponse, Problem, Health,
        Credentials, EmailBody, TokenBody, ResetPasswordBody
    ))
)]
pub struct ApiDoc;

/// Build the HTTP application (sessions + security layers included).
pub fn app(state: AppState) -> Router {
    let store = PostgresSessionStore::new(state.db().clone());
    let env = state.config().env;
    let secure = env == Environment::Production;
    let cookie_name = state.config().session.cookie_name.clone();
    let origin = HeaderValue::from_str(state.config().app_url.trim_end_matches('/'))
        .unwrap_or_else(|_| HeaderValue::from_static("http://localhost:8080"));
    let origin_state = state.clone();

    let session_layer = SessionManagerLayer::new(store)
        .with_name(cookie_name)
        .with_http_only(true)
        .with_same_site(SameSite::Lax)
        .with_secure(secure)
        .with_expiry(Expiry::OnInactivity(TimeDuration::days(14)));

    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(health::health))
        .nest("/api/auth", auth_router(env))
        .with_state(state)
        .split_for_parts();

    let spec = Arc::new(api);
    let spec_for_json = Arc::clone(&spec);

    router
        .merge(Scalar::with_url("/docs", (*spec).clone()))
        .route(
            "/api/openapi.json",
            get(move || {
                let spec = Arc::clone(&spec_for_json);
                async move { Json((*spec).clone()) }
            }),
        )
        .layer(axum::middleware::from_fn_with_state(
            origin_state,
            origin_check,
        ))
        .layer(session_layer)
        // LAST `.layer` is outermost. Request must hit SetRequestId before Trace.
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request| {
                let request_id = request
                    .headers()
                    .get("x-request-id")
                    .and_then(|value| value.to_str().ok())
                    .unwrap_or("");
                tracing::info_span!(
                    "request",
                    method = %request.method(),
                    uri = %request.uri(),
                    request_id,
                )
            }),
        )
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(TimeoutLayer::with_status_code(
            axum::http::StatusCode::GATEWAY_TIMEOUT,
            Duration::from_secs(30),
        ))
        .layer(CompressionLayer::new())
        .layer(
            CorsLayer::new()
                .allow_origin(origin)
                .allow_credentials(true)
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PATCH,
                    Method::DELETE,
                    Method::OPTIONS,
                ])
                .allow_headers([header::CONTENT_TYPE, header::ACCEPT, header::COOKIE]),
        )
        .layer(SetResponseHeaderLayer::overriding(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("referrer-policy"),
            HeaderValue::from_static("no-referrer"),
        ))
}

fn auth_router(env: Environment) -> OpenApiRouter<AppState> {
    let router = OpenApiRouter::new()
        .routes(routes!(auth::register))
        .routes(routes!(auth::login))
        .routes(routes!(auth::logout))
        .routes(routes!(auth::me))
        .routes(routes!(auth::verify_email))
        .routes(routes!(auth::forgot_password))
        .routes(routes!(auth::reset_password));

    if env == Environment::Test {
        return router;
    }

    let governor = GovernorConfigBuilder::default()
        .per_second(2)
        .burst_size(10)
        .finish()
        .expect("governor config");
    router.layer(GovernorLayer::new(governor))
}

/// Combined API spec for later `--export-openapi` use.
pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}
