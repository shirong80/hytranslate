//! Ollama HTTP 클라이언트. Phase 1: streaming generate + prompt builder + 모델 상수.

pub mod client;
pub mod models;
pub mod prompt;

pub use client::{ChunkFlow, OllamaClient};
pub use models::{DEFAULT_MODEL, MODEL_HY_MT2_1_8B, MODEL_HY_MT2_7B};
pub use prompt::build_prompt;
