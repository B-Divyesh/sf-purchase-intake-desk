use std::path::{Path, PathBuf};

use axum::{
    http::{
        header::{CONTENT_SECURITY_POLICY, REFERRER_POLICY, X_CONTENT_TYPE_OPTIONS},
        HeaderValue,
    },
    routing::get,
    Json, Router,
};
use serde::Serialize;
use tower_governor::{
    governor::GovernorConfigBuilder,
    key_extractor::SmartIpKeyExtractor,
    GovernorLayer,
};
use tower_http::{
    services::{ServeDir, ServeFile},
    set_header::SetResponseHeaderLayer,
    trace::TraceLayer,
};

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
    build_sha: &'static str,
}

pub fn build_sha() -> &'static str {
    option_env!("BUILD_SHA")
        .filter(|value| !value.is_empty())
        .or(option_env!("GIT_SHA").filter(|value| !value.is_empty()))
        .or(option_env!("SOURCE_COMMIT").filter(|value| !value.is_empty()))
        .unwrap_or("dev")
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        build_sha: build_sha(),
    })
}

pub fn app(static_dir: impl AsRef<Path>) -> Router {
    let static_dir = PathBuf::from(static_dir.as_ref());
    let index = static_dir.join("index.html");
    let static_files = ServeDir::new(static_dir).not_found_service(ServeFile::new(index));

    let rate_limit = GovernorConfigBuilder::default()
        .per_millisecond(50)
        .burst_size(40)
        .key_extractor(SmartIpKeyExtractor)
        .use_headers()
        .finish()
        .expect("valid rate limit configuration");

    let limited_static = Router::new()
        .fallback_service(static_files)
        .layer(GovernorLayer::new(rate_limit));

    Router::new()
        .route("/health", get(health))
        .merge(limited_static)
        .layer(SetResponseHeaderLayer::if_not_present(
            X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            REFERRER_POLICY,
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            CONTENT_SECURITY_POLICY,
            HeaderValue::from_static(
                "default-src 'self'; base-uri 'self'; connect-src 'self'; font-src 'self'; frame-ancestors 'none'; img-src 'self' data:; object-src 'none'; script-src 'self'; style-src 'self'",
            ),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::HeaderName::from_static("permissions-policy"),
            HeaderValue::from_static("camera=(self), microphone=(), geolocation=()"),
        ))
        .layer(TraceLayer::new_for_http())
}
