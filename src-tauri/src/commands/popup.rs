//! Floating popup 윈도우 제어.
//!
//! - `show_popup`: 메인 디스플레이 중앙 배치 후 표시 + 포커스.
//! - `hide_popup`: 윈도우 숨김. focus 추적 없이 항상 idempotent.
//! - `toggle_popup`: 이미 보이면 숨기고, 아니면 show_popup.

use tauri::{Emitter, LogicalPosition, Manager, Runtime, WebviewWindow};

use crate::errors::{AppError, AppResult};
use crate::events::{POPUP_CLOSED, POPUP_OPENED};

const POPUP_LABEL: &str = "popup";

pub fn get_popup<R: Runtime>(app: &tauri::AppHandle<R>) -> AppResult<WebviewWindow<R>> {
    app.get_webview_window(POPUP_LABEL)
        .ok_or_else(|| AppError::internal("popup window missing"))
}

fn center_on_primary<R: Runtime>(window: &WebviewWindow<R>) -> AppResult<()> {
    // Tauri 의 `center()` 가 단순하면서 디스플레이 안전. 활성 화면 추적 필요 시
    // monitor_from_point 로 교체 — Phase 3 에서는 단순화한다.
    window.center().map_err(AppError::internal)?;
    Ok(())
}

pub fn show<R: Runtime>(app: &tauri::AppHandle<R>) -> AppResult<()> {
    let window = get_popup(app)?;
    if window.is_visible().map_err(AppError::internal)? {
        // 이미 보이는 경우 포커스만 가져온다 — 사용자가 다른 앱에서 단축키를
        // 두 번 누른 상황.
        window.set_focus().map_err(AppError::internal)?;
        return Ok(());
    }
    center_on_primary(&window)?;
    window.show().map_err(AppError::internal)?;
    window.set_focus().map_err(AppError::internal)?;
    let _ = app.emit(POPUP_OPENED, ());
    Ok(())
}

pub fn hide<R: Runtime>(app: &tauri::AppHandle<R>) -> AppResult<()> {
    let window = get_popup(app)?;
    if !window.is_visible().map_err(AppError::internal)? {
        return Ok(());
    }
    window.hide().map_err(AppError::internal)?;
    let _ = app.emit(POPUP_CLOSED, ());
    Ok(())
}

pub fn toggle<R: Runtime>(app: &tauri::AppHandle<R>) -> AppResult<()> {
    let window = get_popup(app)?;
    if window.is_visible().map_err(AppError::internal)? {
        hide(app)
    } else {
        show(app)
    }
}

#[tauri::command]
pub async fn show_popup<R: Runtime>(app: tauri::AppHandle<R>) -> AppResult<()> {
    show(&app)
}

#[tauri::command]
pub async fn hide_popup<R: Runtime>(app: tauri::AppHandle<R>) -> AppResult<()> {
    hide(&app)
}

#[tauri::command]
pub async fn toggle_popup<R: Runtime>(app: tauri::AppHandle<R>) -> AppResult<()> {
    toggle(&app)
}

/// FE 의 popup 윈도우가 마운트되기 전에 호출해도 안전한 placement helper.
/// 단위 테스트용 — 화면 크기 모의가 어렵기에 logical-coord 계산을 분리.
#[allow(dead_code)]
pub fn center_within(
    viewport_w: f64,
    viewport_h: f64,
    popup_w: f64,
    popup_h: f64,
) -> LogicalPosition<f64> {
    LogicalPosition::new(
        ((viewport_w - popup_w) / 2.0).max(0.0),
        ((viewport_h - popup_h) / 2.0).max(0.0),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn center_within_centers_popup() {
        let pos = center_within(1440.0, 900.0, 480.0, 360.0);
        assert_eq!(pos.x, 480.0);
        assert_eq!(pos.y, 270.0);
    }

    #[test]
    fn center_within_clamps_when_popup_larger_than_screen() {
        let pos = center_within(300.0, 200.0, 480.0, 360.0);
        assert_eq!(pos.x, 0.0);
        assert_eq!(pos.y, 0.0);
    }
}
