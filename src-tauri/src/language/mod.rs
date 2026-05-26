//! 입력 언어 식별. Phase 1 은 enum + prompt label 만, Phase 2 에서 자동 감지기 도입.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceLanguage {
    Korean,
    ChineseSimplified,
    ChineseTraditional,
}

impl SourceLanguage {
    /// PRD §8.3 prompt template 에 그대로 들어가는 영문 라벨.
    pub fn prompt_label(self) -> &'static str {
        match self {
            SourceLanguage::Korean => "Korean",
            SourceLanguage::ChineseSimplified => "Simplified Chinese",
            SourceLanguage::ChineseTraditional => "Traditional Chinese",
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
    }

    #[test]
    fn serializes_with_variant_name() {
        let json = serde_json::to_string(&SourceLanguage::Korean).unwrap();
        assert_eq!(json, "\"Korean\"");
        let parsed: SourceLanguage = serde_json::from_str("\"ChineseSimplified\"").unwrap();
        assert_eq!(parsed, SourceLanguage::ChineseSimplified);
    }
}
