# Phase 6 — Code Review v1 Follow-up

소스: `docs/code-review-v1.md` (정적 리뷰, 2026-05-26).
범위: Critical 2건 · Major 8건 · Minor 4건. 사용자 결정 4건 (Q1~Q4) 반영.
완료 기준:
- 각 항목별 `검증` 절을 모두 통과.
- **자동 게이트 (필수, 본 follow-up의 hard pass/fail)**: 작업자가 각 커밋 직후 또는 PR 직전에 아래 명령을 모두 0 종료로 통과시켜야 한다.
  - `npm run lint`
  - `npm run typecheck`
  - `npm run test`
  - `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings`
  - `cargo test --manifest-path src-tauri/Cargo.toml`
- **가이드라인 검토 (best-effort)**: Claude Code 환경에서 작업하는 경우 코드 변경 직후 `.claude/CLAUDE.md`의 "After every code change" 규약에 따라 `Agent(subagent_type="quality-gate", prompt="...")`를 호출. 단, quality-gate는 Claude Code agent에 의존하므로 **외부 CI / 사람 검토자에게는 강제하지 않는다** — 위 자동 게이트 6개 명령이 통과하면 자동 게이트는 닫힌다. agent 호출이 가능한 환경에서는 보고서가 fail로 닫히지 않아야 함.
- Critical 1은 본 범위에서 닫히지 않음 (§11 참고) — 완료 보고서에 "open" 으로 명시.

---

## 0. 사용자 결정 (interview)

| Q | 결정 |
|---|---|
| Q1 자동 감지 UI | `translation:started` payload 확장 — backend resolved language를 FE badge에 매핑. detect_language는 호출하지 않는다. |
| Q2 ModelInstallState | PRD §9.3을 "런타임 상태 + `last_checked_at`" 모델로 조정. settings에 `last_checked_at` 필드만 추가. DB schema는 추가하지 않는다. |
| Q3 E2E + 평가셋 | PRD §14.3의 5종 E2E를 **shell + skip** 스펙으로 작성하고, 평가셋은 골조 + 대표 10개 샘플(`source_text` / `reference_en` 채움, 점수 컬럼은 비움). 100건 채점은 별도 트래킹. |
| Q4 state machine | `debouncing` → `typing`으로 rename + `detecting` 추가. 상태 전이 UI/i18n까지 일관. |

---

## 1. 작업 우선순위

P0 = 사용자 데이터 안전. P1 = PRD §19 DoD 직결. P2 = 정합성.

| # | 항목 | 카테고리 | 우선순위 |
|---|---|---|---|
| 1 | 입력 변경 즉시 취소 | Critical 2 | P0 |
| 2 | DB 경로 PRD 일치 | Major 7 | P0 |
| 3 | 감지 결과 UI 연결 | Major 1 | P1 |
| 4 | 메뉴바 tray menu + DB recent | Major 2 | P1 |
| 5 | 클립보드 feature + inline error | Major 3 | P1 |
| 6 | popup focus/sizing | Major 4 | P1 |
| 7 | Ollama exponential backoff | Major 5 | P1 |
| 8 | state machine (typing/detecting) | Major 8 | P1 |
| 9 | last_checked_at 영속 + PRD 노트 | Major 6 | P2 |
| 10 | E2E 5종 shell + 평가셋 골조 + 10건 (DoD 미충족 유지) | Critical 1 | P2 |
| 11 | Minor 1~4 | Minor | P2 |

> Critical 1은 본 follow-up에서 **부분만** 다루며 PRD §19 DoD는 닫히지 않는다 (§11, §14 참고).

---

## 2. Critical 2 — 입력 변경 즉시 취소

### 현상
`use-translation-controller.ts:128-149` — `sourceText/sourceLanguage/model`이 바뀌면 새 debounce timer만 예약한다. 기존 in-flight 요청은 cancel하지 않으므로, 그 사이 backend가 완료되면 stale 결과가 `output`에 쓰이고 DB에 저장될 수 있다.

### 변경

**FE — `src/features/translation/use-translation-controller.ts`**
1. effect 진입 시 **항상 `cancelInFlight()` 먼저 호출**. 비어 있으면 `setIdle()`, 비어있지 않으면 `setTyping()` 후 debounce timer 예약. (입력 변경 즉시 취소.)
2. `runTranslation()` 내부의 `cancelInFlight()` **유지**. 이유: `retranslateImmediately()`(Cmd+Enter / "다시 번역" 버튼)는 입력 deps가 바뀌지 않아 effect를 트리거하지 않는다. 같은 입력으로 즉시 재번역할 때 기존 in-flight를 정리하는 방어선이 사라지면 두 요청이 병행될 수 있다.
3. 양쪽 cancel이 race를 만들지 않게: `cancelInFlight`는 `inFlightRef`가 null이면 즉시 no-op이고, backend `cancel_translation`은 idempotent (`registry.cancel`이 missing token에서 false 반환). 따라서 두 곳에서 호출돼도 안전.
4. effect dependency에 `setTyping`을 추가.

**FE — `src/features/translation/store.ts`**
1. `markStarted/appendChunk/markCompleted/markCancelled/markError`의 `requestId` 가드는 유지 (이미 OK).
2. `setTyping`/`setDetecting` action 추가 (Major 8 항목과 묶음).

**BE — `src-tauri/src/commands/translate.rs`**
- `cancel_translation`이 worker가 emit하기 직전 cancel을 받았더라도 backend는 이미 `token.is_cancelled()` 분기로 `translation:cancelled`를 emit하고 DB 저장도 막는다. 보강은 **`persist_completed` 호출 직전에 `token.is_cancelled()` 재확인** 한 줄 추가 (race 방지).

### 검증
- Vitest: `use-translation-controller.test.ts` 신규 — fake timers, 모킹된 invoke로
  - (a) 첫 입력 → 500ms 대기 → `translate_stream` 호출
  - (b) 200ms 후 추가 입력 → 즉시 `cancel_translation` 호출 (effect 경로)
  - (c) 입력 변경 없이 `runImmediately()` 두 번 연속 호출 → 두 번째 호출 시작 직전 `cancel_translation`이 첫 번째 requestId로 호출되는지 확인 (runTranslation 경로)
- cargo test: `commands::translate::tests` — 기존 persist 테스트 외에 "completed 직전 cancel 토큰 설정 시 persist 호출되지 않음" 케이스 추가.

---

## 3. Major 7 — DB / settings 경로 PRD 일치 (자동 copy+verify, 수동 cleanup)

### 현상
`commands/mod.rs:69-92` — `app_data_dir()` 사용. macOS에서 identifier(`com.shiron.hytranslate`) 기반 디렉터리로 떨어진다. PRD §9.4는 `~/Library/Application Support/HyTranslate Mac/`.

### 안전 원칙 (코드리뷰 Critical hard stop 2차 반영)

설정/이력은 user-visible state다. **백업이 explicit confirmation을 대체하지 않는다**는 원칙을 받아들여 두 단계로 명확히 분리한다:

| 단계 | 트리거 | 동작 | destructive? |
|---|---|---|---|
| 자동 | 앱 startup | new 경로에 **copy + verify**. legacy는 손대지 않음 (read-only 참조). | ❌ |
| 수동 | 사용자 in-app CTA (`legacy 경로 정리하기`) 클릭 | legacy 우리 파일을 backup-dir로 이동 후 빈 디렉터리면 remove_dir. | ✅ (사용자 동의) |

- 자동 마이그레이션이 verify 실패 → setup은 legacy 경로를 active로 계속 사용 (다음 실행에서 재시도).
- verify 성공해도 cleanup CTA를 누르기 전까지는 legacy 그대로. 양쪽 경로에 동일 데이터가 존재할 뿐(disk overhead 약간).
- cleanup 후 backup-dir은 **무기한 보존**한다. 자동 삭제는 또 다른 destructive auto-execution이므로 도입하지 않는다. backup-dir 삭제는 별도 사용자 액션(같은 설정 패널의 "이전 백업 삭제" CTA)으로만 수행한다 — 본 follow-up 범위 외, §14 후속 트래킹.

### SQLite 마이그레이션 전략 (raw 파일 copy 금지)

`hytranslate.sqlite` + WAL/SHM을 `fs::copy`로 그대로 복사하는 방식은 거부한다:
- 다른 프로세스가 source DB를 열고 있으면 inconsistent snapshot.
- WAL이 checkpoint되지 않은 상태에서 copy하면 destination이 partial.
- "verify"가 PRAGMA user_version과 count(*) 만으로는 깨진 페이지를 탐지 못함.

대체 전략: **rusqlite `backup::Backup` API**. source DB를 read-only로 열고 destination DB(fresh)로 페이지 단위 backup. WAL/SHM은 destination에서 자동 생성되므로 별도 copy 안 함. source에 동시 writer가 있어도 SQLite가 일관된 snapshot을 만들어 준다.

### Dependency 변경

**`src-tauri/Cargo.toml`**
- `rusqlite = { version = "0.32", features = ["bundled", "backup"] }` — `backup` feature 추가 (현재 `bundled`만 활성). `backup::Backup` 사용에 필수.
- `tempfile`은 **dev-dependencies로 유지**. production 마이그레이션 코드는 `tempfile` crate 대신 std 기반 atomic write로 구현:
  - `<dst>.tmp-<uuid>` 경로에 `File::create` + `write_all` + `sync_all`
  - 성공 후 `fs::rename(<dst>.tmp-<uuid>, <dst>)` — 같은 디렉터리이므로 POSIX 원자 보장.
  - 실패 시 tmp 파일 best-effort 삭제.
  - helper: `paths::atomic_write_file(dst: &Path, bytes: &[u8])`. uuid는 이미 의존성에 있음(translate 요청 ID 검증용).

`logs/` 디렉터리는 v1에서 PRD 명시가 약하므로 마이그레이션 대상에서 **제외**. 새 경로에서 새로 만든다.

### 검증 (dependency)
- `cargo build --manifest-path src-tauri/Cargo.toml`이 `backup` feature 활성으로 통과.
- `cargo test`에서 backup helper 호출이 컴파일 + 동작.

### 변경

**BE — `src-tauri/src/paths.rs` (신규)**

```rust
pub fn new_data_dir<R: Runtime>(app: &AppHandle<R>) -> AppResult<PathBuf>;
// home_dir + "Library/Application Support/HyTranslate Mac"
// 미존재 시 create_dir_all. dirs crate 사용 안 함 (Minor).

pub fn legacy_data_dir<R: Runtime>(app: &AppHandle<R>) -> AppResult<Option<PathBuf>>;
// app.path().app_data_dir() — 새 경로와 같으면 Ok(None), 다르면 Ok(Some(p)).

/// 자동 단계의 산출물.
/// **불변식**: migrate_copy_verify가 `Ok`로 반환할 때 new_dir과 legacy_dir 필드는
/// 항상 채워진다. resolve_data_dir는 이 값만 보고 active dir를 고른다.
///
/// 실패 표현:
/// - "verify 실패" (DB 손상 등): Ok 반환, `verified: false`. resolve가 legacy로 fallback.
/// - "초기화 실패" (new 경로 생성/PATH 해석 자체가 실패): `migrate_copy_verify`가 `Err(AppError)` 반환. setup이 받아서 사용자에게 fatal 안내 + 앱 종료. (drive 권한 문제, sandbox 차단 등 — v1에서 정상 흐름으로 복구 불가.)
pub struct MigrationOutcome {
  pub new_dir: PathBuf,                    // 항상 채움 (new_data_dir 결과)
  pub legacy_dir: Option<PathBuf>,         // 항상 채움 (legacy_data_dir 결과)
  pub copied: Vec<PathBuf>,                // 성공한 copy 목록 (실패 시 [])
  pub verified: bool,                      // verify 통과 여부
  pub legacy_has_our_files: bool,          // legacy에 settings.json 또는 DB가 있는지
  pub legacy_cleanable: bool,              // verified && legacy_has_our_files
  pub verify_error: Option<String>,        // verify 실패 시 사람이 읽을 수 있는 메시지 (tracing/UI용)
}

pub fn migrate_copy_verify<R: Runtime>(app: &AppHandle<R>) -> AppResult<MigrationOutcome>;
// `Err`는 path 해석 / new_dir 생성 같은 fatal 케이스에만 사용.
// 그 외 (legacy 조회 실패, DB copy 실패, verify 실패)는 모두 outcome 내부에 흡수.
// legacy는 절대 건드리지 않음.

pub fn resolve_data_dir(outcome: &MigrationOutcome) -> PathBuf;
// 결정 트리 (불변식 덕에 명확):
//   1. outcome.verified == true → outcome.new_dir
//   2. outcome.legacy_has_our_files && outcome.legacy_dir.is_some() → legacy_dir
//      (verify 실패했지만 legacy에 실데이터가 있으면 legacy 사용 — 안전 fallback)
//   3. 그 외 → outcome.new_dir (legacy 자체가 없거나 비어 있음 — 새 설치 첫 실행)

pub fn cleanup_legacy(outcome: &MigrationOutcome) -> AppResult<CleanupReport>;
// 수동 단계. legacy_cleanable == false면 즉시 CleanupReport::Skipped.
// outcome.new_dir / outcome.legacy_dir만으로 fs 작업 가능 — app handle 불필요.
// 우리 파일들을 <new_dir>/legacy-backup-<unix-ts>/ 로 fs::rename.
// 같은 파일시스템이면 원자적; 다른 파일시스템이면 copy + remove.
// rename 후 legacy 디렉터리가 빈 디렉터리면 remove_dir 시도.
// 외부 파일이 남아 있으면 legacy 디렉터리는 그대로 둔다.
```

**setup 사용 패턴**

```rust
let outcome = match paths::migrate_copy_verify(app.handle()) {
  Ok(o) => o,
  Err(err) => {
    // fatal — 사용자에게 안내 후 종료. data_dir 자체를 못 만든 상황이라
    // 부분 fallback도 안전하지 않음.
    tracing::error!(error = ?err, "data dir initialization failed; aborting startup");
    return Err(Box::new(std::io::Error::other(format!(
      "data dir initialization failed: {err:?}"
    ))));
  }
};
tracing::info!(
  verified = outcome.verified,
  legacy_cleanable = outcome.legacy_cleanable,
  copied = outcome.copied.len(),
  verify_error = ?outcome.verify_error,
  "migration outcome",
);
let data_dir = paths::resolve_data_dir(&outcome);
app.manage(Arc::new(outcome));
// settings/db는 data_dir 기준으로 init
```

이전 plan의 `MigrationOutcome::skipped()` 헬퍼는 제거. **fatal 초기화 실패는 `Err`, 그 외(verify 실패 / copy 실패 등 복구 가능 케이스)는 outcome 필드(`verified: false`)로 표현**. `resolve_data_dir`가 항상 정답을 고를 수 있도록 `new_dir`/`legacy_dir`를 outcome이 보존.

**BE — `src-tauri/src/paths/migration.rs`** 또는 paths.rs 내부 helper —
- `copy_settings(src, dst) -> AppResult<()>`: `paths::atomic_write_file` helper 사용. std 기반 `<dst>.tmp-<uuid>` 임시 파일명에 write → `sync_all` → `fs::rename`으로 atomic.
- `copy_database(src, dst) -> AppResult<()>`: rusqlite `backup::Backup::new()` 사용. progress는 무시 (작은 DB).
- `verify_destination(dst_dir) -> AppResult<()>`: 새 settings.json deserialize + 새 DB에서 `PRAGMA user_version` + `SELECT count(*) FROM translation_records`.

**BE — `src-tauri/src/commands/mod.rs`**

위 "setup 사용 패턴" 그대로. `migrate_copy_verify`가 `AppResult<MigrationOutcome>`을 반환하므로 setup은 `Err`를 받아 startup을 중단(fatal)하거나 사용자 안내 후 종료한다. `Ok` 경로에서는 `verified: false`여도 outcome으로 legacy fallback이 자동 결정된다.

**BE — `src-tauri/src/commands/paths.rs` (신규 command 모듈)**

```rust
#[tauri::command]
pub async fn get_legacy_migration_status(
  outcome: tauri::State<'_, Arc<MigrationOutcome>>,
) -> AppResult<MigrationStatusView> {
  // { legacyCleanable: bool, legacyDir: Option<String>, verified: bool }
}

#[tauri::command]
pub async fn cleanup_legacy_data_dir(
  outcome: tauri::State<'_, Arc<MigrationOutcome>>,
) -> AppResult<CleanupReport> {
  paths::cleanup_legacy(&outcome)
}
```

**FE — `src/features/settings/components/settings-panel.tsx`**

- "고급" 또는 "데이터" 섹션 신설.
- mount 시 `get_legacy_migration_status` 호출 → `legacyCleanable === true` 면 inline 안내 배너 노출:
  > "이전 위치(`legacy_dir`)에 데이터 사본이 남아 있습니다. 새 위치로 정상 이전된 것을 확인했다면 정리할 수 있습니다."
  - `[legacy 경로 정리하기]` 버튼 → `cleanup_legacy_data_dir` 호출. 성공 시 배너 사라짐.
  - 확인 모달: "legacy 파일을 `legacy-backup-<ts>/`로 이동합니다. 새 데이터에는 영향이 없습니다." 사용자가 명시 confirm.
- 절대 자동 호출 안 함.

### 검증

- cargo test: `paths::tests` (tempdir 두 개 사용)
  - 빈 양쪽 → `verified: true`, `copied: []`, `legacy_has_our_files: false`, `legacy_cleanable: false`. `resolve_data_dir` 결과는 `new_dir`.
  - legacy에 정상 settings + DB, new 비어 있음 → copy 성공, verify 통과, `legacy_cleanable: true`, **legacy 파일이 그대로 남아있는지** assert. `resolve_data_dir`은 `new_dir`.
  - **legacy DB 손상 → copy 후 verify 실패**: destination에 복사된 파일을 모두 지우고 outcome은 `verified: false`, `legacy_has_our_files: true`, `legacy_cleanable: false`. `resolve_data_dir`은 `legacy_dir` (안전 fallback). legacy 보존.
  - **legacy 자체가 없는 경우 (legacy_data_dir이 None)** → outcome은 `verified: true`, `legacy_dir: None`. `resolve_data_dir`은 `new_dir`.
  - 양쪽 모두 우리 파일 존재 → no-op, `legacy_cleanable: false`. `resolve_data_dir`은 `new_dir`.
  - `atomic_write_file`: write 도중 패닉 시 tmp 파일 잔여, 정상 종료 시 원자적 rename. uuid를 tmp suffix로 사용.
  - SQLite backup API 호출 검증 (별도 helper로 unit-testable).
- cargo test: `paths::cleanup_tests`
  - `legacy_cleanable: false` 일 때 호출 → `CleanupReport::Skipped` 반환, legacy 건드리지 않음.
  - 정상 호출 → 우리 파일들이 backup-dir로 이동, 빈 디렉터리면 `remove_dir` 수행, 외부 파일 보존.
  - rename이 실패하는 환경(cross-filesystem 시뮬레이션) → copy + remove fallback 검증.
- Vitest: `settings-panel.test.tsx` — `legacyCleanable: true` 응답일 때 배너 렌더링, 버튼 클릭 시 confirm 후 `cleanup_legacy_data_dir` 호출.
- 수동: 첫 실행 후 양쪽 경로에 `settings.json`/`hytranslate.sqlite` 둘 다 존재 확인. 설정 UI에서 배너가 보여야 함. CTA 클릭 → `legacy-backup-<ts>/` 생성 + legacy 정리 확인.

---

## 4. Major 1 — 감지 결과 UI 노출 (`translation:started` 확장)

### 변경

**BE — `src-tauri/src/commands/translate.rs`**
1. `StartedPayload`에 `resolved_language: SourceLanguage` 추가 (`#[serde(rename_all = "camelCase")]`).
2. `translate_stream` 내부에서 이미 계산하는 `resolved_language`를 `emit(TRANSLATION_STARTED, ...)` payload에 포함.

**FE — `src/features/translation/types.ts`**
1. `StartedPayload`에 `resolvedLanguage: SourceLanguage` 추가.

**FE — `src/features/translation/store.ts`**
1. `TranslationState`에 `resolvedLanguage: SourceLanguage | null` 추가 (초기값 null).
2. `markStarted({ resolvedLanguage })`에서 store에 저장.
3. `beginRequest`/`setIdle`/`reset`/`setLocalError`에서 null로 리셋.

**FE — `src/features/translation/components/source-language-select.tsx`**
1. `value === 'Auto'`이고 `resolvedLanguage`가 Korean/Simplified/Traditional 중 하나면 select 옆에 작은 badge 노출. 미결정 시 `detected.unknown` 라벨.
2. badge는 i18n 키 `translation.sourceLanguage.detected.*`를 그대로 사용 (이미 ko.ts에 정의됨).

**FE — `src/windows/main/main.tsx`**, **`src/windows/popup/popup.tsx`**
1. `<SourceLanguageSelect>` 호출 부분에 `resolvedLanguage` prop 추가 — store에서 가져온다.

### 검증
- Vitest: `source-language-select.test.tsx` — `value="Auto"` + `resolvedLanguage="ChineseSimplified"`일 때 detected 라벨 렌더링.
- cargo test: `commands::translate` — StartedPayload JSON에 `resolvedLanguage` 키 포함 확인 (serde 테스트).

---

## 5. Major 4 — Popup focus / sizing

### 변경

**BE — `src-tauri/src/commands/popup.rs`**
1. `center_on_primary` → `place_on_active_monitor`로 교체:
   - 현재 mouse cursor 위치 기반으로 `monitor_from_point` 사용해 활성 monitor 추출.
   - 해당 monitor의 size를 가져와 width 480 고정, height는 **`min(content_h, monitor.height * 0.8)`**. content_h는 우선 360 그대로 두고 후속 resize는 FE가 처리.
   - 활성 monitor 중심으로 set_position.

**FE — `src/windows/popup/popup.tsx`**
1. mount 시 textarea focus + **`POPUP_OPENED` listen** 추가: 이벤트 수신 시 `textareaRef.current?.focus()` 호출.
2. 결과 길이에 따른 window 높이 조절: `useEffect`에서 `output` 변경 시 `getCurrentWindow().setSize()`로 content height에 맞춰 조정하되 monitor.height * 0.8 cap 적용 (Tauri API: `currentMonitor().size`).

**`src-tauri/tauri.conf.json`**
- height 360 그대로 유지. `maxHeight`는 동적이므로 conf로는 표현 안 됨.

### 검증
- 수동: 팝업을 단축키로 열기 → Esc로 숨기기 → 다시 단축키로 열기 → 첫 키 입력이 textarea에 들어가는지.
- 수동: 매우 긴 입력을 번역 → 결과 영역에 따라 window가 늘어나되 모니터 80%를 넘지 않는지.
- cargo test: `place_on_active_monitor`를 helper로 분리해 화면 좌표 계산만 단위 테스트.

---

## 6. Major 2 — Tray menu + DB-based recent

### 변경

**BE — `src-tauri/src/menubar/mod.rs`**
1. `TrayIconBuilder`에 **우클릭/우상단 메뉴**(`MenuBuilder`) 추가:
   - `메인 창 열기` → `app.get_webview_window("main").show()+set_focus()`
   - `이력 열기` → main 윈도우 띄우고 FE에 nav 이벤트 emit (or 그냥 main 띄움)
   - `설정 열기` → 동일하게 main + nav 이벤트
   - `종료` → `app.exit(0)`
2. 좌클릭 popover는 그대로 유지.
3. 이벤트 이름 추가: `nav:request { route: "history" | "settings" | "main" }` → `src-tauri/src/events.rs` 상수 신설 + FE listener.

**FE — `src/windows/main/main.tsx`**
1. `nav:request` listen → main window에서 history/settings 패널로 라우트 전환.

**FE — `src/windows/menubar/menubar.tsx`**
1. mount 시 `list_translation_records({ limit: 5 })` 호출 → 결과를 컴포넌트 local state로 저장. `recent` (TranslationStore)은 사용 중단 (Minor 1).
2. popover open 이벤트 (`MENUBAR_OPENED`) 마다 재조회 — DB가 최신 반영되도록.

**FE — `src/features/translation/store.ts`** & **`src/features/translation/types.ts`**
1. `recent`, `RecentTranslation`, `RECENT_LIMIT` 제거 (Minor 1과 묶음). `markCompleted`도 recent 갱신 코드 삭제.

### 검증
- cargo test: tray menu 빌더는 통합 테스트가 어려우므로 컴파일 + 수동 확인.
- Vitest: `menubar.test.tsx` — `list_translation_records` 호출 검증.
- 수동: tray 우클릭 시 메뉴 4개, 클릭 시 main 윈도우 활성 + 라우트 전환.

---

## 7. Major 3 — Clipboard feature + inline error

### Capability 추가 (선행 작업)

**`src-tauri/capabilities/default.json`** — permissions에 추가:
- `clipboard-manager:allow-read-image` — `readImage()` 호출에 필요.

(`clipboard-manager:allow-read-text` / `allow-write-text`는 이미 존재.)

### Clipboard 종류 감지 전략

Tauri 2 `@tauri-apps/plugin-clipboard-manager` API는 `readText()`, `readImage()`만 노출하며 macOS Finder 파일(NSFilenamesPboardType)을 별도 API로 읽을 수 없다. 감지 절차:

1. `readText()` 호출
   - 성공 + `string` non-empty → **text 케이스**
   - 성공 + empty string → 다음 단계
   - **throw → ReadFailed 케이스 (즉시 반환, image probe 안 함)**
2. `readImage()` 호출
   - 성공 → **image 케이스 (ClipboardUnsupported)**
   - throw → **empty 케이스** (PRD §6.4는 file/empty를 같이 안내해도 됨; image API 미지원/플랫폼 차이는 empty로 안전 매핑)

**중요 (2차 코드리뷰 반영)**: **`readText()`의 throw를 empty로 가리지 않는다**. 권한이 빠지거나 plugin이 동작하지 않는 진짜 통합 실패를 "텍스트 없음"으로 숨기면 디버깅이 불가능해진다. ReadFailed로 분리해 사용자에게 "권한/플러그인 문제" 안내.

`readImage()`의 throw는 empty와 의미가 같다 (이미지가 없음). 단, `readImage()`가 미지원 환경(예: vitest jsdom)에서 throw하는 경우에도 empty로 떨어지는 점을 테스트에서 명시.

### 변경

**FE 신규 — `src/features/clipboard/`**
- `ipc.ts`: `readClipboard(): Promise<ClipboardReadResult>`:
  ```ts
  type ClipboardReadResult =
    | { kind: 'text'; text: string }
    | { kind: 'empty' }
    | { kind: 'unsupported' }     // 이미지 감지
    | { kind: 'readFailed'; error: AppError };  // readText throw
  ```
- `hooks.ts`: `usePasteFromClipboard({ onText, onError })` — 메뉴바/팝업에서 동일 로직 재사용. error 콜백은 `empty | unsupported | readFailed` 각각 다른 메시지 매핑.

**FE — `src/lib/ipc/errors.ts`**
- `AppError` 유니온에 추가:
  - `ClipboardEmpty` — readText는 성공했으나 빈 문자열 또는 image API throw (텍스트가 없는 정상 상태)
  - `ClipboardUnsupported` — readImage 성공 (이미지 감지)
  - `ClipboardReadFailed { message: string }` — readText 자체가 throw (권한/플러그인 문제)
- i18n 메시지:
  - `errors.ClipboardEmpty: "클립보드에 텍스트가 없습니다."`
  - `errors.ClipboardUnsupported: "이미지는 번역할 수 없습니다. 텍스트를 복사해 다시 시도해 주세요."`
  - `errors.ClipboardReadFailed: "클립보드를 읽을 수 없습니다. macOS 손쉬운 사용 권한과 앱 권한을 확인해 주세요."` + `[다시 시도]` 액션.

**BE 동등화** — `src-tauri/src/errors.rs`의 `AppError` enum에 동일 세 variant 추가 (직렬화 모양 일치). 백엔드에서 emit하지 않더라도 FE union과 mirror 유지.

**FE — `src/windows/menubar/menubar.tsx`**
1. `handlePasteFromClipboard` → `usePasteFromClipboard` 훅 사용. 에러는 popover 안에 inline 표시 (textarea 위 작은 amber 배너).

**FE — `src/windows/popup/popup.tsx`**
1. 클립보드 붙여넣기 버튼 추가 (메뉴바와 동일 컴포넌트). 동일 훅 사용.

### 검증
- Vitest: `clipboard/ipc.test.ts` — 모킹된 `readText`/`readImage`로 4가지 분기:
  - text 반환 → `{ kind: 'text', text }`
  - text empty + readImage 성공 → `{ kind: 'unsupported' }`
  - text empty + readImage throw → `{ kind: 'empty' }` (이미지 없음 정상)
  - **text throw → `{ kind: 'readFailed', error: { kind: 'ClipboardReadFailed', message: ... } }`** (코드리뷰 Major 반영 회귀 테스트)
- Vitest: `usePasteFromClipboard.test.ts` — 각 분기가 onText/onError 분기로 정확히 라우팅되는지.
- 수동: 빈 클립보드 / 이미지 복사 / Finder 파일 복사 / 텍스트 복사 / capability 누락 시뮬레이션(allow-read-text 제거) → 각각 다른 inline 메시지.

---

## 8. Major 5 — Ollama exponential backoff

### 변경

**FE — `src/features/onboarding/store.ts`** (또는 `lib/backoff.ts` 분리)
1. `tryStartOllama` 후 단일 800ms sleep을 **delays = [250, 500, 1000, 2000, 4000, 8000] ms** 시퀀스로 교체.
2. 각 step에서 `getOllamaStatus()` → `running=true`면 즉시 종료. 끝까지 실패 시 `running=false`를 store에 반영하고 사용자에게 "다시 확인" CTA.
3. cancel 가능: `cancelStartProbe` action — `AbortController` 또는 cancellation flag로 polling 루프 중단.

**FE — `src/lib/backoff.ts`** (신규)
```ts
export async function withExponentialBackoff<T>(
  fn: () => Promise<T>,
  isDone: (v: T) => boolean,
  signal: { cancelled: boolean },
  delays = [250, 500, 1000, 2000, 4000, 8000],
): Promise<T> { ... }
```

### 검증
- Vitest: `backoff.test.ts` — fake timers로 시퀀스 검증, cancel signal 전달 시 중단.
- 수동: Ollama 종료 상태 → 자동 실행 클릭 → 8초 이내 running 전환.

---

## 9. Major 8 — State machine (typing/detecting)

### 변경

**FE — `src/features/translation/types.ts`**
1. `TranslationStatus`:
   ```ts
   type TranslationStatus =
     | 'idle' | 'typing' | 'detecting' | 'translating'
     | 'completed' | 'cancelled' | 'error';
   ```
   `debouncing` 제거.

**FE — `src/features/translation/store.ts`**
1. `setTyping()` action 추가 — status를 `typing`으로, output/error 초기화.
2. `markStarted`에서 `detecting`을 거치지 않고 바로 `translating`. (감지는 backend가 emit하기 직전 동기 작업이므로 race 위험 없음. 사용자 결정에 따라 detecting은 향후 client-side detect 도입 시 활용 — 지금은 enum만 추가하고 active transition은 typing → translating).
3. 단, **활성 transition 유지 의도**(PRD §7.2)대로 `detecting` 상태를 짧게라도 emit하려면 `runTranslation` 시작 직전에 `set({ status: 'detecting' })`을 한 tick 두는 옵션도 있음. 결정: `runTranslation` 진입 시 `detecting`으로 set → invoke 직후 → `translation:started` 도착 시 `translating`. UI는 매우 짧은 detecting 상태를 보일 수 있고 테스트 가능.

**FE — `src/features/translation/use-translation-controller.ts`**
1. 입력 effect에서 비어있지 않으면 `setTyping()` 호출.
2. `runTranslation` 진입 시 `setDetecting()`.

**FE — `src/i18n/ko.ts`**
1. `translation.status.debouncing` → `translation.status.typing` rename + `translation.status.detecting` 추가.

**FE — UI 노출**
- 메인/팝업/메뉴바의 상태 indicator에서 `typing` / `detecting` 라벨 사용.

### 검증
- Vitest: `store.test.ts` 업데이트 — 새 상태 전이 verify.
- typecheck: 새로 추가/변경된 상태가 모든 분기에서 처리되도록 컴파일러가 강제.

---

## 10. Major 6 — last_checked_at + PRD 노트

### 변경

**BE — `src-tauri/src/settings/mod.rs`**
1. `Settings`에 `model_install_state: ModelInstallSummary` 추가 (`#[serde(default)]` + `Default` impl):
   ```rust
   #[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
   #[serde(rename_all = "camelCase")]
   pub struct ModelInstallSummary {
     pub last_checked_at: Option<String>,  // ISO8601, /api/tags 성공 시각
   }
   ```
2. **Throttle 정책**: `get_ollama_status`는 hot path (UI 열림 시마다 호출됨). 매 호출마다 disk write 하면 안 됨.
   - 마지막 persist된 `last_checked_at`과 비교해 **5분(300초) 이상 경과**한 경우에만 settings 업데이트 + persist.
   - throttle은 `SettingsStore`에 helper로 추가: `fn maybe_touch_last_checked(&self, now: DateTime) -> bool` — 내부 lock 안에서 비교 후 조건부 update. 동시 호출 race-safe.
3. throttle 대상이 아니어도 SettingsStore 내부 in-memory snapshot은 항상 갱신(다른 영역의 disk write 시 같이 flush). disk write만 throttle.

**FE TS mirror — `src/features/settings/types.ts`**
1. `Settings`에 `modelInstallState: { lastCheckedAt: string | null }` 추가.
2. `DEFAULT_SETTINGS`에 `modelInstallState: { lastCheckedAt: null }` 추가.

**FE — `src/features/settings/store.test.ts`, `ipc.test.ts`**
1. 새 필드가 round-trip되는지 검증 추가.

**FE — `src/features/settings/components/settings-panel.tsx`**
1. UI 노출은 없음 (v1은 디버그 정보 안 보여줌). 단지 round-trip만 보장.

**docs — `docs/hytranslate-mac-prd.md` §9.3**
1. ModelInstallState 정의 옆에 보조 노트 추가: "v1에서는 ollama_name/installed/recommended는 `/api/tags` 런타임 응답으로 대신하고, last_checked_at만 settings.modelInstallState에 영속화한다. throttle: 5분."

### 검증
- cargo test:
  - settings 직렬화/역직렬화 라운드트립 (새 필드 포함).
  - 기존 settings.json(이전 버전 — 새 필드 없음) → `#[serde(default)]` 로 deserialize 성공.
  - `maybe_touch_last_checked`: 첫 호출에서 update + persist, 5분 이내 재호출은 no-op, 5분 초과 시 다시 update.
- Vitest: settings store가 새 필드를 보존하고 update flow에서 노출하는지.

---

## 11. Critical 1 — E2E shell + 평가셋 골조 (**DoD 미충족 상태 유지**)

> ⚠️ **Framing 명시 (코드리뷰 Major 5 반영)**: 본 follow-up은 Critical 1을 **해결하지 않는다**. 사용자 결정(Q3)에 따라 v1.0 출시 전 별도 작업으로 트래킹한다. 이번 단계의 산출물은 "임시 골조 + 후속 작업 트래킹 기반"이며, PRD §14.1 / §14.2 / §14.3 / §19 (마지막 두 bullet — 필수 unit/integration/E2E 통과, 품질 평가셋 기준 만족)은 **여전히 미충족 상태**이다. 이 항목들은 본 plan 완료 보고에서도 "open" 으로 표시한다.

### 후속 작업으로 떨어지는 항목

이 follow-up에서 다루지 않는 것 (별도 트래킹):
- PRD §14.3의 5종 E2E를 실제 assertion (skip 해제) — Tauri Playwright 통합 환경 필요.
- 평가셋 100건 채점 (40 한국어 + 40 간체 + 20 번체) + 모델별 점수 합산 + 합격선 검증.
- 위 둘이 끝나지 않으면 PRD §19 DoD는 닫히지 않는다.

### E2E 변경

**`tests/e2e/`**
1. `sanity.spec.ts` 유지.
2. 신규 shell 스펙 5개. 모두 `test.skip(true, '<reason>')` 또는 `test.fixme`로 시작 — 실제 dev/build 통합은 후속. assertion plan은 주석으로 남긴다.
   - `onboarding-happy-path.spec.ts`
   - `translate-main.spec.ts`
   - `translate-popup-shortcut.spec.ts`
   - `history-search-favorite.spec.ts`
   - `clipboard-translate.spec.ts`
3. `playwright.config.ts`가 없으면 추가 (혹은 기존 설정 확인).

### 평가셋 변경

**`evals/translation-quality.md`**
1. 표 컬럼: `#`, `언어`, `도메인`, `source_text`, `reference_en`, `hy-mt2-7b`, `hy-mt2-1.8b`, `reviewer`, `note` — 그대로 유지.
2. 대표 10건 채움:
   - 한국어 4건 (일상/비즈니스/IT/법률)
   - 간체 4건 (일상/비즈니스/학술/IT)
   - 번체 2건 (일상/비즈니스)
   - `source_text`와 `reference_en`만 작성. 점수/note 칼럼은 빈칸.
3. 표 위에 "v1.0 출시 전 100건 채점 별도 트래킹" 노트.

### 검증
- `npx playwright test --list`로 5종 + sanity 총 6개 스펙 출력.
- 평가셋 markdown 표 렌더링 확인.

---

## 12. Minor 1~4

| # | 작업 |
|---|---|
| M1 | `TranslationState.recent` 제거 (Major 2와 묶음). 주석 동기화. |
| M2 | `src/features/history/components/history-panel.tsx`의 export 버튼 영역에 inline 안내 추가: "내보낸 파일에는 원문/결과가 평문으로 포함됩니다." i18n 키 `history.export.notice` 신설. |
| M3 | `src-tauri/src/ollama/endpoint.rs` — `https://localhost*`를 allowlist에서 **제외**. 기본 endpoint가 `http://localhost:11434`이고 PRD가 외부 송신을 금지하므로 HTTPS loopback은 의도되지 않음. 변경 후 회귀 테스트 추가. |
| M4 | copy 실패 inline 노출: `src/features/translation/components/translation-panel.tsx` / `src/windows/popup/popup.tsx` / `src/lib/hooks/use-auto-copy-translation.ts`에서 `catch` 시 store에 `copyError` 플래그(또는 AppError) 저장 → UI에 1.5초 inline 메시지. |

### 검증
- M3: cargo test에서 `https://localhost:11434`가 거부되는지 추가.
- M4: Vitest로 copy 실패 시 store 상태 확인.

---

## 13. 작업 순서 / 커밋 단위

각 커밋은 conventional commits + 한국어 본문.

1. `refactor(state): typing/detecting 상태 추가` (Major 8) — state enum + i18n
2. `fix(controller): 입력 변경 즉시 in-flight 취소` (Critical 2) — controller + store + 테스트
3. `feat(translation): 감지 결과 UI badge` (Major 1) — started payload 확장 + select badge
4. `feat(paths): PRD 경로 copy+verify 자동 마이그레이션` (Major 7 자동 단계) — `rusqlite` features에 `backup` 추가 + paths 모듈 + atomic_write_file helper + setup hook
4b. `feat(settings): legacy 경로 정리 CTA` (Major 7 수동 단계) — cleanup_legacy command + settings UI 배너/모달
5. `feat(menubar): tray menu + DB recent` (Major 2 + Minor 1) — 메뉴 + 이력 조회
6. `feat(clipboard): feature 분리 + inline 오류` (Major 3) — feature dir + 훅
7. `feat(popup): active monitor + reopen focus` (Major 4) — popup 명령 + 이벤트
8. `feat(onboarding): Ollama exponential backoff` (Major 5) — backoff lib + store
9. `feat(settings): last_checked_at 영속` (Major 6) — settings + status command
10. `chore(security): https loopback 제거 + export 안내 + copy 실패 표시` (Minor 2/3/4)
11. `test(e2e): 5종 shell + 평가셋 골조` (Critical 1) — playwright 스펙 + evals

---

## 14. 후속 트래킹 (이번 반영 범위 외 — **v1.0 DoD에 직접 영향**)

- **PRD §14.3 5종 E2E 본 구현**: skip 해제 + 실제 assertion. Playwright Tauri 통합 환경 결정 필요.
- **PRD §14.1/14.2 평가셋 100건 채점**: evals 표 채움 + 합격선 검증 (전체 평균 ≥ 4.0, 치명적 오역 ≤ 5%, 언어별 평균 ≥ 3.8). 채점자 / 일정 별도 결정.
- **PRD §19 DoD 닫기**: 위 두 항목 완료 전까지 "필수 unit/integration/E2E 통과" 와 "품질 평가셋 기준 만족" bullet은 미충족.
- legacy-backup 정리 CTA (v1.1 후보): 설정 패널에 "이전 백업 삭제" 버튼 + 확인 모달 추가. 자동 정리는 도입하지 않는다 (§3 참고).
- ModelInstallState DB 영속이 필요하다고 판단되면 v1.1에서 재논의.
- macOS 13 실 디바이스에서 tray menu의 Dock 숨김 모드 동작 확인 (§15).
- Test hygiene: `use-auto-copy-translation.test.tsx`의 React `act(...)` warning 정리 (테스트 자체는 통과). 코드리뷰 verification에서 발견.

---

## 15. 사용자 결정 사항 (확정)

- **legacy 경로 정리 정책 (2차 코드리뷰 반영으로 갱신)**: startup은 새 경로에 **copy + verify만** 자동 수행하며 legacy는 손대지 않는다. legacy 정리는 사용자가 설정 UI의 `[legacy 경로 정리하기]` CTA를 명시적으로 클릭했을 때만 실행되며, 우리 파일을 `legacy-backup-<ts>/`로 이동 후 빈 디렉터리면 `remove_dir` 수행. 외부 파일이 남아 있으면 디렉터리는 보존. → §3 (Major 7) 작업에 반영됨.
- **tray menu Dock 숨김 모드 동작**: 런타임에서 검증 예정. tray menu의 "이력/설정 열기"가 Dock 숨김 모드에서 정상 동작하는지 실행 후 확인. 문제가 발견되면 별도 이슈로 트래킹.
- **backoff cap 8s**: 충분하다고 판단. §8 (Major 5)의 delays = [250, 500, 1000, 2000, 4000, 8000] 시퀀스 그대로 진행.
