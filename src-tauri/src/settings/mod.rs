//! 사용자 Settings 영속화 (PRD §9.2).
//!
//! Phase 2 단계에서는 단순 JSON 파일에 직렬화한다. Phase 4 에서 SQLite 도입 시
//! 마이그레이션 한 번으로 옮긴다.

mod store;

use serde::{Deserialize, Serialize};

pub use store::SettingsStore;

/// PRD §9.2 Settings. 필드 순서는 PRD 표 순서.
///
/// 일부 필드는 Phase 2 에서 UI 와이어링이 없지만 (이력/단축키/Dock 등) 기본값으로
/// 영속화한다. Phase 3+ 에서 UI 가 붙는 시점에 그대로 사용한다.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub global_hotkey: String,
    pub active_model: String,
    pub auto_copy_after_translation: bool,
    pub save_history: bool,
    pub start_at_login: bool,
    pub hide_dock_icon: bool,
    pub ollama_endpoint: String,
    pub theme: Theme,
    pub onboarding_completed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Theme {
    System,
    Light,
    Dark,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            global_hotkey: "Cmd+Shift+T".to_string(),
            active_model: crate::ollama::DEFAULT_MODEL.to_string(),
            auto_copy_after_translation: false,
            save_history: true,
            start_at_login: false,
            hide_dock_icon: false,
            ollama_endpoint: "http://localhost:11434".to_string(),
            theme: Theme::System,
            onboarding_completed: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_prd_section_9_2() {
        let s = Settings::default();
        assert_eq!(s.global_hotkey, "Cmd+Shift+T");
        assert_eq!(s.active_model, crate::ollama::DEFAULT_MODEL);
        assert!(!s.auto_copy_after_translation);
        assert!(s.save_history);
        assert!(!s.start_at_login);
        assert!(!s.hide_dock_icon);
        assert_eq!(s.ollama_endpoint, "http://localhost:11434");
        assert_eq!(s.theme, Theme::System);
        assert!(!s.onboarding_completed);
    }

    #[test]
    fn serializes_with_camel_case_keys() {
        let s = Settings::default();
        let json = serde_json::to_string(&s).unwrap();
        assert!(json.contains("\"globalHotkey\":\"Cmd+Shift+T\""));
        assert!(json.contains("\"activeModel\":"));
        assert!(json.contains("\"autoCopyAfterTranslation\":false"));
        assert!(json.contains("\"saveHistory\":true"));
        assert!(json.contains("\"ollamaEndpoint\":\"http://localhost:11434\""));
        assert!(json.contains("\"theme\":\"System\""));
    }
}
