//! 입력 언어 식별.
//!
//! - `SourceLanguage` enum 은 사용자 명시값 또는 detector 결과를 표현한다.
//! - `Auto` variant 는 두 가지 의미를 갖는다:
//!   1. FE 에서 사용자가 "자동 감지" 를 선택했다는 표시.
//!   2. detector 가 한국어/간체/번체 중 어느 것으로도 분류할 수 없을 때의 결과
//!      (모호한 중국어 입력 등). 이때 prompt 는 generic `Chinese` 라벨을 사용한다 (PRD §8.2).

mod detector;

use serde::{Deserialize, Serialize};

pub use detector::{detect, DetectionResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceLanguage {
    Korean,
    ChineseSimplified,
    ChineseTraditional,
    Auto,
}

impl SourceLanguage {
    /// PRD §8.3 prompt template 에 그대로 들어가는 영문 라벨.
    /// `Auto` 는 detector 가 결정을 내리지 못한 경우의 fallback 으로 `"Chinese"` 를 쓴다 (§8.2).
    pub fn prompt_label(self) -> &'static str {
        match self {
            SourceLanguage::Korean => "Korean",
            SourceLanguage::ChineseSimplified => "Simplified Chinese",
            SourceLanguage::ChineseTraditional => "Traditional Chinese",
            SourceLanguage::Auto => "Chinese",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_label_per_variant() {
        assert_eq!(SourceLanguage::Korean.prompt_label(), "Korean");
        assert_eq!(
            SourceLanguage::ChineseSimplified.prompt_label(),
            "Simplified Chinese"
        );
        assert_eq!(
            SourceLanguage::ChineseTraditional.prompt_label(),
            "Traditional Chinese"
        );
        assert_eq!(SourceLanguage::Auto.prompt_label(), "Chinese");
    }

    #[test]
    fn serializes_with_variant_name() {
        let json = serde_json::to_string(&SourceLanguage::Korean).unwrap();
        assert_eq!(json, "\"Korean\"");
        let parsed: SourceLanguage = serde_json::from_str("\"ChineseSimplified\"").unwrap();
        assert_eq!(parsed, SourceLanguage::ChineseSimplified);
        let auto: SourceLanguage = serde_json::from_str("\"Auto\"").unwrap();
        assert_eq!(auto, SourceLanguage::Auto);
    }
}
