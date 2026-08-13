use std::sync::{Arc, Mutex};

use contracts::{AppError, ErrorCode};
use cpal::{
    FromSample, InterfaceType, Sample, SampleFormat, SizedSample, Stream, StreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use zeroize::Zeroize;

const MAX_RECORDING_SECONDS: u32 = 30;
const MIN_RECORDING_MILLISECONDS: u32 = 120;

pub struct RecordedAudio {
    pub wav_bytes: Vec<u8>,
}

impl Drop for RecordedAudio {
    fn drop(&mut self) {
        self.wav_bytes.zeroize();
    }
}

pub trait AudioBackend: Send + Sync {
    fn microphone_available(&self) -> bool;
    fn start_push_to_talk(&self) -> Result<(), AppError>;
    fn finish_push_to_talk(&self) -> Result<RecordedAudio, AppError>;
    fn stop(&self);
}

#[derive(Default)]
pub struct CpalAudioBackend {
    active: Mutex<Option<ActiveRecording>>,
}

struct ActiveRecording {
    stream: Stream,
    samples: Arc<Mutex<Vec<i16>>>,
    sample_rate: u32,
    channels: u16,
}

impl AudioBackend for CpalAudioBackend {
    fn microphone_available(&self) -> bool {
        preferred_input_device().is_some()
    }

    fn start_push_to_talk(&self) -> Result<(), AppError> {
        self.stop();
        let device = preferred_input_device().ok_or_else(microphone_unavailable)?;
        let device_name = device
            .description()
            .map(|description| description.name().to_owned())
            .unwrap_or_else(|_| "unknown".to_owned());
        tracing::info!(
            component = "audio",
            operation = "push_to_talk_start",
            device = %device_name
        );
        let supported = device.default_input_config().map_err(microphone_error)?;
        let sample_rate = supported.sample_rate();
        let channels = supported.channels();
        let sample_format = supported.sample_format();
        let config: StreamConfig = supported.into();
        let max_samples =
            sample_rate as usize * usize::from(channels) * MAX_RECORDING_SECONDS as usize;
        let samples = Arc::new(Mutex::new(Vec::with_capacity(
            max_samples.min(sample_rate as usize * usize::from(channels) * 5),
        )));
        let stream = match sample_format {
            SampleFormat::I16 => build_input_stream::<i16>(&device, &config, &samples, max_samples),
            SampleFormat::U16 => build_input_stream::<u16>(&device, &config, &samples, max_samples),
            SampleFormat::F32 => build_input_stream::<f32>(&device, &config, &samples, max_samples),
            format => {
                tracing::warn!(component = "audio", operation = "start", ?format);
                return Err(AppError::new(
                    ErrorCode::MicrophoneUnavailable,
                    "Định dạng micrô này chưa được Tro hỗ trợ.",
                    false,
                ));
            }
        }?;
        stream.play().map_err(microphone_error)?;
        let mut active = self
            .active
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *active = Some(ActiveRecording {
            stream,
            samples,
            sample_rate,
            channels,
        });
        Ok(())
    }

    fn finish_push_to_talk(&self) -> Result<RecordedAudio, AppError> {
        let recording = self
            .active
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
            .ok_or_else(|| {
                AppError::new(
                    ErrorCode::InvalidTransition,
                    "Tro chưa bắt đầu nghe. Hãy giữ Command + Option rồi thử lại.",
                    true,
                )
            })?;
        drop(recording.stream);
        let samples = recording
            .samples
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let minimum_samples = recording.sample_rate as usize
            * usize::from(recording.channels)
            * MIN_RECORDING_MILLISECONDS as usize
            / 1_000;
        if samples.len() < minimum_samples {
            return Err(AppError::new(
                ErrorCode::InvalidRequest,
                "Tro chưa nghe đủ câu hỏi. Hãy giữ phím lâu hơn một chút.",
                true,
            ));
        }
        Ok(RecordedAudio {
            wav_bytes: encode_pcm16_wav(&samples, recording.sample_rate, recording.channels)?,
        })
    }

    fn stop(&self) {
        self.active
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
    }
}

fn preferred_input_device() -> Option<cpal::Device> {
    let host = cpal::default_host();
    // Prefer the Mac's integrated microphone. Opening an AirPods/Bluetooth
    // microphone forces its output into the low-bandwidth headset profile,
    // which makes music sound distorted for the duration of recording.
    let built_in = host.input_devices().ok().and_then(|mut devices| {
        devices.find(|device| {
            device
                .description()
                .is_ok_and(|description| description.interface_type() == InterfaceType::BuiltIn)
        })
    });
    built_in.or_else(|| host.default_input_device())
}

fn build_input_stream<T>(
    device: &cpal::Device,
    config: &StreamConfig,
    samples: &Arc<Mutex<Vec<i16>>>,
    max_samples: usize,
) -> Result<Stream, AppError>
where
    T: Sample + SizedSample,
    i16: FromSample<T>,
{
    let captured = Arc::clone(samples);
    device
        .build_input_stream(
            *config,
            move |input: &[T], _| {
                if let Ok(mut output) = captured.try_lock() {
                    let remaining = max_samples.saturating_sub(output.len());
                    output.extend(input.iter().take(remaining).copied().map(i16::from_sample));
                }
            },
            |error| {
                tracing::warn!(
                    component = "audio",
                    operation = "capture_stream",
                    error_code = "microphone_stream_failed",
                    source = %error
                );
            },
            Some(std::time::Duration::from_secs(3)),
        )
        .map_err(microphone_error)
}

fn encode_pcm16_wav(samples: &[i16], sample_rate: u32, channels: u16) -> Result<Vec<u8>, AppError> {
    let data_len = samples
        .len()
        .checked_mul(2)
        .and_then(|length| u32::try_from(length).ok())
        .ok_or_else(|| AppError::new(ErrorCode::Internal, "Âm thanh quá dài.", false))?;
    let riff_len = 36_u32
        .checked_add(data_len)
        .ok_or_else(|| AppError::new(ErrorCode::Internal, "Âm thanh quá dài.", false))?;
    let block_align = channels.saturating_mul(2);
    let byte_rate = sample_rate.saturating_mul(u32::from(block_align));
    let mut wav = Vec::with_capacity(data_len as usize + 44);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&riff_len.to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16_u32.to_le_bytes());
    wav.extend_from_slice(&1_u16.to_le_bytes());
    wav.extend_from_slice(&channels.to_le_bytes());
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&block_align.to_le_bytes());
    wav.extend_from_slice(&16_u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_len.to_le_bytes());
    for sample in samples {
        wav.extend_from_slice(&sample.to_le_bytes());
    }
    Ok(wav)
}

fn microphone_unavailable() -> AppError {
    AppError::new(
        ErrorCode::MicrophoneUnavailable,
        "Không tìm thấy micrô. Hãy kiểm tra thiết bị âm thanh.",
        true,
    )
}

fn microphone_error(error: impl std::fmt::Display) -> AppError {
    tracing::warn!(
        component = "audio",
        operation = "microphone",
        error_code = "microphone_unavailable",
        source = %error
    );
    AppError::new(
        ErrorCode::MicrophoneUnavailable,
        "Tro chưa thể thu âm. Hãy kiểm tra quyền micrô rồi thử lại.",
        true,
    )
}

#[cfg(test)]
mod tests {
    use super::encode_pcm16_wav;

    #[test]
    fn encodes_pcm_samples_as_a_valid_wav_container() {
        let wav =
            encode_pcm16_wav(&[0, i16::MAX, i16::MIN], 48_000, 1).expect("fixture should encode");
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        assert_eq!(&wav[36..40], b"data");
        assert_eq!(u32::from_le_bytes(wav[40..44].try_into().unwrap()), 6);
        assert_eq!(wav.len(), 50);
    }
}
