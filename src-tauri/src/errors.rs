use serde::Serialize;
use thiserror::Error;

/// 공개 에러 — `#[tauri::command]` 가 반환하는 직렬화 shape.
/// FE 의 `src/lib/ipc/errors.ts` 와 1:1 mirror. variant 변경은 양쪽 동시 적용.
#[derive(Debug, Clone, Error, Serialize)]
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

    // PRD §11 의 표 변형 — 권한 producer 는 `AXIsProcessTrustedWithOptions` 등
    // macOS 시스템 API 와 함께 Phase 5 onboarding 에서 도입된다. v1 의 단축키 권한
    // 안내는 settings 패널의 persistent CTA 로 처리된다 (코드리뷰 Medium 1).
    #[error("macOS permission required for feature: {feature}")]
    PermissionRequired { feature: String },

    #[error("invalid shortcut accelerator: {input}")]
    InvalidShortcut { input: String },

    #[error("internal error: {message}")]
    Internal { message: String },
}

impl AppError {
    /// 내부 에러 메시지를 `AppError::Internal` 로 감싸는 헬퍼.
    pub fn internal<E: std::fmt::Display>(err: E) -> Self {
        AppError::Internal {
            message: err.to_string(),
        }
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        // Connect / DNS 실패는 Ollama 미실행으로 간주. 그 외는 Internal 로 폴백.
        // 원문 / 번역 결과는 절대 로그에 남기지 않는다.
        if err.is_connect() || err.is_timeout() {
            tracing::warn!(error.kind = %"OllamaNotRunning", "ollama endpoint not reachable");
            return AppError::OllamaNotRunning;
        }
        if err.is_request() {
            return AppError::Internal {
                message: format!("ollama request error: {err}"),
            };
        }
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
