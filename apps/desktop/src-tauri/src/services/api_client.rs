use contracts::{AppError, ErrorCode};

pub struct ApiClient {
    client: reqwest::Client,
    base_url: String,
}
impl ApiClient {
    pub fn new(base_url: String) -> Result<Self, AppError> {
        if !base_url.starts_with("https://") && !base_url.starts_with("http://127.0.0.1") {
            return Err(AppError::new(
                ErrorCode::InvalidRequest,
                "Địa chỉ máy chủ không được phép.",
                false,
            ));
        }
        Ok(Self {
            client: reqwest::Client::new(),
            base_url,
        })
    }
    pub async fn health(&self) -> bool {
        self.client
            .get(format!("{}/healthz", self.base_url))
            .timeout(std::time::Duration::from_secs(3))
            .send()
            .await
            .is_ok_and(|response| response.status().is_success())
    }
}
