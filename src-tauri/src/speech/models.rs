//! Model inventory and installation (SpeechModelManager).
//!
//! The inventory comes from `models-manifest.json`, so new models can be added
//! without changing application code. Models are never bundled in the app: they are
//! downloaded on demand into `models_dir`, checksum-verified, and then used offline.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use super::download;
use super::error::SpeechError;
use super::provider::CancelToken;
use super::types::{DownloadPhase, DownloadProgress, InstallStatus, ModelInfo};

const MANIFEST: &str = include_str!("../../models-manifest.json");
const MARKER: &str = ".speechlab-installed";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ModelFamily {
    Whisper,
    Canary,
    NemoTransducer,
    GgmlWhisper,
    /// Silero voice-activity detector (a support model, run through the sherpa-onnx VAD API).
    SileroVad,
    /// Piper voice (VITS) run through the sherpa-onnx text-to-speech API.
    PiperVits,
    /// Kokoro multi-voice model run through the sherpa-onnx text-to-speech API.
    Kokoro,
}

/// What a manifest entry is for. Support models (for example the voice-activity detector) are
/// downloadable and checksum-verified like engines, but they are not speech-to-text engines:
/// `list()` leaves them out so the UI engine pickers and the benchmark never see them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ModelRole {
    #[default]
    Stt,
    Support,
    /// Text-to-speech voices (M6): listed by `tts_models()`, never by `list()`.
    Tts,
}

/// Manifest id of the voice-activity detector used by the chunker.
pub const VAD_MODEL_ID: &str = "silero-vad";

/// How the model is distributed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum Packaging {
    /// A `.tar.bz2` archive that extracts to `dirName/`.
    #[default]
    Archive,
    /// A single file saved as `dirName/<files.model>`.
    File,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelFiles {
    pub encoder: Option<String>,
    pub decoder: Option<String>,
    pub joiner: Option<String>,
    pub tokens: Option<String>,
    /// Single-file models (ggml, VITS and Kokoro text-to-speech).
    pub model: Option<String>,
    /// Text-to-speech: speaker embeddings file (Kokoro).
    pub voices: Option<String>,
    /// Text-to-speech: pronunciation lexicon file(s), comma separated (Kokoro).
    pub lexicon: Option<String>,
    /// Text-to-speech: folder of phonemizer data inside the model folder (must exist, not a file).
    pub data_dir: Option<String>,
}

impl ModelFiles {
    pub fn required(&self) -> Vec<&str> {
        let lexicons: Vec<&str> = self.lexicon.as_deref().map(|l| l.split(',').collect()).unwrap_or_default();
        [&self.encoder, &self.decoder, &self.joiner, &self.tokens, &self.model, &self.voices]
            .into_iter()
            .flatten()
            .map(String::as_str)
            .chain(lexicons)
            .collect()
    }

    /// Returns a required file name or a manifest error (never panics on bad manifests).
    pub fn get<'a>(&'a self, field: &'a Option<String>, what: &str) -> Result<&'a str, SpeechError> {
        field
            .as_deref()
            .ok_or_else(|| SpeechError::Engine(format!("manifest is missing files.{what}")))
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelDef {
    pub id: String,
    pub display_name: String,
    pub provider: String,
    pub family: ModelFamily,
    #[serde(default)]
    pub role: ModelRole,
    #[serde(default)]
    pub packaging: Packaging,
    pub languages: Vec<String>,
    pub architecture: String,
    pub quantization: String,
    pub archive_url: String,
    pub archive_bytes: u64,
    /// None = checksum not pinned yet; the computed value is reported after install.
    pub sha256: Option<String>,
    pub dir_name: String,
    pub files: ModelFiles,
    pub license: String,
    pub source_url: String,
    pub runtime: String,
    pub platforms: Vec<String>,
    pub expected_memory_mb: Option<u32>,
}

pub struct ModelManager {
    models_dir: PathBuf,
    defs: Vec<ModelDef>,
}

impl ModelManager {
    pub fn new(models_dir: PathBuf) -> Result<Self, SpeechError> {
        Self::from_manifest(models_dir, MANIFEST)
    }

    pub fn from_manifest(models_dir: PathBuf, manifest: &str) -> Result<Self, SpeechError> {
        let defs: Vec<ModelDef> = serde_json::from_str(manifest)
            .map_err(|e| SpeechError::Engine(format!("invalid model manifest: {e}")))?;
        Ok(Self { models_dir, defs })
    }

    pub fn models_dir(&self) -> &Path {
        &self.models_dir
    }

    pub fn def(&self, id: &str) -> Result<&ModelDef, SpeechError> {
        self.defs
            .iter()
            .find(|d| d.id == id)
            .ok_or_else(|| SpeechError::UnknownModel(id.to_string()))
    }

    pub fn model_dir(&self, def: &ModelDef) -> PathBuf {
        self.models_dir.join(&def.dir_name)
    }

    pub fn is_installed(&self, def: &ModelDef) -> bool {
        let dir = self.model_dir(def);
        let required = def.files.required();
        !required.is_empty()
            && dir.join(MARKER).is_file()
            && required.iter().all(|name| dir.join(name).is_file())
            && def.files.data_dir.as_ref().map_or(true, |d| dir.join(d).is_dir())
    }

    /// Speech-to-text models only; support models are reached through `support_models`.
    pub fn list(&self) -> Vec<ModelInfo> {
        self.info_for(ModelRole::Stt)
    }

    /// Support models (voice-activity detector), kept out of `list()` on purpose.
    pub fn support_models(&self) -> Vec<ModelInfo> {
        self.info_for(ModelRole::Support)
    }

    /// Path of the installed voice-activity-detector model file.
    pub fn vad_model_path(&self) -> Result<PathBuf, SpeechError> {
        let def = self.def(VAD_MODEL_ID)?;
        if !self.is_installed(def) {
            return Err(SpeechError::ModelNotInstalled(def.id.clone()));
        }
        Ok(self.model_dir(def).join(def.files.get(&def.files.model, "model")?))
    }

    /// Text-to-speech voices (M6).
    pub fn tts_models(&self) -> Vec<ModelInfo> {
        self.info_for(ModelRole::Tts)
    }

    fn info_for(&self, role: ModelRole) -> Vec<ModelInfo> {
        self.defs
            .iter()
            .filter(|d| d.role == role)
            .map(|d| {
                let installed = self.is_installed(d);
                ModelInfo {
                    id: d.id.clone(),
                    display_name: d.display_name.clone(),
                    provider: d.provider.clone(),
                    languages: d.languages.clone(),
                    architecture: d.architecture.clone(),
                    quantization: d.quantization.clone(),
                    size_bytes: d.archive_bytes,
                    license: d.license.clone(),
                    source_url: d.source_url.clone(),
                    runtime: d.runtime.clone(),
                    platforms: d.platforms.clone(),
                    expected_memory_mb: d.expected_memory_mb,
                    install_status: if installed {
                        InstallStatus::Installed
                    } else {
                        InstallStatus::NotInstalled
                    },
                    installed_path: installed.then(|| self.model_dir(d).display().to_string()),
                }
            })
            .collect()
    }

    /// Downloads, verifies and installs a model. Returns the SHA-256 of the downloaded file.
    /// This is the only code path that touches the network.
    pub fn install(
        &self,
        id: &str,
        cancel: &CancelToken,
        on_progress: &mut dyn FnMut(DownloadProgress),
    ) -> Result<String, SpeechError> {
        let def = self.def(id)?.clone();
        if def.files.required().is_empty() {
            return Err(SpeechError::Engine(format!("manifest for {id} lists no files")));
        }
        let model_id = def.id.clone();
        let mut emit = |phase: DownloadPhase, done: u64, total: u64| {
            on_progress(DownloadProgress {
                model_id: model_id.clone(),
                phase,
                done_bytes: done,
                total_bytes: total,
            })
        };

        fs::create_dir_all(&self.models_dir).map_err(|e| SpeechError::Download(e.to_string()))?;
        let target = self.model_dir(&def);
        if target.exists() {
            fs::remove_dir_all(&target).map_err(|e| SpeechError::Download(e.to_string()))?;
        }

        let sha = match def.packaging {
            Packaging::Archive => {
                let archive = self
                    .models_dir
                    .join(".partial")
                    .join(format!("{}.tar.bz2", def.dir_name));
                let sha = download::download(
                    &def.archive_url,
                    &archive,
                    def.archive_bytes,
                    def.sha256.as_deref(),
                    cancel,
                    &mut emit,
                )?;
                emit(DownloadPhase::Extract, def.archive_bytes, def.archive_bytes);
                download::extract_tar_bz2(&archive, &self.models_dir)?;
                let _ = fs::remove_file(&archive);
                sha
            }
            Packaging::File => {
                let name = def.files.get(&def.files.model, "model")?;
                fs::create_dir_all(&target).map_err(|e| SpeechError::Download(e.to_string()))?;
                let partial = target.join(format!("{name}.partial"));
                let sha = download::download(
                    &def.archive_url,
                    &partial,
                    def.archive_bytes,
                    def.sha256.as_deref(),
                    cancel,
                    &mut emit,
                )?;
                fs::rename(&partial, target.join(name))
                    .map_err(|e| SpeechError::Download(e.to_string()))?;
                sha
            }
        };

        // Prove the layout matches the manifest before declaring the model installed.
        if let Some(d) = &def.files.data_dir {
            if !target.join(d).is_dir() {
                return Err(SpeechError::Download(format!("expected folder is missing: {}", target.join(d).display())));
            }
        }
        for name in def.files.required() {
            if !target.join(name).is_file() {
                return Err(SpeechError::Download(format!(
                    "download finished but expected file is missing: {}",
                    target.join(name).display()
                )));
            }
        }
        fs::write(target.join(MARKER), format!("sha256={sha}\n"))
            .map_err(|e| SpeechError::Download(e.to_string()))?;
        emit(DownloadPhase::Done, def.archive_bytes, def.archive_bytes);
        Ok(sha)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_manifest_parses_and_has_unique_ids() {
        let m = ModelManager::new(std::env::temp_dir().join("speechlab_models_test")).unwrap();
        let list = m.list();
        assert!(list.len() >= 3);
        let mut ids: Vec<_> = list.iter().map(|i| i.id.clone()).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), list.len());
        assert!(list.iter().all(|i| i.install_status == InstallStatus::NotInstalled));
        assert!(list.iter().all(|i| i.expected_memory_mb.is_none()));
    }

    #[test]
    fn every_manifest_entry_lists_files_and_a_provider() {
        let m = ModelManager::new(std::env::temp_dir().join("speechlab_models_test")).unwrap();
        for d in &m.defs {
            assert!(!d.files.required().is_empty(), "{} has no files", d.id);
            assert!(d.provider == "sherpa-onnx" || d.provider == "whisper-cpp", "{}", d.id);
            assert!(d.sha256.is_some(), "{} must pin a SHA-256", d.id);
            if d.packaging == Packaging::File {
                assert!(d.files.model.is_some(), "{} needs files.model", d.id);
            }
        }
    }

    #[test]
    fn support_models_are_kept_out_of_the_engine_list() {
        let m = ModelManager::new(std::env::temp_dir().join("speechlab_models_test")).unwrap();
        assert!(m.list().iter().all(|i| i.id != VAD_MODEL_ID), "the VAD must not appear in list()");
        assert!(m.support_models().iter().any(|i| i.id == VAD_MODEL_ID));
        let def = m.def(VAD_MODEL_ID).unwrap();
        assert_eq!(def.role, ModelRole::Support);
        assert_eq!(def.family, ModelFamily::SileroVad);
        assert!(def.license.starts_with("MIT"));
        assert!(matches!(m.vad_model_path(), Err(SpeechError::ModelNotInstalled(_))));
    }

    #[test]
    fn tts_voices_are_kept_out_of_the_speech_to_text_list() {
        let m = ModelManager::new(std::env::temp_dir().join("speechlab_models_test")).unwrap();
        let tts = m.tts_models();
        assert!(tts.len() >= 3, "expected the M6 voices in the manifest");
        assert!(tts.iter().all(|i| m.list().iter().all(|s| s.id != i.id)));
        for d in m.defs.iter().filter(|d| d.role == ModelRole::Tts) {
            assert!(matches!(d.family, ModelFamily::PiperVits | ModelFamily::Kokoro), "{}", d.id);
            assert!(d.files.data_dir.is_some(), "{} needs the phonemizer data folder", d.id);
            assert!(!d.license.trim().is_empty());
        }
    }

    #[test]
    fn lexicon_list_counts_as_required_files() {
        let f = ModelFiles { lexicon: Some("a.txt,b.txt".into()), model: Some("m.onnx".into()), ..Default::default() };
        assert_eq!(f.required(), vec!["m.onnx", "a.txt", "b.txt"]);
    }

    #[test]
    fn unknown_model_is_an_error() {
        let m = ModelManager::new(std::env::temp_dir().join("speechlab_models_test")).unwrap();
        assert!(matches!(m.def("nope"), Err(SpeechError::UnknownModel(_))));
    }

    #[test]
    fn invalid_manifest_is_reported() {
        assert!(ModelManager::from_manifest(PathBuf::from("."), "{not json").is_err());
    }
}
