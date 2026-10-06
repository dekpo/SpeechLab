//! Model download, verification and extraction.
//!
//! This is the ONLY module that uses the network. Inference code never calls it,
//! so recognition keeps working with the network disconnected.

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use std::time::Duration;

use sha2::{Digest, Sha256};

use super::error::SpeechError;
use super::provider::CancelToken;
use super::types::DownloadPhase;

pub type ProgressFn<'a> = &'a mut dyn FnMut(DownloadPhase, u64, u64);

/// Downloads `url` to `dest`, returning the SHA-256 (lowercase hex) of the received bytes.
/// If `expected_sha256` is given and differs, the file is deleted and an error is returned.
pub fn download(
    url: &str,
    dest: &Path,
    expected_len: u64,
    expected_sha256: Option<&str>,
    cancel: &CancelToken,
    progress: ProgressFn,
) -> Result<String, SpeechError> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).map_err(|e| SpeechError::Download(e.to_string()))?;
    }
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(20))
        .timeout_read(Duration::from_secs(60))
        .try_proxy_from_env(true)
        .build();
    let response = agent
        .get(url)
        .call()
        .map_err(|e| SpeechError::Download(format!("GET {url}: {e}")))?;
    let total = response
        .header("Content-Length")
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(expected_len);

    let mut reader = response.into_reader();
    let mut file = File::create(dest).map_err(|e| SpeechError::Download(e.to_string()))?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 256 * 1024];
    let mut done: u64 = 0;
    let mut last_report = 0u64;

    loop {
        if cancel.is_cancelled() {
            drop(file);
            let _ = fs::remove_file(dest);
            return Err(SpeechError::Cancelled);
        }
        let n = reader
            .read(&mut buf)
            .map_err(|e| SpeechError::Download(format!("read error: {e}")))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
        file.write_all(&buf[..n])
            .map_err(|e| SpeechError::Download(e.to_string()))?;
        done += n as u64;
        if done - last_report >= 1_000_000 {
            last_report = done;
            progress(DownloadPhase::Download, done, total);
        }
    }
    file.flush().map_err(|e| SpeechError::Download(e.to_string()))?;
    progress(DownloadPhase::Download, done, total);

    progress(DownloadPhase::Verify, done, total);
    let actual = hex(&hasher.finalize());
    if let Some(expected) = expected_sha256 {
        if !expected.eq_ignore_ascii_case(&actual) {
            let _ = fs::remove_file(dest);
            return Err(SpeechError::Download(format!(
                "checksum mismatch: expected {expected}, got {actual}"
            )));
        }
    }
    Ok(actual)
}

/// Extracts a `.tar.bz2` archive into `target_dir` (path traversal is rejected by `tar`).
pub fn extract_tar_bz2(archive: &Path, target_dir: &Path) -> Result<(), SpeechError> {
    let file = File::open(archive).map_err(|e| SpeechError::Download(e.to_string()))?;
    let decoder = bzip2::read::BzDecoder::new(file);
    tar::Archive::new(decoder)
        .unpack(target_dir)
        .map_err(|e| SpeechError::Download(format!("extract failed: {e}")))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_encodes_lowercase() {
        assert_eq!(hex(&[0x00, 0xab, 0xff]), "00abff");
    }

    #[test]
    fn extract_rejects_garbage_archive() {
        let p = std::env::temp_dir().join("speechlab_garbage.tar.bz2");
        std::fs::write(&p, b"not an archive").unwrap();
        let out = std::env::temp_dir().join("speechlab_garbage_out");
        assert!(extract_tar_bz2(&p, &out).is_err());
        let _ = std::fs::remove_file(p);
    }
}
