//! 메뉴바 트레이 아이콘 + popover 윈도우.
//!
//! - 트레이 좌클릭 → menubar 윈도우를 트레이 아이콘 아래 ~4px 띄워 표시.
//! - 트레이 우클릭 (또는 메뉴 키) → 4개 항목 ContextMenu: 메인 / 이력 / 설정 / 종료.
//! - 메뉴바 윈도우는 blur 시 자동 hide (외부 click-out UX).
//! - 위치 계산은 `compute_anchor` 로 분리해 단위 테스트 가능.

mod positioning;

use serde::Serialize;
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, PhysicalPosition, Runtime,
};

use crate::errors::{AppError, AppResult};
use crate::events::{MENUBAR_CLOSED, MENUBAR_OPENED, NAV_REQUEST};

pub use positioning::compute_anchor;

const MENUBAR_LABEL: &str = "menubar";
const MAIN_LABEL: &str = "main";
const MENUBAR_TRAY_ID: &str = "menubar";

const MENU_ID_OPEN_MAIN: &str = "menubar.open_main";
const MENU_ID_OPEN_HISTORY: &str = "menubar.open_history";
const MENU_ID_OPEN_SETTINGS: &str = "menubar.open_settings";
const MENU_ID_QUIT: &str = "menubar.quit";

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct NavRequestPayload {
    route: &'static str,
}

pub fn install<R: Runtime>(app: &tauri::App<R>) -> AppResult<()> {
    let icon = app
        .default_window_icon()
        .ok_or_else(|| AppError::internal("default window icon missing"))?
        .clone();

    let menu = MenuBuilder::new(app)
        .item(
            &MenuItemBuilder::with_id(MENU_ID_OPEN_MAIN, "메인 창 열기")
                .build(app)
                .map_err(AppError::internal)?,
        )
        .item(
            &MenuItemBuilder::with_id(MENU_ID_OPEN_HISTORY, "이력 열기")
                .build(app)
                .map_err(AppError::internal)?,
        )
        .item(
            &MenuItemBuilder::with_id(MENU_ID_OPEN_SETTINGS, "설정 열기")
                .build(app)
                .map_err(AppError::internal)?,
        )
        .separator()
        .item(
            &MenuItemBuilder::with_id(MENU_ID_QUIT, "종료")
                .build(app)
                .map_err(AppError::internal)?,
        )
        .build()
        .map_err(AppError::internal)?;

    TrayIconBuilder::with_id(MENUBAR_TRAY_ID)
        .icon(icon)
        .icon_as_template(true)
        .menu(&menu)
        // 좌클릭은 popover, 우클릭/우상단 보조키는 OS 가 menu 를 띄운다.
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            MENU_ID_OPEN_MAIN => focus_main_and_route(app, "translate"),
            MENU_ID_OPEN_HISTORY => focus_main_and_route(app, "history"),
            MENU_ID_OPEN_SETTINGS => focus_main_and_route(app, "settings"),
            MENU_ID_QUIT => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                rect,
                ..
            } = event
            else {
                return;
            };
            let app = tray.app_handle();
            let Some(window) = app.get_webview_window(MENUBAR_LABEL) else {
                return;
            };
            let scale = window.scale_factor().unwrap_or(1.0);
            let pos = rect.position.to_physical::<f64>(scale);
            let tsize = rect.size.to_physical::<f64>(scale);
            let popover_w = window
                .outer_size()
                .ok()
                .map(|s| s.width as f64)
                .unwrap_or(320.0);
            let (x, y) = compute_anchor(pos.x, pos.y, tsize.width, tsize.height, popover_w);
            let _ = window.set_position(PhysicalPosition::new(x, y));
            let _ = window.show();
            let _ = window.set_focus();
            let _ = app.emit(MENUBAR_OPENED, ());
        })
        .build(app)
        .map_err(AppError::internal)?;

    if let Some(window) = app.get_webview_window(MENUBAR_LABEL) {
        let w = window.clone();
        let app_handle = app.handle().clone();
        window.on_window_event(move |event| {
            if let tauri::WindowEvent::Focused(false) = event {
                let _ = w.hide();
                let _ = app_handle.emit(MENUBAR_CLOSED, ());
            }
        });
    }

    Ok(())
}

fn focus_main_and_route<R: Runtime>(app: &tauri::AppHandle<R>, route: &'static str) {
    if let Some(window) = app.get_webview_window(MAIN_LABEL) {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
    let _ = app.emit(NAV_REQUEST, NavRequestPayload { route });
}
