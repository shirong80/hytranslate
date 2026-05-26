//! Prompt builder. PRD §8.3 fixed template. target=English 하드코딩.

use crate::language::SourceLanguage;

/// PRD §8.3 의 prompt template. `source_text` 는 절대 정규화하지 않는다.
pub fn build_prompt(source_language: SourceLanguage, source_text: &str) -> String {
    format!(
        "Translate the following segment from {} into English.\n\
         Output only the translation. Do not add explanations, preambles, quotation marks, or markdown.\n\
         \n\
         {}",
        source_language.prompt_label(),
        source_text
    )
}

/// `num_predict` 동적 산정. PRD §8.3 기본 512, 입력 길이에 비례하되 cap.
pub fn num_predict_for(source_text: &str) -> u32 {
    let len = source_text.chars().count();
    let scaled = (len as u32).saturating_mul(2).max(512);
    scaled.min(4096)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn korean_prompt_contains_source_label_and_text() {
        let text = "안녕하세요. 오늘 회의는 오후 3시에 시작합니다.";
        let prompt = build_prompt(SourceLanguage::Korean, text);
        assert_eq!(
            prompt,
            "Translate the following segment from Korean into English.\n\
             Output only the translation. Do not add explanations, preambles, quotation marks, or markdown.\n\
             \n\
             안녕하세요. 오늘 회의는 오후 3시에 시작합니다."
        );
    }

    #[test]
    fn simplified_chinese_uses_simplified_label() {
        let prompt = build_prompt(SourceLanguage::ChineseSimplified, "你好世界");
        assert!(prompt
            .starts_with("Translate the following segment from Simplified Chinese into English."));
        assert!(prompt.ends_with("你好世界"));
    }

    #[test]
    fn traditional_chinese_uses_traditional_label() {
        let prompt = build_prompt(SourceLanguage::ChineseTraditional, "你好世界");
        assert!(prompt
            .starts_with("Translate the following segment from Traditional Chinese into English."));
    }

    #[test]
    fn auto_falls_back_to_generic_chinese_label() {
        // PRD §8.2 — detector 가 결정을 내리지 못해 Auto 가 prompt 까지 흘러올 때
        // generic `Chinese` 라벨로 호출한다.
        let prompt = build_prompt(SourceLanguage::Auto, "人山人海");
        assert!(prompt.starts_with("Translate the following segment from Chinese into English."));
    }

    #[test]
    fn does_not_strip_or_normalize_source_text() {
        let messy = "  spaced\n\nmulti\tline  ";
        let prompt = build_prompt(SourceLanguage::Korean, messy);
        assert!(prompt.ends_with(messy));
    }

    #[test]
    fn num_predict_scales_with_input_and_caps() {
        assert_eq!(num_predict_for("hi"), 512);
        assert_eq!(num_predict_for(&"가".repeat(300)), 600);
        assert_eq!(num_predict_for(&"가".repeat(5000)), 4096);
    }
}
