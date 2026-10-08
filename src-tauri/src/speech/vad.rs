//! Silero voice-activity detection through the sherpa-onnx VAD API (D-035).
//!
//! Gives the chunker (`chunking.rs`) the places where someone speaks. The Silero model is a
//! small support model (`models-manifest.json`, id `silero-vad`, MIT licence), downloaded
//! like the engines and used offline.

use std::path::Path;

use sherpa_onnx::{SileroVadModelConfig, VadModelConfig, VoiceActivityDetector};

use super::chunking::{Span, CHUNK_RATE};
use super::error::SpeechError;

/// Speech probability above which a window counts as speech.
const THRESHOLD: f32 = 0.5;
/// A pause must last this long (seconds) to end a speech span.
const MIN_SILENCE_S: f32 = 0.4;
/// Bursts shorter than this (seconds) are ignored.
const MIN_SPEECH_S: f32 = 0.25;
/// Silero analyses 512-sample windows at 16 kHz.
const WINDOW: usize = 512;
/// The detector must never cut speech itself: the chunker owns that decision (quietest point).
const MAX_SPEECH_S: f32 = 3600.0;

/// Detects speech spans in 16 kHz mono samples.
pub fn silero_speech_spans(model: &Path, samples: &[f32]) -> Result<Vec<Span>, SpeechError> {
    let config = VadModelConfig {
        silero_vad: SileroVadModelConfig {
            model: Some(model.to_string_lossy().into_owned()),
            threshold: THRESHOLD,
            min_silence_duration: MIN_SILENCE_S,
            min_speech_duration: MIN_SPEECH_S,
            window_size: WINDOW as i32,
            max_speech_duration: MAX_SPEECH_S,
        },
        sample_rate: CHUNK_RATE as i32,
        num_threads: 1,
        provider: Some("cpu".into()),
        ..Default::default()
    };
    let buffer_s = samples.len() as f32 / CHUNK_RATE as f32 + 5.0;
    let vad = VoiceActivityDetector::create(&config, buffer_s).ok_or_else(|| {
        SpeechError::Engine(format!("sherpa-onnx could not create the VAD from {}", model.display()))
    })?;

    let mut spans = Vec::new();
    let mut drain = |vad: &VoiceActivityDetector| {
        while let Some(seg) = vad.front() {
            let start = seg.start().max(0) as usize;
            spans.push(Span { start, end: start + seg.n().max(0) as usize });
            vad.pop();
        }
    };
    for window in samples.chunks(WINDOW) {
        vad.accept_waveform(window);
        drain(&vad);
    }
    vad.flush();
    drain(&vad);
    Ok(spans)
}
