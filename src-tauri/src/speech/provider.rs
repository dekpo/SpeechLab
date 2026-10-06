use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use super::error::SpeechError;
use super::types::{
    ProviderInfo, SynthesizeRequest, SynthesizeResult, TranscribeRequest, TranscribeResult,
    VoiceInfo,
};

/// Cooperative cancellation flag shared between the command layer and a running provider.
#[derive(Debug, Clone, Default)]
pub struct CancelToken(Arc<AtomicBool>);

impl CancelToken {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn cancel(&self) {
        self.0.store(true, Ordering::SeqCst);
    }
    pub fn reset(&self) {
        self.0.store(false, Ordering::SeqCst);
    }
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

pub trait SpeechToTextProvider: Send + Sync {
    fn info(&self) -> ProviderInfo;
    fn transcribe(
        &self,
        request: &TranscribeRequest,
        cancel: &CancelToken,
    ) -> Result<TranscribeResult, SpeechError>;
}

// Declared now so the contract is stable; first implementation arrives in M6.
#[allow(dead_code)]
pub trait TextToSpeechProvider: Send + Sync {
    fn info(&self) -> ProviderInfo;
    fn voices(&self) -> Result<Vec<VoiceInfo>, SpeechError>;
    fn synthesize(
        &self,
        request: &SynthesizeRequest,
        cancel: &CancelToken,
    ) -> Result<SynthesizeResult, SpeechError>;
}
