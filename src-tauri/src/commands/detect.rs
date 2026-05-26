//! Phase 2: `detect_language` 명령 (PRD §10.3).
//!
//! 짧고 동기적이지만 `#[tauri::command]` 규약상 `async fn` 유지.

use serde::Deserialize;

use crate::errors::AppResult;
use crate::language::{detect, DetectionResult};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectRequest {
    pub text: String,
}

#[tauri::command]
pub async fn detect_language(request: DetectRequest) -> AppResult<DetectionResult> {
    Ok(detect(&request.text))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::language::SourceLanguage;

    #[tokio::test]
    async fn detect_language_returns_korean_for_hangul() {
        let r = detect_language(DetectRequest {
            text: "안녕하세요".to_string(),
        })
        .await
        .unwrap();
        assert_eq!(r.language, SourceLanguage::Korean);
    }
}
