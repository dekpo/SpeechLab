//! Local store for audio clips (microphone recordings and converted imports).
//!
//! Clips are plain WAV files in the app data directory. They never leave the machine; the
//! UI offers a delete action so recordings can be removed. Deleting is restricted to files
//! inside the store directory.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::error::SpeechError;
use super::wav;

/// Upper bound for one clip (about 100 minutes of 16 kHz mono 16-bit audio).
const MAX_CLIP_BYTES: usize = 200 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipInfo {
    pub path: String,
    pub duration_ms: u64,
    pub sample_rate: u32,
    pub bytes: u64,
}

pub struct ClipStore {
    dir: PathBuf,
}

impl ClipStore {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// Validates and stores a WAV payload, returning its path and measured duration.
    pub fn save(&self, bytes: &[u8]) -> Result<ClipInfo, SpeechError> {
        if bytes.len() < 44 || bytes.len() > MAX_CLIP_BYTES {
            return Err(SpeechError::InvalidRequest(format!(
                "clip size {} bytes is outside the accepted range",
                bytes.len()
            )));
        }
        if &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
            return Err(SpeechError::InvalidRequest("clip is not a WAV payload".into()));
        }
        fs::create_dir_all(&self.dir).map_err(|e| SpeechError::Audio(e.to_string()))?;
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        let path = self.dir.join(format!("clip-{stamp}.wav"));
        fs::write(&path, bytes).map_err(|e| SpeechError::Audio(e.to_string()))?;
        // Decode once: proves the file is usable and gives the duration.
        match wav::read_wav_mono(&path) {
            Ok(audio) => Ok(ClipInfo {
                path: path.display().to_string(),
                duration_ms: audio.duration_ms(),
                sample_rate: audio.sample_rate,
                bytes: bytes.len() as u64,
            }),
            Err(e) => {
                let _ = fs::remove_file(&path);
                Err(e)
            }
        }
    }

    /// Resolves `path` and refuses anything that is not a `.wav` inside the store directory.
    fn resolve_inside(&self, path: &str) -> Result<PathBuf, SpeechError> {
        let target = fs::canonicalize(path)
            .map_err(|e| SpeechError::InvalidRequest(format!("cannot resolve path: {e}")))?;
        let root = fs::canonicalize(&self.dir)
            .map_err(|e| SpeechError::InvalidRequest(format!("clip store not available: {e}")))?;
        if !target.starts_with(&root) || target.extension().and_then(|e| e.to_str()) != Some("wav") {
            return Err(SpeechError::InvalidRequest(
                "refusing to access a file outside the clip store".into(),
            ));
        }
        Ok(target)
    }

    /// Deletes a clip; refuses any path outside the store directory.
    pub fn delete(&self, path: &str) -> Result<(), SpeechError> {
        let target = self.resolve_inside(path)?;
        fs::remove_file(target).map_err(|e| SpeechError::Audio(e.to_string()))
    }

    /// Raw bytes of a stored clip (for playback); refuses paths outside the store.
    pub fn read(&self, path: &str) -> Result<Vec<u8>, SpeechError> {
        let target = self.resolve_inside(path)?;
        fs::read(target).map_err(|e| SpeechError::Audio(e.to_string()))
    }

    /// Every readable clip currently in the store, oldest first. Unreadable files are skipped.
    pub fn list(&self) -> Vec<ClipInfo> {
        let Ok(entries) = fs::read_dir(&self.dir) else {
            return Vec::new();
        };
        let mut paths: Vec<PathBuf> = entries
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("wav"))
            .collect();
        paths.sort();
        paths
            .into_iter()
            .filter_map(|p| {
                let audio = wav::read_wav_mono(&p).ok()?;
                Some(ClipInfo {
                    path: p.display().to_string(),
                    duration_ms: audio.duration_ms(),
                    sample_rate: audio.sample_rate,
                    bytes: fs::metadata(&p).map(|m| m.len()).unwrap_or(0),
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wav_bytes(seconds: u32) -> Vec<u8> {
        let mut cur = std::io::Cursor::new(Vec::new());
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 16000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut w = hound::WavWriter::new(&mut cur, spec).unwrap();
        for _ in 0..(16000 * seconds) {
            w.write_sample(1000i16).unwrap();
        }
        w.finalize().unwrap();
        cur.into_inner()
    }

    fn store(name: &str) -> ClipStore {
        let dir = std::env::temp_dir().join(format!("speechlab_clips_{name}"));
        let _ = fs::remove_dir_all(&dir);
        ClipStore::new(dir)
    }

    #[test]
    fn saves_a_valid_wav_and_reports_duration() {
        let s = store("save");
        let info = s.save(&wav_bytes(2)).unwrap();
        assert_eq!(info.duration_ms, 2000);
        assert_eq!(info.sample_rate, 16000);
        assert!(Path::new(&info.path).is_file());
        let _ = fs::remove_dir_all(s.dir());
    }

    #[test]
    fn rejects_non_wav_and_tiny_payloads() {
        let s = store("reject");
        assert!(s.save(b"not a wav at all, definitely not, no no no no no no no no").is_err());
        assert!(s.save(&[0u8; 10]).is_err());
        let _ = fs::remove_dir_all(s.dir());
    }

    #[test]
    fn list_returns_saved_clips_and_read_refuses_outside_paths() {
        let s = store("list");
        assert!(s.list().is_empty());
        let a = s.save(&wav_bytes(1)).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(5));
        let b = s.save(&wav_bytes(2)).unwrap();
        let listed = s.list();
        assert_eq!(listed.len(), 2);
        assert_eq!(listed[0].path, a.path);
        assert_eq!(listed[1].duration_ms, 2000);
        assert_eq!(s.read(&b.path).unwrap().len() as u64, b.bytes);
        let outside = std::env::temp_dir().join("speechlab_outside_read.wav");
        fs::write(&outside, wav_bytes(1)).unwrap();
        assert!(s.read(&outside.display().to_string()).is_err());
        let _ = fs::remove_file(outside);
        let _ = fs::remove_dir_all(s.dir());
    }

    #[test]
    fn delete_works_inside_the_store_and_refuses_outside() {
        let s = store("delete");
        let info = s.save(&wav_bytes(1)).unwrap();
        s.delete(&info.path).unwrap();
        assert!(!Path::new(&info.path).exists());

        let outside = std::env::temp_dir().join("speechlab_outside_clip.wav");
        fs::write(&outside, wav_bytes(1)).unwrap();
        let err = s.delete(&outside.display().to_string());
        assert!(err.is_err());
        assert!(outside.exists(), "file outside the store must survive");
        let _ = fs::remove_file(outside);
        let _ = fs::remove_dir_all(s.dir());
    }
}
