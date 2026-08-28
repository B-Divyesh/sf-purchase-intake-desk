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
