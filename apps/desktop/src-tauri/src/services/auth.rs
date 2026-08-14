use std::time::Duration;

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use contracts::{
    ApiEnvelope, AppError, DeviceTokenResponse, ErrorCode, GoogleAuthCompleteRequest,
    GoogleAuthStartRequest, GoogleAuthStartResponse, RegisterDeviceRequest,
};
use rand::RngCore;
use reqwest::Response;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use zeroize::Zeroizing;

use super::{llm::LlmConfig, secrets};

const MAX_AUTH_RESPONSE_BYTES: usize = 65_536;
const MAX_CALLBACK_REQUEST_BYTES: usize = 8_192;
const GOOGLE_LOGIN_TIMEOUT: Duration = Duration::from_secs(180);

pub struct AuthGateway {
    client: reqwest::Client,
}

impl Default for AuthGateway {
    fn default() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

impl AuthGateway {
    pub async fn restore_session(&self, config: &LlmConfig) -> Result<bool, AppError> {
        let Some(token) = secrets::load_device_token()? else {
            return Ok(false);
        };
        let response = self
            .client
            .post(format!(
                "{}/v1/devices/refresh",
                config.backend_url.trim_end_matches('/')
            ))
            .bearer_auth(token.as_str())
            .timeout(Duration::from_secs(10))
            .send()
            .await
            .map_err(auth_unavailable)?;
        let status = response.status();
        let bytes = read_bounded(response).await?;
        if !status.is_success() {
            if status == reqwest::StatusCode::UNAUTHORIZED {
                secrets::delete_device_token()?;
                return Ok(false);
            }
            return Err(remote_error(&bytes).unwrap_or_else(|| auth_unavailable(status)));
        }
        save_session(&bytes)?;
        Ok(true)
    }

    pub async fn sign_in_with_google(&self, config: &LlmConfig) -> Result<(), AppError> {
        let listener = TcpListener::bind(("127.0.0.1", 0))
            .await
            .map_err(auth_unavailable)?;
        let port = listener.local_addr().map_err(auth_unavailable)?.port();
        let redirect_uri = format!("http://127.0.0.1:{port}");
        let flow = GoogleLoginFlow::new();

        let response = self
            .client
            .post(format!(
                "{}/v1/auth/google/start",
                config.backend_url.trim_end_matches('/')
            ))
            .timeout(Duration::from_secs(10))
            .json(&GoogleAuthStartRequest {
                redirect_uri: redirect_uri.clone(),
                state: flow.state.clone(),
                nonce: flow.nonce.clone(),
                code_challenge: flow.code_challenge.clone(),
            })
            .send()
            .await
            .map_err(auth_unavailable)?;
        let status = response.status();
        let bytes = read_bounded(response).await?;
        if !status.is_success() {
            return Err(remote_error(&bytes).unwrap_or_else(|| auth_unavailable(status)));
        }
        let start: ApiEnvelope<GoogleAuthStartResponse> =
            serde_json::from_slice(&bytes).map_err(|_| auth_protocol_error())?;
        let authorization_url =
            validate_authorization_url(&start.data.authorization_url, &redirect_uri, &flow)?;
        webbrowser::open(authorization_url.as_str()).map_err(browser_unavailable)?;

        let code = tokio::time::timeout(
            GOOGLE_LOGIN_TIMEOUT,
            wait_for_google_callback(listener, &flow.state),
        )
        .await
        .map_err(|_| login_timeout())??;
        let public_id = secrets::load_or_create_device_public_id()?;
        let response = self
            .client
            .post(format!(
                "{}/v1/auth/google/complete",
                config.backend_url.trim_end_matches('/')
            ))
            .timeout(Duration::from_secs(15))
            .json(&GoogleAuthCompleteRequest {
                code,
                code_verifier: flow.code_verifier.to_string(),
                redirect_uri,
                nonce: flow.nonce,
                public_id,
                app_version: env!("CARGO_PKG_VERSION").to_owned(),
                platform: std::env::consts::OS.to_owned(),
            })
            .send()
            .await
            .map_err(auth_unavailable)?;
        let status = response.status();
        let bytes = read_bounded(response).await?;
        if !status.is_success() {
            return Err(remote_error(&bytes).unwrap_or_else(|| auth_unavailable(status)));
        }
        save_session(&bytes)
    }

    pub async fn sign_in(
        &self,
        config: &LlmConfig,
        invite_code: &str,
        accepted_age_scope: bool,
    ) -> Result<(), AppError> {
        let invite_code = validate_invite(invite_code)?;
        if !accepted_age_scope {
            return Err(AppError::new(
                ErrorCode::InvalidRequest,
                "Bạn cần xác nhận đủ 18 tuổi để tiếp tục.",
                false,
            ));
        }
        let response = self
            .client
            .post(format!(
                "{}/v1/devices/register",
                config.backend_url.trim_end_matches('/')
            ))
            .timeout(Duration::from_secs(10))
            .json(&RegisterDeviceRequest {
                invite_code: invite_code.to_owned(),
                public_id: secrets::load_or_create_device_public_id()?,
                app_version: env!("CARGO_PKG_VERSION").to_owned(),
                platform: std::env::consts::OS.to_owned(),
                accepted_age_scope,
            })
            .send()
            .await
            .map_err(auth_unavailable)?;
        let status = response.status();
        let bytes = read_bounded(response).await?;
        if !status.is_success() {
            return Err(remote_error(&bytes).unwrap_or_else(|| auth_unavailable(status)));
        }
        save_session(&bytes)
    }
}

struct GoogleLoginFlow {
    state: String,
    nonce: String,
    code_verifier: Zeroizing<String>,
    code_challenge: String,
}

impl GoogleLoginFlow {
    fn new() -> Self {
        let code_verifier = Zeroizing::new(random_urlsafe());
        let code_challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(code_verifier.as_bytes()));
        Self {
            state: random_urlsafe(),
            nonce: random_urlsafe(),
            code_verifier,
            code_challenge,
        }
    }
}

fn random_urlsafe() -> String {
    let mut bytes = [0_u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

fn validate_authorization_url(
    value: &str,
    redirect_uri: &str,
    flow: &GoogleLoginFlow,
) -> Result<url::Url, AppError> {
    let url = url::Url::parse(value).map_err(|_| auth_protocol_error())?;
    if url.scheme() != "https"
        || url.host_str() != Some("accounts.google.com")
        || url.port_or_known_default() != Some(443)
        || url.path() != "/o/oauth2/v2/auth"
        || url.username() != ""
        || url.password().is_some()
        || url.fragment().is_some()
        || unique_query_value(&url, "redirect_uri").as_deref() != Some(redirect_uri)
        || unique_query_value(&url, "state").as_deref() != Some(flow.state.as_str())
        || unique_query_value(&url, "nonce").as_deref() != Some(flow.nonce.as_str())
        || unique_query_value(&url, "code_challenge").as_deref()
            != Some(flow.code_challenge.as_str())
        || unique_query_value(&url, "code_challenge_method").as_deref() != Some("S256")
    {
        return Err(auth_protocol_error());
    }
    Ok(url)
}

async fn wait_for_google_callback(
    listener: TcpListener,
    expected_state: &str,
) -> Result<String, AppError> {
    loop {
        let (mut stream, _) = listener.accept().await.map_err(auth_unavailable)?;
        let request =
            match tokio::time::timeout(Duration::from_secs(2), read_callback_request(&mut stream))
                .await
            {
                Ok(Some(request)) => request,
                _ => {
                    respond_to_browser(&mut stream, false).await;
                    continue;
                }
            };
        match parse_google_callback(&request, expected_state) {
            CallbackResult::Code(code) => {
                respond_to_browser(&mut stream, true).await;
                return Ok(code);
            }
            CallbackResult::Cancelled => {
                respond_to_browser(&mut stream, false).await;
                return Err(login_cancelled());
            }
            CallbackResult::Ignore => {
                respond_to_browser(&mut stream, false).await;
            }
        }
    }
}

async fn read_callback_request(stream: &mut TcpStream) -> Option<Vec<u8>> {
    let mut request = Vec::new();
    let mut chunk = [0_u8; 1_024];
    loop {
        let read = stream.read(&mut chunk).await.ok()?;
        if read == 0 {
            return None;
        }
        if request.len().saturating_add(read) > MAX_CALLBACK_REQUEST_BYTES {
            return None;
        }
        request.extend_from_slice(&chunk[..read]);
        if request.windows(4).any(|window| window == b"\r\n\r\n") {
            return Some(request);
        }
    }
}

enum CallbackResult {
    Code(String),
    Cancelled,
    Ignore,
}

fn parse_google_callback(request: &[u8], expected_state: &str) -> CallbackResult {
    let Ok(request) = std::str::from_utf8(request) else {
        return CallbackResult::Ignore;
    };
    let Some(first_line) = request.lines().next() else {
        return CallbackResult::Ignore;
    };
    let parts = first_line.split_whitespace().collect::<Vec<_>>();
    if parts.len() != 3 || parts[0] != "GET" || !parts[2].starts_with("HTTP/1.") {
        return CallbackResult::Ignore;
    }
    let Ok(url) = url::Url::parse(&format!("http://127.0.0.1{}", parts[1])) else {
        return CallbackResult::Ignore;
    };
    if url.path() != "/"
        || unique_query_value(&url, "state").as_deref() != Some(expected_state)
        || unique_query_value(&url, "iss").as_deref() != Some("https://accounts.google.com")
    {
        return CallbackResult::Ignore;
    }
    if unique_query_value(&url, "error").is_some() {
        return CallbackResult::Cancelled;
    }
    let Some(code) = unique_query_value(&url, "code") else {
        return CallbackResult::Ignore;
    };
    if code.is_empty() || code.len() > 2_048 || code.contains(char::is_whitespace) {
        return CallbackResult::Ignore;
    }
    CallbackResult::Code(code)
}

fn unique_query_value(url: &url::Url, key: &str) -> Option<String> {
    let mut values = url
        .query_pairs()
        .filter(|(name, _)| name == key)
        .map(|(_, value)| value.into_owned());
    let value = values.next()?;
    if values.next().is_some() {
        return None;
    }
    Some(value)
}

async fn respond_to_browser(stream: &mut TcpStream, success: bool) {
    let (status, title, message) = if success {
        (
            "200 OK",
            "Đã kết nối với Tro",
            "Bạn có thể đóng cửa sổ này và quay lại Tro.",
        )
    } else {
        (
            "400 Bad Request",
            "Chưa thể kết nối",
            "Hãy quay lại Tro và thử đăng nhập lại.",
        )
    };
    let body = format!(
        "<!doctype html><html lang=\"vi\"><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width\"><title>{title}</title><style>body{{font:16px system-ui;display:grid;place-items:center;min-height:100vh;margin:0;background:#f8f6ef;color:#27251f}}main{{max-width:420px;text-align:center;padding:32px}}h1{{font-size:28px}}p{{color:#676158;line-height:1.6}}</style><main><h1>{title}</h1><p>{message}</p></main></html>"
    );
    let headers = format!(
        "HTTP/1.1 {status}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Security-Policy: default-src 'none'; style-src 'unsafe-inline'\r\nCache-Control: no-store\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _result = stream.write_all(headers.as_bytes()).await;
    let _result = stream.write_all(body.as_bytes()).await;
    let _result = stream.shutdown().await;
}

fn save_session(bytes: &[u8]) -> Result<(), AppError> {
    let envelope: ApiEnvelope<DeviceTokenResponse> =
        serde_json::from_slice(bytes).map_err(|_| auth_protocol_error())?;
    let token = Zeroizing::new(envelope.data.device_token);
    if !(32..=256).contains(&token.len()) || token.contains(char::is_whitespace) {
        return Err(auth_protocol_error());
    }
    secrets::save_device_token(token.as_str())
}

fn remote_error(bytes: &[u8]) -> Option<AppError> {
    serde_json::from_slice::<RemoteErrorEnvelope>(bytes)
        .map(|remote| remote.error)
        .ok()
}

fn validate_invite(invite: &str) -> Result<&str, AppError> {
    let invite = invite.trim();
    if !(4..=128).contains(&invite.len())
        || !invite
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '-')
    {
        return Err(AppError::new(
            ErrorCode::InviteInvalid,
            "Mã truy cập không đúng định dạng.",
            false,
        ));
    }
    Ok(invite)
}

async fn read_bounded(mut response: Response) -> Result<Vec<u8>, AppError> {
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(auth_unavailable)? {
        if bytes.len().saturating_add(chunk.len()) > MAX_AUTH_RESPONSE_BYTES {
            return Err(auth_protocol_error());
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

#[derive(Deserialize)]
struct RemoteErrorEnvelope {
    error: AppError,
}

fn auth_unavailable(error: impl std::fmt::Display) -> AppError {
    tracing::warn!(
        component = "auth",
        operation = "device_session",
        error_code = "provider_unavailable",
        source = %error
    );
    AppError::new(
        ErrorCode::ProviderUnavailable,
        "Tro chưa kết nối được máy chủ. Hãy thử lại.",
        true,
    )
}

fn browser_unavailable(error: impl std::fmt::Display) -> AppError {
    tracing::warn!(
        component = "auth",
        operation = "open_system_browser",
        error_code = "browser_unavailable",
        source = %error
    );
    AppError::new(
        ErrorCode::ProviderUnavailable,
        "Tro chưa thể mở trình duyệt để đăng nhập Google.",
        true,
    )
}

fn login_cancelled() -> AppError {
    AppError::new(ErrorCode::Cancelled, "Đăng nhập Google đã bị hủy.", false)
}

fn login_timeout() -> AppError {
    AppError::new(
        ErrorCode::Cancelled,
        "Đã hết thời gian đăng nhập Google. Hãy thử lại.",
        false,
    )
}

fn auth_protocol_error() -> AppError {
    AppError::new(
        ErrorCode::ProviderProtocolError,
        "Máy chủ đăng nhập trả về dữ liệu chưa hợp lệ.",
        true,
    )
}

#[cfg(test)]
mod tests {
    use super::{
        CallbackResult, GoogleLoginFlow, parse_google_callback, validate_authorization_url,
        validate_invite,
    };

    #[test]
    fn invite_accepts_only_bounded_ascii_code_characters() {
        assert_eq!(validate_invite(" TRO-LOCAL ").unwrap(), "TRO-LOCAL");
        assert!(validate_invite("bad code").is_err());
        assert!(validate_invite("x").is_err());
        assert!(validate_invite("TRO-💜").is_err());
    }

    #[test]
    fn callback_requires_the_original_state_and_google_issuer() {
        let valid = b"GET /?state=expected&code=one-time-code&iss=https%3A%2F%2Faccounts.google.com HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n";
        assert!(matches!(
            parse_google_callback(valid, "expected"),
            CallbackResult::Code(code) if code == "one-time-code"
        ));
        assert!(matches!(
            parse_google_callback(valid, "different"),
            CallbackResult::Ignore
        ));
        let forged_issuer = b"GET /?state=expected&code=one-time-code&iss=https%3A%2F%2Fexample.com HTTP/1.1\r\n\r\n";
        assert!(matches!(
            parse_google_callback(forged_issuer, "expected"),
            CallbackResult::Ignore
        ));
        let missing_issuer = b"GET /?state=expected&code=one-time-code HTTP/1.1\r\n\r\n";
        assert!(matches!(
            parse_google_callback(missing_issuer, "expected"),
            CallbackResult::Ignore
        ));
    }

    #[test]
    fn browser_url_is_pinned_to_google_and_the_local_flow() {
        let flow = GoogleLoginFlow::new();
        let redirect = "http://127.0.0.1:49152";
        let mut url =
            url::Url::parse("https://accounts.google.com/o/oauth2/v2/auth").expect("static URL");
        url.query_pairs_mut().extend_pairs([
            ("redirect_uri", redirect),
            ("state", flow.state.as_str()),
            ("nonce", flow.nonce.as_str()),
            ("code_challenge", flow.code_challenge.as_str()),
            ("code_challenge_method", "S256"),
        ]);
        assert!(validate_authorization_url(url.as_str(), redirect, &flow).is_ok());
        url.set_host(Some("example.com")).expect("valid host");
        assert!(validate_authorization_url(url.as_str(), redirect, &flow).is_err());
    }
}
