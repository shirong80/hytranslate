//! `get_settings` / `update_settings` (PRD §10, §9.2).
//!
//! 영속화는 `SettingsStore` 가 담당. 커맨드 레이어는 검증 + diff 기반 system reconcile.
//! Phase 3 부터 단축키 / dock / autostart 변경 시 즉시 system 상태에 반영한다.
//!
//! 단축키 변경은 register-then-rollback transaction (코드리뷰 High 1): syntax 검증 →
//! 시스템 register → settings 디스크 저장 → persist 실패 시 hotkey 도 이전 값으로 복구.
//! unusable hotkey 가 settings.json 에 남는 상태를 만들지 않는다.

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
    // PRD §12 — non-loopback endpoint 저장 거부. 설정 단계에서 막아두면 번역 시점에
    // NetworkBlocked 가 발생하는 일이 없다.
    if !is_endpoint_allowed(&settings.ollama_endpoint) {
        return Err(AppError::NetworkBlocked);
    }
    // 단축키 형식이 잘못된 경우 system reconcile 전에 빠르게 거부.
    let _ = shortcuts::parser::parse(&settings.global_hotkey)?;

    let prev = store.get();
    let hotkey_changed = prev.global_hotkey != settings.global_hotkey;
    let previous_hotkey = prev.global_hotkey.clone();

    // 1) 새 단축키를 먼저 시스템에 등록. 실패 시 settings 디스크 변경 없이 즉시 에러.
    if hotkey_changed {
        shortcuts::try_swap(&app, &previous_hotkey, &settings.global_hotkey)?;
    }

    // 2) settings 디스크 저장. 실패 시 hotkey 를 이전 값으로 rollback 한 뒤 에러를 그대로 전파.
    let saved = match store.update(settings) {
        Ok(s) => s,
        Err(e) => {
            if hotkey_changed {
                // best-effort rollback. rollback 도 실패하면 단축키 비활성 상태로 남지만
                // settings 는 여전히 이전 값이므로 재시작 시 정상 재등록된다.
                let _ = shortcuts::try_rollback(&app, &previous_hotkey);
            }
            return Err(e);
        }
    };

    // 3) Dock / autostart 변경 적용. 실패는 로그로만 — settings 는 이미 일관 상태이고,
    // 재시작 시 setup() 가 동일 reconcile 을 다시 시도한다.
    if prev.hide_dock_icon != saved.hide_dock_icon {
        if let Err(err) = system::apply_dock_hidden(&app, saved.hide_dock_icon) {
            tracing::warn!(error = ?err, "dock activation policy update failed");
        }
    }
    if prev.start_at_login != saved.start_at_login {
        if let Err(err) = system::apply_autostart(&app, saved.start_at_login) {
            tracing::warn!(error = ?err, "autostart update failed");
        }
    }

    Ok(saved)
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
