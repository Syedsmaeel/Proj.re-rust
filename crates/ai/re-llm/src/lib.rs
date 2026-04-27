//! re-llm: Local LLM inference for Proj.re-rust
//!
//! First model: Microsoft Phi-3-mini-4k-instruct (open weights, MIT)
//! Backend: candle (Hugging Face's Rust ML framework — pure Rust, no Python).
//!
//! Used by `re-agent` (the OpenClaw → Rust port) as its model provider, so the
//! whole agent loop can run with no remote API and no third-party runtime.

pub mod phi;

pub use phi::{GenerationConfig, PhiBackend, PhiVariant};
