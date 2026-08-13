use api::{AppConfig, AppState, FakeProvider, FakeTutorProvider, MemoryRepository};
use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
use contracts::RegisterDeviceRequest;
use std::sync::Arc;

#[tokio::test]
async fn invite_is_redeemed_and_token_is_opaque() {
    let config = Arc::new(AppConfig::test());
    let repository = Arc::new(MemoryRepository::default());
    let salt = SaltString::encode_b64(b"0123456789abcdef").expect("valid static salt");
    let candidate = format!("TRO-TEST{}", config.invite_code_pepper.expose());
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
            invite_code: "TRO-TEST".to_owned(),
            public_id: "device-fixture".to_owned(),
            app_version: "0.1.0".to_owned(),
            platform: "macos".to_owned(),
            accepted_age_scope: true,
        },
    )
    .await
    .expect("invite works");
    assert!(token.device_token.len() >= 40);
    assert!(!token.device_token.contains("TRO-TEST"));
}
