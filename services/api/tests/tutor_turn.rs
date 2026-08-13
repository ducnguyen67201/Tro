use std::sync::Arc;

use api::{AppConfig, AppState, FakeProvider, FakeTutorProvider, MemoryRepository, build_router};
use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use contracts::{
    ApiEnvelope, ImageMime, RegisterDeviceRequest, ScreenFrameMeta, TutorTurnMetadata,
    TutorTurnResponse,
};
use http_body_util::BodyExt;
use tower::ServiceExt;

async fn authenticated_state() -> (AppState, String) {
    let mut config = AppConfig::test();
    config.device_daily_screenshots = 1;
    let config = Arc::new(config);
    let repository = Arc::new(MemoryRepository::default());
    let salt = SaltString::encode_b64(b"0123456789abcdef").expect("valid static salt");
    let candidate = format!("TRO-TUTOR{}", config.invite_code_pepper.expose());
    let hash = Argon2::default()
        .hash_password(candidate.as_bytes(), &salt)
        .expect("fixture hash")
        .to_string();
    repository.seed_invite_hash(hash, 1);
    let state = AppState::new(
        config,
        repository,
        Arc::new(FakeProvider::default()),
        Arc::new(FakeTutorProvider::default()),
    );
    let token = api::services::device_tokens::register_device(
        &state,
        RegisterDeviceRequest {
            invite_code: "TRO-TUTOR".to_owned(),
            public_id: "tutor-device".to_owned(),
            app_version: "0.1.0".to_owned(),
            platform: "macos".to_owned(),
            accepted_age_scope: true,
        },
    )
    .await
    .expect("fixture registration")
    .device_token;
    (state, token)
}

fn multipart_body() -> (String, Vec<u8>) {
    let boundary = "tro-test-boundary";
    let metadata = serde_json::to_vec(&TutorTurnMetadata {
        locale: "vi-VN".to_owned(),
        frame: ScreenFrameMeta {
            frame_id: "fixture".to_owned(),
            monitor_id: "main".to_owned(),
            width_px: 100,
            height_px: 100,
            origin_x_px: 0,
            origin_y_px: 0,
            scale_factor: 1.0,
            layout_generation: 1,
            mime_type: ImageMime::Jpeg,
        },
    })
    .expect("metadata serializes");
    let mut wav = vec![0_u8; 44];
    wav[0..4].copy_from_slice(b"RIFF");
    wav[8..12].copy_from_slice(b"WAVE");
    let jpeg = [0xff, 0xd8, 0xff, 0xd9];
    let mut body = Vec::new();
    add_part(
        &mut body,
        boundary,
        "metadata",
        "metadata.json",
        "application/json",
        &metadata,
    );
    add_part(
        &mut body,
        boundary,
        "audio",
        "question.wav",
        "audio/wav",
        &wav,
    );
    add_part(
        &mut body,
        boundary,
        "screenshot",
        "screen.jpg",
        "image/jpeg",
        &jpeg,
    );
    body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());
    (boundary.to_owned(), body)
}

fn add_part(
    body: &mut Vec<u8>,
    boundary: &str,
    name: &str,
    filename: &str,
    content_type: &str,
    value: &[u8],
) {
    body.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"{name}\"; filename=\"{filename}\"\r\nContent-Type: {content_type}\r\n\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(value);
    body.extend_from_slice(b"\r\n");
}

#[tokio::test]
async fn tutor_turn_requires_device_auth_and_returns_only_guidance() {
    let (state, token) = authenticated_state().await;
    let (boundary, body) = multipart_body();
    let unauthorized = build_router(state.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/tutor/turns")
                .header(
                    "content-type",
                    format!("multipart/form-data; boundary={boundary}"),
                )
                .body(Body::from(body.clone()))
                .expect("static request"),
        )
        .await
        .expect("router responds");
    assert_eq!(unauthorized.status(), StatusCode::UNAUTHORIZED);

    let response = build_router(state.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/tutor/turns")
                .header("authorization", format!("Bearer {token}"))
                .header(
                    "content-type",
                    format!("multipart/form-data; boundary={boundary}"),
                )
                .body(Body::from(body))
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
    let envelope: ApiEnvelope<TutorTurnResponse> =
        serde_json::from_slice(&bytes).expect("response contract");
    assert_eq!(envelope.data.guidance, "Hãy bắt đầu từ dữ kiện đầu tiên.");
    assert!(!String::from_utf8_lossy(&bytes).contains("provider"));

    let (boundary, body) = multipart_body();
    let limited = build_router(state)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/tutor/turns")
                .header("authorization", format!("Bearer {token}"))
                .header(
                    "content-type",
                    format!("multipart/form-data; boundary={boundary}"),
                )
                .body(Body::from(body))
                .expect("static request"),
        )
        .await
        .expect("router responds");
    assert_eq!(limited.status(), StatusCode::TOO_MANY_REQUESTS);
}
