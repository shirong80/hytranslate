//! Ollama HTTP 클라이언트. Phase 2 부터는 endpoint 가 per-call 인자.

pub mod client;
pub mod endpoint;
pub mod models;
pub mod prompt;

pub use client::{ChunkFlow, OllamaClient, PullChunk};
pub use endpoint::is_endpoint_allowed;
pub use models::{DEFAULT_MODEL, MODEL_HY_MT2_1_8B, MODEL_HY_MT2_7B};
pub use prompt::build_prompt;
