use axum::{Json, http::StatusCode, response::IntoResponse};
use contracts::{AppError, ErrorCode};
use serde::Serialize;

#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub app: AppError,
}

#[derive(Serialize)]
struct ErrorEnvelope {
    error: ErrorBody,
}

#[derive(Serialize)]
struct ErrorBody {
    code: &'static str,
    message_vi: String,
    retryable: bool,
    request_id: Option<String>,
}

impl ApiError {
    pub fn invalid(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            app: AppError::new(ErrorCode::InvalidRequest, message, false),
        }
    }

    pub fn unauthorized() -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            app: AppError::new(
                ErrorCode::AuthExpired,
                "Phiên thiết bị đã hết hạn. Hãy đăng nhập lại bằng mã mời.",
                false,
            ),
        }
    }

    pub fn provider() -> Self {
        Self {
            status: StatusCode::BAD_GATEWAY,
            app: AppError::new(
                ErrorCode::ProviderUnavailable,
                "Dịch vụ AI đang tạm gián đoạn. Hãy thử lại sau.",
                true,
            ),
        }
    }

    pub fn disabled(feature: &'static str) -> Self {
        Self {
            status: StatusCode::SERVICE_UNAVAILABLE,
            app: AppError::new(
                ErrorCode::ProviderUnavailable,
                format!("Tính năng {feature} đang tạm dừng để đảm bảo an toàn."),
                false,
            ),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let body = ErrorEnvelope {
            error: ErrorBody {
                code: self.app.code.as_str(),
                message_vi: self.app.message_vi,
                retryable: self.app.retryable,
                request_id: self.app.request_id,
            },
        };
        (self.status, Json(body)).into_response()
    }
}
