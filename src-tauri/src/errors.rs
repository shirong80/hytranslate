use serde::Serialize;
use thiserror::Error;

/// 공개 에러 — `#[tauri::command]` 가 반환하는 직렬화 shape.
/// FE 의 `src/lib/ipc/errors.ts` 와 1:1 mirror. variant 변경은 양쪽 동시 적용.
#[derive(Debug, Error, Serialize)]
#[serde(tag = "kind")]
pub enum AppError {
    #[error("Ollama is not available")]
    OllamaUnavailable,

    #[error("Ollama is not running")]
    OllamaNotRunning,

    #[error("model is not installed: {model}")]
    ModelMissing { model: String },

    #[error("input exceeds the {limit}-character limit")]
    InputTooLong { limit: usize },

    #[error("operation cancelled")]
    Cancelled,

    #[error("network access blocked")]
    NetworkBlocked,

    #[error("internal error: {message}")]
    Internal { message: String },
}

impl AppError {
    /// 내부 에러 메시지를 `AppError::Internal` 로 감싸는 헬퍼.
    /// Phase 1 진입 시 `From<reqwest::Error>` / `From<rusqlite::Error>` 를 추가하여 확장한다.
    pub fn internal<E: std::fmt::Display>(err: E) -> Self {
        AppError::Internal {
            message: err.to_string(),
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_with_kind_discriminator() {
        let err = AppError::ModelMissing {
            model: "hf.co/tencent/Hy-MT2-7B-GGUF:Q4_K_M".to_string(),
        };
        let json = serde_json::to_string(&err).expect("serializable");
        assert!(json.contains(r#""kind":"ModelMissing""#));
        assert!(json.contains(r#""model":"hf.co/tencent/Hy-MT2-7B-GGUF:Q4_K_M""#));
    }

    #[test]
    fn cancelled_has_no_extra_fields() {
        let json = serde_json::to_string(&AppError::Cancelled).expect("serializable");
        assert_eq!(json, r#"{"kind":"Cancelled"}"#);
    }

    #[test]
    fn internal_helper_wraps_display() {
        let err = AppError::internal("boom");
        match err {
            AppError::Internal { message } => assert_eq!(message, "boom"),
            _ => panic!("expected Internal variant"),
        }
    }
}
