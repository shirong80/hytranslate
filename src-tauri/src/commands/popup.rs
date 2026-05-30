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

/// popup 의 NSWindow 에 전체화면 overlay 속성(collection behavior + window level)을 건다.
///
/// tao 의 `set_visible_on_all_workspaces(true)` 는 `CanJoinAllSpaces` 만 켜고 정작
/// 전체화면 overlay 에 필요한 `FullScreenAuxiliary` 는 켜지 않으므로(Tauri #11488) 직접
/// 설정한다. 멱등이라 cold-show 시 그리고 order-front 직후(#5566 release reset 대비)
/// 반복 호출해도 안전하다.
///
/// collection behavior 는 상호배타 그룹(Spaces: CanJoinAllSpaces↔MoveToActiveSpace /
/// FullScreen: Primary↔Auxiliary↔None)을 가지므로, 단순 OR 은 같은 그룹 비트를 동시에
/// 켜 동작 미정의를 부른다. 각 그룹의 다른 멤버를 먼저 제거한 뒤 원하는 비트를 켜
/// 무관한 기존 비트는 보존한다.
#[cfg(target_os = "macos")]
fn apply_fullscreen_overlay(ns_window: &objc2_app_kit::NSWindow) {
    use objc2_app_kit::NSWindowCollectionBehavior;

    // NSStatusWindowLevel — 메뉴 막대 아래, 전체화면 콘텐츠 위.
    const NS_STATUS_WINDOW_LEVEL: isize = 25;

    let mut behavior = ns_window.collectionBehavior();
    behavior &= !NSWindowCollectionBehavior::MoveToActiveSpace;
    behavior |= NSWindowCollectionBehavior::CanJoinAllSpaces;
    behavior &= !(NSWindowCollectionBehavior::FullScreenPrimary
        | NSWindowCollectionBehavior::FullScreenNone);
    behavior |= NSWindowCollectionBehavior::FullScreenAuxiliary;
    ns_window.setCollectionBehavior(behavior);

    ns_window.setLevel(NS_STATUS_WINDOW_LEVEL);
}

/// 커서가 있는 monitor 중심의 popup 좌상단을 `setFrameTopLeftPoint` 용 Cocoa 좌표로
/// 환산한다. 커서 추출 실패 시 primary 로 fallback하고, monitor API 자체가 실패하면
/// `None`(호출부가 Tauri 기본 center 로 처리).
#[cfg(target_os = "macos")]
fn cocoa_placement<R: Runtime>(window: &WebviewWindow<R>) -> AppResult<Option<(f64, f64)>> {
    let active = window
        .cursor_position()
        .ok()
        .and_then(|pos| window.monitor_from_point(pos.x, pos.y).ok().flatten())
        .or_else(|| window.primary_monitor().ok().flatten());
    let (Some(monitor), Some(primary)) = (
        active,
        window.primary_monitor().map_err(AppError::internal)?,
    ) else {
        return Ok(None);
    };

    // monitor 좌표는 physical px. center_on_monitor 로 popup 좌상단(physical)을 구한 뒤
    // tao 와 동일한 변환으로 Cocoa points 좌표를 만든다.
    let mon_pos = monitor.position();
    let mon_size = monitor.size();
    let outer = window.outer_size().map_err(AppError::internal)?;
    let (cx, cy) = center_on_monitor(
        mon_pos.x as f64,
        mon_pos.y as f64,
        mon_size.width as f64,
        mon_size.height as f64,
        outer.width as f64,
        outer.height as f64,
    );
    let scale = window.scale_factor().map_err(AppError::internal)?;
    let primary_points_high = primary.size().height as f64 / primary.scale_factor();
    Ok(Some(physical_top_left_to_cocoa_point(
        cx,
        cy,
        scale,
        primary_points_high,
    )))
}

/// 새로 띄우는 경로(cold-show).
///
/// macOS: tao 의 `set_position` 은 `exec_async` 라 메인스레드에서 동기 실행되는
/// `show()`(= `make_key_and_order_front_sync`)보다 늦게 적용된다. 그래서 order-front
/// 시점의 프레임이 stale 이고, 듀얼모니터+전체화면(커서가 전체화면 모니터)에서 popup 이
/// 엉뚱한 Space 로 떠 보이지 않는다. 이를 피하려 ns_window 에 위치를 **먼저 동기로** 박은
/// 뒤 order-front 한다.
///
/// NSWindow 조작은 AppKit 메인스레드 전용이다. 이 경로는 단축키 핸들러(메인스레드)뿐
/// 아니라 `show_popup`/`toggle_popup` 커맨드(웹뷰 invoke → async 런타임 워커 스레드)에서도
/// 도달하므로 네이티브 시퀀스를 `run_on_main_thread` 로 메인스레드에 디스패치한다. 한
/// 클로저 안에서 순서대로 실행되니 race 제거 효과는 유지된다. 표시가 끝난 뒤에야
/// `POPUP_OPENED` 를 emit 해야 FE 의 textarea focus 가 key window 위에서 동작한다.
#[cfg(target_os = "macos")]
fn cold_show<R: Runtime>(window: &WebviewWindow<R>) -> AppResult<()> {
    let win = window.clone();
    window
        .run_on_main_thread(move || match cold_show_on_main(&win) {
            Ok(()) => {
                let _ = win.emit(POPUP_OPENED, ());
            }
            Err(e) => tracing::warn!(error = ?e, "popup cold-show failed on main thread"),
        })
        .map_err(AppError::internal)
}

/// `cold_show` 가 메인스레드에서 실행하는 실제 네이티브 시퀀스.
/// `set_focus`(= makeKeyAndOrderFront + activateIgnoringOtherApps)는 전체화면에서 Space
/// 전환을 유발하므로 쓰지 않고 `makeKeyAndOrderFront` 까지만 한다.
#[cfg(target_os = "macos")]
fn cold_show_on_main<R: Runtime>(window: &WebviewWindow<R>) -> AppResult<()> {
    use objc2_app_kit::NSWindow;
    use objc2_foundation::NSPoint;

    let ptr = window.ns_window().map_err(AppError::internal)? as *const NSWindow;
    // SAFETY: `cold_show` 가 run_on_main_thread 로 메인스레드 실행을 보장하고, tao 가
    // WebviewWindow 수명 동안 NSWindow 를 살려둔다. 만들어지는 `&NSWindow` 외 가변 별칭 없음.
    let ns_window: &NSWindow = unsafe { &*ptr };

    apply_fullscreen_overlay(ns_window);
    match cocoa_placement(window)? {
        // 위치를 order-front '이전에' 동기 적용 — race 의 근본 제거.
        Some((x, y)) => ns_window.setFrameTopLeftPoint(NSPoint::new(x, y)),
        // monitor API 실패 — Tauri 기본 center 로 fallback.
        None => window.center().map_err(AppError::internal)?,
    }
    ns_window.makeKeyAndOrderFront(None);
    // release 빌드에서 level/behavior 가 리셋되는 선례(#5566) — order-front 직후 재적용.
    apply_fullscreen_overlay(ns_window);
    Ok(())
}

/// 비-macOS 는 컴파일 호환용(앱은 macOS 전용). 기본 center 후 표시.
#[cfg(not(target_os = "macos"))]
fn cold_show<R: Runtime>(window: &WebviewWindow<R>) -> AppResult<()> {
    window.center().map_err(AppError::internal)?;
    window.show().map_err(AppError::internal)?;
    window.set_focus().map_err(AppError::internal)?;
    let _ = window.emit(POPUP_OPENED, ());
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
    // cold_show 가 표시 완료 후 POPUP_OPENED 를 emit 한다(macOS 는 메인스레드 디스패치).
    cold_show(&window)
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

/// `cocoa_placement` 가 쓰는 좌표 산출 헬퍼. monitor 의 좌상단 좌표 + 크기 →
/// popup 좌상단(physical px). 순수 함수라 단위 테스트로 배치 회귀를 막는다.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
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

/// `center_on_monitor` 가 구한 physical top-left 좌표(popup 좌상단)를
/// `NSWindow::setFrameTopLeftPoint` 가 받는 Cocoa 좌표(points, 원점은 primary 좌하단,
/// +y 위쪽)로 변환한다.
///
/// tao 의 `set_outer_position` 은 비동기(`exec_async`)라 동기 `show()`
/// (`make_key_and_order_front_sync` → 메인스레드 inline)와 실행 순서가 뒤집힌다.
/// cold-show 시 order-front 시점의 프레임이 stale 이라 듀얼모니터+전체화면에서 popup 이
/// 엉뚱한 Space 에 떠 안 보인다. 이를 피하려 위치를 ns_window 에 **동기**로 적용하는데,
/// 그때 tao 의 `window_position` 변환을 그대로 재현해 기존 배치 결과를 보존한다:
///   logical = physical / scale
///   cocoa_top_left = (logical.x, primary_points_high - logical.y)
/// `primary_points_high` 는 primary monitor 의 points 높이로, tao 가 쓰는
/// `CGDisplay::main().pixels_high()` 와 같다(= primary `size.height / scale_factor`).
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn physical_top_left_to_cocoa_point(
    phys_x: f64,
    phys_y: f64,
    scale: f64,
    primary_points_high: f64,
) -> (f64, f64) {
    (phys_x / scale, primary_points_high - phys_y / scale)
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

    /// 비-Retina(scale 1): primary 중앙 배치 → Cocoa 좌상단은 X 동일, Y 만 뒤집힘.
    #[test]
    fn cocoa_point_flips_y_on_non_retina_primary() {
        // 1920x1080 primary, popup 480x360 중앙 → physical top-left (720, 360).
        let (px, py) = center_on_monitor(0.0, 0.0, 1920.0, 1080.0, 480.0, 360.0);
        assert_eq!((px, py), (720.0, 360.0));
        // scale 1, primary 1080pt → Cocoa 좌상단 (720, 1080 - 360) = (720, 720).
        let (cx, cy) = physical_top_left_to_cocoa_point(px, py, 1.0, 1080.0);
        assert_eq!((cx, cy), (720.0, 720.0));
    }

    /// Retina(scale 2): physical 좌표는 logical 로 나눈 뒤 primary points 높이로 뒤집힌다.
    #[test]
    fn cocoa_point_divides_by_scale_on_retina() {
        // 1440x900 logical = 2880x1800 physical primary. popup 480x360 logical
        // (= outer 960x720 physical) 중앙 → physical top-left (960, 540).
        let (px, py) = center_on_monitor(0.0, 0.0, 2880.0, 1800.0, 960.0, 720.0);
        assert_eq!((px, py), (960.0, 540.0));
        // scale 2, primary 900pt → (960/2, 900 - 540/2) = (480, 630).
        let (cx, cy) = physical_top_left_to_cocoa_point(px, py, 2.0, 900.0);
        assert_eq!((cx, cy), (480.0, 630.0));
    }

    /// 멀티모니터: secondary 가 primary 우측이면 X 는 보존되고 Y 는 primary 높이 기준.
    #[test]
    fn cocoa_point_preserves_x_on_right_secondary() {
        // primary 1920x1080 우측에 1920x1080 secondary(origin x=1920). popup 480x360 중앙.
        let (px, py) = center_on_monitor(1920.0, 0.0, 1920.0, 1080.0, 480.0, 360.0);
        assert_eq!((px, py), (2640.0, 360.0));
        let (cx, cy) = physical_top_left_to_cocoa_point(px, py, 1.0, 1080.0);
        // X 는 그대로(primary 우측), Y 는 primary 높이 기준 뒤집기.
        assert_eq!((cx, cy), (2640.0, 720.0));
    }

    /// 멀티모니터: secondary 가 primary 좌측이면 음수 X offset 이 보존된다.
    #[test]
    fn cocoa_point_preserves_negative_x_on_left_secondary() {
        let (cx, cy) = physical_top_left_to_cocoa_point(-1200.0, 200.0, 1.0, 1080.0);
        assert_eq!((cx, cy), (-1200.0, 880.0));
    }
}
