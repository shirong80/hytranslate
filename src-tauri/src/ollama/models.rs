//! Hy-MT2 GGUF 모델 식별자 (PRD §8.4).

pub const MODEL_HY_MT2_7B: &str = "hf.co/tencent/Hy-MT2-7B-GGUF:Q4_K_M";
pub const MODEL_HY_MT2_1_8B: &str = "hf.co/tencent/Hy-MT2-1.8B-GGUF:Q4_K_M";

/// Phase 1 디폴트. Settings UI 도입 (Phase 2) 전까지 하드코딩으로 사용한다.
pub const DEFAULT_MODEL: &str = MODEL_HY_MT2_7B;
