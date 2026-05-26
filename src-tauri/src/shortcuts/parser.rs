//! Electron-style accelerator 문자열 (`Cmd+Shift+T`) 을 `tauri_plugin_global_shortcut::Shortcut`
//! 으로 변환한다. 도메인 전용 — `tauri::` 의존을 모듈 최소 표면으로 유지.
//!
//! 지원 modifier: `Cmd` / `Command` / `Super` / `Meta` (전부 SUPER 로 매핑),
//! `Shift`, `Ctrl` / `Control`, `Alt` / `Option` / `Opt`. `CmdOrCtrl` /
//! `CommandOrControl` 은 macOS 에서 SUPER 로 처리.
//!
//! Key code 는 단일 영문자 / 숫자 / F-key / 화살표 / Space / Tab 위주로
//! 한정한다. 매칭은 ASCII-uppercase 무시.

use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut};

use crate::errors::{AppError, AppResult};

pub fn parse(accelerator: &str) -> AppResult<Shortcut> {
    let trimmed = accelerator.trim();
    if trimmed.is_empty() {
        return Err(AppError::InvalidShortcut {
            input: accelerator.to_string(),
        });
    }

    let mut modifiers = Modifiers::empty();
    let mut key_code: Option<Code> = None;

    for raw in trimmed.split('+') {
        let token = raw.trim();
        if token.is_empty() {
            return Err(AppError::InvalidShortcut {
                input: accelerator.to_string(),
            });
        }

        if let Some(modifier) = parse_modifier(token) {
            // 동일 modifier bit 가 두 번 등장하면 사용자 입력 실수일 가능성이 크다.
            // 예: `Cmd+CmdOrCtrl+T` 는 둘 다 SUPER 로 매핑되어 silent 하게 `Cmd+T` 와 동일
            // 등록되었다. 이는 PRD §8.5 의 "유효성 검증" 정신에 어긋난다 — 거부한다.
            if modifiers.contains(modifier) {
                return Err(AppError::InvalidShortcut {
                    input: accelerator.to_string(),
                });
            }
            modifiers |= modifier;
            continue;
        }

        if key_code.is_some() {
            return Err(AppError::InvalidShortcut {
                input: accelerator.to_string(),
            });
        }
        key_code = Some(parse_code(token).ok_or_else(|| AppError::InvalidShortcut {
            input: accelerator.to_string(),
        })?);
    }

    let code = key_code.ok_or_else(|| AppError::InvalidShortcut {
        input: accelerator.to_string(),
    })?;
    Ok(Shortcut::new(Some(modifiers), code))
}

fn parse_modifier(token: &str) -> Option<Modifiers> {
    match token.to_ascii_lowercase().as_str() {
        // macOS Cmd 키. Tauri 2 의 Modifiers::SUPER 가 Cmd 에 매핑된다.
        "cmd" | "command" | "super" | "meta" => Some(Modifiers::SUPER),
        // macOS 에서 CmdOrCtrl 은 Cmd 우선.
        "cmdorctrl" | "commandorcontrol" => Some(Modifiers::SUPER),
        "shift" => Some(Modifiers::SHIFT),
        "ctrl" | "control" => Some(Modifiers::CONTROL),
        "alt" | "option" | "opt" => Some(Modifiers::ALT),
        _ => None,
    }
}

fn parse_code(token: &str) -> Option<Code> {
    let lowered = token.to_ascii_lowercase();
    if lowered.len() == 1 {
        let ch = lowered.chars().next().unwrap();
        if ch.is_ascii_alphabetic() {
            return Some(match ch {
                'a' => Code::KeyA,
                'b' => Code::KeyB,
                'c' => Code::KeyC,
                'd' => Code::KeyD,
                'e' => Code::KeyE,
                'f' => Code::KeyF,
                'g' => Code::KeyG,
                'h' => Code::KeyH,
                'i' => Code::KeyI,
                'j' => Code::KeyJ,
                'k' => Code::KeyK,
                'l' => Code::KeyL,
                'm' => Code::KeyM,
                'n' => Code::KeyN,
                'o' => Code::KeyO,
                'p' => Code::KeyP,
                'q' => Code::KeyQ,
                'r' => Code::KeyR,
                's' => Code::KeyS,
                't' => Code::KeyT,
                'u' => Code::KeyU,
                'v' => Code::KeyV,
                'w' => Code::KeyW,
                'x' => Code::KeyX,
                'y' => Code::KeyY,
                'z' => Code::KeyZ,
                _ => unreachable!(),
            });
        }
        if ch.is_ascii_digit() {
            return Some(match ch {
                '0' => Code::Digit0,
                '1' => Code::Digit1,
                '2' => Code::Digit2,
                '3' => Code::Digit3,
                '4' => Code::Digit4,
                '5' => Code::Digit5,
                '6' => Code::Digit6,
                '7' => Code::Digit7,
                '8' => Code::Digit8,
                '9' => Code::Digit9,
                _ => unreachable!(),
            });
        }
    }
    match lowered.as_str() {
        "space" => Some(Code::Space),
        "tab" => Some(Code::Tab),
        "enter" | "return" => Some(Code::Enter),
        "escape" | "esc" => Some(Code::Escape),
        "left" => Some(Code::ArrowLeft),
        "right" => Some(Code::ArrowRight),
        "up" => Some(Code::ArrowUp),
        "down" => Some(Code::ArrowDown),
        "f1" => Some(Code::F1),
        "f2" => Some(Code::F2),
        "f3" => Some(Code::F3),
        "f4" => Some(Code::F4),
        "f5" => Some(Code::F5),
        "f6" => Some(Code::F6),
        "f7" => Some(Code::F7),
        "f8" => Some(Code::F8),
        "f9" => Some(Code::F9),
        "f10" => Some(Code::F10),
        "f11" => Some(Code::F11),
        "f12" => Some(Code::F12),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_cmd_shift_t() {
        let expected = Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyT);
        assert_eq!(parse("Cmd+Shift+T").unwrap(), expected);
    }

    #[test]
    fn parses_case_insensitive_and_whitespace() {
        let expected = Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyT);
        assert_eq!(parse(" command + SHIFT + t ").unwrap(), expected);
    }

    #[test]
    fn parses_cmdorctrl_as_super_on_macos() {
        let expected = Shortcut::new(Some(Modifiers::SUPER | Modifiers::ALT), Code::KeyL);
        assert_eq!(parse("CmdOrCtrl+Alt+L").unwrap(), expected);
    }

    #[test]
    fn parses_function_key_with_no_modifier() {
        let expected = Shortcut::new(Some(Modifiers::empty()), Code::F5);
        assert_eq!(parse("F5").unwrap(), expected);
    }

    #[test]
    fn rejects_empty_input() {
        assert!(matches!(parse(""), Err(AppError::InvalidShortcut { .. })));
    }

    #[test]
    fn rejects_double_separators() {
        assert!(matches!(
            parse("Cmd++T"),
            Err(AppError::InvalidShortcut { .. })
        ));
    }

    #[test]
    fn rejects_unknown_modifier_or_key() {
        assert!(matches!(
            parse("Hyper+T"),
            Err(AppError::InvalidShortcut { .. })
        ));
        assert!(matches!(
            parse("Cmd+Shift+UnknownKey"),
            Err(AppError::InvalidShortcut { .. })
        ));
    }

    #[test]
    fn rejects_two_keys_no_modifier() {
        assert!(matches!(
            parse("A+B"),
            Err(AppError::InvalidShortcut { .. })
        ));
    }

    #[test]
    fn rejects_duplicate_modifier_aliases() {
        // 코드리뷰 Low 1 회귀 — `Cmd` 와 `CmdOrCtrl` 는 둘 다 SUPER 비트.
        assert!(matches!(
            parse("Cmd+CmdOrCtrl+T"),
            Err(AppError::InvalidShortcut { .. })
        ));
        // 같은 이름이 두 번 와도 거부.
        assert!(matches!(
            parse("Shift+Shift+T"),
            Err(AppError::InvalidShortcut { .. })
        ));
        // Command 와 Cmd, Option 과 Alt 같은 동의어도 같은 비트라 거부.
        assert!(matches!(
            parse("Command+Cmd+T"),
            Err(AppError::InvalidShortcut { .. })
        ));
        assert!(matches!(
            parse("Alt+Option+T"),
            Err(AppError::InvalidShortcut { .. })
        ));
    }
}
