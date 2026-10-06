use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderKind {
    Stt,
    Tts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Capabilities {
    pub supports_cancellation: bool,
    pub supports_language_auto_detect: bool,
    pub supports_streaming: bool,
    pub supports_speed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderInfo {
    pub id: String,
    pub display_name: String,
    pub kind: ProviderKind,
    /// True for placeholder providers; their output must never be used as benchmark data.
    pub is_mock: bool,
    pub languages: Vec<String>,
    pub capabilities: Capabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscribeRequest {
    pub provider_id: String,
    pub model_id: String,
    /// Explicit language code ("fr", "en").
    pub language: String,
    /// Path of a WAV file on disk.
    pub audio_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscribeResult {
    pub provider_id: String,
    pub model_id: String,
    pub language: String,
    pub text: String,
    /// Inference time only (stream creation + decode), excluding model loading.
    pub processing_ms: u64,
    /// Time spent loading the model; 0 when the model was already loaded (warm).
    pub load_ms: u64,
    pub cold_start: bool,
    pub audio_ms: Option<u64>,
    /// Real-time factor = processing_ms / audio_ms (below 1.0 is faster than real time).
    pub rtf: Option<f64>,
    pub threads: u32,
    pub is_mock: bool,
}

// --- TTS contract (declared in M1, implemented in M6) ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SynthesizeRequest {
    pub provider_id: String,
    pub voice_id: String,
    pub language: String,
    pub text: String,
    /// 1.0 = normal speed. Providers that do not support it must report so in `Capabilities`.
    pub speed: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SynthesizeResult {
    pub provider_id: String,
    pub voice_id: String,
    pub wav_path: String,
    pub generation_ms: u64,
    pub audio_ms: u64,
    pub is_mock: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceInfo {
    pub id: String,
    pub display_name: String,
    pub language: String,
    /// "female", "male" or "unknown" - never guessed.
    pub gender: String,
    pub license: String,
}

// --- Model inventory ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InstallStatus {
    NotInstalled,
    Installed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    pub id: String,
    pub display_name: String,
    pub provider: String,
    pub languages: Vec<String>,
    pub architecture: String,
    pub quantization: String,
    /// Size of the download archive in bytes.
    pub size_bytes: u64,
    pub license: String,
    pub source_url: String,
    pub runtime: String,
    pub platforms: Vec<String>,
    /// None = not measured yet (never guessed).
    pub expected_memory_mb: Option<u32>,
    pub install_status: InstallStatus,
    pub installed_path: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DownloadPhase {
    Download,
    Verify,
    Extract,
    Done,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress {
    pub model_id: String,
    pub phase: DownloadPhase,
    pub done_bytes: u64,
    pub total_bytes: u64,
}
