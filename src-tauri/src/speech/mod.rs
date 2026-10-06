//! Engine-agnostic speech contract.
//!
//! The UI and the Tauri commands only know the traits and types defined here.
//! Each engine (sherpa-onnx, whisper.cpp, ...) is an adapter implementing the traits.

pub mod error;
pub mod mock;
pub mod provider;
pub mod registry;
pub mod types;
