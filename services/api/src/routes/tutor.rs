use axum::{Json, extract::Multipart, extract::State, http::HeaderMap};
use contracts::{ApiEnvelope, ImageMime, TutorTurnMetadata, TutorTurnResponse};
use uuid::Uuid;
use zeroize::Zeroizing;

use crate::{
    error::ApiError,
    services::{device_tokens, tutor::TutorMedia},
    state::AppState,
};

const METADATA_MAX_BYTES: usize = 16 * 1024;

pub async fn turn(
    State(state): State<AppState>,
    headers: HeaderMap,
    multipart: Multipart,
) -> Result<Json<ApiEnvelope<TutorTurnResponse>>, ApiError> {
    if !state.config.tutor_enabled {
        return Err(ApiError::disabled("trợ lý học tập"));
    }
    let bearer = device_tokens::bearer_value(
        headers
            .get("authorization")
            .and_then(|value| value.to_str().ok()),
    )?;
    let device_id = device_tokens::authenticate(&state, bearer).await?;
    let (metadata, mut audio, mut screenshot) = parse_multipart(multipart).await?;
    validate_media(&state, &metadata, &audio, &screenshot)?;
    let reserved = state
        .repository
        .reserve_tutor_usage(device_id, state.config.device_daily_screenshots)
        .await
        .map_err(|_| ApiError::provider())?;
    if !reserved {
        return Err(ApiError {
            status: axum::http::StatusCode::TOO_MANY_REQUESTS,
            app: contracts::AppError::new(
                contracts::ErrorCode::RateLimited,
                "Bạn đã đạt giới hạn câu hỏi có màn hình hôm nay.",
                false,
            ),
        });
    }
    let guidance = state
        .tutor_provider
        .complete(TutorMedia {
            audio_wav: std::mem::take(&mut *audio),
            screenshot_jpeg: std::mem::take(&mut *screenshot),
        })
        .await?;
    Ok(Json(ApiEnvelope {
        data: TutorTurnResponse { guidance },
        request_id: format!("req_{}", Uuid::new_v4().simple()),
    }))
}

async fn parse_multipart(
    mut multipart: Multipart,
) -> Result<(TutorTurnMetadata, Zeroizing<Vec<u8>>, Zeroizing<Vec<u8>>), ApiError> {
    let mut metadata = None;
    let mut audio = None;
    let mut screenshot = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| ApiError::invalid("Invalid multipart body."))?
    {
        match field.name() {
            Some("metadata") if metadata.is_none() => {
                if field.content_type() != Some("application/json") {
                    return Err(ApiError::invalid("Metadata content type is invalid."));
                }
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|_| ApiError::invalid("Invalid metadata."))?;
                if bytes.len() > METADATA_MAX_BYTES {
                    return Err(ApiError::invalid("Metadata exceeds the safe limit."));
                }
                metadata = Some(
                    serde_json::from_slice(&bytes)
                        .map_err(|_| ApiError::invalid("Invalid metadata JSON."))?,
                );
            }
            Some("audio") if audio.is_none() => {
                if field.content_type() != Some("audio/wav") {
                    return Err(ApiError::invalid("Audio content type is invalid."));
                }
                audio = Some(Zeroizing::new(
                    field
                        .bytes()
                        .await
                        .map_err(|_| ApiError::invalid("Invalid audio."))?
                        .to_vec(),
                ));
            }
            Some("screenshot") if screenshot.is_none() => {
                if field.content_type() != Some("image/jpeg") {
                    return Err(ApiError::invalid("Screenshot content type is invalid."));
                }
                screenshot = Some(Zeroizing::new(
                    field
                        .bytes()
                        .await
                        .map_err(|_| ApiError::invalid("Invalid screenshot."))?
                        .to_vec(),
                ));
            }
            _ => return Err(ApiError::invalid("Unknown or duplicate multipart field.")),
        }
    }
    Ok((
        metadata.ok_or_else(|| ApiError::invalid("Missing metadata."))?,
        audio.ok_or_else(|| ApiError::invalid("Missing audio."))?,
        screenshot.ok_or_else(|| ApiError::invalid("Missing screenshot."))?,
    ))
}

fn validate_media(
    state: &AppState,
    metadata: &TutorTurnMetadata,
    audio: &[u8],
    screenshot: &[u8],
) -> Result<(), ApiError> {
    if metadata.locale != "vi-VN" && metadata.locale != "en" {
        return Err(ApiError::invalid("Ngôn ngữ chưa được hỗ trợ."));
    }
    let frame = &metadata.frame;
    if frame.mime_type != ImageMime::Jpeg
        || frame.frame_id.is_empty()
        || frame.frame_id.len() > 128
        || frame.monitor_id.is_empty()
        || frame.monitor_id.len() > 128
        || frame.width_px == 0
        || frame.height_px == 0
        || frame.width_px > state.config.screenshot_max_edge_px
        || frame.height_px > state.config.screenshot_max_edge_px
        || !frame.scale_factor.is_finite()
        || frame.scale_factor <= 0.0
        || frame.scale_factor > 8.0
    {
        return Err(ApiError::invalid("Thông tin màn hình không hợp lệ."));
    }
    if audio.len() < 44
        || audio.len() > state.config.tutor_audio_max_bytes
        || audio.get(0..4) != Some(b"RIFF")
        || audio.get(8..12) != Some(b"WAVE")
    {
        return Err(ApiError::invalid(
            "Âm thanh vượt giới hạn hoặc không hợp lệ.",
        ));
    }
    if screenshot.len() < 4
        || screenshot.len() > state.config.screenshot_max_bytes
        || screenshot.get(0..2) != Some(&[0xff, 0xd8])
        || screenshot.get(screenshot.len().saturating_sub(2)..) != Some(&[0xff, 0xd9])
    {
        return Err(ApiError::invalid(
            "Ảnh màn hình vượt giới hạn hoặc không hợp lệ.",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use contracts::{ImageMime, ScreenFrameMeta, TutorTurnMetadata};

    use crate::{AppConfig, AppState, FakeProvider, FakeTutorProvider, MemoryRepository};

    use super::validate_media;

    fn state() -> AppState {
        AppState::new(
            Arc::new(AppConfig::test()),
            Arc::new(MemoryRepository::default()),
            Arc::new(FakeProvider::default()),
            Arc::new(FakeTutorProvider::default()),
        )
    }

    fn metadata() -> TutorTurnMetadata {
        TutorTurnMetadata {
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
        }
    }

    #[test]
    fn rejects_mislabeled_or_malformed_media() {
        let mut wav = vec![0_u8; 44];
        wav[0..4].copy_from_slice(b"RIFF");
        wav[8..12].copy_from_slice(b"WAVE");
        assert!(validate_media(&state(), &metadata(), &wav, &[0xff, 0xd8, 0xff, 0xd9]).is_ok());
        assert!(
            validate_media(
                &state(),
                &metadata(),
                b"not-wave",
                &[0xff, 0xd8, 0xff, 0xd9]
            )
            .is_err()
        );
        assert!(validate_media(&state(), &metadata(), &wav, b"not-jpeg").is_err());
    }
}
