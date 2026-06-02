# Bugfix — 듀얼 모니터에서 popup 을 커서가 있는 화면에 표시

범위: `Cmd+Shift+T` 로 popup 을 새로 표시할 때의 macOS 모니터 선택과 좌표 변환, 그리고
popup resize 위치 복원 경로의 경쟁 상태를 점검한다.

이 문서는 `docs/plans/08-popup-active-screen-fullscreen-overlay.md` 의 후속 계획이다. `08` 에서
구현한 nonactivating `NSPanel`, 전체화면 Space overlay, 키 포커스 정책은 유지한다.

---

## 0. 현상

듀얼 모니터 환경에서 커서를 보조 모니터에 둔 채 `Cmd+Shift+T` 를 눌러도 popup 이 커서가
있는 데스크탑에 나타나지 않을 수 있다.

PRD §6.3 과 기존 결정(`docs/plans/08-popup-active-screen-fullscreen-overlay.md` §0)에 따르면
활성 화면은 **커서가 위치한 모니터**다. popup 은 해당 모니터 중앙에 나타나야 한다.

### 관찰 가능한 증상

1. 보조 모니터에 커서를 두고 popup 을 열어도 primary 모니터에 나타난다.
2. 화면 배율이 다른 모니터 사이에서 popup 이 중앙이 아닌 위치에 나타나거나 화면 밖으로
   밀릴 수 있다.
3. 번역 결과가 streaming 중일 때 popup 을 닫고 다른 화면에서 다시 열면 이전 위치로
   되돌아갈 가능성이 있다.
4. popup 이 이미 보이는 상태에서 다른 화면으로 커서를 옮기고 단축키를 누르면 popup 은
   이동하지 않고 숨겨진다. 이는 현재 toggle 정책이며, 수정 여부는 별도 제품 결정이다.

---

## 1. 현재 실행 경로

### 1.1 단축키에서 popup 표시까지

```text
Cmd+Shift+T
  → src-tauri/src/shortcuts/mod.rs::install handler
  → src-tauri/src/commands/popup.rs::toggle
  → popup 이 숨겨져 있으면 show
  → cold_show
  → run_on_main_thread
  → cold_show_on_main
  → cocoa_placement
  → NSWindow::setFrameTopLeftPoint
  → orderFrontRegardless + makeKeyWindow
```

`cold_show_on_main` 의 동기 위치 적용은 필요하다. Tauri/tao 의 비동기 `set_position` 보다
먼저 프레임을 설정해 전체화면 Space 에 stale frame 으로 나타나는 회귀를 막기 때문이다.

### 1.2 현재 모니터 선택

`src-tauri/src/commands/popup.rs:95-99`

```rust
let active = window
    .cursor_position()
    .ok()
    .and_then(|pos| window.monitor_from_point(pos.x, pos.y).ok().flatten())
    .or_else(|| window.primary_monitor().ok().flatten());
```

cursor 기반 모니터 판정이 실패하면 오류를 노출하지 않고 primary 모니터로 fallback 한다.
따라서 좌표 계약이 깨지면 사용자에게는 "항상 primary 화면에 뜬다"는 현상으로 보인다.

### 1.3 현재 Cocoa 좌표 변환

`src-tauri/src/commands/popup.rs:109-127`

1. 선택된 monitor 의 위치, 크기와 popup 크기를 physical px 로 가져온다.
2. `center_on_monitor` 로 physical px 기준 좌상단을 계산한다.
3. 전체 좌표를 popup 윈도우의 현재 `window.scale_factor()` 로 나눈다.
4. primary 모니터 높이를 기준으로 Y축을 뒤집어 Cocoa points 로 바꾼다.

이 계산은 모든 모니터가 같은 배율일 때는 맞을 수 있지만, 서로 다른 배율의 모니터를 하나의
전역 physical 좌표계처럼 취급한다.

---

## 2. 원인 가설

### H1. Tauri/tao cursor 좌표(physical)와 monitor 판정 좌표(points)의 단위가 어긋난다

**우선순위: 가장 높음. 소스로 확정됨(§11 검증 기록). 즉시 수정한다.**

고정된 의존성:

| crate               | version  |
| ------------------- | -------- |
| `tauri`             | `2.11.2` |
| `tauri-runtime-wry` | `2.11.2` |
| `tao`               | `0.35.3` |

`tao 0.35.3` macOS 구현(소스 확인):

- `platform_impl/macos/util/mod.rs:101-106 cursor_position`
  - `NSEvent.mouseLocation`(Cocoa **points**, 좌하단 원점)을 읽는다.
  - `.to_physical(primary_monitor().scale_factor())` 로 **primary 배율을 곱한 physical px** 를 반환한다.
- `platform_impl/macos/monitor.rs:163-173 from_point`
  - 전달받은 좌표를 `CGDisplayBounds`(**points**, 좌상단 원점)와 `CGRectContainsPoint` 로 직접 비교한다.
- `tauri-runtime-wry-2.11.2/src/lib.rs:3125,3144` 의 `monitor_from_point`/`cursor_position` 은
  두 값을 변환 없이 tao 로 pass-through 한다.

즉 `cursor_position()` 은 `points × primary_scale`, `monitor_from_point()` 은 `points` 를 기대한다.
primary 배율이 1 보다 크면(= Retina primary) 커서 physical 좌표가 어느 display 의 points bounds
에도 들지 않아 `None` 이 되고, 앱 코드가 primary fallback 을 선택한다.

> **범위 정정**: 이 버그는 엄밀히 "mixed-DPI" 가 아니라 **primary 가 non-1x(Retina)** 인 모든
> 구성에서 발생한다. 듀얼 Retina(둘 다 2x)에서도 보조 화면 커서는 `points×2` 가 되어 매칭이
> 깨진다. 단일 Retina 모니터에서는 매칭이 깨져도 fallback 이 곧 primary 라서 결과가 우연히 맞아
> 증상이 가려진다(기존 테스트가 놓친 이유). MacBook 내장 Retina + 외장 모니터는 가장 흔한
> 재현 구성이다. §5.1 테스트 매트릭스 4조합은 이 범위를 모두 커버하므로 그대로 유지한다.

> 1차 원인 문장: `popup.rs::cocoa_placement` 가 tao 의 `cursor_position()`(physical=points×primary_scale)
> 결과를 points 기준 `monitor_from_point()` 에 그대로 전달하고, 매칭 실패를 primary fallback 으로
> 숨기기 때문에 Retina primary 환경에서 popup 이 커서가 없는 primary 화면에 나타난다.

### H2. 선택된 화면과 popup 의 기존 화면 배율을 혼합한다

**우선순위: 높음. 소스로 확정됨. H1 과 함께 제거한다.**

`src-tauri/src/commands/popup.rs:112-127`

```rust
let scale = window.scale_factor().map_err(AppError::internal)?;
```

`mon_pos`/`mon_size` 는 대상 모니터 배율 기준 physical(`points × target_scale`)이고
`outer` 는 popup 현재 화면 배율 기준 physical 인데, `physical_top_left_to_cocoa_point` 는
이 좌표를 popup 의 **현재** 화면 배율(`window.scale_factor()`)로 나눈다. 대상 배율과 popup
현재 배율이 다르면(예: popup 이 Retina 화면에 있었고 다음에 일반 DPI 보조 화면으로 가야 함)
차원이 어긋나 중앙 배치가 크게 틀어진다.

> Y 반전 기준 자체(`primary.size().height / scale == CGDisplay::main().pixels_high()`)는 tao 의
> `window_position` 규약과 일치해 정상이다. 버그는 그 변환에 **혼합 배율 physical 좌표**를
> 입력한다는 데 있다. H1 미수정 상태에서는 fallback→primary 로 대상==popup 화면이 되어 배율이
> 우연히 일치, 증상이 "primary 중앙 배치(화면만 틀림)" 로 나타난다. H1 을 고쳐 대상이 비-primary
> 가 되거나 popup 의 직전 화면 배율이 대상과 다르면 H2 가 독립적으로 드러난다. 따라서 두 가설을
> 같은 PR 에서 좌표계 통일(§4.1)로 함께 제거한다.

### H3. 숨김/재표시와 FE resize 위치 복원이 경쟁한다

**우선순위: 중간. 구조적으로 확정(아래). H1/H2 와 같은 PR 에서 함께 수정한다.**

구조 확인: `popup.tsx` 는 `POPUP_OPENED` 만 listen 하고 `POPUP_CLOSED` 소비처는 없다(정의만
존재). `cancelled` 플래그는 effect cleanup(=`output` 변경 또는 unmount)에서만 set 되는데,
`hide()`=`orderOut` 은 webview 를 unmount 하지 않으므로 hide→재표시 사이에 set 되지 않는다.
따라서 `await win.setSize` 도중 popup 이 숨겨졌다가 다른 화면에서 다시 열리면 `cancelled` 은
여전히 false 이고, 늦게 완료된 `setPosition(topLeft)` 가 새 배치를 이전 위치로 덮는다.

`src/windows/popup/popup.tsx:84-115`

```ts
const topLeft = await win.outerPosition();
await win.setSize(new LogicalSize(480, target));
if (!cancelled) await win.setPosition(topLeft);
```

번역 chunk 로 `output` 이 바뀔 때마다 비동기 resize 가 실행된다. `POPUP_CLOSED` 이벤트는
Rust 에서 emit 되지만 FE 는 소비하지 않는다. `outerPosition()` 이후 popup 이 닫히거나 다른
화면에서 다시 열린 경우, 늦게 완료된 `setPosition(topLeft)` 가 새 배치를 이전 위치로
덮어쓸 수 있다.

### H4. toggle 정책은 화면 간 summon 동작을 제공하지 않는다

**우선순위: 제품 결정 필요. 이번 버그 수정에 자동 포함하지 않는다.**

`src-tauri/src/commands/popup.rs::toggle` 은 popup 이 보이면 항상 `hide()` 한다. 다른 화면에
떠 있는 popup 을 현재 커서 화면으로 옮기는 동작은 없다.

현재 정책을 유지하면:

1. popup 이 다른 화면에 보이는 상태에서 `Cmd+Shift+T` → 숨김
2. 다시 `Cmd+Shift+T` → 현재 커서 화면에 새로 표시

단축키를 "toggle" 이 아니라 "summon-or-hide-on-current-monitor" 로 바꿀지는 별도 UX 결정으로
남긴다.

---

## 3. 기존 테스트가 놓친 이유

`src-tauri/src/commands/popup.rs` 의 기존 Rust 테스트 7개는 통과한다.

- `center_on_monitor` 의 산술 검증
- 동일 배율의 secondary origin offset 검증
- physical → Cocoa 변환 시 Y축 반전 검증
- 단일 Retina 배율 검증

하지만 아래 계약은 검증하지 않는다.

1. `cursor_position()` → `monitor_from_point()` 실제 연결
2. primary 와 secondary 의 배율이 다른 경우
3. popup 이 이전 화면에서 가진 배율과 목표 화면 배율이 다른 경우
4. 숨김/재표시와 streaming resize 가 겹치는 경우

`tests/e2e/translate-popup-shortcut.spec.ts` 도 assertion 계획만 있고 `test.skip` 상태다.

---

## 4. 수정 원칙

### 4.1 macOS popup 배치는 Cocoa points 좌표계 하나로 계산한다

AppKit `NSWindow::setFrameTopLeftPoint` 를 호출할 것이므로 중간에 Tauri physical px 좌표로
왕복하지 않는다.

macOS main thread 에서:

1. `NSEvent::mouseLocation` 으로 Cocoa points 커서 위치를 읽는다. `mouseLocation` 과
   `NSScreen.frame` 은 동일한 전역 Cocoa 공간(좌하단 원점, +y 위)이라 배율이 개입하지 않는다.
2. `NSScreen::screens(mtm)` 중 `frame` 이 커서를 포함하는 화면을 찾는다.
3. 대상 `NSScreen.frame` 과 `NSWindow.frame().size`(둘 다 points)로 중앙 좌상단을 Cocoa points
   로 직접 계산한다. `setFrameTopLeftPoint` 도 같은 전역 공간을 받으므로 Y 반전·`primary_points_high`
   가 더 이상 필요 없다(helper 입력에 배율 인자 없음).
4. 중앙 배치 전에 선택 화면의 80% cap(`POPUP_MAX_HEIGHT_RATIO`)으로 popup 높이를 제한한다
   (`capped_centered_placement` → 필요 시 `setContentSize`). 다른 높이의 화면에서 다시 열 때도
   (output 미변경 → FE resize 미발생) PRD §6.3 의 80% 제한을 cold-show 가 직접 보장한다.
5. `NSWindow::setFrameTopLeftPoint` 를 동기 호출한다.
6. 커서가 어느 화면에도 없으면(rare) `NSScreen::screens(mtm)[0]` 으로 fallback 하고 `tracing::debug!`
   를 남긴다. fallback 대상은 `NSScreen::mainScreen`(포커스 화면, 가변)이 **아니라** `screens[0]`
   (메뉴바·좌표 원점 화면)을 쓴다 — 결정적이어야 하기 때문이다.

화면 중앙 기준은 `frame`(전체 화면)을 쓴다. 기존 동작(모니터 전체 중앙)과 일치하며, 메뉴바·Dock
영역을 제외하려면 `visibleFrame` 으로 바꿀 수 있으나 v1 에서는 동작 보존을 위해 `frame` 유지.

이 방식은 모니터별 backing scale 차이를 계산에서 완전히 제거한다.

### 4.2 fullscreen overlay 정책은 유지한다

아래 동작은 기존 전체화면 회귀 방지에 필요하므로 변경하지 않는다.

- `HyPopupPanel` class-swizzle
- `NSWindowStyleMask::NonactivatingPanel`
- `CanJoinAllSpaces | FullScreenAuxiliary`
- `NSStatusWindowLevel = 25`
- 위치를 먼저 동기 적용한 뒤 `orderFrontRegardless()`
- `makeKeyWindow()`

### 4.3 resize 크기 변경 + top-left 보존은 Rust 가 메인스레드에서 동기로 처리한다

FE 에서 `setSize`→`setPosition` 으로 나눠 호출하면 둘 다 tao 의 `set_content_size_async`/
`set_frame_top_left_point_async`(= `DispatchQueue::main().exec_async`)로 enqueue 되어 적용 완료를
JS 가 기다릴 수 없다. 그래서 늦게 실행된 이전 세대의 `setContentSize` 가 close→reopen 직후의
`cold_show_on_main` 배치를 미는 경합이 남는다(FE generation 가드로는 enqueue 된 native 호출을
취소할 수 없음). 좌상단 스냅샷을 async 로 미리 잡아 나중에 적용하는 구조 자체가 race 의 원인이다.

따라서 크기 변경과 top-left 보존을 **Rust `resize_popup` 커맨드**로 옮겨 `run_on_main_thread`
한 클로저에서 동기로 적용한다.

1. FE 는 monitor 80% cap 으로 목표 height 만 계산해 `invoke('resize_popup', { height })` 한다.
   더 최신 output 에 추월되면(effect cleanup 의 `cancelled`) invoke 자체를 건너뛴다(coalesce).
2. Rust 는 메인스레드에서 `ns_window.frame()` 으로 **현재** top-left 를 읽어 `setContentSize`
   후 `setFrameTopLeftPoint` 로 되돌린다. async 스냅샷이 아니라 적용 시점의 live 좌표라 stale
   복원이 원천 차단된다.
3. `resize_popup` 의 Rust 상한도 `screen.frame * POPUP_MAX_HEIGHT_RATIO`(80%)다. cold-show·FE 와
   같은 기준이라, 지연된 resize 가 cold-show 의 80% cap 을 다시 넘지 못하고(신뢰 불가 IPC 도 동일
   상한), 정책 기준이 세 경로에서 일치한다.
4. `resize_popup` 와 `cold_show_on_main` 은 같은 `run_on_main_thread` 경로라 FIFO 로 직렬화된다.
   순서가 어떻든 결과가 옳다: resize 가 먼저면 cold_show 가 마지막에 재중앙배치, cold_show 가
   먼저면 resize 가 그때의 중앙 top-left 를 읽어 보존.

이 구조에서 FE 의 generation/visible/`POPUP_CLOSED` 추적은 더 이상 필요 없다(잘못된 레이어의
보정이었음). top-left 보존은 Rust 단위 테스트(`top_left_to_preserve`)로 회귀를 막는다.

### 4.4 다른 높이 화면으로 재오픈 시 output 높이를 복구한다

cold-show 는 높이를 화면 80% 로 **축소만** 한다(축소는 PRD 보장). 작은 화면에서 줄어든 popup 을
다시 큰 화면에서 열면(`output` 미변경 → output effect 미발생) 긴 결과에 맞는 높이가 복구되지
않는다(scroll 가능하지만 자동 확장이 안 됨). 그래서 `POPUP_OPENED` 수신 시 FE 가 현재 화면 기준
으로 높이를 다시 계산해 `resize_popup` 을 호출한다(`applyResize` 공유 콜백, output 은 store 에서
직접 읽어 listener stale 회피). resize_popup 이 top-left 를 보존하므로 streaming 과 동일하게 위에서
아래로 확장된다.

---

## 5. 구현 계획

### 5.1 Phase A — runtime probe 로 H1/H2 확인

수정 전 dev 빌드에서 popup cold-show 경로에 임시 debug 로그를 추가한다. 로그에는 원문이나
번역 결과를 포함하지 않는다.

필수 필드:

```text
cursor_cocoa
tauri_cursor_physical
screens[{ frame_points, backing_scale }]
selected_screen
popup_frame_points
fallback_used
```

검증 구성:

| primary  | secondary | 기대           |
| -------- | --------- | -------------- |
| Retina   | Retina    | 커서 화면 선택 |
| Retina   | 일반 DPI  | 커서 화면 선택 |
| 일반 DPI | Retina    | 커서 화면 선택 |
| 일반 DPI | 일반 DPI  | 커서 화면 선택 |

H1/H2 가 확인되면 임시 로그는 제거하거나 `tracing::debug!` 로 최소화한다.

> Phase A 는 H2/H3 의 정확한 런타임 수치 확보·재현 검증용이다. H1/H2 자체는 §11 에서 소스로
> 확정됐고 Retina-primary Mac 에서 결정적으로 재현되므로, 수정을 probe 가용성에 묶지 않는다.
> Retina primary Mac 이 있으면 §5.2 수정을 먼저 적용하고 Phase A 는 수동 검증(§6.3)으로 갈음한다.

### 5.2 Phase B — `src-tauri/src/commands/popup.rs`

1. `cocoa_placement` 를 AppKit Cocoa points 기반 helper 로 교체한다.
2. `window.cursor_position()`, `window.monitor_from_point()`, `window.scale_factor()`,
   `primary.size().height / primary.scale_factor()` 의 배치 경로 의존을 제거한다.
3. `NSEvent.mouseLocation` 과 `NSScreen.screens` 를 main thread 에서만 호출한다.
4. pure helper 를 분리한다.

예상 helper 계약(둘 다 순수 함수, 단위 테스트 대상):

```rust
// 커서를 포함하는 화면 인덱스. 없으면 None → 호출부가 screens[0] fallback.
fn screen_index_at(
    frames: &[(f64, f64, f64, f64)], // (x, y, w, h) Cocoa points
    cursor_x: f64,
    cursor_y: f64,
) -> Option<usize>

// 대상 화면 frame + popup 크기 → setFrameTopLeftPoint 용 좌상단(Cocoa points).
fn center_top_left_in_cocoa_points(
    screen_x: f64,
    screen_y: f64,
    screen_w: f64,
    screen_h: f64,
    popup_w: f64,
    popup_h: f64,
) -> (f64, f64)
//   x     = screen_x + (screen_w - popup_w) / 2
//   top_y = screen_y + (screen_h + popup_h) / 2   // Cocoa +y 위 → 좌상단은 bottom+height
```

5. `src-tauri/Cargo.toml` 의 `objc2-app-kit` feature 에 `NSEvent`, `NSScreen` 을 추가한다(확인:
   현재 `["NSWindow","NSResponder","NSPanel"]` 만 활성). 두 feature 는 `objc2-foundation/NSArray`,
   `NSGeometry`, `objc2-core-foundation` 을 transitively 켜므로 `NSScreen::screens` 가 돌려주는
   `NSArray<NSScreen>` iterate 와 `NSRect` 필드 접근에 충분하다. 새 crate 또는 broad feature
   bundle 은 추가하지 않는다.
6. `NSScreen` 은 `MainThreadOnly` 라 `screens(mtm)` 에 `MainThreadMarker` 가 필요하다.
   `cold_show_on_main` 은 `run_on_main_thread` 로 메인스레드가 보장되므로 SAFETY 주석과 함께
   `MainThreadMarker::new_unchecked()` 로 얻는다(파일의 기존 unsafe 규약과 일치).
   `NSEvent::mouseLocation()` 은 인자 없는 class method 라 marker 불필요.

### 5.3 Phase C — resize 를 Rust 로 이동

`src-tauri/src/commands/popup.rs` + `src-tauri/src/commands/mod.rs`:

1. `resize_popup { height }` 커맨드 추가. macOS 는 `run_on_main_thread` 에서 `ns_window.frame()`
   → `top_left_to_preserve` → `setContentSize` → `setFrameTopLeftPoint` 를 동기 적용한다.
   비-macOS 는 `current_monitor` 기준 cap 으로 `set_size` fallback(컴파일 호환).
2. `top_left_to_preserve(origin_x, origin_y, frame_height)` 순수 helper 로 분리(테스트 대상).
3. **IPC 입력 검증**: `height` 는 신뢰 불가 웹뷰 입력이므로 `clamp_popup_height(height, cap)` 로
   sanitize 한다 — 비유한값은 최소 높이로, `[POPUP_MIN_HEIGHT, cap]` 로 clamp. cap 은
   `resize_cap_for_screen(screen_height)`: 화면을 알면 `frame.height * POPUP_MAX_HEIGHT_RATIO`(80%,
   cold-show·FE 와 동일), screenless 면 `POPUP_MIN_HEIGHT`(growth 거부 — 이전 4000pt fallback 제거).
4. `mod.rs` invoke_handler 에 `popup::resize_popup` 등록. Rust 에 seq/generation state 는 두지
   않는다(latest-wins 는 FE single-flight 로 보장 — 6번. 전역 static 은 webview reload 시
   FE 카운터와 어긋나는 새 결함을 만들어 배제).

`src/windows/popup/popup.tsx` + `src/windows/popup/resize.ts`:

5. `resize.ts` 는 `computePopupHeight`(순수) + `createPopupResizer`(인스턴스별 single-flight 클로저)
   만 남긴다. 기존 generation/visible/`POPUP_CLOSED`/`setPosition` 로직은 제거한다(Rust 로 이동).
6. **latest-wins(single-flight)**: `createPopupResizer` 가 resize_popup 호출을 직렬화한다 — 한 번에
   하나만 in-flight 로 보내고 대기 중 target 은 최신값으로 coalesce. 직렬화로 enqueue 순서 = 호출
   순서 → resize_popup 의 FIFO 적용 → 최신 output 높이가 최종 적용된다(Rust 전역 state 불필요).
   부분 실패도 견딘다: 다음 작업을 잇기 전에 직전 chain 의 rejection 을 `catch` 로 흡수해, resize
   가 한 번 실패해도 queue 가 영구히 멈추지 않는다(P1-1). 또한 대기 중 target 에 `isCancelled` 를
   함께 저장해 invoke 직전 다시 검사하므로, enqueue 후 effect cleanup 으로 취소된 stale resize 는
   실행되지 않는다(P2-1).
7. popup window 의 Rust 커맨드 계약은 `src/features/popup/ipc.ts` typed wrapper(`resizePopup(height)`,
   `hidePopup()`)로 고정한다 — entrypoint 에 magic command string 을 두지 않는다(아키텍처: 컴포넌트는
   `invoke()` 직접 호출 금지). `popup.tsx` 의 공유 `applyResize` 콜백이 monitor cap 으로 height 를
   계산해 single-flight resizer(`useMemo` 인스턴스)로 `resizePopup` 을 호출한다. output
   effect(streaming)와 `POPUP_OPENED` listener(재오픈 시 현재 화면 기준 높이 복구) 양쪽에서
   호출한다 — 재오픈 시 output 은 store 에서 직접 읽어 listener stale 을 피한다.

### 5.3.1 최소 권한 — popup capability 정리

FE 가 더 이상 `set_size`/`set_position` 을 호출하지 않으므로 `capabilities/popup-window.json` 에서
`core:window:allow-set-size`, `core:window:allow-set-position` 를 제거하고 `allow-start-dragging`
만 남긴다. 이 두 권한은 plugin window 의 `get_window(window, label)` 가 호출자가 넘긴 `label` 로
대상 윈도우를 찾기 때문에(`tauri-2.11.2/src/window/plugin.rs:13`), popup renderer 가 침해되면
`label: "main"`/`"menubar"` 로 다른 앱 윈도우까지 조작할 수 있다(window-scope 가 아님). 드래그는
`start-dragging` 으로 유지된다.

### 5.4 Phase D — toggle UX 결정

이번 수정과 분리한다. 필요하면 별도 계획에서 아래 정책을 비교한다.

| 정책             | 동작                                              |
| ---------------- | ------------------------------------------------- |
| 현재 toggle 유지 | 보이는 popup 은 첫 단축키에서 숨김                |
| summon 우선      | 커서 화면과 popup 화면이 다르면 이동, 같으면 숨김 |
| 항상 summon      | 보이는 popup 도 현재 커서 화면으로 이동하고 focus |

---

## 6. 회귀 테스트 계획

### 6.1 Rust unit

`src-tauri/src/commands/popup.rs`

`center_top_left_in_cocoa_points`:

1. Cocoa points 기준으로 primary 화면 중앙 좌상단을 계산한다.
2. 우측 secondary 화면 origin 을 보존한다.
3. 좌측 secondary 화면의 음수 origin 을 보존한다.
4. 위/아래로 배치된 secondary 화면의 Y origin 을 보존한다.
5. 화면 배율은 helper 입력에 없음을 확인한다. 같은 points frame 은 배율과 무관하게 같은 결과.

`screen_index_at`:

6. 커서가 든 화면 인덱스를 반환한다(다중 화면).
7. 음수 origin 화면(좌/하측 배치) 안의 커서도 매칭한다.
8. 어느 화면에도 없으면 `None`(호출부 screens[0] fallback).

기존 physical 변환 helper(`center_on_monitor`, `physical_top_left_to_cocoa_point`)와 그 테스트는
새 좌표 계약으로 대체해 제거한다.

`top_left_to_preserve`:

9. resize 직전 top-left(`origin + 현재 높이`)를 보존한다 — height 가 바뀌어도 top 모서리 고정.
10. 음수 origin(좌/하측 화면)도 그대로 반영한다.

`clamp_popup_height`(IPC 입력 검증):

11. 정상값은 그대로, 음수·0·최소 미만은 `POPUP_MIN_HEIGHT` 로.
12. 매우 큰 유한값은 max 로, 비유한값(NaN/Inf)은 `POPUP_MIN_HEIGHT` 로.
13. max < min 인 비정상 화면값도 최소로 수렴.

`capped_centered_placement`(cold-show 재배치):

14. 큰 화면에서 커진 popup 을 작은 화면에서 다시 열면 80% cap 으로 줄이고 그 높이로 중앙배치.
15. cap 안의 높이는 그대로 두고 그 높이로 중앙배치.

`resize_cap_for_screen`(IPC 상한):

16. 화면을 알면 80% cap, screenless(None)면 `POPUP_MIN_HEIGHT`(4000 fallback 우회 차단).

### 6.2 Frontend unit

`src/windows/popup/resize.test.ts`.

`computePopupHeight`:

1. 빈 output 은 360 하한.
2. output 길이에 비례해 커지되 monitor 80% cap 안.
3. 큰 output 은 80% cap 으로 clamp.
4. 같은 긴 output 을 더 큰 화면에서 재측정하면 더 큰 높이로 복구된다(P2-1 재오픈 확장).

`createPopupResizer`(single-flight):

5. 추월되지 않으면 계산된 height 로 `resize` 를 1회 호출한다.
6. 이미 cancelled 면 `resize` 를 호출하지 않는다.
7. 겹친 호출을 직렬화해, 먼저 시작된 resize 가 늦게 끝나도 마지막에 최신 height 가 적용된다
   (latest-wins). 이로써 P2-1 의 resize 순서 역전을 막는다.
8. 첫 `resize` 가 reject 돼도 queue 가 복구돼 다음 요청이 실제로 적용된다(P1-1).
9. enqueue 후 취소된 대기 요청은 in-flight resize 가 끝나도 invoke 되지 않는다(P2-1 stale 차단).

`src/features/popup/ipc.ts` typed wrapper:

10. `resizePopup(height)` 는 `resize_popup` 을 `{ height }` payload 로 invoke 한다.
11. `hidePopup()` 는 `hide_popup` 을 payload 없이 invoke 한다.

> top-left 보존(close→reopen stale 차단)은 Rust `top_left_to_preserve` 단위 테스트와 §6.3
> 수동 검증 #3·#6 으로 검증한다. FE async mock 으로는 tao 내부 `exec_async` queue 를 모델링할
> 수 없어 이 경합을 단위로 재현하지 못한다(그래서 Rust 동기 적용으로 옮겼다).

### 6.3 E2E / 수동 검증

`tests/e2e/translate-popup-shortcut.spec.ts` 의 skip 해제는 macOS 권한과 다중 화면 제어가 가능한
테스트 harness 를 마련한 뒤 수행한다. 그 전까지 아래 항목은 dev와 release `.app` 양쪽에서
수동 hard pass/fail 로 확인한다.

1. primary 커서 → `Cmd+Shift+T` → primary 중앙 표시.
2. secondary 커서 → `Cmd+Shift+T` → secondary 중앙 표시.
3. Retina ↔ 일반 DPI 양방향 이동 후 각각 올바른 화면 중앙 표시.
4. secondary 에 네이티브 전체화면 앱이 있을 때 popup overlay + textarea focus 유지.
5. `Esc` 로 닫고 반대 화면에서 다시 열었을 때 새 화면 중앙 표시.
6. 번역 streaming 중 `Esc` 로 닫고 반대 화면에서 다시 열어도 이전 위치로 돌아가지 않음.
7. 긴 결과로 높이가 커져도 popup top-left 가 흔들리지 않음.
8. Dock 아이콘 숨김 ON/OFF 양쪽에서 1-7 반복.
9. Mission Control / Space 이동 후 popup 잔류나 잘못된 Space 전환이 없음.

---

## 7. 자동 검증

변경 후 저장소 루트에서 실행한다.

```bash
npm run lint
npm run typecheck
npm run test
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
npm run tauri:build
```

`npm run tauri:build` 는 NSPanel, window level, collection behavior 가 dev와 release에서 다르게
동작했던 선례 때문에 필수다.

---

## 8. 영향 범위와 비목표

### 영향 범위

- `src-tauri/src/commands/popup.rs` (placement 재작성 + `resize_popup` + 입력 검증 + 테스트)
- `src-tauri/src/commands/mod.rs` (`resize_popup` invoke_handler 등록)
- `src-tauri/Cargo.toml` (`objc2-app-kit` 에 `NSEvent`, `NSScreen` feature 추가)
- `src-tauri/capabilities/popup-window.json` (generic window setter 권한 제거 — 최소 권한)
- `src/windows/popup/popup.tsx` (resize 를 typed wrapper 호출로 단순화)
- `src/windows/popup/resize.ts` + `resize.test.ts` (height 계산 + coalesce + queue 복구/취소 재검사)
- `src/features/popup/ipc.ts` + `ipc.test.ts` (popup typed IPC wrapper: `resizePopup`, `hidePopup`)

### 비목표

- main / menubar 윈도우 위치 정책 변경
- 단축키 파서 변경
- Ollama, 번역 streaming protocol, DB 변경
- Tauri capability 확대(오히려 축소함)
- H4 toggle UX 변경

---

## 9. 완료 조건

1. 듀얼 모니터(Retina primary·mixed-DPI 포함)에서 popup 이 항상 커서가 있는 화면 중앙에 나타난다.
2. 네이티브 전체화면 Space overlay 와 textarea focus가 유지된다.
3. streaming resize 가 새 화면 배치를 이전 위치로 덮어쓰지 않는다(크기 변경·top-left 보존을
   Rust 가 메인스레드에서 동기 적용해 async enqueue 경합을 제거).
4. Rust 단위 테스트에 Cocoa points 배치 + `top_left_to_preserve` 회귀 guard 가 있다.
5. frontend 단위 테스트에 height 계산 + coalesce guard 가 있다.
6. `resize_popup` 의 신뢰 불가 height 입력이 Rust 에서 clamp 되고(screenless 포함) 단위 테스트로
   검증된다.
7. popup capability 는 최소 권한(`start-dragging` 만)으로, 다른 윈도우 조작 표면이 없다.
8. 다른 높이의 화면에서 다시 열어도 popup 높이가 그 화면의 80% 를 넘지 않는다. cold-show 가
   `capped_centered_placement` 로 cap 을 재적용하고, `resize_popup` 상한도 `frame * 0.8`(screenless
   면 최소 높이)라 지연된 resize 도 cap 을 넘지 못한다(Rust 단위 테스트로 검증).
9. 작은 화면에서 줄어든 popup 을 큰 화면에서 다시 열면 `POPUP_OPENED` 시 output 높이가 복구된다
   (FE `applyResize`; `computePopupHeight` 재확장 단위 테스트로 검증).
10. streaming 중 resize 순서가 뒤집혀도 최신 output 높이가 최종 적용된다(FE single-flight
    `createPopupResizer`; latest-wins 단위 테스트로 검증).
11. §7 자동 검증이 모두 통과한다.
12. dev와 release `.app` 에서 §6.3 수동 검증이 통과한다.

---

## 10. 조사 기록

2026-06-02 기준:

- `cargo test --manifest-path src-tauri/Cargo.toml popup -- --nocapture`
  - popup 관련 Rust 테스트 `7 passed`, `0 failed`
  - mixed-DPI 실제 모니터 선택은 검증하지 않음
- `tests/e2e/translate-popup-shortcut.spec.ts`
  - `test.skip` 상태
- `POPUP_CLOSED`
  - Rust emit과 FE 상수 정의는 있으나 popup FE listener 없음
- 현재 워크트리
  - 조사 시작 시 clean
- 현재 Codex 실행 환경
  - AppKit 화면 목록이 노출되지 않아 물리 듀얼 모니터 runtime 재현 불가
  - 실제 듀얼 모니터 Mac에서 Phase A probe 필요

---

## 11. 검증 기록 (소스 대조, 2026-06-02)

고정 의존성 소스를 직접 읽어 H1~H3 을 확정했다. `~/.cargo/registry/.../` 경로.

| 주장                                    | 판정 | 근거                                                                                                                                                            |
| --------------------------------------- | ---- | --------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| cursor 좌표는 `points × primary_scale`  | ✅   | `tao-0.35.3/.../util/mod.rs:101-106` — `NSEvent.mouseLocation` → `.to_physical(primary_monitor().scale_factor())`                                               |
| monitor 판정은 `points` 기준            | ✅   | `tao-0.35.3/.../monitor.rs:163-173` — `CGRectContainsPoint(CGDisplayBounds, point)`. `position()`/`size()` 도 `from_logical(CGDisplayBounds/pixels, own_scale)` |
| Tauri 가 두 값을 변환 없이 pass-through | ✅   | `tauri-runtime-wry-2.11.2/src/lib.rs:3125,3144`                                                                                                                 |
| H2 배율 혼합                            | ✅   | `popup.rs:112-127` — target physical ÷ popup 현재 scale                                                                                                         |
| H3 FE 가 `POPUP_CLOSED` 미소비          | ✅   | `popup.tsx:17,57` 은 `POPUP_OPENED` 만 listen. grep 결과 FE 소비처 0                                                                                            |
| `objc2-app-kit` feature 부족            | ✅   | `Cargo.toml:50` = `["NSWindow","NSResponder","NSPanel"]` (NSScreen/NSEvent 없음)                                                                                |

API 시그니처 확인(`objc2-app-kit-0.3.2`, `objc2-0.6.4`, `objc2-foundation-0.3.2`):

- `NSEvent::mouseLocation() -> NSPoint` (class method, marker 불필요)
- `NSScreen::screens(mtm: MainThreadMarker) -> Retained<NSArray<NSScreen>>`
- `NSScreen::frame(&self) -> NSRect`, `visibleFrame(&self) -> NSRect`
- `NSRect = CGRect { origin: CGPoint{x,y}, size: CGSize{width,height} }` — 필드 public
- `NSArray::iter()` 로 순회, `MainThreadMarker::new_unchecked()` 로 marker 획득
