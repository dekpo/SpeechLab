//! Placeholder STT provider used only to prove the UI -> Rust -> provider round trip in M1.
//! It performs NO recognition. Its output must never be used as benchmark data.

use std::time::Instant;

use super::error::SpeechError;
use super::provider::{CancelToken, SpeechToTextProvider};
use super::types::{Capabilities, ProviderInfo, ProviderKind, TranscribeRequest, TranscribeResult};

pub const MOCK_ID: &str = "mock-stt";

pub struct MockSttProvider;

impl SpeechToTextProvider for MockSttProvider {
    fn info(&self) -> ProviderInfo {
        ProviderInfo {
            id: MOCK_ID.into(),
            display_name: "Mock STT (no recognition)".into(),
            kind: ProviderKind::Stt,
            is_mock: true,
            languages: vec!["fr".into(), "en".into()],
            capabilities: Capabilities {
                supports_cancellation: true,
                supports_language_auto_detect: false,
                supports_streaming: false,
                supports_speed: false,
            },
        }
    }

    fn transcribe(
        &self,
        request: &TranscribeRequest,
        cancel: &CancelToken,
    ) -> Result<TranscribeResult, SpeechError> {
        let start = Instant::now();
        if request.audio_path.trim().is_empty() {
            return Err(SpeechError::InvalidRequest("audio path is empty".into()));
        }
        if cancel.is_cancelled() {
            return Err(SpeechError::Cancelled);
        }
        Ok(TranscribeResult {
            provider_id: MOCK_ID.into(),
            model_id: request.model_id.clone(),
            language: request.language.clone(),
            text: "[MOCK] no real transcription was performed".into(),
            processing_ms: start.elapsed().as_millis() as u64,
            load_ms: 0,
            cold_start: false,
            audio_ms: None,
            rtf: None,
            threads: 1,
            is_mock: true,
        })
    }
}
