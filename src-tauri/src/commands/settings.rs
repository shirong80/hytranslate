//! `get_settings` / `update_settings` (PRD §10, §9.2).
//!
//! 영속화는 `SettingsStore` 가 담당. 커맨드 레이어는 검증 + system transaction 을 거친다.
//! Phase 3 의 모든 OS-mutating settings (hotkey / dock / autostart) 는 transactional 로
//! 적용된다. 한 step 이 실패하면 이미 적용된 step 들이 역순으로 rollback 되고,
//! settings.json 은 변경되지 않는다 — UI 가 "저장됨" 으로 표시하지만 macOS 상태는
//! 변경되지 않는 mismatch 를 방지한다 (코드리뷰 second-pass High 1).

use std::sync::Arc;

use crate::commands::system;
use crate::errors::{AppError, AppResult};
use crate::ollama::is_endpoint_allowed;
use crate::settings::{Settings, SettingsStore};
use crate::shortcuts;

#[tauri::command]
pub async fn get_settings(store: tauri::State<'_, Arc<SettingsStore>>) -> AppResult<Settings> {
    Ok(store.get())
}

#[tauri::command]
pub async fn update_settings<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    store: tauri::State<'_, Arc<SettingsStore>>,
    settings: Settings,
) -> AppResult<Settings> {
    // PRD §12 — non-loopback endpoint 저장 거부.
    if !is_endpoint_allowed(&settings.ollama_endpoint) {
        return Err(AppError::NetworkBlocked);
    }
    // 단축키 형식이 잘못된 경우 system reconcile 전에 빠르게 거부.
    let _ = shortcuts::parser::parse(&settings.global_hotkey)?;

    let prev = store.get();
    let hotkey_change = (prev.global_hotkey != settings.global_hotkey)
        .then(|| (prev.global_hotkey.clone(), settings.global_hotkey.clone()));
    let dock_change = (prev.hide_dock_icon != settings.hide_dock_icon)
        .then_some((prev.hide_dock_icon, settings.hide_dock_icon));
    let autostart_change = (prev.start_at_login != settings.start_at_login)
        .then_some((prev.start_at_login, settings.start_at_login));

    // settings 는 closure 안에서 한 번 소비된다 — Option 으로 감싸 take().
    let mut settings_holder = Some(settings);
    let store_clone = store.inner().clone();

    system::orchestrate_settings_apply(
        hotkey_change,
        dock_change,
        autostart_change,
        |prev, next| shortcuts::try_swap(&app, prev, next),
        |hidden| system::apply_dock_hidden(&app, hidden),
        |enabled| system::apply_autostart(&app, enabled),
        || {
            let s = settings_holder
                .take()
                .ok_or_else(|| AppError::internal("settings consumed twice"))?;
            store_clone.update(s).map(|_| ())
        },
    )?;

    Ok(store.get())
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
        // tauri::State 직접 생성이 불가능하므로 검증 로직 자체를 확인한다.
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
