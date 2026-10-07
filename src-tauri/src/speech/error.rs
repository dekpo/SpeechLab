use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpeechError {
    UnknownProvider(String),
    UnknownModel(String),
    ModelNotInstalled(String),
    InvalidRequest(String),
    Cancelled,
    Audio(String),
    Download(String),
    Engine(String),
}

impl fmt::Display for SpeechError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpeechError::UnknownProvider(id) => write!(f, "unknown provider: {id}"),
            SpeechError::UnknownModel(id) => write!(f, "unknown model: {id}"),
            SpeechError::ModelNotInstalled(id) => {
                write!(f, "model not installed: {id} (install it first)")
            }
            SpeechError::InvalidRequest(msg) => write!(f, "invalid request: {msg}"),
            SpeechError::Cancelled => write!(f, "operation cancelled"),
            SpeechError::Audio(msg) => write!(f, "audio error: {msg}"),
            SpeechError::Download(msg) => write!(f, "download error: {msg}"),
            SpeechError::Engine(msg) => write!(f, "engine error: {msg}"),
        }
    }
}

impl std::error::Error for SpeechError {}
