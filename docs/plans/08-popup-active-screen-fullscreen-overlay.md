# Feature — `Cmd+Shift+T` Popup 을 현재 활성 화면(전체화면 포함) 위에 표시

범위: `popup` 윈도우의 표시 동작 한정. `main` / `menubar` 윈도우, 단축키 파서, FE 컴포넌트는 손대지 않음.
플랫폼: macOS 전용 (앱 자체가 macOS-only).

완료 기준:
- 아래 §7 수동 검증 항목 전부 통과 (특히 **다른 앱의 네이티브 전체화면 위에 popup 이 뜨고 키 입력 포커스를 얻는다**).
- **자동 게이트 (필수, hard pass/fail)**:
  - `npm run lint`
  - `npm run typecheck`
  - `npm run test`
  - `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings`
  - `cargo test --manifest-path src-tauri/Cargo.toml`
- **가이드라인 검토**: Rust 코드 변경 직후 `.claude/CLAUDE.md` "After every code change" 규약대로 `Agent(subagent_type="quality-gate")` 호출.
- **릴리스 빌드 확인 (필수)**: 네이티브 window-level / collection-behavior 호출은 dev 와 packaged(release) 동작이 갈린 선례가 있음(Tauri #5566). `npm run tauri:build` 산출물(DMG/.app)로도 §7 을 재검증.

---

## 0. 사용자 결정 (확정)

| Q | 결정 | 근거 |
|---|---|---|
| "활성 화면" 의 정의 | **커서가 위치한 모니터** | 사용자 확정 (2026-05-29). Spotlight/Raycast 관례이며 기존 `place_on_active_monitor` 가 이미 이 방식. 최전면 앱 key window 기준은 타 앱 윈도우 조회가 필요해 v1 범위 밖 — 채택하지 않음. |

→ 멀티모니터에서 *커서는 A, 전체화면은 B* 인 경우 popup 은 **커서 모니터 A** 에 뜬다. 이 동작이 의도된 사양이다.

---

## 1. 요구사항

> `Command + Shift + T` 단축키로 Popup 을 띄울 때, 현재 활성 화면(전체화면 모드 포함)에 나타나도록 한다.

두 개의 하위 요구사항으로 분해된다:

| # | 요구사항 | 현재 상태 |
|---|---|---|
| R1 | 여러 디스플레이 중 **현재 활성 화면**에 popup 배치 | **이미 구현됨** — `place_on_active_monitor` (커서 모니터 기준) |
| R2 | 다른 앱이 **네이티브 전체화면**일 때 그 Space 위에 popup overlay + 키 포커스 | **미구현** — 본 작업의 핵심 |

---

## 2. 현재 동작 분석 (코드 추적)

### 2.1 단축키 → popup toggle 경로
`src-tauri/src/shortcuts/mod.rs:25-31` — 전역 단축키 핸들러는 `ShortcutState::Pressed` 시 `popup::toggle(&handle)` 한 줄만 호출. 어떤 화면/Space 인지는 신경쓰지 않음.

### 2.2 popup 표시 경로
`src-tauri/src/commands/popup.rs`
- `toggle()` → 보이면 `hide()`, 아니면 `show()`.
- `show()` (45-59): 이미 보이면 `set_focus()` 만; 아니면 `place_on_active_monitor()` → `window.show()` → `window.set_focus()`.
- `place_on_active_monitor()` (22-43): `cursor_position()` → `monitor_from_point()` 로 **커서가 있는 모니터**를 구하고, 그 모니터 중앙(physical px)에 `set_position()`. 실패 시 `primary_monitor()` → `window.center()` 순으로 fallback.

→ **R1(활성 화면 배치)은 이미 충족.** 커서가 위치한 모니터를 "활성 화면"으로 간주한다(§9 가정 참조).

### 2.3 popup 윈도우 설정
`src-tauri/tauri.conf.json` popup 정의:
```
decorations:false, transparent:true, alwaysOnTop:true, skipTaskbar:true, visible:false, shadow:true
```
- `alwaysOnTop:true` → tao 가 `NSFloatingWindowLevel`(=3) 로 올림. 일반 창 위에는 뜨지만, 네이티브 전체화면 Space 의 콘텐츠 위로는 보장되지 않음.
- 전체화면 overlay 에 필요한 `NSWindowCollectionBehaviorFullScreenAuxiliary` 는 어디에서도 설정되지 않음.

### 2.4 activation policy
`src-tauri/src/settings/mod.rs:56` — `hide_dock_icon: false` 기본값 → 기본 `ActivationPolicy::Regular`. (Accessory 는 Dock 숨김 토글 시에만.)

---

## 3. 근본 원인 — 왜 전체화면 위에 안 뜨는가

macOS 네이티브 전체화면 앱은 **전용 Space** 를 점유한다. 일반 윈도우는 자신이 생성된 Space 에만 속하므로, 전체화면 Space 위로는 나타나지 못한다.

`library-docs-fetcher` 리서치로 확인한 핵심 사실(버전: tauri 2.11.2 / tao 0.35.3 / objc2 0.6.4 / objc2-app-kit 0.3.2 — 전부 `Cargo.lock` 에 이미 존재):

1. **`WebviewWindow::set_visible_on_all_workspaces(true)` 만으로는 부족하다.** tao 0.35.3 구현은 `NSWindowCollectionBehavior::CanJoinAllSpaces` **만** 켜고 `FullScreenAuxiliary` 는 켜지 않는다. 다른 앱의 전체화면 Space 위로 뜨려면 `FullScreenAuxiliary` 가 필수 (Tauri #11488, not-planned 로 닫힘).
2. **`alwaysOnTop` 의 floating level(3) 도 부족.** 전체화면 콘텐츠보다 확실히 위에 두려면 더 높은 window level 이 필요.
3. 해결책: 네이티브 `NSWindow` 를 `window.ns_window()` 로 받아 objc2 로 직접
   - `collectionBehavior = CanJoinAllSpaces | FullScreenAuxiliary`
   - `setLevel(NSStatusWindowLevel = 25)`
   를 설정한다. (`cocoa`/`objc` 구버전 API 가 아니라 이미 트리에 있는 **objc2** 사용.)
4. **포커스/Space 전환 주의(Regular policy):** FullScreenAuxiliary 가 없으면 `set_focus()` 가 popup 의 홈 Space 로 Space 를 전환시켜 사용자를 전체화면에서 끌어낸다. FullScreenAuxiliary 를 켜 popup 이 현재(전체화면) Space 에 합류하면, 활성화가 제자리에서 일어나 전환이 발생하지 않는다 — 이것이 본 수정이 포커스 문제까지 해결하는 이유.

---

## 4. 설계 결정

### 채택안 (A) — popup `NSWindow` 에 objc2 로 overlay 속성 직접 설정
- 변경 최소. popup 한 윈도우에만 영향. activation policy/Dock 정책 불변.
- `CanJoinAllSpaces | FullScreenAuxiliary` + `NSStatusWindowLevel` → overlay + 키 포커스 모두 해결될 것으로 기대.
- 기존 `place_on_active_monitor` + `set_focus()` 흐름 그대로 유지.

### 보류안 (B) — `tauri-nspanel` (nonactivating NSPanel) 로 전환
- Raycast/Spotlight 의 "정석" 해법. 앱을 활성화하지 않고도 키 입력 가능.
- 다만 윈도우 종류 자체를 바꾸는 큰 변경 → v1 범위 초과.
- **fallback 으로만 문서화.** §7 검증에서 (A) 가 포커스를 못 얻거나 Space 전환이 남으면 그때 별도 이슈로 (B) 검토.

### 적용 시점
`show()` 의 **not-visible 분기**(실제로 새로 띄우는 경로)에서 `place_on_active_monitor()` 직전에 1회 호출. 매 cold-show 마다 재적용 → #5566 의 release-build 리셋 가능성에 견고. (이미 보이는 상태의 re-focus 분기는 직전 show 에서 적용됐으므로 불필요.)

### Window level 선택
`NSStatusWindowLevel = 25` — 메뉴 막대 아래, 앱 콘텐츠/전체화면 위. (메뉴까지 덮으려면 `NSPopUpMenuWindowLevel = 101` 도 가능하나 25 로 충분.) 코드에 명명 상수로 박아 의도를 드러낸다.

### 4.1 collection behavior 는 clobber 가 아닌 group-safe read-modify-write (Codex 리뷰 반영)

Codex adversarial 리뷰가 "기존 collection behavior 를 통째로 덮어쓴다"고 지적. 소스 검증 결과:

- **현재는 라이브 버그 아님**: `tao 0.35.3` 은 창 생성 시 collection behavior 를 설정하지 않고(유일한 `setCollectionBehavior` 호출은 `set_visible_on_all_workspaces` 내부), 본 계획은 그 API 를 호출하지 않는다 → 팝업의 시작값은 `Default`(0). clobber 와 additive 가 동일.
- **그래도 방어적으로 보존**: 향후 `visibleOnAllWorkspaces` config 추가나 tao 업그레이드 시 외부에서 설정된 비트(`ParticipatesInCycle` 등)를 지우지 않도록 read-modify-write 로 둔다.
- **단, Codex 의 naive `behavior |= ...` 제안은 거부**: collection behavior 는 상호배타 그룹(Spaces / FullScreen)을 가지므로, 기존에 `MoveToActiveSpace` 또는 `FullScreenPrimary` 가 켜져 있으면 단순 OR 이 같은 그룹의 두 비트를 동시에 켜 **충돌 마스크**(동작 미정의)를 만든다. → 각 그룹의 다른 멤버를 **먼저 clear** 한 뒤 원하는 비트를 켠다(§5.2 스니펫).

확인: `objc2-app-kit 0.3.2` 의 `NSWindowCollectionBehavior` 는 bitflags 로 `CanJoinAllSpaces`(1<<0) / `MoveToActiveSpace`(1<<1) / `FullScreenPrimary`(1<<7) / `FullScreenAuxiliary`(1<<8) / `FullScreenNone`(1<<9) 상수를 모두 노출 → 위 스니펫 컴파일 가능.

---

## 5. 구현 계획 (파일별)

### 5.1 `src-tauri/Cargo.toml` — macOS-gated 네이티브 의존성 추가
두 crate 모두 tao/wry 경유로 이미 트리에 존재하므로 컴파일 트리 증가 없음.
```toml
[target.'cfg(target_os = "macos")'.dependencies]
objc2 = "0.6"
objc2-app-kit = { version = "0.3", default-features = false, features = ["NSWindow", "NSResponder"] }
```
> **`default-features = false` 필수**: objc2-app-kit 의 default feature 묶음을 켜면 `objc2-core-video` 가 `Cargo.lock` 에 새로 추가된다(wry 는 이를 끌어오지 않음). `NSWindow` / `NSResponder` feature 만 켜면 기존 트리에 이미 있는 crate 만 쓰여 신규 패키지 0개. security.md 의 의존성 최소화 원칙 준수.
> `objc2-foundation` 은 `NSWindowLevel`(=`isize`) 정수 리터럴만 쓰면 불필요 — 추가하지 않음.

### 5.2 `src-tauri/src/commands/popup.rs` — overlay 헬퍼 추가 + show() 결선

**(a) macOS 구현 + 비-macOS no-op 스텁 (cfg 분기):**
```rust
/// popup 을 다른 앱의 네이티브 전체화면 Space 위에도 띄우기 위한 NSWindow 속성 설정.
/// CanJoinAllSpaces|FullScreenAuxiliary + NSStatusWindowLevel.
/// tao 의 set_visible_on_all_workspaces 는 FullScreenAuxiliary 를 켜지 않아(Tauri #11488)
/// objc2 로 직접 설정한다.
#[cfg(target_os = "macos")]
fn apply_fullscreen_overlay<R: Runtime>(window: &WebviewWindow<R>) -> AppResult<()> {
    use objc2_app_kit::{NSWindow, NSWindowCollectionBehavior};

    // NSStatusWindowLevel — 메뉴 막대 아래, 전체화면 콘텐츠 위.
    const NS_STATUS_WINDOW_LEVEL: isize = 25;

    let ptr = window.ns_window().map_err(AppError::internal)? as *const NSWindow;
    // SAFETY: tao 가 WebviewWindow 수명 동안 NSWindow 를 살려둔다.
    let ns_window: &NSWindow = unsafe { &*ptr };

    // 기존 collection behavior 를 보존하며 필요한 비트만 조정한다(§4.1 참조).
    // macOS collection behavior 는 상호배타 그룹으로 묶여 있어
    //   Spaces:     CanJoinAllSpaces ↔ MoveToActiveSpace
    //   FullScreen: FullScreenPrimary ↔ FullScreenAuxiliary ↔ FullScreenNone
    // 단순 OR 은 같은 그룹 비트를 동시에 켜 충돌 마스크를 만든다(Apple 동작 미정의).
    // 각 그룹의 다른 멤버를 먼저 제거한 뒤 원하는 비트를 켠다.
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
```

**(b) `show()` not-visible 분기에서 placement 직전 호출:**
```rust
    apply_fullscreen_overlay(&window)?;
    place_on_active_monitor(&window)?;
    window.show().map_err(AppError::internal)?;
    window.set_focus().map_err(AppError::internal)?;
```
> `set_focus()` 유지. (A) 로 포커스가 안정적이지 않으면 §4 보류안(B) 또는 `orderFrontRegardless` 로 escalate.

### 5.3 FE / 설정 / 이벤트
변경 없음. 단축키 경로(`shortcuts/mod.rs`), `toggle`/`hide`, `POPUP_OPENED` 이벤트, popup React 컴포넌트 모두 그대로.

---

## 6. 테스트 계획

### 6.1 자동 (단위)
- objc2 호출은 실제 `NSWindow` 가 필요해 단위 테스트 불가 → 검증은 §7 수동/릴리스 빌드로.
- 기존 `center_within` / `center_on_monitor` 모니터 중앙 계산 테스트 **그대로 유지**(R1 회귀 방지).
- `apply_fullscreen_overlay` 의 비-macOS 스텁이 `Ok` 인지 확인하는 trivial 테스트는 가치 낮아 생략. 명명 상수 `NS_STATUS_WINDOW_LEVEL = 25` 의 의미는 주석으로 충분.

### 6.2 회귀 영향 표면
- popup 외 윈도우 무영향.
- `place_on_active_monitor` 로직 불변 → 멀티모니터 배치 회귀 없음.

---

## 7. 수동 검증 (핵심 — hard pass/fail)

dev(`npm run tauri:dev`)와 release(`npm run tauri:build` 산출물) **양쪽**에서:

1. **싱글 모니터 일반 데스크탑**: 임의 앱 위에서 `Cmd+Shift+T` → popup 이 화면 중앙에 뜨고 입력란에 바로 타이핑 가능.
2. **네이티브 전체화면 위 (핵심)**: Safari/동영상 등을 초록 버튼으로 네이티브 전체화면 → `Cmd+Shift+T` → **전체화면에서 빠져나가지 않고** 그 위에 popup overlay, 한국어/중국어 입력 시 키 입력이 popup 으로 들어감.
3. **멀티 모니터**: 커서를 보조 모니터에 둔 상태로 단축키 → 보조 모니터 중앙에 popup. 한 모니터만 전체화면일 때 커서 있는 모니터에 뜨는지 확인.
4. **toggle**: 떠 있는 상태에서 다시 단축키 → 숨김. `Esc` 로도 닫힘(기존 동작 유지).
5. **Dock 숨김 모드(Accessory)**: 설정에서 Dock 아이콘 숨김 ON 후 1·2 재확인 — overlay/포커스 정상.
6. **포커스 회귀**: popup 닫은 뒤 직전 전체화면 앱이 그대로 전체화면을 유지하는지(Space 전환 잔상 없음).
7. **collection-behavior 부작용 회귀(§4.1)**: Mission Control / Exposé 진입 시 popup 이 비정상 배치·잔류하지 않는지, 일반(비전체화면) 데스크탑에서 popup 표시/숨김·Space 이동이 기존과 동일한지. group-safe read-modify-write 로 무관 비트가 보존됐는지 간접 확인.

---

## 8. 리스크 / 가정 / fallback

### 결정 (확정 — §0 참조)
- **"활성 화면" = 커서가 위치한 모니터.** 사용자 확정. 이미 `place_on_active_monitor` 에 구현됨. 멀티모니터+전체화면에서 커서와 전체화면 모니터가 다를 때 popup 은 커서 모니터에 뜨며, 이는 의도된 사양이다.

### 리스크
| 리스크 | 대응 |
|---|---|
| (A) 로도 전체화면에서 키 포커스 미획득 / Space 전환 잔존 | 보류안(B) `tauri-nspanel` 또는 `orderFrontRegardless` 로 escalate (별도 이슈). |
| dev OK / release 에서 level·behavior 리셋(#5566) | show() 마다 재적용으로 1차 방어. release 빌드 필수 검증(§7). 그래도 리셋되면 setup() 시점 추가 적용 병행. |
| tao 업그레이드로 `set_visible_on_all_workspaces` 가 FullScreenAuxiliary 까지 켜게 바뀜 | 그 경우 objc2 패치 불필요해질 수 있음 — 업그레이드 시 재확인. |
| `set_focus` 의 macOS 2.3+ 회귀(#12834) | overlay 가 안정적이면 무관. 문제 시 `orderFrontRegardless` 대체. |

### 보안/프라이버시
네트워크·로깅·DB 영향 없음. 순수 윈도우 표시 동작 변경 → PRD §12 제약 무관.

---

## 9. 작업 순서 (실행 시)

1. `tasks/todo.md` 에 체크리스트 기록.
2. `Cargo.toml` 의존성 추가 (§5.1).
3. `popup.rs` 헬퍼 + show() 결선 (§5.2).
4. `cargo fmt` / `cargo clippy -D warnings` / `cargo test` 통과.
5. `Agent(subagent_type="quality-gate")` 호출.
6. dev + release 빌드로 §7 수동 검증.
7. 커밋(Conventional Commits, Korean title): `feat(popup): 전역 단축키 popup 을 전체화면 포함 활성 화면 위에 표시`.
