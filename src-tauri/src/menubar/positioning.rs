//! 트레이 아이콘 위치 + 메뉴바 popover 크기를 받아 표시 좌표를 계산한다.
//!
//! 단위는 Tauri 의 `TrayIconEvent::Click.rect` 가 제공하는 physical pixel 그대로 사용한다.
//! 출력은 (x, y) i32 — `WebviewWindow::set_position(PhysicalPosition::new(x, y))` 에 그대로 투입.
//!
//! 정책:
//! - x 는 트레이 아이콘 중앙 기준으로 popover 너비의 절반만큼 좌측 시프트.
//! - y 는 트레이 아이콘 하단에서 4px 갭.

const POPOVER_GAP_PX: i32 = 4;

pub fn compute_anchor(
    tray_x: f64,
    tray_y: f64,
    tray_w: f64,
    tray_h: f64,
    popover_w: f64,
) -> (i32, i32) {
    let center_x = tray_x + tray_w / 2.0;
    let x = (center_x - popover_w / 2.0).round() as i32;
    let y = (tray_y + tray_h).round() as i32 + POPOVER_GAP_PX;
    (x, y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centers_popover_under_tray() {
        // 트레이 아이콘이 (1000, 0) 위치에서 22x22 라면, 320 너비 popover 는
        // 중앙 1011 기준으로 좌측 정렬되어 x = 1011 - 160 = 851
        let (x, y) = compute_anchor(1000.0, 0.0, 22.0, 22.0, 320.0);
        assert_eq!(x, 851);
        assert_eq!(y, 22 + POPOVER_GAP_PX);
    }

    #[test]
    fn handles_non_integer_tray_position() {
        let (x, y) = compute_anchor(1500.5, 0.0, 22.0, 22.0, 320.0);
        let expected_x: i32 = (1500.5_f64 + 11.0_f64 - 160.0_f64).round() as i32;
        assert_eq!(x, expected_x);
        assert_eq!(y, 22 + POPOVER_GAP_PX);
    }

    #[test]
    fn popover_wider_than_tray_still_centers() {
        let (x, _y) = compute_anchor(100.0, 30.0, 30.0, 22.0, 400.0);
        // 중앙 115, 절반 200 → -85
        assert_eq!(x, -85);
    }
}
