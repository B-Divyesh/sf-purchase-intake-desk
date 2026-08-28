use std::path::{Path, PathBuf};

use axum::{
    body::Body,
    http::{
        header::{CACHE_CONTROL, CONTENT_SECURITY_POLICY, REFERRER_POLICY, X_CONTENT_TYPE_OPTIONS},
        HeaderValue, Request, StatusCode,
    },
    middleware::{self, Next},
    response::Response,
    routing::get,
    Json, Router,
};
use serde::Serialize;
use tower_governor::{
    governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor, GovernorLayer,
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

fn known_path(path: &str) -> bool {
    path == "/"
        || path == "/health"
        || matches!(path, "/demo" | "/start" | "/app" | "/privacy" | "/terms")
        || path.starts_with("/demo/")
        || path.starts_with("/app/")
        || path.starts_with("/assets/")
        || path.starts_with("/fonts/")
        || matches!(
            path,
            "/index.html"
                | "/404.html"
                | "/favicon.svg"
                | "/apple-touch-icon.png"
                | "/apple-touch-icon.svg"
                | "/og-image.svg"
                | "/manifest.webmanifest"
                | "/robots.txt"
                | "/sitemap.xml"
                | "/staticwebapp.config.json"
                | "/sw.js"
        )
}

async fn response_policy(request: Request<Body>, next: Next) -> Response {
    let path = request.uri().path().to_owned();
    let mut response = next.run(request).await;
    if response.status() == StatusCode::OK && !known_path(&path) {
        *response.status_mut() = StatusCode::NOT_FOUND;
    }
    let cache = if path == "/sw.js"
        || path == "/index.html"
        || path == "/404.html"
        || !path.contains('.')
    {
        "no-cache, no-store, must-revalidate"
    } else if path.starts_with("/assets/") || path.starts_with("/fonts/") {
        "public, max-age=31536000, immutable"
    } else {
        "public, max-age=3600"
    };
    response
        .headers_mut()
        .insert(CACHE_CONTROL, HeaderValue::from_static(cache));
    response
}

pub fn app(static_dir: impl AsRef<Path>) -> Router {
    let static_dir = PathBuf::from(static_dir.as_ref());
    let index = static_dir.join("index.html");
    let static_files = ServeDir::new(static_dir).fallback(ServeFile::new(index));

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
                "default-src 'self'; base-uri 'self'; connect-src 'self'; font-src 'self'; frame-ancestors 'none'; img-src 'self' data: blob:; object-src 'none'; script-src 'self'; style-src 'self'",
            ),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::HeaderName::from_static("strict-transport-security"),
            HeaderValue::from_static("max-age=31536000; includeSubDomains"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::HeaderName::from_static("permissions-policy"),
            HeaderValue::from_static("camera=(self), microphone=(), geolocation=()"),
        ))
        .layer(middleware::from_fn(response_policy))
        .layer(TraceLayer::new_for_http())
}
