use std::sync::Arc;

use api::{
    AppConfig, AppState, FakeProvider, FakeTutorProvider, MemoryRepository, Repository,
    build_router,
    services::{
        device_tokens::authenticate,
        google_auth::{GoogleIdentity, IdentityProvider},
    },
};
use async_trait::async_trait;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use contracts::{ApiEnvelope, DeviceTokenResponse, GoogleAuthCompleteRequest};
use http_body_util::BodyExt;
use tower::ServiceExt;

#[derive(Default)]
struct FakeGoogle;

#[async_trait]
impl IdentityProvider for FakeGoogle {
    async fn exchange_google_code(
        &self,
        _config: &AppConfig,
        request: &GoogleAuthCompleteRequest,
    ) -> Result<GoogleIdentity, api::error::ApiError> {
        assert_eq!(request.code, "google-authorization-code");
        Ok(GoogleIdentity {
            subject: "google-user-123".to_owned(),
        })
    }
}

#[tokio::test]
async fn a_device_cannot_be_relinked_to_a_different_google_account() {
    let repository = MemoryRepository::default();
    let original = repository
        .upsert_google_device("subject-one", "public-device", "0.1.0", "macos")
        .await
        .expect("repository works");
    let attempted_relink = repository
        .upsert_google_device("subject-two", "public-device", "0.1.0", "macos")
        .await
        .expect("repository works");

    assert!(original.is_some());
    assert!(attempted_relink.is_none());
}

#[tokio::test]
async fn verified_google_login_issues_an_opaque_device_session() {
    let state = AppState::new(
        Arc::new(AppConfig::test()),
        Arc::new(MemoryRepository::default()),
        Arc::new(FakeProvider::default()),
        Arc::new(FakeTutorProvider::default()),
    )
    .with_identity_provider(Arc::new(FakeGoogle));
    let request = GoogleAuthCompleteRequest {
        code: "google-authorization-code".to_owned(),
        code_verifier: "v".repeat(43),
        redirect_uri: "http://127.0.0.1:49152".to_owned(),
        nonce: "n".repeat(43),
        public_id: "stable-device-public-id".to_owned(),
        app_version: "0.1.0".to_owned(),
        platform: "macos".to_owned(),
    };
    let response = build_router(state.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/google/complete")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&request).expect("serialize login request"),
                ))
                .expect("static request"),
        )
        .await
        .expect("router responds");

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("body collects")
        .to_bytes();
    let envelope: ApiEnvelope<DeviceTokenResponse> =
        serde_json::from_slice(&bytes).expect("device session response");
    assert!(envelope.data.device_token.len() >= 40);
    assert!(!envelope.data.device_token.contains("google-user-123"));
    assert!(
        authenticate(&state, &envelope.data.device_token)
            .await
            .is_ok()
    );
}
