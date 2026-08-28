use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use intake_desk_api::app;
use serde_json::Value;
use tower::ServiceExt;

#[tokio::test]
async fn health_reports_status_and_build_identity() {
    let response = app("../dist")
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .expect("valid request"),
        )
        .await
        .expect("health response");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["x-content-type-options"], "nosniff");

    let body = response
        .into_body()
        .collect()
        .await
        .expect("read health body")
        .to_bytes();
    let payload: Value = serde_json::from_slice(&body).expect("valid JSON");

    assert_eq!(payload["status"], "ok");
    assert!(payload["build_sha"]
        .as_str()
        .is_some_and(|sha| !sha.is_empty()));
}

#[tokio::test]
async fn static_routes_rate_limit_by_first_forwarded_ip() {
    let app = app("../dist");
    let mut limited = None;
    for _ in 0..48 {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/robots.txt")
                    .header("x-forwarded-for", "203.0.113.12, 10.0.0.8")
                    .body(Body::empty())
                    .expect("valid request"),
            )
            .await
            .expect("static response");
        if response.status() == StatusCode::TOO_MANY_REQUESTS {
            limited = Some(response);
            break;
        }
    }

    let response = limited.expect("burst must be rate limited");
    assert!(response.headers().contains_key("retry-after"));
}

#[tokio::test]
async fn deep_links_return_the_spa_with_success_status() {
    let fixture =
        std::env::temp_dir().join(format!("intake-desk-static-fixture-{}", std::process::id()));
    std::fs::create_dir_all(&fixture).expect("create fixture directory");
    std::fs::write(
        fixture.join("index.html"),
        "<!doctype html><main>app</main>",
    )
    .expect("write fixture index");

    let response = app(&fixture)
        .oneshot(
            Request::builder()
                .uri("/demo/receive/po-nb-1047")
                .header("x-forwarded-for", "203.0.113.50")
                .body(Body::empty())
                .expect("valid request"),
        )
        .await
        .expect("deep link response");

    assert_eq!(response.status(), StatusCode::OK);
    std::fs::remove_dir_all(&fixture).expect("remove fixture directory");
}

#[tokio::test]
async fn unknown_routes_return_the_spa_with_not_found_status() {
    let fixture = std::env::temp_dir().join(format!(
        "intake-desk-not-found-fixture-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&fixture).expect("create fixture directory");
    std::fs::write(
        fixture.join("index.html"),
        "<!doctype html><main>app</main>",
    )
    .expect("write fixture index");

    let response = app(&fixture)
        .oneshot(
            Request::builder()
                .uri("/not-a-real-route")
                .header("x-forwarded-for", "203.0.113.51")
                .body(Body::empty())
                .expect("valid request"),
        )
        .await
        .expect("not-found response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(
        response.headers()["cache-control"],
        "no-cache, no-store, must-revalidate"
    );
    std::fs::remove_dir_all(&fixture).expect("remove fixture directory");
}

#[tokio::test]
async fn hashed_assets_receive_immutable_cache_policy() {
    let fixture =
        std::env::temp_dir().join(format!("intake-desk-cache-fixture-{}", std::process::id()));
    std::fs::create_dir_all(fixture.join("assets")).expect("create assets directory");
    std::fs::write(fixture.join("assets/app-abc123.js"), "export {}").expect("write asset");

    let response = app(&fixture)
        .oneshot(
            Request::builder()
                .uri("/assets/app-abc123.js")
                .header("x-forwarded-for", "203.0.113.52")
                .body(Body::empty())
                .expect("valid request"),
        )
        .await
        .expect("asset response");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers()["cache-control"],
        "public, max-age=31536000, immutable"
    );
    std::fs::remove_dir_all(&fixture).expect("remove fixture directory");
}
