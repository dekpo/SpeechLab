mod commands;
pub mod speech;

use commands::AppState;
use speech::registry::ProviderRegistry;

pub fn run() {
    tauri::Builder::default()
        .manage(AppState::new(ProviderRegistry::with_default_providers()))
        .invoke_handler(tauri::generate_handler![
            commands::list_stt_providers,
            commands::transcribe,
            commands::cancel_transcription,
        ])
        .run(tauri::generate_context!())
        .expect("error while running the SpeechLab application");
}
