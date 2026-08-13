use api::{AppConfig, AppState, FakeProvider, FakeTutorProvider, MemoryRepository, build_router};
use axum::{body::Body, http::Request};
use http_body_util::BodyExt;
use std::sync::Arc;
use tower::ServiceExt;

#[tokio::test]
async fn health_is_content_free() {
    let state = AppState::new(
        Arc::new(AppConfig::test()),
        Arc::new(MemoryRepository::default()),
        Arc::new(FakeProvider::default()),
        Arc::new(FakeTutorProvider::default()),
    );
    let response = build_router(state)
        .oneshot(
            Request::builder()
                .uri("/healthz")
                .body(Body::empty())
                .expect("static request"),
        )
        .await
        .expect("router responds");
    assert!(response.status().is_success());
    let body = response
        .into_body()
        .collect()
        .await
        .expect("body collects")
        .to_bytes();
    assert_eq!(&body[..], br#"{"status":"ok"}"#);
}
