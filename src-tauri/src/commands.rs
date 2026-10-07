use std::sync::Arc;

use tauri::State;

use crate::speech::provider::CancelToken;
use crate::speech::registry::ProviderRegistry;
use crate::speech::types::{ProviderInfo, TranscribeRequest, TranscribeResult};

pub struct AppState {
    registry: Arc<ProviderRegistry>,
    cancel: CancelToken,
}

impl AppState {
    pub fn new(registry: ProviderRegistry) -> Self {
        Self {
            registry: Arc::new(registry),
            cancel: CancelToken::new(),
        }
    }
}

#[tauri::command]
pub fn list_stt_providers(state: State<'_, AppState>) -> Vec<ProviderInfo> {
    state.registry.list_stt()
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
