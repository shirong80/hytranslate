# Bugfix — 메인 창 닫은 뒤 Dock 아이콘 클릭이 동작하지 않음

이슈: [shirong80/hytranslate#1](https://github.com/shirong80/hytranslate/issues/1)
범위: 메인 창(`main`) 한정. popup / menubar 윈도우는 손대지 않음.
완료 기준:
- 아래 §6 수동 검증 5항목 전부 통과.
- **자동 게이트 (필수, hard pass/fail)**:
  - `npm run lint`
  - `npm run typecheck`
  - `npm run test`
  - `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings`
  - `cargo test --manifest-path src-tauri/Cargo.toml`
- **가이드라인 검토**: 코드 변경 직후 `.claude/CLAUDE.md` 의 "After every code change" 규약에 따라 `Agent(subagent_type="quality-gate")` 호출.

---

## 1. 사용자 결정 (interview)

| Q | 결정 |
|---|---|
| Q1 닫기(X) 정책 | 창 hide, 앱은 살아 있음 (macOS 표준). |
| Q2 Dock 아이콘 클릭 동작 | 항상 메인 창 `show()` + `unminimize()` + `set_focus()`. |
| Q3 이슈 범위 | Narrow — 메인 창만. popup / menubar lifecycle audit 는 별도 이슈로 분리. |
| Q4 `unminimize()` 호출 | 호출 — Dock 클릭 시 최소화도 함께 풀어주는 표준 macOS 동작과 일치. |
| Q5 로깅 레벨 | `debug!` — 운영 노이즈 최소화. |

---

## 2. 증상

1. 앱 실행 → 메인 창 좌상단 빨간 닫기(X) 클릭 → 창 사라짐.
2. Dock 아이콘 클릭 → 무반응. 메인 창 복귀 불가.
3. `Cmd+Shift+T` 만 동작하지만 popup 만 뜸 — 메인 창 복귀 경로 아님.

기대 동작 (macOS 표준): 닫기 = 창 숨김, Dock 클릭 = 메인 창 `show()` + `focus`.

---

## 3. 원인 (코드 추적)

### 3.1 메인 창 close 가 hide 가 아니라 destroy 됨
`src-tauri/tauri.conf.json:14-26` — `main` 윈도우 정의에 close 관련 별도 핸들러가 없음. Tauri 2 의 기본 동작은 `WindowEvent::CloseRequested → Destroy`. webview 가 파괴된 뒤 단순 `show()` 만으로는 복귀 불가.

### 3.2 `RunEvent::Reopen` 핸들러 없음
`src-tauri/src/lib.rs:14`, `src-tauri/src/commands/mod.rs:55-168` 어디에도 macOS Dock 클릭 시 발생하는 `NSApplicationDelegate#applicationShouldHandleReopen:hasVisibleWindows:` 대응 코드가 없음. `tauri::Builder::run(ctx)` 한 줄로 종료 → 외부 이벤트 루프에 후크할 수 없는 구조.

### 3.3 `Cmd+Shift+T` 는 popup 전용
`src-tauri/src/shortcuts/mod.rs` — 단축키가 `popup::toggle_popup` 으로 연결됨. 메인 창 복귀 경로와 무관.

### 3.4 종합
`hide_dock_icon: false` (`src-tauri/src/settings/mod.rs:56`, default) 이므로 Dock 아이콘은 떠 있는데, 클릭 이벤트를 잡는 코드가 없어 죽은 아이콘이 되는 구조.

---

## 4. 변경 사항

### 4.1 `src-tauri/src/commands/mod.rs` — main 윈도우 close 정책

`setup` 클로저 안에서 다음 시점에 main 윈도우 `on_window_event` 등록:
- 위치: `system::apply_autostart(...)` 호출 직후 (setup 의 다른 OS-mutating 초기화 끝난 뒤).
- 동작: `WindowEvent::CloseRequested { api, .. }` 수신 시
  1. `api.prevent_close()` 호출.
  2. `window.hide()` 호출.
  3. `tracing::debug!(window = "main", "close-to-hide intercepted")` 한 줄.
- **플랫폼 분기**: 핸들러 부착 블록 전체를 `#[cfg(target_os = "macos")]` 로 감싼다. macOS 가 아닐 경우 등록 자체를 건너뛰어 Tauri 기본 동작(destroy) 을 유지 — 본 v1 은 macOS 전용 (PRD §1) 이라 비-macOS 동작은 형식적 보호이지만, "어디서 어떻게 분기하는지" 를 명확히 한다.

**구현 패턴** — `menubar/mod.rs:112-121` 의 `let w = window.clone();` 패턴을 그대로 따른다. `on_window_event` 콜로저가 `window` 를 capture-by-move 하므로, 부착 전에 clone 을 떠 두어야 콜로저 내부에서 `w.hide()` 호출이 가능. 예:

```rust
#[cfg(target_os = "macos")]
{
    if let Some(window) = app.get_webview_window("main") {
        let w = window.clone();
        window.on_window_event(move |event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = w.hide();
                tracing::debug!(window = "main", "close-to-hide intercepted");
            }
        });
    } else {
        tracing::warn!(window = "main", "close handler not attached — window missing");
    }
}
```

> 주의 — Tauri 2.11.2 의 `WebviewWindow::on_window_event` 는 `Result` 를 반환하지 않는다. 부착 자체에 대한 에러 처리는 불필요. 경고는 `get_webview_window("main")` 이 `None` 인 경우에만 의미가 있음.

### 4.2 `src-tauri/src/lib.rs` — `RunEvent::Reopen` 후크

현재:
```rust
commands::register(builder)
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
```

변경 후 (의사 코드):
```rust
#[cfg(target_os = "macos")]
use tauri::Manager;  // get_webview_window 가 Manager trait 메서드라 필수.

// ... pub fn run() { ... }

let app = commands::register(builder)
    .build(tauri::generate_context!())
    .expect("error while building tauri application");

app.run(|app_handle, event| {
    #[cfg(target_os = "macos")]
    if let tauri::RunEvent::Reopen { has_visible_windows: _, .. } = &event {
        if let Some(window) = app_handle.get_webview_window("main") {
            let _ = window.show();
            let _ = window.unminimize();
            let _ = window.set_focus();
            tracing::debug!(window = "main", "reopen requested");
        }
    }
    let _ = (app_handle, event);
});
```

- **`use tauri::Manager;` import 필수** — `AppHandle::get_webview_window` 는 `Manager` trait 의 메서드라 import 없이는 컴파일되지 않는다 (현행 `src-tauri/src/lib.rs:1` 에 `use` 구문 없음).
- **import 에 `#[cfg(target_os = "macos")]` 어트리뷰트 부착 필수** — `Manager` 는 macOS 분기 안에서만 쓰이므로, cfg 없이 import 만 두면 비-macOS 빌드에서 unused-import warning → `cargo clippy -- -D warnings` 에서 fail.
- `has_visible_windows` 는 무시 — popup / menubar 가 화면에 있어도 사용자가 의도한 동작은 메인 창 복귀이기 때문.
- macOS 가 아닐 경우 클로저 body 는 `let _ = (app_handle, event);` 만 남음 (no-op).
- 기존 `setup` / `commands::register` 흐름은 그대로 사용. builder 종단부만 두 줄로 분리.

### 4.3 `src-tauri/src/events.rs`
변경 없음. 신규 이벤트 도입 안 함 (FE 가 알아야 할 정보가 없음).

### 4.4 FE
변경 없음. webview 가 살아 있는 상태에서 다시 보이는 것이므로 Zustand store / route / in-flight state 복원 이슈 없음.

---

## 5. 회귀 위험 평가

| 시나리오 | 현재 | 변경 후 | 위험 |
|---|---|---|---|
| 닫기 → Dock 클릭 | 무반응 | 메인 창 복귀 | 의도된 변화 |
| 닫기 → `Cmd+Shift+T` | popup 만 뜸 | popup 만 뜸 (변화 없음) | 없음 |
| 트레이 메뉴 "메인 창 열기" | 정상 동작 | 정상 동작 | `focus_main_and_route` 가 이미 `show()` + `unminimize()` + `set_focus()` 호출 (`menubar/mod.rs:127-133`) — 동일 패턴 |
| 트레이 메뉴 "종료" | `app.exit(0)` | `app.exit(0)` | 없음 |
| `hide_dock_icon: true` (메뉴바 전용) | 닫기 시 트레이 통해서만 복귀 가능 | 동일 (Dock 자체가 없으므로 Reopen 발생 안 함) | 없음 |
| 마지막 창 닫혀도 앱 살아 있음 | 살아 있음 (menubar / popup webview 가 있음) | 살아 있음 | 동일 |
| 닫힘 후 메모리 | webview destroy → 메모리 해제 | webview 유지 → 메모리 보존 | webview 1개 분 메모리 상주. 정상 macOS 앱 동작이라 수용 |

---

## 6. 테스트 전략

### 6.1 자동 회귀 테스트: 없음

본 변경은 Tauri runtime + macOS NSApplication 이벤트 (`RunEvent::Reopen`, `WindowEvent::CloseRequested`) 에 직접 의존한다. 두 이벤트 모두 OS / Tauri 이벤트 루프가 송신하므로 단위 테스트로 재현할 수 없고, Tauri 가 제공하는 mock runtime 도 reopen 시뮬레이션은 지원하지 않는다. pure helper 로 추출할 분기 로직도 너무 단순 (`if let Some(w) = ... { w.show(); w.unminimize(); w.set_focus(); }`) 해서 테스트 가치가 비용을 못 따라간다.

대신 다음을 게이트로 둔다:
- `cargo fmt --check` / `cargo clippy -- -D warnings` / `cargo test` — 컴파일 + 기존 단위 테스트 통과 (회귀 없음 보장).
- §6.2 수동 체크리스트 5항목 — 사람이 실제 동작 확인.

향후 E2E (PRD §14.3) 가 들어오면 그 시점에 Playwright 의 Tauri driver 로 close → reopen flow 를 자동화 검토.

### 6.2 수동 검증 체크리스트 (PR 본문에 복사)

- [ ] `npm run tauri:dev` 실행 후 메인 창 닫기 → Dock 아이콘 클릭 → 메인 창이 같은 입력 / 상태로 복귀.
- [ ] 메인 창 닫기 → `Cmd+Shift+T` → popup 정상 동작 (regression 없음).
- [ ] `hide_dock_icon: true` 설정 → 메인 창 닫기 → 트레이 "메인 창 열기" → 정상 복귀 (regression 없음).
- [ ] 메인 창 닫기 → 트레이 "메인 창 열기" → 정상 복귀.
- [ ] 트레이 "종료" → 앱 완전 종료 (프로세스 사라짐).

추가로 비-macOS 빌드가 깨지지 않는지 확인:
- [ ] `cargo check --target x86_64-unknown-linux-gnu --manifest-path src-tauri/Cargo.toml` (선택적, 로컬 toolchain 있으면).

---

## 7. 비범위 (out of scope)

- popup / menubar 의 close · focus 정책 통일 — 별도 audit 이슈로 분리.
- Windows / Linux 의 close 정책 — v1 은 macOS 전용 (PRD §1).
- 트레이 메뉴 "종료" 와 close 의 의미 분리 — 이미 구현됨 (`MENU_ID_QUIT → app.exit(0)`).
- `RunEvent::ExitRequested` 후크 — 본 이슈와 무관, 현행 유지.

---

## 8. 작업 절차

1. 브랜치 생성: `fix/main-window-dock-reopen` (선택 — 직접 main 위에서 작업해도 무방, 프로젝트 기본 = 단일 브랜치).
2. `src-tauri/src/commands/mod.rs` — close 핸들러 등록 (§4.1, `let w = window.clone()` 패턴).
3. `src-tauri/src/lib.rs` — `use tauri::Manager;` 추가 + builder 종단부 분리 + `RunEvent::Reopen` 후크 (§4.2).
4. `cargo fmt` / `cargo clippy` / `cargo test` 통과 확인.
5. `npm run lint` / `npm run typecheck` / `npm run test` 통과 확인 (FE 무변경이라 통과 기대).
6. `npm run tauri:dev` 으로 §6 수동 체크리스트 5항목 모두 통과.
7. `Agent(subagent_type="quality-gate", prompt="...")` 호출.
8. 커밋 (Conventional Commits, Korean title) — 예: `fix: 메인 창 닫은 뒤 Dock 클릭으로 복귀 가능하도록 수정 (#1)`.
9. PR 생성 (Korean title, 본문에 §6 체크리스트 포함, `Closes #1`).

---

## 9. 관련 코드 (현 시점 기준)

- `src-tauri/src/lib.rs:14` — `run()` 진입점, builder 종단부 변경 대상.
- `src-tauri/src/commands/mod.rs:68-135` — `setup` 클로저, close 핸들러 부착 위치.
- `src-tauri/tauri.conf.json:14-26` — main 윈도우 정의 (구조 변경 없음).
- `src-tauri/src/menubar/mod.rs:126-133` — `focus_main_and_route`, show + unminimize + set_focus 참고 패턴.
- `src-tauri/src/settings/mod.rs:56` — `hide_dock_icon: false` 기본값.
