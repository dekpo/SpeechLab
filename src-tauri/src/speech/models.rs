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
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelFiles {
    pub encoder: String,
    pub decoder: String,
    pub joiner: Option<String>,
    pub tokens: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelDef {
    pub id: String,
    pub display_name: String,
    pub provider: String,
    pub family: ModelFamily,
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
        let f = &def.files;
        let mut required = vec![MARKER, &f.encoder, &f.decoder, &f.tokens];
        if let Some(j) = &f.joiner {
            required.push(j);
        }
        required.iter().all(|name| dir.join(name).is_file())
    }

    pub fn list(&self) -> Vec<ModelInfo> {
        self.defs
            .iter()
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

    /// Downloads, verifies and extracts a model. Returns the SHA-256 of the archive.
    /// This is the only code path that touches the network.
    pub fn install(
        &self,
        id: &str,
        cancel: &CancelToken,
        on_progress: &mut dyn FnMut(DownloadProgress),
    ) -> Result<String, SpeechError> {
        let def = self.def(id)?.clone();
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
        let partial = self.models_dir.join(".partial");
        let archive = partial.join(format!("{}.tar.bz2", def.dir_name));

        let sha = download::download(
            &def.archive_url,
            &archive,
            def.archive_bytes,
            def.sha256.as_deref(),
            cancel,
            &mut emit,
        )?;

        emit(DownloadPhase::Extract, def.archive_bytes, def.archive_bytes);
        let target = self.model_dir(&def);
        if target.exists() {
            fs::remove_dir_all(&target).map_err(|e| SpeechError::Download(e.to_string()))?;
        }
        download::extract_tar_bz2(&archive, &self.models_dir)?;
        let _ = fs::remove_file(&archive);

        // Prove the layout matches the manifest before declaring the model installed.
        let f = &def.files;
        let mut required = vec![&f.encoder, &f.decoder, &f.tokens];
        if let Some(j) = &f.joiner {
            required.push(j);
        }
        for name in required {
            if !target.join(name).is_file() {
                return Err(SpeechError::Download(format!(
                    "archive extracted but expected file is missing: {}",
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
    fn unknown_model_is_an_error() {
        let m = ModelManager::new(std::env::temp_dir().join("speechlab_models_test")).unwrap();
        assert!(matches!(m.def("nope"), Err(SpeechError::UnknownModel(_))));
    }

    #[test]
    fn invalid_manifest_is_reported() {
        assert!(ModelManager::from_manifest(PathBuf::from("."), "{not json").is_err());
    }
}
