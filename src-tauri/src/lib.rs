mod commands;
pub mod speech;

use std::path::PathBuf;
use std::sync::Arc;

use tauri::Manager;

use commands::AppState;
use speech::clips::ClipStore;
use speech::models::ModelManager;
use speech::registry::ProviderRegistry;

/// Models live outside the repository. `SPEECHLAB_MODELS_DIR` overrides the default
/// (`<app data dir>/models`), which is handy for sharing one folder with the CLI example.
pub fn resolve_models_dir(default_app_data: PathBuf) -> PathBuf {
    std::env::var_os("SPEECHLAB_MODELS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| default_app_data.join("models"))
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data = app.path().app_data_dir()?;
            let clips = ClipStore::new(app_data.join("recordings"));
            let models = Arc::new(
                ModelManager::new(resolve_models_dir(app_data)).map_err(|e| e.to_string())?,
            );
            let registry = ProviderRegistry::with_default_providers(Arc::clone(&models));
            app.manage(AppState::new(registry, models, clips));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_stt_providers,
            commands::list_models,
            commands::transcribe,
            commands::cancel_transcription,
            commands::install_model,
            commands::cancel_model_download,
            commands::save_clip,
            commands::delete_clip,
            commands::list_clips,
            commands::read_clip,
            commands::compare_texts,
            commands::list_scripts,
            commands::list_samples,
            commands::create_sample,
            commands::delete_sample,
            commands::read_sample_audio,
            commands::list_tts_providers,
            commands::list_tts_models,
            commands::list_tts_voices,
            commands::synthesize,
            commands::cancel_synthesis,
            commands::read_tts_audio,
            commands::clear_tts_audio,
        ])
        .run(tauri::generate_context!())
        .expect("error while running the SpeechLab application");
}
