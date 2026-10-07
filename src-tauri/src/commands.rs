use std::sync::Arc;

use tauri::{AppHandle, Emitter, State};

use crate::speech::clips::{ClipInfo, ClipStore};
use crate::speech::dataset::{
    self, Dataset, Sample, ScriptItem, SourceInfo, Speaker, ValidationIssue,
};
use crate::speech::metrics::{compare_texts as compare, TextComparison};
use crate::speech::models::ModelManager;
use crate::speech::provider::CancelToken;
use crate::speech::registry::ProviderRegistry;
use crate::speech::types::{ModelInfo, ProviderInfo, TranscribeRequest, TranscribeResult};

pub struct AppState {
    registry: Arc<ProviderRegistry>,
    models: Arc<ModelManager>,
    clips: ClipStore,
    cancel: CancelToken,
    download_cancel: CancelToken,
}

impl AppState {
    pub fn new(registry: ProviderRegistry, models: Arc<ModelManager>, clips: ClipStore) -> Self {
        Self {
            registry: Arc::new(registry),
            models,
            clips,
            cancel: CancelToken::new(),
            download_cancel: CancelToken::new(),
        }
    }
}

#[tauri::command]
pub fn list_stt_providers(state: State<'_, AppState>) -> Vec<ProviderInfo> {
    state.registry.list_stt()
}

#[tauri::command]
pub fn list_models(state: State<'_, AppState>) -> Vec<ModelInfo> {
    state.models.list()
}

/// Runs on a blocking worker so long inferences never freeze the UI.
#[tauri::command]
pub async fn transcribe(
    state: State<'_, AppState>,
    request: TranscribeRequest,
) -> Result<TranscribeResult, String> {
    let registry = Arc::clone(&state.registry);
    let cancel = state.cancel.clone();
    cancel.reset();
    tauri::async_runtime::spawn_blocking(move || {
        registry
            .stt(&request.provider_id)
            .and_then(|p| p.transcribe(&request, &cancel))
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("worker failed: {e}"))?
}

#[tauri::command]
pub fn cancel_transcription(state: State<'_, AppState>) {
    state.cancel.cancel();
}

/// Downloads and installs a model (the only network operation). Emits
/// `model-download-progress` events. Returns the archive SHA-256.
#[tauri::command]
pub async fn install_model(
    app: AppHandle,
    state: State<'_, AppState>,
    model_id: String,
) -> Result<String, String> {
    let models = Arc::clone(&state.models);
    let cancel = state.download_cancel.clone();
    cancel.reset();
    tauri::async_runtime::spawn_blocking(move || {
        models
            .install(&model_id, &cancel, &mut |p| {
                let _ = app.emit("model-download-progress", p);
            })
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("worker failed: {e}"))?
}

#[tauri::command]
pub fn cancel_model_download(state: State<'_, AppState>) {
    state.download_cancel.cancel();
}

/// Stores a WAV clip sent as a raw binary body (microphone recording or converted import).
#[tauri::command]
pub fn save_clip(
    request: tauri::ipc::Request<'_>,
    state: State<'_, AppState>,
) -> Result<ClipInfo, String> {
    match request.body() {
        tauri::ipc::InvokeBody::Raw(bytes) => state.clips.save(bytes).map_err(|e| e.to_string()),
        _ => Err("save_clip expects a raw binary body".into()),
    }
}

#[tauri::command]
pub fn delete_clip(state: State<'_, AppState>, path: String) -> Result<(), String> {
    state.clips.delete(&path).map_err(|e| e.to_string())
}

/// WER, CER and a word diff between a reference (or baseline) and a hypothesis.
#[tauri::command]
pub fn compare_texts(reference: String, hypothesis: String) -> TextComparison {
    compare(&reference, &hypothesis)
}

/// Clips already stored on disk (so recordings survive a reload and can still be deleted).
#[tauri::command]
pub fn list_clips(state: State<'_, AppState>) -> Vec<ClipInfo> {
    state.clips.list()
}

/// WAV bytes of a stored clip, for playback in the UI.
#[tauri::command]
pub fn read_clip(
    state: State<'_, AppState>,
    path: String,
) -> Result<tauri::ipc::Response, String> {
    state
        .clips
        .read(&path)
        .map(tauri::ipc::Response::new)
        .map_err(|e| e.to_string())
}

// ---------- Benchmark dataset recorder (M5b) ----------

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatasetStatus {
    root: String,
    samples: Vec<Sample>,
    issues: Vec<ValidationIssue>,
}

/// The sentences to read, from `benchmark/scripts/`.
#[tauri::command]
pub fn list_scripts() -> Result<Vec<ScriptItem>, String> {
    dataset::load_scripts(&dataset::default_root()).map_err(|e| e.to_string())
}

/// Samples already recorded (public and private), with validation issues.
#[tauri::command]
pub fn list_samples() -> Result<DatasetStatus, String> {
    let root = dataset::default_root();
    let ds = Dataset::load(&root).map_err(|e| e.to_string())?;
    Ok(DatasetStatus {
        root: root.display().to_string(),
        samples: ds.samples,
        issues: ds.issues,
    })
}

/// Turns a stored clip into a benchmark sample for the given script sentence, then removes the
/// temporary clip. The reference transcript is the script text, never typed by hand.
#[tauri::command]
pub fn create_sample(
    state: State<'_, AppState>,
    script_id: String,
    speaker: Speaker,
    source: SourceInfo,
    private: bool,
    clip_path: String,
) -> Result<Sample, String> {
    let root = dataset::default_root();
    let script = dataset::load_scripts(&root)
        .map_err(|e| e.to_string())?
        .into_iter()
        .find(|s| s.id == script_id)
        .ok_or_else(|| format!("unknown script id: {script_id}"))?;
    let wav = state.clips.read(&clip_path).map_err(|e| e.to_string())?;
    let sample = dataset::create_sample(&root, &script, speaker, source, private, &wav)
        .map_err(|e| e.to_string())?;
    let _ = state.clips.delete(&clip_path);
    Ok(sample)
}

#[tauri::command]
pub fn delete_sample(sample_id: String) -> Result<(), String> {
    dataset::delete_sample(&dataset::default_root(), &sample_id).map_err(|e| e.to_string())
}

/// WAV bytes of a recorded benchmark sample (to listen to it again).
#[tauri::command]
pub fn read_sample_audio(sample_id: String) -> Result<tauri::ipc::Response, String> {
    let id = dataset::sanitize_id(&sample_id);
    if id.is_empty() || id != sample_id {
        return Err("invalid sample id".into());
    }
    let path = dataset::default_root().join("audio").join(format!("{id}.wav"));
    std::fs::read(path)
        .map(tauri::ipc::Response::new)
        .map_err(|e| e.to_string())
}
