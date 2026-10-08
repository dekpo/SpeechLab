//! Engine-agnostic speech contract.
//!
//! The UI and the Tauri commands only know the traits and types defined here.
//! Each engine (sherpa-onnx, whisper.cpp, ...) is an adapter implementing the traits.

pub mod benchmark;
pub mod chunking;
pub mod clips;
pub mod critical;
pub mod dataset;
pub mod download;
pub mod error;
pub mod metrics;
pub mod mock;
pub mod models;
pub mod numbers;
pub mod probe;
pub mod postcorrect;
pub mod provider;
pub mod registry;
pub mod sherpa;
pub mod termstudy;
pub mod types;
pub mod vad;
pub mod wav;
pub mod whisper_cpp;
