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

    Router::new()
        .route("/health", get(health))
        .fallback_service(static_files)
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
        .layer(TraceLayer::new_for_http())
}
