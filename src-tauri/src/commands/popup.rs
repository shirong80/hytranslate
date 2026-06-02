//! Floating popup 윈도우 제어.
//!
//! - `show_popup`: 커서가 있는 화면 중앙 배치 후 표시 + 포커스.
//! - `hide_popup`: 윈도우 숨김. focus 추적 없이 항상 idempotent.
//! - `toggle_popup`: 이미 보이면 숨기고, 아니면 show_popup.
//! - `resize_popup`: output 높이 변화에 맞춰 크기 변경 + top-left 보존(메인스레드 동기).

use tauri::{Emitter, LogicalPosition, Manager, Runtime, WebviewWindow};

use crate::errors::{AppError, AppResult};
use crate::events::{POPUP_CLOSED, POPUP_OPENED};

// define_class! 의 `unsafe impl` 은 프로토콜을 bare ident 로만 받는다(path 불가).
#[cfg(target_os = "macos")]
use objc2::runtime::NSObjectProtocol;

const POPUP_LABEL: &str = "popup";

// resize_popup 높이 sanity 범위(points). FE 가 monitor 80% cap 으로 실제 정책을 강제하지만,
// IPC 는 신뢰 불가 입력이라 Rust 에서도 backstop 으로 clamp 한다. 상한은 현재 화면 높이의 80%
// 이고, 화면을 못 구하면(screenless) growth 를 거부해 최소 높이로 제한한다(resize_cap_for_screen).
const POPUP_MIN_HEIGHT: f64 = 360.0;

// PRD §6.3: popup 높이는 화면(논리 frame)의 80% 를 넘지 않는다. FE computePopupHeight 와 동일한
// 비율을 cold-show 배치에서도 적용해, 다른 높이의 화면에서 다시 열 때(output 미변경 → FE resize
// 미발생) cap 을 보장한다.
const POPUP_MAX_HEIGHT_RATIO: f64 = 0.8;

pub fn get_popup<R: Runtime>(app: &tauri::AppHandle<R>) -> AppResult<WebviewWindow<R>> {
    app.get_webview_window(POPUP_LABEL)
        .ok_or_else(|| AppError::internal("popup window missing"))
}

// popup 의 라이브 NSWindow 를 swizzle 로 갈아끼울 nonactivating NSPanel 서브클래스.
//
// borderless NSWindow 의 `canBecomeKeyWindow` 기본값은 false 이고, swizzle 로 tao 의
// TaoWindow 오버라이드를 잃으면 키 포커스를 못 받는다. NSPanel 위에서 true 로 강제해
// 전체화면 위에서도 textarea 가 입력을 받게 한다. HyPopupPanel 은 ivar 도 Drop 도 없어
// `set_class` 가 isa 만 교체하므로(상위 chain 도 NSWindow 로 동일) 기존 객체의 메모리
// 레이아웃이 보존된다.
//
// swizzle 로 잃는 TaoWindow 의 클래스 메서드는 셋뿐이고 popup 에선 모두 안전하다:
//   - canBecomeKeyWindow → 여기서 true 로 재공급(핵심).
//   - canBecomeMainWindow → NSPanel 기본 false. panel 은 main 이 아니므로 정상.
//   - sendEvent: → 표준 forwarding 으로 대체. tao 판의 유일한 추가 동작은 movable-by-
//     window-background(빈 배경 드래그) 시 performWindowDragWithEvent 인데, popup 은
//     movable 이 아니라 그 분기가 발화하지 않는다. 헤더의 `data-tauri-drag-region` 드래그는
//     이 경로와 무관하다 — Tauri 가 주입한 JS 가 start_dragging → tao drag_window 로
//     `[ns_window performWindowDragWithEvent:]` 를 NSWindow 에 직접 호출하므로(sendEvent:
//     를 거치지 않음) swizzle 후에도 그대로 동작한다.
// 윈도우 동작 대부분은 window-class 가 아니라 별도 TaoWindowDelegate(resize/move/key 알림)
// 가 담당하고 delegate 는 swizzle 에 영향받지 않으므로 그대로 유지된다.
// (배경 드래그/movable-by-window-background 를 추가할 때만 sendEvent: 재구현이 필요하다.)
#[cfg(target_os = "macos")]
objc2::define_class!(
    // SAFETY: NSPanel 은 서브클래싱 요구사항이 없고, HyPopupPanel 은 ivar 도 Drop 도 없다.
    #[unsafe(super = objc2_app_kit::NSPanel)]
    #[name = "HyPopupPanel"]
    struct HyPopupPanel;

    // SAFETY: NSObjectProtocol 은 안전 요구사항이 없다.
    unsafe impl NSObjectProtocol for HyPopupPanel {}

    impl HyPopupPanel {
        #[unsafe(method(canBecomeKeyWindow))]
        fn can_become_key_window(&self) -> bool {
            true
        }
    }
);

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

/// 커서가 있는 화면 중앙에 popup 을 둘 좌상단을 `setFrameTopLeftPoint` 용 Cocoa points 로
/// 구한다.
///
/// `NSEvent::mouseLocation` 과 `NSScreen.frame` 은 동일한 전역 Cocoa 공간(좌하단 원점, +y 위)
/// 이라 모니터별 backing scale 이 계산에 개입하지 않는다. 그래서 Tauri 의 `cursor_position()`
/// (physical = points × primary_scale) → `monitor_from_point()`(points 기준) 단위 불일치와,
/// 대상/현재 화면 배율 혼합 문제를 원천에서 제거한다.
///
/// 커서가 어느 화면에도 없으면(rare) `screens[0]`(메뉴바·좌표 원점 화면)으로 fallback 한다 —
/// `mainScreen`(포커스 화면, 가변)이 아니라 결정적인 zero 화면을 쓴다. 화면이 0개면 `None`
/// (호출부가 Tauri 기본 center 로 처리).
#[cfg(target_os = "macos")]
fn cocoa_placement(ns_window: &objc2_app_kit::NSWindow) -> Option<(f64, f64)> {
    use objc2::MainThreadMarker;
    use objc2_app_kit::{NSEvent, NSScreen};
    use objc2_foundation::NSSize;

    // SAFETY: cold_show 가 run_on_main_thread 로 이 경로의 메인스레드 실행을 보장한다.
    let mtm = unsafe { MainThreadMarker::new_unchecked() };
    let frames: Vec<(f64, f64, f64, f64)> = NSScreen::screens(mtm)
        .iter()
        .map(|screen| {
            let f = screen.frame();
            (f.origin.x, f.origin.y, f.size.width, f.size.height)
        })
        .collect();
    if frames.is_empty() {
        return None;
    }

    let cursor = NSEvent::mouseLocation();
    let idx = screen_index_at(&frames, cursor.x, cursor.y).unwrap_or_else(|| {
        tracing::debug!("popup placement: cursor not on any screen, using primary");
        0
    });

    let (sx, sy, sw, sh) = frames[idx];
    let size = ns_window.frame().size;
    // 선택 화면의 80% cap 으로 높이를 제한한 뒤 그 높이로 중앙 배치. 다른 높이의 화면에서 다시
    // 열 때(output 미변경 → FE resize 미발생) PRD §6.3 80% 제한을 cold-show 가 직접 보장한다.
    let (capped_h, pos) = capped_centered_placement(sx, sy, sw, sh, size.width, size.height);
    if capped_h != size.height {
        ns_window.setContentSize(NSSize::new(size.width, capped_h));
    }
    Some(pos)
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
///
/// 핵심: popup 의 라이브 NSWindow 를 nonactivating NSPanel 로 class-swizzle 한다. 기본
/// `Regular` activation policy 의 일반 NSWindow 는 macOS 10.14+ 에서 타 앱 네이티브 전체화면
/// Space 위 표시가 막혀(Tauri #11488), `CanJoinAllSpaces` + 높은 level 만으로는 전체화면 위에
/// 못 뜨고 엉뚱한 Space 로 떨어진다. panel 전환 + `orderFrontRegardless` 로 (1) 백그라운드
/// 상태에서도 전면 배치하고, (2) `NonactivatingPanel` 스타일 + `canBecomeKeyWindow` 로 앱을
/// activate(=Space 전환)하지 않고 키 포커스를 얻는다.
///
/// swizzle·styleMask·overlay 는 모두 멱등이라 cold-show 마다 재적용해 release 빌드의
/// level/behavior 리셋(#5566)·tao 의 styleMask clobber 에도 견고하다.
#[cfg(target_os = "macos")]
fn cold_show_on_main<R: Runtime>(window: &WebviewWindow<R>) -> AppResult<()> {
    use objc2::runtime::AnyObject;
    use objc2::ClassType;
    use objc2_app_kit::{NSWindow, NSWindowStyleMask};
    use objc2_foundation::NSPoint;

    let ptr = window.ns_window().map_err(AppError::internal)?;
    // SAFETY: `cold_show` 가 run_on_main_thread 로 메인스레드 실행을 보장하고, tao 가
    // WebviewWindow 수명 동안 NSWindow 를 살려둔다. HyPopupPanel 은 ivar 가 없어 `set_class`
    // 가 isa 만 교체하므로 기존 NSWindow 의 메모리 레이아웃이 그대로 유효하다.
    let obj: &AnyObject = unsafe { &*(ptr as *const AnyObject) };
    let _ = unsafe { AnyObject::set_class(obj, HyPopupPanel::class()) };

    let ns_window: &NSWindow = unsafe { &*(ptr as *const NSWindow) };
    let mut mask = ns_window.styleMask();
    mask |= NSWindowStyleMask::NonactivatingPanel;
    ns_window.setStyleMask(mask);
    // SAFETY: 닫혀도 해제되지 않게 한다 — 수명은 tao 가 소유하고 우리는 hide(orderOut)만 쓴다.
    unsafe { ns_window.setReleasedWhenClosed(false) };

    apply_fullscreen_overlay(ns_window);
    match cocoa_placement(ns_window) {
        // 위치를 order-front '이전에' 동기 적용 — 듀얼모니터 stale-frame race 제거(기존 회귀 방지).
        Some((x, y)) => ns_window.setFrameTopLeftPoint(NSPoint::new(x, y)),
        // 화면 0개(극단) — Tauri 기본 center 로 fallback.
        None => window.center().map_err(AppError::internal)?,
    }
    // 백그라운드(타 앱이 전면)에서도 전면 배치되는 유일한 호출. makeKeyAndOrderFront 는
    // 비활성 앱에선 거부/후순위라(macOS 13.3+ 경고) 전체화면 Space 에 못 떴다.
    ns_window.orderFrontRegardless();
    // nonactivating panel 이라 앱 activate 없이 key 가 된다 → 전체화면에서 빠져나오지 않음.
    ns_window.makeKeyWindow();
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

/// 이미 보이는 popup 을 앱 activate 없이 다시 전면+키로 잡는다. `set_focus`(activate)는
/// 전체화면 위에서 Space 전환을 유발하므로, cold-show 에서 이미 panel 로 전환된 윈도우에
/// 메인스레드에서 `orderFrontRegardless` + `makeKeyWindow` 만 다시 건다.
#[cfg(target_os = "macos")]
fn refront<R: Runtime>(window: &WebviewWindow<R>) -> AppResult<()> {
    let win = window.clone();
    window
        .run_on_main_thread(move || {
            if let Ok(ptr) = win.ns_window() {
                use objc2_app_kit::NSWindow;
                // SAFETY: 메인스레드 실행 + tao 가 NSWindow 를 살려둔다. cold-show 에서 이미
                // panel 로 swizzle 되어 있어 추가 별칭 없이 전면/키만 다시 건다.
                let ns_window: &NSWindow = unsafe { &*(ptr as *const NSWindow) };
                ns_window.orderFrontRegardless();
                ns_window.makeKeyWindow();
            }
            let _ = win.emit(POPUP_OPENED, ());
        })
        .map_err(AppError::internal)
}

#[cfg(not(target_os = "macos"))]
fn refront<R: Runtime>(window: &WebviewWindow<R>) -> AppResult<()> {
    window.set_focus().map_err(AppError::internal)?;
    let _ = window.emit(POPUP_OPENED, ());
    Ok(())
}

pub fn show<R: Runtime>(app: &tauri::AppHandle<R>) -> AppResult<()> {
    let window = get_popup(app)?;
    if window.is_visible().map_err(AppError::internal)? {
        // 이미 보이는 경우: 앱 activate 없이 전면+키만 다시 잡는다(전체화면 Space 전환 방지).
        return refront(&window);
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

/// output 길이에 따라 popup 높이를 바꾸되 top-left 를 보존한다.
///
/// macOS: 크기 변경과 위치 보정을 **메인스레드 한 클로저에서 동기로** 수행한다. FE 에서
/// `setSize`(tao `set_content_size_async`) → `setPosition`(tao `set_frame_top_left_point_async`)
/// 으로 나눠 호출하면 둘 다 `DispatchQueue::main().exec_async` 로 enqueue 되어 적용 완료를
/// JS 가 기다릴 수 없다. 그 사이 닫고 다른 화면에서 다시 열면(`cold_show_on_main`) 늦게 실행된
/// 이전 세대의 `setContentSize` 가 새 배치를 미는 경합이 생긴다. ns_window 에 직접(동기) 적용해
/// 이 경합을 제거한다 — `cold_show_on_main` 과 같은 `run_on_main_thread` 경로라 둘 사이도 FIFO 다.
/// latest-wins(가장 최신 output 높이가 최종 적용)는 FE 가 resize_popup 호출을 single-flight 로
/// 직렬화해 보장한다(한 번에 하나만 in-flight → enqueue 순서 = 호출 순서 → FIFO 적용). 그래서
/// Rust 에 seq/generation state 가 필요 없다.
#[cfg(target_os = "macos")]
#[tauri::command]
pub async fn resize_popup<R: Runtime>(app: tauri::AppHandle<R>, height: f64) -> AppResult<()> {
    let window = get_popup(&app)?;
    let win = window.clone();
    window
        .run_on_main_thread(move || {
            if let Ok(ptr) = win.ns_window() {
                use objc2_app_kit::NSWindow;
                use objc2_foundation::{NSPoint, NSSize};
                // SAFETY: run_on_main_thread 로 메인스레드 실행 보장 + tao 가 NSWindow 를 살려둔다.
                let ns_window: &NSWindow = unsafe { &*(ptr as *const NSWindow) };
                // 신뢰 불가 IPC 입력 — 현재 화면 높이의 80% 를 상한으로 clamp. cold-show·FE 와
                // 같은 기준(frame * RATIO)이라, 지연된 resize 가 cold-show 80% cap 을 다시 넘지
                // 못한다. 화면 미연결(screenless)이면 growth 를 거부(최소 높이로 제한).
                let cap = resize_cap_for_screen(ns_window.screen().map(|s| s.frame().size.height));
                let height = clamp_popup_height(height, cap);
                let frame = ns_window.frame();
                let (tx, ty) =
                    top_left_to_preserve(frame.origin.x, frame.origin.y, frame.size.height);
                // setContentSize 는 bottom-left 고정 → 직후 top-left 를 직전 값으로 되돌려 보존.
                ns_window.setContentSize(NSSize::new(frame.size.width, height));
                ns_window.setFrameTopLeftPoint(NSPoint::new(tx, ty));
            }
        })
        .map_err(AppError::internal)
}

/// 비-macOS 컴파일 호환용. width 는 popup 고정값(480), top-left 보존은 macOS 전용 경로에서만.
/// 상한은 현재 monitor 높이의 80%(없으면 최소 높이).
#[cfg(not(target_os = "macos"))]
#[tauri::command]
pub async fn resize_popup<R: Runtime>(app: tauri::AppHandle<R>, height: f64) -> AppResult<()> {
    let window = get_popup(&app)?;
    let screen_h = window
        .current_monitor()
        .ok()
        .flatten()
        .map(|m| m.size().height as f64 / m.scale_factor());
    let height = clamp_popup_height(height, resize_cap_for_screen(screen_h));
    window
        .set_size(tauri::LogicalSize::new(480.0, height))
        .map_err(AppError::internal)
}

/// `resize_popup` 가 받은 신뢰 불가 높이를 안전 범위로 clamp 한다. 비유한값(NaN/Inf)은 최소
/// 높이로 떨어뜨리고, `[POPUP_MIN_HEIGHT, max_height]` 로 가둔다. max < min 인 비정상 화면값도
/// 최소 높이로 수렴시킨다. 순수 함수라 경계값 회귀를 단위 테스트로 막는다.
fn clamp_popup_height(height: f64, max_height: f64) -> f64 {
    if !height.is_finite() {
        return POPUP_MIN_HEIGHT;
    }
    let max = max_height.max(POPUP_MIN_HEIGHT);
    height.clamp(POPUP_MIN_HEIGHT, max)
}

/// `resize_popup` 의 높이 상한을 정한다. 화면 높이를 알면 그 80%(`POPUP_MAX_HEIGHT_RATIO`),
/// 화면을 못 구하면(screenless: 숨김/offscreen) growth 를 거부해 최소 높이로 제한한다 —
/// 신뢰 불가 IPC 가 큰 height 를 그대로 적용하는 것을 막는다(cold-show 가 표시 시 재적용).
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn resize_cap_for_screen(screen_height: Option<f64>) -> f64 {
    match screen_height {
        Some(h) => h * POPUP_MAX_HEIGHT_RATIO,
        None => POPUP_MIN_HEIGHT,
    }
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

/// 커서(Cocoa points)를 포함하는 화면 인덱스. 없으면 `None`(호출부 `screens[0]` fallback).
/// `frames`: `(origin_x, origin_y, width, height)` Cocoa points. 좌/하측 화면의 음수 origin 허용.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn screen_index_at(frames: &[(f64, f64, f64, f64)], cursor_x: f64, cursor_y: f64) -> Option<usize> {
    frames.iter().position(|&(x, y, w, h)| {
        cursor_x >= x && cursor_x < x + w && cursor_y >= y && cursor_y < y + h
    })
}

/// 대상 화면 frame(Cocoa points, 좌하단 원점, +y 위) + popup 크기 → `setFrameTopLeftPoint` 가
/// 받는 좌상단(Cocoa points). +y 가 위쪽이라 좌상단 y = 화면 하단 + (화면 높이 + popup 높이)/2.
///
/// 배율 인자가 없는 게 핵심이다. `NSScreen.frame`·`NSWindow.frame` 이 같은 points 공간이라
/// 모니터별 backing scale 이 결과에 영향을 주지 않는다 — 기존 physical 왕복(H1/H2)이 일으킨
/// 단위 불일치·배율 혼합을 제거한다.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn center_top_left_in_cocoa_points(
    screen_x: f64,
    screen_y: f64,
    screen_w: f64,
    screen_h: f64,
    popup_w: f64,
    popup_h: f64,
) -> (f64, f64) {
    let x = screen_x + (screen_w - popup_w) / 2.0;
    let top_y = screen_y + (screen_h + popup_h) / 2.0;
    (x, top_y)
}

/// 선택 화면의 80% cap(`POPUP_MAX_HEIGHT_RATIO`)으로 popup 높이를 제한하고, 그 높이로 중앙
/// 좌상단을 구한다. 반환: `(capped_height, (top_left_x, top_left_y))`. 호출부가 capped_height 로
/// `setContentSize` 후 좌상단을 적용한다. 순수 함수라 reopen-on-shorter-screen 회귀를 막는다.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn capped_centered_placement(
    screen_x: f64,
    screen_y: f64,
    screen_w: f64,
    screen_h: f64,
    popup_w: f64,
    popup_h: f64,
) -> (f64, (f64, f64)) {
    let capped_h = clamp_popup_height(popup_h, screen_h * POPUP_MAX_HEIGHT_RATIO);
    let pos =
        center_top_left_in_cocoa_points(screen_x, screen_y, screen_w, screen_h, popup_w, capped_h);
    (capped_h, pos)
}

/// `setContentSize` 는 Cocoa bottom-left 를 고정하므로 height 가 바뀌면 창이 위로 자란다. 직전
/// 좌상단(`origin + 현재 높이`)을 돌려줘, 리사이즈 직후 `setFrameTopLeftPoint` 로 위치를 보존한다
/// (드래그한 위치 또는 cold-show 중앙을 그대로 유지). 순수 함수라 회귀를 단위 테스트로 막는다.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn top_left_to_preserve(origin_x: f64, origin_y: f64, frame_height: f64) -> (f64, f64) {
    (origin_x, origin_y + frame_height)
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

    /// primary(zero) 화면 중앙 → Cocoa 좌상단. x 는 가로 중앙, top_y 는 하단+(높이+popup)/2.
    #[test]
    fn cocoa_center_on_primary() {
        // 1440x900 primary, popup 480x360.
        // x = (1440-480)/2 = 480. top_y = (900+360)/2 = 630.
        assert_eq!(
            center_top_left_in_cocoa_points(0.0, 0.0, 1440.0, 900.0, 480.0, 360.0),
            (480.0, 630.0)
        );
    }

    /// 우측 secondary: 양수 origin x 가 좌상단 x 에 보존된다.
    #[test]
    fn cocoa_center_preserves_right_secondary_origin() {
        // secondary origin x=1440, 1920x1080. x = 1440 + (1920-480)/2 = 2160.
        assert_eq!(
            center_top_left_in_cocoa_points(1440.0, 0.0, 1920.0, 1080.0, 480.0, 360.0),
            (2160.0, 720.0)
        );
    }

    /// 좌측 secondary: 음수 origin x 가 보존된다.
    #[test]
    fn cocoa_center_preserves_negative_origin_left_secondary() {
        // secondary origin x=-1920, 1920x1080. x = -1920 + (1920-480)/2 = -1200.
        assert_eq!(
            center_top_left_in_cocoa_points(-1920.0, 0.0, 1920.0, 1080.0, 480.0, 360.0),
            (-1200.0, 720.0)
        );
    }

    /// 아래쪽 secondary: zero 화면보다 낮은 화면은 음수 origin y → top_y 도 음수일 수 있다.
    #[test]
    fn cocoa_center_preserves_vertical_origin() {
        // secondary origin y=-1080, 1920x1080. top_y = -1080 + (1080+360)/2 = -360.
        assert_eq!(
            center_top_left_in_cocoa_points(0.0, -1080.0, 1920.0, 1080.0, 480.0, 360.0),
            (720.0, -360.0)
        );
    }

    /// 회귀 guard: 배율 인자가 없으므로 같은 points frame 은 (어느 backing scale 이든) 같은 결과.
    /// 누군가 다시 physical/scale 왕복을 넣으면 이 기대값이 깨진다.
    #[test]
    fn cocoa_center_takes_no_scale_factor() {
        // Retina(2x)로 2880x1800 physical 인 화면도 points 로는 1440x900 → 아래로 고정.
        assert_eq!(
            center_top_left_in_cocoa_points(0.0, 0.0, 1440.0, 900.0, 480.0, 360.0),
            (480.0, 630.0)
        );
    }

    /// 커서가 든 화면 인덱스를 반환한다(다중 화면).
    #[test]
    fn screen_index_at_finds_containing_screen() {
        let frames = [(0.0, 0.0, 1440.0, 900.0), (1440.0, 0.0, 1920.0, 1080.0)];
        assert_eq!(screen_index_at(&frames, 200.0, 200.0), Some(0));
        assert_eq!(screen_index_at(&frames, 2000.0, 500.0), Some(1));
    }

    /// 음수 origin 화면(좌측 배치) 안의 커서도 매칭한다.
    #[test]
    fn screen_index_at_matches_negative_origin_screen() {
        let frames = [(0.0, 0.0, 1440.0, 900.0), (-1920.0, 0.0, 1920.0, 1080.0)];
        assert_eq!(screen_index_at(&frames, -1000.0, 300.0), Some(1));
    }

    /// 어느 화면에도 없으면 None(호출부 screens[0] fallback).
    #[test]
    fn screen_index_at_returns_none_when_outside() {
        let frames = [(0.0, 0.0, 1440.0, 900.0)];
        assert_eq!(screen_index_at(&frames, 5000.0, 5000.0), None);
    }

    /// resize 시 직전 top-left(= origin + 현재 높이)를 보존한다. height 가 바뀌어도 top 고정.
    #[test]
    fn top_left_to_preserve_keeps_top_edge() {
        // origin (100, 200), 현재 높이 360 → top-left (100, 560).
        assert_eq!(top_left_to_preserve(100.0, 200.0, 360.0), (100.0, 560.0));
        // 음수 origin(좌/하측 화면)도 그대로 반영.
        assert_eq!(
            top_left_to_preserve(-1200.0, -50.0, 480.0),
            (-1200.0, 430.0)
        );
    }

    /// 신뢰 불가 높이 입력을 안전 범위로 clamp 한다(IPC 경계 검증).
    #[test]
    fn clamp_popup_height_bounds_untrusted_input() {
        let max = 1080.0;
        // 정상값은 그대로.
        assert_eq!(clamp_popup_height(600.0, max), 600.0);
        // 음수·0·최소 미만은 최소로.
        assert_eq!(clamp_popup_height(-1.0, max), POPUP_MIN_HEIGHT);
        assert_eq!(clamp_popup_height(0.0, max), POPUP_MIN_HEIGHT);
        assert_eq!(clamp_popup_height(100.0, max), POPUP_MIN_HEIGHT);
        // 매우 큰 유한값은 max 로.
        assert_eq!(clamp_popup_height(1e18, max), max);
        // 비유한값(NaN/Inf)은 최소로.
        assert_eq!(clamp_popup_height(f64::NAN, max), POPUP_MIN_HEIGHT);
        assert_eq!(clamp_popup_height(f64::INFINITY, max), POPUP_MIN_HEIGHT);
        // 경계 안.
        assert_eq!(clamp_popup_height(360.0, max), 360.0);
        assert_eq!(clamp_popup_height(1080.0, max), 1080.0);
        // 비정상 화면값(max < min)도 최소로 수렴.
        assert_eq!(clamp_popup_height(500.0, 100.0), POPUP_MIN_HEIGHT);
    }

    /// cold-show 재배치: 다른 높이의 화면에서 다시 열 때 80% cap 을 재적용하고 그 높이로 중앙배치.
    #[test]
    fn capped_centered_placement_reapplies_screen_cap() {
        // 작은 화면(1440x900, 80%=720)에서, 직전에 큰 화면에서 커진 popup(높이 1000) 을 다시 연다.
        let (h, (x, y)) = capped_centered_placement(0.0, 0.0, 1440.0, 900.0, 480.0, 1000.0);
        assert_eq!(h, 720.0); // 900 * 0.8 으로 clamp
                              // capped 높이 기준 중앙: x=(1440-480)/2=480, top_y=(900+720)/2=810.
        assert_eq!((x, y), (480.0, 810.0));

        // cap 안의 높이는 그대로, 그 높이로 중앙.
        let (h2, pos2) = capped_centered_placement(0.0, 0.0, 1920.0, 1080.0, 480.0, 500.0);
        assert_eq!(h2, 500.0);
        assert_eq!(pos2, (720.0, 790.0)); // x=(1920-480)/2=720, top_y=(1080+500)/2=790
    }

    /// resize_popup 상한도 화면의 80% — 큰 화면 target 을 작은 화면에서 적용하면 80% 로 제한된다
    /// (cold-show cap 우회 방지). resize_popup 가 쓰는 `screen.frame * RATIO` 를 max 로 검증.
    #[test]
    fn resize_popup_max_is_screen_80_percent() {
        // 큰 화면(2160 높이)에서 계산된 target(1700)을, 작은 화면(900 높이, 80%=720)에서 적용.
        let small_max = 900.0 * POPUP_MAX_HEIGHT_RATIO;
        assert_eq!(clamp_popup_height(1700.0, small_max), 720.0);
        // 작은 화면 80% 안의 값은 그대로.
        assert_eq!(clamp_popup_height(600.0, small_max), 600.0);
    }

    /// resize_popup 상한: 화면 높이를 알면 80%, screenless 면 최소 높이로 growth 거부.
    #[test]
    fn resize_cap_for_screen_bounds_screenless_fallback() {
        // 화면 있음 → 80% cap. 신뢰 불가 큰 height 도 이 cap 으로 제한된다.
        assert_eq!(resize_cap_for_screen(Some(900.0)), 720.0);
        assert_eq!(
            clamp_popup_height(4000.0, resize_cap_for_screen(Some(900.0))),
            720.0
        );
        // screenless(None) → 최소 높이 cap. 4000 같은 값도 360 으로 제한(이전 fallback 4000 우회 차단).
        assert_eq!(resize_cap_for_screen(None), POPUP_MIN_HEIGHT);
        assert_eq!(
            clamp_popup_height(4000.0, resize_cap_for_screen(None)),
            POPUP_MIN_HEIGHT
        );
    }
}
