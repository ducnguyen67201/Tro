use async_trait::async_trait;
use contracts::{AppError, ErrorCode};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RealtimeEvent {
    Transcript(String),
    AudioChunk(Vec<u8>),
    ResponseComplete,
    Disconnected,
}

#[async_trait]
pub trait RealtimeTransport: Send + Sync {
    async fn connect(&self, ephemeral_secret: &str) -> Result<(), AppError>;
    async fn close(&self);
}
pub struct NativeRealtimeTransport;
#[async_trait]
impl RealtimeTransport for NativeRealtimeTransport {
    async fn connect(&self, ephemeral_secret: &str) -> Result<(), AppError> {
        if ephemeral_secret.len() < 20 {
            Err(AppError::new(
                ErrorCode::AuthExpired,
                "Phiên giọng nói đã hết hạn.",
                true,
            ))
        } else {
            Ok(())
        }
    }
    async fn close(&self) {}
}
