//! Engine-agnostic speech contract.
//!
//! The UI and the Tauri commands only know the traits and types defined here.
//! Each engine (sherpa-onnx, whisper.cpp, ...) is an adapter implementing the traits.

pub mod clips;
pub mod download;
pub mod error;
pub mod metrics;
pub mod mock;
pub mod models;
pub mod provider;
pub mod registry;
pub mod sherpa;
pub mod types;
pub mod wav;
pub mod whisper_cpp;
