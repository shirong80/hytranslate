//! 전역 단축키 등록. `tauri-plugin-global-shortcut` 위에 얇은 도메인 facade.
//!
//! - `install`: 플러그인 빌드 (handler 캡처) + 초기 단축키 등록.
//! - `reconcile`: settings 변경 시 호출 — 이전 단축키를 모두 해제 후 새 단축키 등록.
//! - 핸들러는 plugin 단일 등록. 어떤 단축키가 눌려도 popup toggle 한다.
//!
//! macOS 손쉬운 사용 권한 부재 시 register 자체는 실패하지 않는다 (handler 가 안 불릴 뿐).
//! UI 차원의 권한 안내는 settings 패널의 도움말 + Onboarding (Phase 5) 에서 다룬다.

pub mod parser;

use tauri::Runtime;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use crate::commands::popup;
use crate::errors::{AppError, AppResult};

/// 플러그인 등록 + 초기 단축키 1회 register. `setup()` 안에서 한 번 호출.
pub fn install<R: Runtime>(app: &tauri::App<R>, initial_hotkey: &str) -> AppResult<()> {
    let handle = app.handle().clone();
    app.handle()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |_app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        if let Err(e) = popup::toggle(&handle) {
                            tracing::warn!(error = ?e, "popup toggle from shortcut failed");
                        }
                    }
                })
                .build(),
        )
        .map_err(AppError::internal)?;
    let handle_for_register = app.handle().clone();
    register(&handle_for_register, initial_hotkey)
}

/// 단일 단축키 register. 이전 등록은 호출자가 unregister 한 뒤 호출하라 — 일반 경로는
/// `reconcile` 을 사용한다. 파싱 실패는 `InvalidShortcut`, 등록 실패는 `Internal`.
pub fn register<R: Runtime>(app: &tauri::AppHandle<R>, accelerator: &str) -> AppResult<()> {
    let shortcut = parser::parse(accelerator)?;
    app.global_shortcut()
        .register(shortcut)
        .map_err(AppError::internal)
}

/// settings 변경 시 호출. 모든 단축키를 해제하고 새 단축키만 등록한다.
/// 새 단축키 파싱이 실패하면 이전 단축키는 그대로 유지 — 사용자가 입력 실수로 인해
/// "단축키 없음" 상태가 되지 않도록 한다.
pub fn reconcile<R: Runtime>(app: &tauri::AppHandle<R>, accelerator: &str) -> AppResult<()> {
    let shortcut = parser::parse(accelerator)?;
    let gs = app.global_shortcut();
    gs.unregister_all().map_err(AppError::internal)?;
    gs.register(shortcut).map_err(AppError::internal)
}
