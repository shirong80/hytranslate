//! 입력 텍스트의 source language 분류 (PRD §8.2).
//!
//! 알고리즘:
//! 1. Hangul Unicode block 에 속하는 문자가 하나라도 있으면 Korean.
//!    (PRD §8.2: "한글만 포함한 입력은 Korean 으로 감지" + Korean-Hanja 혼용도 Korean.)
//! 2. CJK Unified Ideograph 가 있으면 frequency table 비교로 간체/번체 결정.
//! 3. simplified marker 와 traditional marker 가 같거나 모두 없으면 `Auto`.
//! 4. CJK / Hangul 어느 것도 없으면 `Auto` (모델 prompt 는 "Chinese" fallback).
//!
//! Frequency table 은 모든 케이스를 완벽히 커버하지 않는다. 사용자 override 가 safety net.

use std::collections::HashSet;
use std::sync::OnceLock;

use serde::Serialize;

use crate::language::SourceLanguage;

/// 간체 중국어 전용 글자. 번체에는 동일 의미의 다른 글자가 있다.
/// 모든 글자를 망라하지 않으며, PRD 의 명확한 sample 을 정확히 분류하는 데 우선순위.
const SIMPLIFIED_ONLY: &str = "这个们国学时来说现经还发当进实让应见觉战单简关间问门网写处计离该选论队区务万与业东严丽举习乱争测试题张长马鸟龙风飞凤鱼车听语书钟银钢铁页头脸开亲讲华贵庆动办";

/// 번체 중국어 전용 글자.
const TRADITIONAL_ONLY: &str = "這個們國學時來說現經還發當進實讓應見覺戰單簡關間問門網寫處計離該選論隊區務萬與業東嚴麗舉習亂爭測試題張長馬鳥龍風飛鳳魚車聽語書鐘銀鋼鐵頁頭臉開親講華貴慶動辦";

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectionResult {
    pub language: SourceLanguage,
    /// 0.0..=1.0 — 분류 신뢰도. Auto 인 경우 0.0.
    pub confidence: f32,
}

pub fn detect(text: &str) -> DetectionResult {
    let mut hangul: usize = 0;
    let mut cjk: usize = 0;
    let mut simplified_hits: usize = 0;
    let mut traditional_hits: usize = 0;

    let simplified = simplified_set();
    let traditional = traditional_set();

    for ch in text.chars() {
        if is_hangul(ch) {
            hangul += 1;
        } else if is_cjk_unified(ch) {
            cjk += 1;
            if simplified.contains(&ch) {
                simplified_hits += 1;
            } else if traditional.contains(&ch) {
                traditional_hits += 1;
            }
        }
    }

    let lang_total = hangul + cjk;
    if lang_total == 0 {
        // CJK/한글 글자가 하나도 없는 입력. detector 는 `Auto` 로 보고하고
        // 호출자가 fallback 정책을 정한다.
        return DetectionResult {
            language: SourceLanguage::Auto,
            confidence: 0.0,
        };
    }

    if hangul > 0 {
        // Korean text 는 Hangul 을 핵심 표기로 쓴다. 혼용 Hanja 는 Chinese 가 아니다.
        let confidence = hangul as f32 / lang_total as f32;
        return DetectionResult {
            language: SourceLanguage::Korean,
            confidence,
        };
    }

    // 여기부터는 CJK 만 존재. simplified / traditional marker 비교.
    let marker_total = simplified_hits + traditional_hits;
    if marker_total == 0 {
        // CJK 는 있으나 결정적 marker 가 없음 → 모호한 Chinese.
        return DetectionResult {
            language: SourceLanguage::Auto,
            confidence: 0.0,
        };
    }

    if simplified_hits > traditional_hits {
        return DetectionResult {
            language: SourceLanguage::ChineseSimplified,
            confidence: simplified_hits as f32 / marker_total as f32,
        };
    }
    if traditional_hits > simplified_hits {
        return DetectionResult {
            language: SourceLanguage::ChineseTraditional,
            confidence: traditional_hits as f32 / marker_total as f32,
        };
    }
    DetectionResult {
        language: SourceLanguage::Auto,
        confidence: 0.0,
    }
}

fn simplified_set() -> &'static HashSet<char> {
    static S: OnceLock<HashSet<char>> = OnceLock::new();
    S.get_or_init(|| SIMPLIFIED_ONLY.chars().collect())
}

fn traditional_set() -> &'static HashSet<char> {
    static T: OnceLock<HashSet<char>> = OnceLock::new();
    T.get_or_init(|| TRADITIONAL_ONLY.chars().collect())
}

fn is_hangul(ch: char) -> bool {
    matches!(
        ch as u32,
        0xAC00..=0xD7A3       // Hangul Syllables
        | 0x1100..=0x11FF     // Hangul Jamo
        | 0x3130..=0x318F     // Hangul Compatibility Jamo
        | 0xA960..=0xA97F     // Hangul Jamo Extended-A
        | 0xD7B0..=0xD7FF     // Hangul Jamo Extended-B
    )
}

fn is_cjk_unified(ch: char) -> bool {
    matches!(
        ch as u32,
        0x3400..=0x4DBF        // CJK Unified Ideographs Extension A
        | 0x4E00..=0x9FFF      // CJK Unified Ideographs
        | 0xF900..=0xFAFF      // CJK Compatibility Ideographs
        | 0x20000..=0x2A6DF    // CJK Unified Ideographs Extension B
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_table_collision_between_simplified_and_traditional() {
        let s = simplified_set();
        let t = traditional_set();
        for ch in s.iter() {
            assert!(
                !t.contains(ch),
                "char {ch:?} appears in both simplified and traditional tables"
            );
        }
    }

    #[test]
    fn pure_korean_detected_as_korean() {
        let r = detect("안녕하세요. 오늘 회의는 오후 3시에 시작합니다.");
        assert_eq!(r.language, SourceLanguage::Korean);
        assert!(r.confidence > 0.9, "confidence={}", r.confidence);
    }

    #[test]
    fn korean_with_hanja_still_detected_as_korean() {
        let r = detect("漢字가 포함된 문장이라도 한국어로 분류된다.");
        assert_eq!(r.language, SourceLanguage::Korean);
    }

    #[test]
    fn simplified_chinese_detected() {
        let r = detect("你好世界,这是一个简单的测试。请让我们一起验证。");
        assert_eq!(r.language, SourceLanguage::ChineseSimplified);
        assert!(r.confidence > 0.5);
    }

    #[test]
    fn traditional_chinese_detected() {
        let r = detect("你好世界,這是一個簡單的測試。請讓我們一起驗證。");
        assert_eq!(r.language, SourceLanguage::ChineseTraditional);
        assert!(r.confidence > 0.5);
    }

    #[test]
    fn ambiguous_cjk_without_markers_is_auto() {
        // 간체/번체 공통 글자만 사용. detector 는 결정 불가 → Auto.
        let r = detect("人山人海");
        assert_eq!(r.language, SourceLanguage::Auto);
    }

    #[test]
    fn empty_input_is_auto_with_zero_confidence() {
        let r = detect("");
        assert_eq!(r.language, SourceLanguage::Auto);
        assert_eq!(r.confidence, 0.0);
    }

    #[test]
    fn non_cjk_input_is_auto() {
        // 영문/숫자만 입력 — detector 가 분류 불가.
        let r = detect("Hello world 123");
        assert_eq!(r.language, SourceLanguage::Auto);
    }

    #[test]
    fn serializes_with_camel_case_fields() {
        let r = DetectionResult {
            language: SourceLanguage::Korean,
            confidence: 0.95,
        };
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains("\"language\":\"Korean\""));
        assert!(json.contains("\"confidence\":0.95"));
    }
}
