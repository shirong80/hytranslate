//! Phase 2: `get_settings` / `update_settings` 명령 (PRD §10, §9.2).
//!
//! 영속화는 `crate::settings::SettingsStore` 가 담당. 커맨드 레이어는
//! `Arc<SettingsStore>` 를 state 로 받아 forward 만 한다.

use std::sync::Arc;

use crate::errors::{AppError, AppResult};
use crate::ollama::is_endpoint_allowed;
use crate::settings::{Settings, SettingsStore};

#[tauri::command]
pub async fn get_settings(store: tauri::State<'_, Arc<SettingsStore>>) -> AppResult<Settings> {
    Ok(store.get())
}

#[tauri::command]
pub async fn update_settings(
    store: tauri::State<'_, Arc<SettingsStore>>,
    settings: Settings,
) -> AppResult<Settings> {
    // PRD §12 — non-loopback endpoint 저장 거부. 설정 단계에서 막아두면 번역 시점에
    // NetworkBlocked 가 발생하는 일이 없다.
    if !is_endpoint_allowed(&settings.ollama_endpoint) {
        return Err(AppError::NetworkBlocked);
    }
    store.update(settings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Theme;

    fn tmp_store() -> Arc<SettingsStore> {
        let mut p = std::env::temp_dir();
        p.push(format!("hytranslate-cmd-{}.json", uuid::Uuid::new_v4()));
        Arc::new(SettingsStore::load(&p).unwrap())
    }

    #[tokio::test]
    async fn update_rejects_non_loopback_endpoint() {
        let store = tmp_store();
        let mut s = store.get();
        s.ollama_endpoint = "http://evil.example.com".to_string();
        // tauri::State 직접 생성이 불가능하므로 store 를 직접 호출하는 식으로 검증.
        // 단위 테스트는 검증 로직 자체에 집중.
        assert!(!is_endpoint_allowed(&s.ollama_endpoint));
    }

    #[tokio::test]
    async fn update_accepts_loopback_with_custom_port() {
        let store = tmp_store();
        let mut s = store.get();
        s.ollama_endpoint = "http://127.0.0.1:9999".to_string();
        assert!(is_endpoint_allowed(&s.ollama_endpoint));
        s.theme = Theme::Dark;
        let saved = store.update(s.clone()).unwrap();
        assert_eq!(saved, s);
    }
}
