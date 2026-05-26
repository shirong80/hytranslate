//! 전역 단축키 등록. `tauri-plugin-global-shortcut` 위에 얇은 도메인 facade.
//!
//! - `install`: 플러그인 빌드 (handler 캡처) + 초기 단축키 등록.
//! - `try_swap`: settings 변경 시 호출 — register-then-rollback 가능한 transactional swap.
//! - 핸들러는 plugin 단일 등록. 어떤 단축키가 눌려도 popup toggle 한다.
//!
//! macOS 손쉬운 사용 권한 부재 시 register 자체는 실패하지 않는다 (handler 가 안 불릴 뿐).
//! 권한 producer 는 Phase 5 onboarding 에서 `AXIsProcessTrustedWithOptions` 와 함께
//! 도입된다. v1 에서는 settings 패널의 persistent CTA 로 안내한다.

pub mod parser;

use tauri::Runtime;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

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

/// 단일 단축키 register. 이전 등록은 호출자가 unregister 한 뒤 호출하라. 일반 경로는
/// `try_swap` 을 사용한다. 파싱 실패는 `InvalidShortcut`, 등록 실패는 `Internal`.
pub fn register<R: Runtime>(app: &tauri::AppHandle<R>, accelerator: &str) -> AppResult<()> {
    let shortcut = parser::parse(accelerator)?;
    app.global_shortcut()
        .register(shortcut)
        .map_err(AppError::internal)
}

/// 이전 → 새 단축키로 transactional swap. 새 단축키 등록이 실패하면 이전 단축키를
/// 즉시 복구하고 에러를 반환한다. 호출자(`update_settings`) 는 swap 이 성공한 뒤에만
/// settings 를 디스크에 저장해야 한다 — settings 가 unusable hotkey 를 담는 상황을
/// 막는다. (코드리뷰 High 1)
pub fn try_swap<R: Runtime>(
    app: &tauri::AppHandle<R>,
    previous_accelerator: &str,
    next_accelerator: &str,
) -> AppResult<()> {
    let gs = app.global_shortcut();
    orchestrate_swap(
        previous_accelerator,
        next_accelerator,
        || gs.unregister_all().map_err(AppError::internal),
        |sc| gs.register(*sc).map_err(AppError::internal),
    )
}

/// settings 가 디스크에 저장된 뒤 hotkey rollback 이 필요한 경우 호출. swap 이 성공해
/// 새 단축키가 활성인 상태에서 persist 가 실패하면, 이 함수가 이전 단축키로 되돌린다.
pub fn try_rollback<R: Runtime>(
    app: &tauri::AppHandle<R>,
    previous_accelerator: &str,
) -> AppResult<()> {
    let gs = app.global_shortcut();
    let prev = parser::parse(previous_accelerator)?;
    gs.unregister_all().map_err(AppError::internal)?;
    gs.register(prev).map_err(AppError::internal)
}

/// Tauri 의존을 분리한 순수 swap orchestration. 새 단축키 등록 실패 시 이전 단축키
/// 등록을 시도한다. 단위 테스트로 호출 시퀀스를 검증한다.
pub(crate) fn orchestrate_swap<U, R>(
    previous_accelerator: &str,
    next_accelerator: &str,
    mut unregister_all: U,
    mut register: R,
) -> AppResult<()>
where
    U: FnMut() -> AppResult<()>,
    R: FnMut(&Shortcut) -> AppResult<()>,
{
    // 새 accel 파싱은 unregister 전에. 입력 검증 실패 시 이전 단축키를 그대로 둔다.
    let next = parser::parse(next_accelerator)?;
    unregister_all()?;
    if let Err(err) = register(&next) {
        // 새 단축키 register 실패 → 이전 단축키 복구. 복구도 실패하면 단축키 비활성
        // 상태로 남지만 settings 는 아직 변경되지 않았으므로 재시작 시 다시 시도된다.
        if let Ok(prev) = parser::parse(previous_accelerator) {
            let _ = register(&prev);
        }
        return Err(err);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    use tauri_plugin_global_shortcut::{Code, Modifiers};

    fn key_t() -> Shortcut {
        Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyT)
    }
    fn key_l() -> Shortcut {
        Shortcut::new(Some(Modifiers::SUPER | Modifiers::ALT), Code::KeyL)
    }

    #[test]
    fn orchestrate_swap_happy_path_registers_only_next() {
        let log = RefCell::new(Vec::<String>::new());
        let unreg = || {
            log.borrow_mut().push("unregister_all".into());
            Ok(())
        };
        let reg = |sc: &Shortcut| {
            log.borrow_mut().push(format!("register:{:?}", sc.key));
            Ok(())
        };
        orchestrate_swap("Cmd+Shift+T", "Cmd+Alt+L", unreg, reg).unwrap();
        assert_eq!(
            log.into_inner(),
            vec![
                "unregister_all".to_string(),
                format!("register:{:?}", key_l().key),
            ]
        );
    }

    #[test]
    fn orchestrate_swap_failure_restores_previous() {
        let log = RefCell::new(Vec::<String>::new());
        let unreg = || {
            log.borrow_mut().push("unregister_all".into());
            Ok(())
        };
        let reg = |sc: &Shortcut| {
            log.borrow_mut().push(format!("register:{:?}", sc.key));
            if sc.key == Code::KeyL {
                Err(AppError::internal("simulated registration failure"))
            } else {
                Ok(())
            }
        };
        let err = orchestrate_swap("Cmd+Shift+T", "Cmd+Alt+L", unreg, reg).unwrap_err();
        assert!(matches!(err, AppError::Internal { .. }));
        assert_eq!(
            log.into_inner(),
            vec![
                "unregister_all".to_string(),
                format!("register:{:?}", key_l().key),
                format!("register:{:?}", key_t().key),
            ]
        );
    }

    #[test]
    fn orchestrate_swap_invalid_next_does_not_unregister() {
        let log = RefCell::new(Vec::<String>::new());
        let unreg = || {
            log.borrow_mut().push("unregister_all".into());
            Ok(())
        };
        let reg = |_sc: &Shortcut| {
            log.borrow_mut().push("register".into());
            Ok(())
        };
        let err = orchestrate_swap("Cmd+Shift+T", "Hyper+T", unreg, reg).unwrap_err();
        assert!(matches!(err, AppError::InvalidShortcut { .. }));
        // 파싱이 unregister 전에 일어나므로 시스템 상태는 그대로다.
        assert!(log.into_inner().is_empty());
    }

    #[test]
    fn orchestrate_swap_unregister_failure_skips_register() {
        let log = RefCell::new(Vec::<String>::new());
        let unreg = || {
            log.borrow_mut().push("unregister_all".into());
            Err(AppError::internal("unregister boom"))
        };
        let reg = |_sc: &Shortcut| {
            log.borrow_mut().push("register".into());
            Ok(())
        };
        let err = orchestrate_swap("Cmd+Shift+T", "Cmd+Alt+L", unreg, reg).unwrap_err();
        assert!(matches!(err, AppError::Internal { .. }));
        assert_eq!(log.into_inner(), vec!["unregister_all".to_string()]);
    }
}
