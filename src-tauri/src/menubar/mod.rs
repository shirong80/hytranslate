//! 메뉴바 트레이 아이콘 + popover 윈도우.
//!
//! - 트레이 좌클릭 → menubar 윈도우를 트레이 아이콘 아래 ~4px 띄워 표시.
//! - 메뉴바 윈도우는 blur 시 자동 hide (외부 click-out UX).
//! - 위치 계산은 `compute_anchor` 로 분리해 단위 테스트 가능.

mod positioning;

use tauri::{
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, PhysicalPosition, Runtime,
};

use crate::errors::{AppError, AppResult};
use crate::events::{MENUBAR_CLOSED, MENUBAR_OPENED};

pub use positioning::compute_anchor;

const MENUBAR_LABEL: &str = "menubar";
const MENUBAR_TRAY_ID: &str = "menubar";

pub fn install<R: Runtime>(app: &tauri::App<R>) -> AppResult<()> {
    let icon = app
        .default_window_icon()
        .ok_or_else(|| AppError::internal("default window icon missing"))?
        .clone();

    TrayIconBuilder::with_id(MENUBAR_TRAY_ID)
        .icon(icon)
        .icon_as_template(true)
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
