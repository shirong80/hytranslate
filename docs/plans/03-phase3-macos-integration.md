# Phase 3 — macOS 시스템 통합 계획서

PRD §15 단계 3 / §6.3–6.5 / §9.1 / §11 참조. Phase 1·2 작업물(번역 루프, 언어 감지, 설정 영속화) 위에 올린다.

## 범위 (Scope)

이번 단계의 deliverable 은 **macOS 네이티브 UX 표면** 다섯 가지다:

1. **전역 단축키** (`Cmd+Shift+T`) — 어떤 앱에서든 팝업을 띄움
2. **플로팅 팝업 윈도우** — 480px × 자동 높이, 화면 중앙, 입력 5,000자 제한
3. **메뉴바 트레이 + popover** — 트레이 아이콘 클릭 시 ~320px popover, blur 시 자동 hide
4. **클립보드 통합** — 자동 복사 (Settings 옵션) + 팝업 Cmd+C
5. **시작 시 실행 (autostart)** + **Dock 아이콘 숨김 (activation policy)**

## Locked Decisions (Phase 3)

| 항목 | 결정 |
|---|---|
| 단축키 plugin | `tauri-plugin-global-shortcut` 2.x |
| 클립보드 plugin | `tauri-plugin-clipboard-manager` 2.x |
| 자동시작 plugin | `tauri-plugin-autostart` 2.x (MacosLauncher::LaunchAgent) |
| 트레이 / Activation policy | Tauri 2 내장 `TrayIconBuilder` / `app.set_activation_policy()` |
| Cmd 표기 | `Modifiers::SUPER` (확정) |
| 단축키 파서 입력 형식 | Electron-style: `Cmd+Shift+T`, `CommandOrControl+Shift+L` |
| 팝업 위치 | 메인 디스플레이 중앙 (Phase 3 단순화); 향후 활성 화면 추적은 Phase 5 |
| 팝업 입력 한도 | 5,000자 (PRD §6.3) |
| 메인 입력 한도 | 30,000자 (변경 없음) |
| 메뉴바 popover 너비 | 320px |
| 메뉴바 popover 높이 | 가변 (기본 480, 최대 640) |
| Recent 5 | Phase 3 에서는 in-memory ring buffer (FE) — DB 통합은 Phase 4 |
| Dock 숨김 토글 | `ActivationPolicy::Accessory` ↔ `Regular` — 메뉴바 모드 진입과 분리 |
| 메뉴바 동작 | 항상 활성 (트레이는 settings 와 무관하게 표시) |
| 단축키 권한 부재 | inline 에러 `PermissionRequired` + 시스템 설정 열기 CTA |
| 팝업 hide trigger | Esc, blur, 번역 완료 후 5초 (PRD §6.3 미정의이지만 macOS UX 통례에 따라 blur on) |

`PermissionRequired` 변형은 `AppError` 에 신규 추가한다 (PRD §11 에 이미 명시).

## 컴포넌트 / 모듈 변경

### Rust (`src-tauri/`)

- `Cargo.toml` — 세 플러그인 추가
- `commands/mod.rs` — `setup()` 안에서 플러그인 init + 트레이/단축키/창 wiring
- `shortcuts/mod.rs` — `parse_shortcut`, `register`, `unregister`, `reconcile_with_settings`
- `menubar/mod.rs` — `install_tray(app)`, `position_popover_under_tray(rect)`
- `commands/popup.rs` (신규) — `show_popup`, `hide_popup`, `toggle_popup`
- `commands/clipboard.rs` (신규) — `copy_text { text }` (FE 보조용)
- `commands/system.rs` (신규) — `apply_dock_hidden`, `apply_autostart`
- `commands/settings.rs` — `update_settings` 핸들러에서 단축키/autostart/dock 변경분에 대해 reconcile 호출
- `errors.rs` — `AppError::PermissionRequired { feature: String }` 추가
- `tauri.conf.json` — 트레이 권장 윈도우 옵션, capabilities/default.json 권한 추가

### Frontend (`src/`)

- `src/lib/ipc/events.ts` + `src-tauri/src/events.rs` — `popup:opened`, `popup:closed` (단순)
- `src/features/translation/types.ts` — `POPUP_INPUT_LIMIT = 5_000` 추가
- `src/features/translation/use-translation-controller.ts` — `inputLimit` 옵션화
- `src/features/translation/components/translation-panel.tsx` — `variant: 'main' | 'popup' | 'menubar'` prop으로 cap 분기
- `src/features/settings/components/settings-panel.tsx` — 단축키, autostart, dock, 자동복사 토글 추가
- `src/windows/popup/popup.tsx` — popup 변형 패널 + Esc/Cmd+C/Cmd+Enter
- `src/windows/menubar/menubar.tsx` — compact 패널 + recent 5 in-memory
- `src/features/clipboard/` (신규 feature) — `useAutoCopy`, `copyToClipboard` 헬퍼
- `src/i18n/ko.ts` — 단축키/자동시작/Dock/자동복사 라벨 추가

## Acceptance — Phase 3 완료 기준

1. **단축키**: 백그라운드 상태에서 `Cmd+Shift+T` → 팝업 표시 + 입력 포커스. 다시 누르면 toggle.
2. **단축키 변경**: Settings 에서 hotkey 문자열 변경 → 즉시 재등록, 이전 키 unregister 확인.
3. **권한 없음**: 첫 호출 시 macOS Accessibility 권한 prompt; 거부 시 inline `PermissionRequired` 메시지 + "시스템 설정 열기" CTA.
4. **팝업 입력 5,000자**: 초과 시 `InputTooLong { limit: 5000 }` inline 에러; 메인 30,000자 cap 은 그대로.
5. **메뉴바 popover**: 트레이 클릭 시 트레이 아래 ~4px 띄워서 표시; 다른 윈도우 포커스 시 hide.
6. **Dock 숨김 토글**: `Settings.hide_dock_icon` true → `Accessory` (Dock 사라짐); false → `Regular`. 실시간 적용.
7. **자동 시작**: `Settings.start_at_login` true → LaunchAgent 등록; 재부팅 후 launch (수동 확인 가능, 자동 검증 X).
8. **자동 복사**: 번역 완료 시 `Settings.auto_copy_after_translation` true 면 결과를 클립보드에 write.
9. **팝업 Cmd+C**: 결과 박스에 포커스 / 또는 결과가 있을 때 Cmd+C 누르면 결과 복사 + 토스트(또는 chip).
10. **번역 흐름 유지**: 메인 / 팝업 / 메뉴바 모두 Phase 1·2 IPC 계약을 그대로 사용 — duplicate 코드 없음.

## 테스트

- **Rust unit**: `parse_shortcut("Cmd+Shift+T")` → `(SUPER|SHIFT, KeyT)`; 잘못된 형식은 `InvalidShortcut` 에러; `Cmd+CmdOrCtrl` 같은 모호 케이스 처리.
- **Rust unit**: `position_popover_under_tray(rect, popup_width=320)` → 트레이 중앙 x 기준으로 좌측 정렬, y 는 trayBottom + gap.
- **Rust unit**: `reconcile_with_settings` 가 이전 단축키 unregister 후 새 단축키 register 순서를 호출하는지 (mock GlobalShortcut trait).
- **FE Vitest**: `use-translation-controller` 가 `inputLimit` prop 을 받아 cap 을 적용하는지.
- **FE Vitest**: `useAutoCopy` 가 완료 이벤트에서만 write 하고 cancel/error 에서는 무시하는지.
- **E2E**: 본 Phase 의 E2E 는 Tauri WebDriver 미지원으로 보류 (PRD §15 도 phase 3 에 E2E 명시 없음). Phase 5 에서 onboarding flow 와 함께 다시 검토.

## 진행 순서

1. plugin 의존성 추가 + capabilities
2. `AppError::PermissionRequired` + 이벤트 상수 추가
3. shortcuts 모듈
4. popup 윈도우 commands + 단축키 → toggle 연결
5. tray + activation policy
6. autostart, clipboard 명령
7. FE popup / menubar 변형
8. settings UI 확장 + settings 변경 시 reconcile
9. 테스트
10. quality-gate 통과 후 커밋
