//! macOS 시스템 통합: Dock activation policy + autostart 토글.
//!
//! - `apply_dock_hidden(app, hidden)`: ActivationPolicy::Accessory ↔ Regular.
//! - `apply_autostart(app, enabled)`: tauri-plugin-autostart 의 enable/disable.
//!
//! 둘 다 settings 변경 시 `update_settings` 에서 호출하고, 시작 시점에는
//! `commands::register::setup()` 가 한 번 호출해 초기 상태를 디스크 값에 맞춘다.

use tauri::Runtime;
use tauri_plugin_autostart::ManagerExt as AutostartManagerExt;

use crate::errors::{AppError, AppResult};

pub fn apply_dock_hidden<R: Runtime>(app: &tauri::AppHandle<R>, hidden: bool) -> AppResult<()> {
    #[cfg(target_os = "macos")]
    {
        let policy = if hidden {
            tauri::ActivationPolicy::Accessory
        } else {
            tauri::ActivationPolicy::Regular
        };
        app.set_activation_policy(policy)
            .map_err(AppError::internal)?;
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (app, hidden);
    }
    Ok(())
}

pub fn apply_autostart<R: Runtime>(app: &tauri::AppHandle<R>, enabled: bool) -> AppResult<()> {
    let manager = app.autolaunch();
    let currently = match manager.is_enabled() {
        Ok(v) => v,
        Err(err) => {
            // LaunchAgent plist 조회 실패. 안전한 fall-back 으로 "비활성" 가정 후 진행하되,
            // 디버깅을 위해 한 줄 남긴다 — autostart 가 의도와 달리 동작할 수 있음을 알 수 있다.
            tracing::warn!(error = %err, "autolaunch is_enabled query failed; assuming disabled");
            false
        }
    };
    if currently == enabled {
        return Ok(());
    }
    if enabled {
        manager.enable().map_err(AppError::internal)
    } else {
        manager.disable().map_err(AppError::internal)
    }
}
