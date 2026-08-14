use std::time::Duration;

use async_trait::async_trait;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use contracts::{
    DeviceTokenResponse, GoogleAuthCompleteRequest, GoogleAuthStartRequest, GoogleAuthStartResponse,
};
use hmac::{Hmac, Mac};
use openidconnect::{
    AccessTokenHash, AuthorizationCode, ClientId, ClientSecret, IssuerUrl, Nonce,
    OAuth2TokenResponse, PkceCodeVerifier, RedirectUrl, TokenResponse,
    core::{CoreClient, CoreProviderMetadata},
    reqwest,
};
use sha2::{Digest, Sha256};

use crate::{
    config::AppConfig, error::ApiError, services::device_tokens::issue_token, state::AppState,
};

const GOOGLE_ISSUER: &str = "https://accounts.google.com";
const GOOGLE_AUTHORIZATION_ENDPOINT: &str = "https://accounts.google.com/o/oauth2/v2/auth";

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GoogleIdentity {
    pub subject: String,
}

#[async_trait]
pub trait IdentityProvider: Send + Sync {
    async fn exchange_google_code(
        &self,
        config: &AppConfig,
        request: &GoogleAuthCompleteRequest,
    ) -> Result<GoogleIdentity, ApiError>;
}

#[derive(Default)]
pub struct GoogleIdentityProvider;

#[async_trait]
impl IdentityProvider for GoogleIdentityProvider {
    async fn exchange_google_code(
        &self,
        config: &AppConfig,
        request: &GoogleAuthCompleteRequest,
    ) -> Result<GoogleIdentity, ApiError> {
        let (client_id, client_secret) = google_credentials(config)?;
        let http_client = reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|_| google_unavailable())?;
        let metadata = CoreProviderMetadata::discover_async(
            IssuerUrl::new(GOOGLE_ISSUER.to_owned()).map_err(|_| google_unavailable())?,
            &http_client,
        )
        .await
        .map_err(|_| google_unavailable())?;
        let client = CoreClient::from_provider_metadata(
            metadata,
            ClientId::new(client_id.to_owned()),
            client_secret.map(|secret| ClientSecret::new(secret.to_owned())),
        )
        .set_redirect_uri(
            RedirectUrl::new(request.redirect_uri.clone()).map_err(|_| invalid_login())?,
        );
        let token_response = client
            .exchange_code(AuthorizationCode::new(request.code.clone()))
            .map_err(|_| invalid_login())?
            .set_pkce_verifier(PkceCodeVerifier::new(request.code_verifier.clone()))
            .request_async(&http_client)
            .await
            .map_err(|_| invalid_login())?;
        let id_token = token_response.id_token().ok_or_else(invalid_login)?;
        let verifier = client.id_token_verifier();
        let claims = id_token
            .claims(&verifier, &Nonce::new(request.nonce.clone()))
            .map_err(|_| invalid_login())?;

        if let Some(expected_hash) = claims.access_token_hash() {
            let signing_algorithm = id_token.signing_alg().map_err(|_| invalid_login())?;
            let signing_key = id_token
                .signing_key(&verifier)
                .map_err(|_| invalid_login())?;
            let actual_hash = AccessTokenHash::from_token(
                token_response.access_token(),
                signing_algorithm,
                signing_key,
            )
            .map_err(|_| invalid_login())?;
            if actual_hash != *expected_hash {
                return Err(invalid_login());
            }
        }

        Ok(GoogleIdentity {
            subject: claims.subject().as_str().to_owned(),
        })
    }
}

pub fn start_google_login(
    config: &AppConfig,
    request: &GoogleAuthStartRequest,
) -> Result<GoogleAuthStartResponse, ApiError> {
    validate_start_request(request)?;
    let (client_id, _) = google_credentials(config)?;
    let mut authorization_url =
        url::Url::parse(GOOGLE_AUTHORIZATION_ENDPOINT).map_err(|_| google_unavailable())?;
    authorization_url.query_pairs_mut().extend_pairs([
        ("client_id", client_id),
        ("redirect_uri", request.redirect_uri.as_str()),
        ("response_type", "code"),
        ("scope", "openid email"),
        ("state", request.state.as_str()),
        ("nonce", request.nonce.as_str()),
        ("code_challenge", request.code_challenge.as_str()),
        ("code_challenge_method", "S256"),
        ("prompt", "select_account"),
    ]);
    Ok(GoogleAuthStartResponse {
        authorization_url: authorization_url.into(),
    })
}

pub async fn complete_google_login(
    state: &AppState,
    request: GoogleAuthCompleteRequest,
) -> Result<DeviceTokenResponse, ApiError> {
    validate_complete_request(&request)?;
    let identity = state
        .identity_provider
        .exchange_google_code(&state.config, &request)
        .await?;
    if identity.subject.is_empty() || identity.subject.len() > 255 {
        return Err(invalid_login());
    }
    let subject_hmac = subject_hmac(
        state.config.device_token_hmac_key.expose(),
        &identity.subject,
    )?;
    let public_id_hash = hex_sha256(request.public_id.as_bytes());
    let device_id = state
        .repository
        .upsert_google_device(
            &subject_hmac,
            &public_id_hash,
            &request.app_version,
            &request.platform,
        )
        .await
        .map_err(|_| ApiError::provider())?
        .ok_or_else(ApiError::unauthorized)?;
    issue_token(state, device_id).await
}

fn google_credentials(config: &AppConfig) -> Result<(&str, Option<&str>), ApiError> {
    config
        .google_oauth_client_id
        .as_deref()
        .map(|client_id| {
            (
                client_id,
                config
                    .google_oauth_client_secret
                    .as_ref()
                    .map(|secret| secret.expose()),
            )
        })
        .ok_or_else(|| ApiError::disabled("đăng nhập Google"))
}

fn validate_start_request(request: &GoogleAuthStartRequest) -> Result<(), ApiError> {
    validate_redirect_uri(&request.redirect_uri)?;
    if !valid_urlsafe_secret(&request.state, 43, 128)
        || !valid_urlsafe_secret(&request.nonce, 43, 128)
        || !valid_urlsafe_secret(&request.code_challenge, 43, 128)
    {
        return Err(ApiError::invalid("Yêu cầu đăng nhập Google chưa hợp lệ."));
    }
    Ok(())
}

fn validate_complete_request(request: &GoogleAuthCompleteRequest) -> Result<(), ApiError> {
    validate_redirect_uri(&request.redirect_uri)?;
    if request.code.is_empty()
        || request.code.len() > 2_048
        || request.code.contains(char::is_whitespace)
        || !valid_urlsafe_secret(&request.code_verifier, 43, 128)
        || !valid_urlsafe_secret(&request.nonce, 43, 128)
        || request.public_id.is_empty()
        || request.public_id.len() > 128
        || request.app_version.is_empty()
        || request.app_version.len() > 32
        || request.platform.is_empty()
        || request.platform.len() > 32
    {
        return Err(ApiError::invalid("Yêu cầu đăng nhập Google chưa hợp lệ."));
    }
    Ok(())
}

fn validate_redirect_uri(value: &str) -> Result<(), ApiError> {
    let redirect = url::Url::parse(value)
        .map_err(|_| ApiError::invalid("Địa chỉ hoàn tất đăng nhập chưa hợp lệ."))?;
    if redirect.scheme() != "http"
        || redirect.host_str() != Some("127.0.0.1")
        || redirect.port().is_none()
        || redirect.path() != "/"
        || redirect.username() != ""
        || redirect.password().is_some()
        || redirect.query().is_some()
        || redirect.fragment().is_some()
    {
        return Err(ApiError::invalid("Địa chỉ hoàn tất đăng nhập chưa hợp lệ."));
    }
    Ok(())
}

fn valid_urlsafe_secret(value: &str, minimum: usize, maximum: usize) -> bool {
    (minimum..=maximum).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~'))
}

fn subject_hmac(key: &str, subject: &str) -> Result<String, ApiError> {
    let mut mac = HmacSha256::new_from_slice(key.as_bytes()).map_err(|_| ApiError::provider())?;
    mac.update(b"google-subject\0");
    mac.update(subject.as_bytes());
    Ok(URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes()))
}

fn hex_sha256(value: &[u8]) -> String {
    let digest = Sha256::digest(value);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn invalid_login() -> ApiError {
    ApiError {
        status: axum::http::StatusCode::UNAUTHORIZED,
        app: contracts::AppError::new(
            contracts::ErrorCode::AuthExpired,
            "Tro chưa thể xác minh tài khoản Google. Hãy thử đăng nhập lại.",
            false,
        ),
    }
}

fn google_unavailable() -> ApiError {
    ApiError {
        status: axum::http::StatusCode::BAD_GATEWAY,
        app: contracts::AppError::new(
            contracts::ErrorCode::ProviderUnavailable,
            "Google đang tạm thời không phản hồi. Hãy thử lại sau.",
            true,
        ),
    }
}

#[cfg(test)]
mod tests {
    use contracts::GoogleAuthStartRequest;

    use super::{start_google_login, validate_redirect_uri};

    #[test]
    fn authorization_url_contains_state_nonce_and_pkce() {
        let secret = "a".repeat(43);
        let response = start_google_login(
            &crate::config::AppConfig::test(),
            &GoogleAuthStartRequest {
                redirect_uri: "http://127.0.0.1:49152".to_owned(),
                state: secret.clone(),
                nonce: secret.clone(),
                code_challenge: secret,
            },
        )
        .expect("valid Google start request");
        let url = url::Url::parse(&response.authorization_url).expect("authorization URL");
        let query = url
            .query_pairs()
            .collect::<std::collections::HashMap<_, _>>();
        assert_eq!(url.host_str(), Some("accounts.google.com"));
        assert_eq!(
            query.get("code_challenge_method").map(|v| v.as_ref()),
            Some("S256")
        );
        assert_eq!(query.get("scope").map(|v| v.as_ref()), Some("openid email"));
        assert_eq!(query.get("state").map(|v| v.len()), Some(43));
        assert_eq!(query.get("nonce").map(|v| v.len()), Some(43));
    }

    #[test]
    fn callback_must_be_an_exact_ipv4_loopback_url() {
        assert!(validate_redirect_uri("http://127.0.0.1:49152").is_ok());
        for value in [
            "http://localhost:49152",
            "http://127.0.0.1",
            "http://127.0.0.1:49152/other",
            "https://127.0.0.1:49152",
            "http://127.0.0.1:49152?next=bad",
        ] {
            assert!(validate_redirect_uri(value).is_err(), "accepted {value}");
        }
    }
}
