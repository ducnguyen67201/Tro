use contracts::{RealtimeSecretRequest, RealtimeSecretResponse};
use uuid::Uuid;

use crate::{error::ApiError, state::AppState};

pub async fn mint(
    state: &AppState,
    device_id: Uuid,
    request: RealtimeSecretRequest,
) -> Result<RealtimeSecretResponse, ApiError> {
    if !state.config.realtime_enabled {
        return Err(ApiError::disabled("trò chuyện giọng nói"));
    }
    if request.locale != "vi-VN" && request.locale != "en" {
        return Err(ApiError::invalid("Ngôn ngữ chưa được hỗ trợ."));
    }
    let usage = state
        .repository
        .usage_today(device_id)
        .await
        .map_err(|_| ApiError::provider())?;
    if usage.realtime_seconds >= state.config.device_daily_realtime_seconds {
        return Err(ApiError {
            status: axum::http::StatusCode::TOO_MANY_REQUESTS,
            app: contracts::AppError::new(
                contracts::ErrorCode::RateLimited,
                "Bạn đã dùng hết thời lượng giọng nói hôm nay.",
                false,
            ),
        });
    }
    state
        .provider
        .create_realtime_secret(&request.locale, &request.safety_identifier_hash)
        .await
}
