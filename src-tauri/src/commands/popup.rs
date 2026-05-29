//! Floating popup 윈도우 제어.
//!
//! - `show_popup`: 메인 디스플레이 중앙 배치 후 표시 + 포커스.
//! - `hide_popup`: 윈도우 숨김. focus 추적 없이 항상 idempotent.
//! - `toggle_popup`: 이미 보이면 숨기고, 아니면 show_popup.

use tauri::{Emitter, LogicalPosition, Manager, PhysicalPosition, Runtime, WebviewWindow};

use crate::errors::{AppError, AppResult};
use crate::events::{POPUP_CLOSED, POPUP_OPENED};

const POPUP_LABEL: &str = "popup";

pub fn get_popup<R: Runtime>(app: &tauri::AppHandle<R>) -> AppResult<WebviewWindow<R>> {
    app.get_webview_window(POPUP_LABEL)
        .ok_or_else(|| AppError::internal("popup window missing"))
}

/// popup 을 다른 앱의 네이티브 전체화면 Space 위에도 띄우기 위한 NSWindow 속성 설정.
///
/// tao 의 `set_visible_on_all_workspaces(true)` 는 `CanJoinAllSpaces` 만 켜고 정작
/// 전체화면 overlay 에 필요한 `FullScreenAuxiliary` 는 켜지 않으므로(Tauri #11488),
/// `ns_window()` 로 받은 NSWindow 에 collection behavior 와 window level 을 직접 설정한다.
///
/// collection behavior 는 상호배타 그룹(Spaces: CanJoinAllSpaces↔MoveToActiveSpace /
/// FullScreen: Primary↔Auxiliary↔None)을 가지므로, 단순 OR 은 같은 그룹 비트를 동시에
/// 켜 동작 미정의를 부른다. 각 그룹의 다른 멤버를 먼저 제거한 뒤 원하는 비트를 켜
/// 무관한 기존 비트는 보존한다.
#[cfg(target_os = "macos")]
fn apply_fullscreen_overlay<R: Runtime>(window: &WebviewWindow<R>) -> AppResult<()> {
    use objc2_app_kit::{NSWindow, NSWindowCollectionBehavior};

    // NSStatusWindowLevel — 메뉴 막대 아래, 전체화면 콘텐츠 위.
    const NS_STATUS_WINDOW_LEVEL: isize = 25;

    let ptr = window.ns_window().map_err(AppError::internal)? as *const NSWindow;
    // SAFETY: tao 가 WebviewWindow 수명 동안 NSWindow 를 살려둔다. 메인 스레드(setup /
    // 단축키 핸들러)에서만 호출된다.
    let ns_window: &NSWindow = unsafe { &*ptr };

    let mut behavior = ns_window.collectionBehavior();
    behavior &= !NSWindowCollectionBehavior::MoveToActiveSpace;
    behavior |= NSWindowCollectionBehavior::CanJoinAllSpaces;
    behavior &= !(NSWindowCollectionBehavior::FullScreenPrimary
        | NSWindowCollectionBehavior::FullScreenNone);
    behavior |= NSWindowCollectionBehavior::FullScreenAuxiliary;
    ns_window.setCollectionBehavior(behavior);

    ns_window.setLevel(NS_STATUS_WINDOW_LEVEL);
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn apply_fullscreen_overlay<R: Runtime>(_window: &WebviewWindow<R>) -> AppResult<()> {
    Ok(())
}

/// Major 4 — 마우스 커서가 있는 monitor 중심에 popup 을 배치. 둘 이상의 디스플레이가
/// 있을 때 사용자가 보던 화면이 활성 monitor 로 우선시된다. cursor 추출에 실패하면
/// primary 의 중심으로 fallback.
fn place_on_active_monitor<R: Runtime>(window: &WebviewWindow<R>) -> AppResult<()> {
    let monitor = window
        .cursor_position()
        .ok()
        .and_then(|pos| window.monitor_from_point(pos.x, pos.y).ok().flatten())
        .or_else(|| window.primary_monitor().ok().flatten());
    let Some(monitor) = monitor else {
        // monitor API 자체가 실패 — Tauri 기본 center 로 fallback.
        return window.center().map_err(AppError::internal);
    };
    // monitor 의 좌표는 이미 physical px — scale_factor 를 곱하지 않는다.
    let mon_pos = monitor.position();
    let mon_size = monitor.size();

    let outer = window.outer_size().map_err(AppError::internal)?;
    let cx = mon_pos.x as f64 + (mon_size.width as f64 - outer.width as f64) / 2.0;
    let cy = mon_pos.y as f64 + (mon_size.height as f64 - outer.height as f64) / 2.0;
    window
        .set_position(PhysicalPosition::new(cx, cy))
        .map_err(AppError::internal)?;
    Ok(())
}

pub fn show<R: Runtime>(app: &tauri::AppHandle<R>) -> AppResult<()> {
    let window = get_popup(app)?;
    if window.is_visible().map_err(AppError::internal)? {
        // 이미 보이는 경우 포커스만 가져온다 — 사용자가 다른 앱에서 단축키를
        // 두 번 누른 상황.
        window.set_focus().map_err(AppError::internal)?;
        let _ = app.emit(POPUP_OPENED, ());
        return Ok(());
    }
    // cold-show 마다 overlay 속성을 재적용한다. release 빌드에서 level/behavior 가
    // 리셋되는 선례가 있어(Tauri #5566) 매 표시 시점에 보장한다.
    apply_fullscreen_overlay(&window)?;
    place_on_active_monitor(&window)?;
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

/// Major 4 — `place_on_active_monitor` 의 좌표 산출만 분리해 단위 테스트.
/// monitor 의 좌상단 좌표 + 크기 → popup 좌상단 좌표.
#[allow(dead_code)]
pub fn center_on_monitor(
    mon_x: f64,
    mon_y: f64,
    mon_w: f64,
    mon_h: f64,
    popup_w: f64,
    popup_h: f64,
) -> (f64, f64) {
    let x = mon_x + (mon_w - popup_w) / 2.0;
    let y = mon_y + (mon_h - popup_h) / 2.0;
    (x, y)
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

    /// Major 4 — secondary monitor 의 origin offset 이 반영돼야 한다.
    #[test]
    fn center_on_monitor_respects_origin_offset() {
        // 1440x900 primary 우측에 1920x1080 sub-monitor 가 붙은 케이스.
        let (x, y) = center_on_monitor(1440.0, 0.0, 1920.0, 1080.0, 480.0, 360.0);
        assert_eq!(x, 1440.0 + (1920.0 - 480.0) / 2.0);
        assert_eq!(y, (1080.0 - 360.0) / 2.0);
    }
}
