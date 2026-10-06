use std::sync::Arc;

use tauri::{AppHandle, Emitter, State};

use crate::speech::models::ModelManager;
use crate::speech::provider::CancelToken;
use crate::speech::registry::ProviderRegistry;
use crate::speech::types::{ModelInfo, ProviderInfo, TranscribeRequest, TranscribeResult};

pub struct AppState {
    registry: Arc<ProviderRegistry>,
    models: Arc<ModelManager>,
    cancel: CancelToken,
    download_cancel: CancelToken,
}

impl AppState {
    pub fn new(registry: ProviderRegistry, models: Arc<ModelManager>) -> Self {
        Self {
            registry: Arc::new(registry),
            models,
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
