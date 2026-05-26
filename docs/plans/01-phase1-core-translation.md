# Phase 1 — 핵심 번역 루프 (Core Translation Loop)

> **상태**: 작성 완료 — 즉시 구현 진행
> **참조**: `docs/hytranslate-mac-prd.md` §15.1 / §8.1 / §8.3 / §10.1 / §10.2 / §10.4, `.claude/rules/*`, `docs/plans/00-initial-setup.md`
> **선행 조건**: `00-initial-setup.md` 모든 산출물 완료. `npm run tauri:dev` 으로 빈 메인 창이 뜨는 상태.

---

## 1. 목적

PRD §15.1 의 완료 기준 3 줄을 모두 만족시킨다.

1. 한국어 또는 중국어 입력을 영어로 streaming 번역한다.
2. 입력 변경 시 이전 요청이 취소된다.
3. 번역 완료 시간이 표시된다.

그리고 PRD §8.1 수용 기준 4 줄을 만족한다.

- 정상 환경에서 첫 출력 chunk 가 도착하기 전까지 UI 가 멈추지 않는다.
- 진행 중인 요청 중 입력을 바꾸면 이전 결과가 최종 결과로 저장되지 않는다.
- streaming 중 partial UTF-8 때문에 깨진 문자가 표시되지 않는다.
- 번역 완료 후 `duration_ms` 가 기록된다 (Phase 1 에서는 UI 표시 + 이벤트 payload. DB 저장은 Phase 4).

## 2. 범위 (포함 / 제외)

**포함**

- Ollama HTTP client (`/api/generate` streaming 만)
- prompt builder (PRD §8.3 의 고정 template, target=English 하드코딩)
- `translate_stream` / `cancel_translation` Tauri command
- in-flight 요청 `CancellationToken` 맵
- UTF-8 안전 streaming 누적기
- 메인 윈도우 UI: 입력 textarea / 출력 영역 / 소스 언어 드롭다운 / 상태 표시 / `duration_ms` 표시 / 복사 / 재번역
- 500ms 디바운스 + 입력 변경 시 이전 요청 취소 + `Cmd+Enter` 즉시 번역
- 시스템/라이트/다크 테마 따라가기 (Settings 화면 없이 OS 설정 자동 추종)
- 메인 창 입력 30,000자 cap (초과 시 inline 에러)
- 한국어 i18n 키 (메인 윈도우 노출분)
- Rust unit test: prompt builder 3종 / Ollama mock streaming / 취소
- FE unit test: translation store 상태 전이 / ipc 래퍼

**제외 (다른 Phase 로 미룸)**

- 자동 언어 감지 → Phase 2
- 수동 override UI 확장 (Phase 1 에서는 드롭다운으로 갈음)
- 설정 화면 / 설정 저장 → Phase 2
- 단축키 / 팝업 / 메뉴바 / 클립보드 / autostart → Phase 3
- SQLite 이력 저장 → Phase 4
- 온보딩 / 모델 다운로드 / 설치 안내 → Phase 5
- Playwright E2E 신규 spec (sanity 만 유지)

## 3. 결정

| 항목            | 결정                                                                                                           |
| --------------- | -------------------------------------------------------------------------------------------------------------- |
| 소스 언어 처리  | **수동 드롭다운만**. 옵션: Korean / ChineseSimplified / ChineseTraditional. 디폴트 Korean. Auto/감지는 Phase 2 |
| 디폴트 모델     | `hf.co/tencent/Hy-MT2-7B-GGUF:Q4_K_M` (상수 하드코딩, Settings UI 없음)                                        |
| Ollama endpoint | `http://localhost:11434` 상수 (Settings UI 없음. Phase 2 에서 Settings 도입)                                   |
| prompt builder  | PRD §8.3 template 고정. `temperature 0.3`, `top_p 0.9`, `num_predict 512`                                      |
| 취소 메커니즘   | `tokio_util::sync::CancellationToken` + `DashMap<RequestId, CancellationToken>`                                |
| 이벤트 payload  | tauri-ipc 규칙 그대로 (`requestId`, `delta`, `fullText`, `durationMs`, `error`)                                |
| 디바운스        | 500ms. `Cmd+Enter` 는 디바운스 무시                                                                            |
| 30,000자 초과   | `AppError::InputTooLong { limit: 30000 }` inline 표시                                                          |
| 테마            | `prefers-color-scheme` listener + `<html>` `dark` 클래스 토글. Settings 화면 없으므로 자동만                   |
| 로깅            | `tracing::info!` 라이프사이클만. `source_text` / `translated_text` 절대 로그 금지                              |

## 4. 추가 의존성 (이번 Phase 에서 잠금)

`src-tauri/Cargo.toml`

```toml
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls", "stream"] }
futures-util = "0.3"
tokio-util = "0.7"
dashmap = "6"
```

dev-dependencies:

```toml
wiremock = "0.6"
tokio = { version = "1", features = ["rt-multi-thread", "macros", "sync", "time"] }
```

> FE 신규 npm 의존성 없음.

## 5. 아키텍처 (Phase 1 추가분)

```
src/
├── features/
│   └── translation/
│       ├── components/
│       │   ├── translation-panel.tsx        # 입력 + 출력 + 상태
│       │   ├── source-language-select.tsx   # 드롭다운
│       │   └── inline-error.tsx             # AppError → UI 메시지
│       ├── store.ts                         # Zustand: status / chunks / error / requestId / startedAt / durationMs
│       ├── ipc.ts                           # invoke('translate_stream'), invoke('cancel_translation'), event listeners
│       └── types.ts                         # SourceLanguage, TranslationStatus
├── i18n/
│   └── ko.ts                                # 라벨/에러 메시지 추가
└── windows/
    └── main/
        └── main.tsx                         # AppRoot → translation panel mount

src-tauri/src/
├── ollama/
│   ├── client.rs        # OllamaClient::generate_stream
│   ├── models.rs        # MODEL_7B / MODEL_1_8B 상수
│   ├── prompt.rs        # build_prompt(source_language, source_text)
│   └── mod.rs
├── language/
│   └── mod.rs           # SourceLanguage enum + Display (감지기는 Phase 2)
├── commands/
│   ├── translate.rs     # translate_stream / cancel_translation 핸들러
│   └── mod.rs           # register
├── errors.rs            # AppError + From<reqwest::Error> 추가
└── events.rs            # 기존 상수 + (이번 Phase 사용분)
```

## 6. 단계별 작업

### Step 1 — 라이브러리 문서 확인

- Tauri 2 `Emitter` / `AppHandle::emit_to` 시그니처 (윈도우별 이벤트 emit)
- reqwest 0.12 `bytes_stream` + `Response::error_for_status`
- `tokio_util::sync::CancellationToken` 의 `is_cancelled` / `cancelled()`
- wiremock 0.6 streaming/chunked response 지원 패턴

→ 결과는 본 문서가 아닌 코드 주석으로 필요 시 남긴다.

### Step 2 — Rust 도메인 모듈

1. `language/mod.rs`: `SourceLanguage` enum (`Korean`, `ChineseSimplified`, `ChineseTraditional`) + `to_prompt_label()`
2. `ollama/models.rs`: `MODEL_HY_MT2_7B`, `MODEL_HY_MT2_1_8B` 상수
3. `ollama/prompt.rs`: `build_prompt(lang, text) -> String` 순수 함수 + unit test 3종
4. `ollama/client.rs`:
   - `OllamaClient::new(endpoint, http)` (싱글톤은 main 에서 1회 생성)
   - `generate_stream(req: GenerateRequest, on_chunk: F)` async — chunk 콜백 패턴
   - 내부 `String` UTF-8 누적기 (`from_utf8_lossy` 금지, 안전 split 필요)
5. `errors.rs`: `From<reqwest::Error>` → connect 에러는 `OllamaNotRunning`, 그 외 `Internal { message }`
6. `events.rs`: 변경 없음 (기존 상수 그대로)

### Step 3 — 명령 핸들러

`commands/translate.rs`:

- 입력 길이 30,000자 초과 시 `Err(AppError::InputTooLong { limit: 30_000 })`
- `requestId` 검증 (UUID v4)
- `CancellationToken` 생성, `state.tokens` 에 삽입
- `tokio::spawn` 으로 worker 실행 — emit `translation:started`, chunk 수신 시 emit `translation:chunk`, 종료 시 `translation:completed` / `translation:cancelled` / `translation:error`
- 종료 시 `state.tokens` 에서 제거

`cancel_translation`:

- `state.tokens.get(&request_id).map(|t| t.cancel())`
- 즉시 `Ok(())`. 실제 cancellation 처리는 worker 가 토큰 확인 후 emit

### Step 4 — FE translation feature

- `types.ts`: `SourceLanguage = 'Korean' | 'ChineseSimplified' | 'ChineseTraditional'`, `TranslationStatus = 'idle' | 'debouncing' | 'translating' | 'completed' | 'cancelled' | 'error'`
- `store.ts`: Zustand store
  - state: `sourceText`, `sourceLanguage`, `output`, `status`, `error`, `requestId`, `startedAt`, `durationMs`
  - actions: `setSourceText`, `setSourceLanguage`, `start`, `appendChunk`, `complete`, `cancel`, `fail`, `reset`
- `ipc.ts`: `translateStream(req)`, `cancelTranslation(id)`, `listenTranslationEvents(handlers)`
- 컴포넌트: 메인 윈도우에서 mount. textarea 입력, 500ms debounce → `start()`, `Cmd+Enter` 즉시 → `start()`, 입력 중 변경 → 이전 `cancel()` 후 새 `start()`

### Step 5 — 메인 윈도우 UI

- 한국어 라벨, lucide 아이콘
- 상단: 모델 badge (`Hy-MT2 7B`) + 상태 indicator
- 좌측: 입력 textarea (max 30,000자 시각 표시)
- 우측: 출력 영역 (`aria-live="polite"`)
- 하단 우측: `duration_ms` 표시 + 복사 버튼
- 좌측 아래: 소스 언어 드롭다운
- 에러 발생 시 출력 영역 자리에 inline error

### Step 6 — 테마

- `src/lib/theme.ts`: `applyTheme(mode)` — Settings 없이 'System' 고정 사용
- `<html>` 의 `dark` 클래스를 `prefers-color-scheme: dark` 에 따라 토글
- 시스템 변화 listener 등록

### Step 7 — 한국어 i18n

`src/i18n/ko.ts` 에 키 추가:

- `translation.input.placeholder`
- `translation.input.tooLong` (`{limit}` 치환)
- `translation.output.placeholder`
- `translation.status.idle` / `.translating` / `.completed` / `.cancelled` / `.error`
- `translation.copy` / `.retranslate`
- `translation.sourceLanguage.label` / `.korean` / `.chineseSimplified` / `.chineseTraditional`
- `errors.OllamaUnavailable` / `OllamaNotRunning` / `ModelMissing` / `TranslationFailed`

### Step 8 — 테스트

**Rust** (`cargo test --manifest-path src-tauri/Cargo.toml`)

- `ollama::prompt::tests` — 3 언어별 prompt 검증 (PRD §8.3 수용 기준)
- `ollama::client::tests` — wiremock 으로 chunked 응답 모킹 후 누적 검증
- `commands::translate::tests` — 토큰 등록/취소, worker 가 cancellation 후 chunk 무시

**FE** (`vitest run`)

- `features/translation/store.test.ts` — start → chunk → complete 전이, cancel 처리, fail 처리
- `features/translation/ipc.test.ts` — `invoke` / `listen` mock 으로 wrapper 검증

### Step 9 — 검증

- `npm run format:check`
- `npm run lint`
- `npm run typecheck`
- `npm run test`
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`
- `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm run tauri:dev` 수동 smoke (사용자가 Ollama + 모델 있는 환경에서 확인)

## 7. 위험

| 위험                                                             | 완화                                                                        |
| ---------------------------------------------------------------- | --------------------------------------------------------------------------- |
| `tauri::generate_handler!` 매크로가 module 분리된 함수를 못 잡음 | `commands/mod.rs` 가 핸들러 함수를 `pub use` 로 끌어올림                    |
| reqwest 0.12 stream feature 와 default feature 충돌              | `default-features = false` + rustls-tls 명시                                |
| Ollama 미설치 환경에서 dev/test 가 멈춤                          | client 는 connect 실패 → `OllamaNotRunning` 매핑. Unit test 는 wiremock     |
| Tauri 2 emit_to (label 지정) 시그니처 변경 가능                  | Step 1 문서 확인 후 진행                                                    |
| UTF-8 partial codepoint 시각화 깨짐                              | `String` 누적기 + `str::from_utf8` 잔여 바이트 보존 패턴. Unit test 로 검증 |

## 8. 산출물 체크리스트

**파일 생성**

- [ ] `docs/plans/01-phase1-core-translation.md` (본 문서)
- [ ] `src-tauri/src/ollama/{client,models,prompt}.rs` + `mod.rs` 업데이트
- [ ] `src-tauri/src/language/mod.rs` (`SourceLanguage`)
- [ ] `src-tauri/src/commands/translate.rs` + `mod.rs` 업데이트
- [ ] `src-tauri/src/errors.rs` 확장 (`From<reqwest::Error>`)
- [ ] `src-tauri/Cargo.toml` 의존성 추가
- [ ] `src/features/translation/{store,ipc,types}.ts`
- [ ] `src/features/translation/components/{translation-panel,source-language-select,inline-error}.tsx`
- [ ] `src/lib/theme.ts` + `src/windows/main/main.tsx` 진입점 갱신
- [ ] `src/i18n/ko.ts` 키 추가

**검증 통과**

- [ ] `npm run format:check`
- [ ] `npm run lint`
- [ ] `npm run typecheck`
- [ ] `npm run test`
- [ ] `cargo fmt --check`
- [ ] `cargo clippy -- -D warnings`
- [ ] `cargo check`
- [ ] `cargo test`

**런타임 smoke** (사용자 검토 단계)

- [ ] Ollama + Hy-MT2 7B 모델이 준비된 환경에서 `npm run tauri:dev` → 한국어 입력 → 영어 출력 streaming 확인
- [ ] 입력 도중 다른 문장으로 바꾸면 이전 요청이 취소되고 새 요청이 시작됨
- [ ] 완료 후 `duration_ms` 표시

## 9. 완료 후 흐름

1. Phase 1 산출물 단일 커밋 (`feat(phase1): core translation loop`).
2. 사용자 검토 대기. Phase 2 진입은 별도 지시 후.
