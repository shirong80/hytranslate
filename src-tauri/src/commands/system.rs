//! macOS 시스템 통합: Dock activation policy + autostart 토글.
//!
//! - `apply_dock_hidden(app, hidden)`: ActivationPolicy::Accessory ↔ Regular.
//! - `apply_autostart(app, enabled)`: tauri-plugin-autostart 의 enable/disable.
//! - `orchestrate_settings_apply`: settings 변경 transaction. 한 step 실패 시 이전 step 들을
//!   역순으로 rollback 한다 — settings 가 macOS 상태와 어긋난 채 저장되는 상황을 막는다
//!   (코드리뷰 second-pass High 1).
//!
//! 둘 다 settings 변경 시 `update_settings` 에서 호출하고, 시작 시점에는
//! `commands::register::setup()` 가 한 번 호출해 초기 상태를 디스크 값에 맞춘다.

use tauri::Runtime;
use tauri_plugin_autostart::ManagerExt as AutostartManagerExt;

use crate::errors::{AppError, AppResult};

pub fn apply_dock_hidden<R: Runtime>(app: &tauri::AppHandle<R>, hidden: bool) -> AppResult<()> {
    #[cfg(target_os = "macos")]
    {
        let policy = if hidden {
            tauri::ActivationPolicy::Accessory
        } else {
            tauri::ActivationPolicy::Regular
        };
        app.set_activation_policy(policy)
            .map_err(AppError::internal)?;
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (app, hidden);
    }
    Ok(())
}

pub fn apply_autostart<R: Runtime>(app: &tauri::AppHandle<R>, enabled: bool) -> AppResult<()> {
    let manager = app.autolaunch();
    let currently = match manager.is_enabled() {
        Ok(v) => v,
        Err(err) => {
            // LaunchAgent plist 조회 실패. 안전한 fall-back 으로 "비활성" 가정 후 진행하되,
            // 디버깅을 위해 한 줄 남긴다 — autostart 가 의도와 달리 동작할 수 있음을 알 수 있다.
            tracing::warn!(error = %err, "autolaunch is_enabled query failed; assuming disabled");
            false
        }
    };
    if currently == enabled {
        return Ok(());
    }
    if enabled {
        manager.enable().map_err(AppError::internal)
    } else {
        manager.disable().map_err(AppError::internal)
    }
}

/// settings 변경 transaction 의 pure orchestration. Tauri 의존을 분리해 단위 테스트
/// 가능하도록 closures 로 step 을 받는다.
///
/// 순서: hotkey → dock → autostart → persist. 각 step 의 apply 실패 시 이전까지
/// 적용된 step 들을 역순으로 rollback 한 뒤 첫 error 를 반환한다. `swap_hotkey` 는
/// (previous_baseline, target) 인자로 호출되며, rollback 시에는 (current_target, previous)
/// 순으로 다시 호출되어 등록을 되돌린다 (대칭). `apply_dock` / `apply_autostart` 는
/// 단일 bool 을 받고, rollback 시 이전 값으로 다시 호출된다.
#[allow(clippy::too_many_arguments)]
pub(crate) fn orchestrate_settings_apply<HK, DK, AS, P>(
    hotkey_change: Option<(String, String)>,
    dock_change: Option<(bool, bool)>,
    autostart_change: Option<(bool, bool)>,
    mut swap_hotkey: HK,
    mut apply_dock: DK,
    mut apply_autostart: AS,
    persist: P,
) -> AppResult<()>
where
    HK: FnMut(&str, &str) -> AppResult<()>,
    DK: FnMut(bool) -> AppResult<()>,
    AS: FnMut(bool) -> AppResult<()>,
    P: FnOnce() -> AppResult<()>,
{
    let mut hotkey_applied: Option<(String, String)> = None;
    let mut dock_applied: Option<bool> = None;
    let mut autostart_applied: Option<bool> = None;

    if let Some((prev, next)) = &hotkey_change {
        swap_hotkey(prev, next)?;
        hotkey_applied = Some((prev.clone(), next.clone()));
    }

    if let Some((prev, next)) = dock_change {
        if let Err(e) = apply_dock(next) {
            if let Some((prev_h, next_h)) = &hotkey_applied {
                let _ = swap_hotkey(next_h, prev_h);
            }
            return Err(e);
        }
        dock_applied = Some(prev);
    }

    if let Some((prev, next)) = autostart_change {
        if let Err(e) = apply_autostart(next) {
            if let Some(p) = dock_applied {
                let _ = apply_dock(p);
            }
            if let Some((prev_h, next_h)) = &hotkey_applied {
                let _ = swap_hotkey(next_h, prev_h);
            }
            return Err(e);
        }
        autostart_applied = Some(prev);
    }

    if let Err(e) = persist() {
        if let Some(p) = autostart_applied {
            let _ = apply_autostart(p);
        }
        if let Some(p) = dock_applied {
            let _ = apply_dock(p);
        }
        if let Some((prev_h, next_h)) = &hotkey_applied {
            let _ = swap_hotkey(next_h, prev_h);
        }
        return Err(e);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    /// 호출 log + 단계별 실패 시뮬레이션을 캡쳐하는 mock fixture.
    struct Mocks {
        log: RefCell<Vec<String>>,
        fail_dock_on: RefCell<Option<bool>>,
        fail_autostart_on: RefCell<Option<bool>>,
        fail_persist: RefCell<bool>,
    }

    impl Mocks {
        fn new() -> Self {
            Self {
                log: RefCell::new(Vec::new()),
                fail_dock_on: RefCell::new(None),
                fail_autostart_on: RefCell::new(None),
                fail_persist: RefCell::new(false),
            }
        }
    }

    fn run(
        mocks: &Mocks,
        hotkey: Option<(&str, &str)>,
        dock: Option<(bool, bool)>,
        autostart: Option<(bool, bool)>,
    ) -> AppResult<()> {
        orchestrate_settings_apply(
            hotkey.map(|(a, b)| (a.to_string(), b.to_string())),
            dock,
            autostart,
            |prev, next| {
                mocks
                    .log
                    .borrow_mut()
                    .push(format!("hotkey:{prev}->{next}"));
                Ok(())
            },
            |v| {
                mocks.log.borrow_mut().push(format!("dock:{v}"));
                if *mocks.fail_dock_on.borrow() == Some(v) {
                    return Err(AppError::internal("dock fail"));
                }
                Ok(())
            },
            |v| {
                mocks.log.borrow_mut().push(format!("autostart:{v}"));
                if *mocks.fail_autostart_on.borrow() == Some(v) {
                    return Err(AppError::internal("autostart fail"));
                }
                Ok(())
            },
            || {
                mocks.log.borrow_mut().push("persist".into());
                if *mocks.fail_persist.borrow() {
                    Err(AppError::internal("persist fail"))
                } else {
                    Ok(())
                }
            },
        )
    }

    #[test]
    fn happy_path_runs_all_steps_in_order() {
        let mocks = Mocks::new();
        run(
            &mocks,
            Some(("Cmd+Shift+T", "Cmd+Alt+L")),
            Some((false, true)),
            Some((false, true)),
        )
        .unwrap();
        assert_eq!(
            mocks.log.into_inner(),
            vec![
                "hotkey:Cmd+Shift+T->Cmd+Alt+L".to_string(),
                "dock:true".into(),
                "autostart:true".into(),
                "persist".into(),
            ]
        );
    }

    #[test]
    fn dock_failure_rolls_back_hotkey() {
        let mocks = Mocks::new();
        *mocks.fail_dock_on.borrow_mut() = Some(true);
        let err = run(
            &mocks,
            Some(("Cmd+Shift+T", "Cmd+Alt+L")),
            Some((false, true)),
            Some((false, true)),
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Internal { .. }));
        // hotkey 적용 → dock 시도 실패 → hotkey rollback (next→prev). autostart / persist 미시도.
        assert_eq!(
            mocks.log.into_inner(),
            vec![
                "hotkey:Cmd+Shift+T->Cmd+Alt+L".to_string(),
                "dock:true".into(),
                "hotkey:Cmd+Alt+L->Cmd+Shift+T".into(),
            ]
        );
    }

    #[test]
    fn autostart_failure_rolls_back_dock_and_hotkey() {
        let mocks = Mocks::new();
        *mocks.fail_autostart_on.borrow_mut() = Some(true);
        let err = run(
            &mocks,
            Some(("Cmd+Shift+T", "Cmd+Alt+L")),
            Some((false, true)),
            Some((false, true)),
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Internal { .. }));
        assert_eq!(
            mocks.log.into_inner(),
            vec![
                "hotkey:Cmd+Shift+T->Cmd+Alt+L".to_string(),
                "dock:true".into(),
                "autostart:true".into(),
                // rollback 역순: dock(prev=false) → hotkey(next→prev).
                "dock:false".into(),
                "hotkey:Cmd+Alt+L->Cmd+Shift+T".into(),
            ]
        );
    }

    #[test]
    fn persist_failure_rolls_back_all_applied_steps() {
        let mocks = Mocks::new();
        *mocks.fail_persist.borrow_mut() = true;
        let err = run(
            &mocks,
            Some(("Cmd+Shift+T", "Cmd+Alt+L")),
            Some((false, true)),
            Some((false, true)),
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Internal { .. }));
        // 모든 step 적용 후 persist 실패 → autostart → dock → hotkey 순으로 rollback.
        assert_eq!(
            mocks.log.into_inner(),
            vec![
                "hotkey:Cmd+Shift+T->Cmd+Alt+L".to_string(),
                "dock:true".into(),
                "autostart:true".into(),
                "persist".into(),
                "autostart:false".into(),
                "dock:false".into(),
                "hotkey:Cmd+Alt+L->Cmd+Shift+T".into(),
            ]
        );
    }

    #[test]
    fn only_changed_steps_are_invoked() {
        // 변경 없는 step 은 호출되지 않는다.
        let mocks = Mocks::new();
        run(&mocks, None, Some((false, true)), None).unwrap();
        assert_eq!(
            mocks.log.into_inner(),
            vec!["dock:true".to_string(), "persist".into()]
        );
    }
}
