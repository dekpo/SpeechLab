use std::sync::Arc;

use super::error::SpeechError;
use super::models::ModelManager;
use super::provider::SpeechToTextProvider;
use super::sherpa::SherpaOnnxProvider;
use super::types::ProviderInfo;
use super::whisper_cpp::WhisperCppProvider;

/// Holds every registered provider. Adding an engine = implementing the trait and registering it here.
pub struct ProviderRegistry {
    stt: Vec<Box<dyn SpeechToTextProvider>>,
}

impl ProviderRegistry {
    pub fn new(stt: Vec<Box<dyn SpeechToTextProvider>>) -> Self {
        Self { stt }
    }

    pub fn with_default_providers(models: Arc<ModelManager>) -> Self {
        // Real engines are registered here. The mock provider is intentionally NOT
        // registered: it exists for tests only.
        Self::new(vec![
            Box::new(SherpaOnnxProvider::new(Arc::clone(&models))),
            Box::new(WhisperCppProvider::new(models)),
        ])
    }

    pub fn list_stt(&self) -> Vec<ProviderInfo> {
        self.stt.iter().map(|p| p.info()).collect()
    }

    pub fn stt(&self, id: &str) -> Result<&dyn SpeechToTextProvider, SpeechError> {
        self.stt
            .iter()
            .find(|p| p.info().id == id)
            .map(|p| p.as_ref())
            .ok_or_else(|| SpeechError::UnknownProvider(id.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::speech::mock::{MockSttProvider, MOCK_ID};
    use crate::speech::provider::CancelToken;
    use crate::speech::types::TranscribeRequest;

    fn mock_registry() -> ProviderRegistry {
        ProviderRegistry::new(vec![Box::new(MockSttProvider)])
    }

    fn request(path: &str) -> TranscribeRequest {
        TranscribeRequest {
            provider_id: MOCK_ID.into(),
            model_id: "none".into(),
            language: "fr".into(),
            audio_path: path.into(),
            vocabulary: Vec::new(),
        }
    }

    #[test]
    fn lists_the_mock_provider_flagged_as_mock() {
        let list = mock_registry().list_stt();
        assert_eq!(list.len(), 1);
        assert!(list[0].is_mock);
    }

    #[test]
    fn default_registry_exposes_both_engines_and_no_mock() {
        let models =
            Arc::new(ModelManager::new(std::env::temp_dir().join("speechlab_reg")).unwrap());
        let list = ProviderRegistry::with_default_providers(models).list_stt();
        let ids: Vec<_> = list.iter().map(|p| p.id.as_str()).collect();
        assert_eq!(ids, vec!["sherpa-onnx", "whisper-cpp"]);
        assert!(list.iter().all(|p| !p.is_mock));
    }

    #[test]
    fn unknown_provider_is_an_error() {
        let reg = mock_registry();
        assert!(matches!(reg.stt("nope"), Err(SpeechError::UnknownProvider(_))));
    }

    #[test]
    fn mock_rejects_empty_path_and_honours_cancellation() {
        let reg = mock_registry();
        let p = reg.stt(MOCK_ID).unwrap();
        let cancel = CancelToken::new();
        assert!(matches!(
            p.transcribe(&request(""), &cancel),
            Err(SpeechError::InvalidRequest(_))
        ));
        cancel.cancel();
        assert_eq!(
            p.transcribe(&request("a.wav"), &cancel).unwrap_err(),
            SpeechError::Cancelled
        );
        cancel.reset();
        assert!(p.transcribe(&request("a.wav"), &cancel).unwrap().is_mock);
    }

    #[test]
    fn json_contract_uses_camel_case() {
        let json = serde_json::to_string(&mock_registry().list_stt()[0]).unwrap();
        assert!(json.contains("\"displayName\"") && json.contains("\"isMock\":true"));
    }
}
