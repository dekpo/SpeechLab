use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpeechError {
    UnknownProvider(String),
    InvalidRequest(String),
    Cancelled,
    Engine(String),
}

impl fmt::Display for SpeechError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpeechError::UnknownProvider(id) => write!(f, "unknown provider: {id}"),
            SpeechError::InvalidRequest(msg) => write!(f, "invalid request: {msg}"),
            SpeechError::Cancelled => write!(f, "operation cancelled"),
            SpeechError::Engine(msg) => write!(f, "engine error: {msg}"),
        }
    }
}

impl std::error::Error for SpeechError {}
