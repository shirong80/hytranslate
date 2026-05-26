# Phase 2 — 언어 감지와 설정 (Language Detection & Settings)

> **상태**: 작성 완료 — 즉시 구현 진행
> **참조**: `docs/hytranslate-mac-prd.md` §15.2 / §8.2 / §8.3 / §7.5 / §9.2 / §10.3, `.claude/rules/*`, `docs/plans/01-phase1-core-translation.md`
> **선행 조건**: Phase 1 완료. 모든 검증 그린 상태.

---

## 1. 목적

PRD §15.2 의 완료 기준 3 줄을 모두 만족시킨다.

1. 한국어, 간체, 번체 샘플을 감지한다.
2. override 값이 prompt 에 반영된다.
3. 기본 설정이 앱 재시작 후 유지된다.

그리고 PRD §8.2 수용 기준을 만족한다.

- 한글만 포함한 입력은 Korean 으로 감지한다.
- 간체 중국어 샘플은 ChineseSimplified 로 감지한다.
- 번체 중국어 샘플은 ChineseTraditional 로 감지한다.
- 사용자가 override 하면 이후 번역 요청에 override 값이 사용된다.

## 2. 범위 (포함 / 제외)

**포함**

- 언어 감지 함수 `language::detect(&str) -> DetectionResult`
  - Hangul 비율 우선 → Korean
  - CJK 한자 비율로 Chinese 후보
  - 간체/번체는 frequency table 로 분리
  - 모호하면 `Auto` (prompt 에서는 generic `Chinese` fallback)
- `detect_language` Tauri 커맨드 (PRD §10.3)
- `SourceLanguage::Auto` variant 도입 (Rust / TS 동기화)
- `translate_stream` 입력이 `Auto` 인 경우 backend 가 detect 후 prompt 생성
- prompt builder fallback 라벨 `"Chinese"` 추가
- `settings` 모듈 + `Settings` struct (PRD §9.2 모든 필드)
- 영속화: `~/Library/Application Support/HyTranslate Mac/settings.json` (Phase 4 SQLite 도입 전까지 단순 JSON 파일)
- `get_settings` / `update_settings` Tauri 커맨드
- `OllamaClient` endpoint 인자화 (Settings 값 사용)
- FE `src/features/settings/` 신설 (types / ipc / store)
- 메인 윈도우에 설정 화면 라우팅 (별도 윈도우 없이 패널 토글)
- 설정 화면 v1 UI 필드 3 종: 활성 모델 / Ollama endpoint / 테마
- 번역 dropdown 에 `Auto` 추가, 기본값 `Auto`
- 한국어 i18n 키 추가

**제외 (다음 Phase)**

- 전역 단축키 / autostart / Dock 숨김 / 자동복사 토글 UI (Phase 3)
- 이력 ON/OFF / 전체 삭제 UI (Phase 4)
- 온보딩 / 모델 추천 (Phase 5)
- SQLite (Phase 4)
- 문장별 혼합 언어 감지 (PRD §8.2 명시 제외)

## 3. 결정 사항

| 결정                            | 선택                                                   | 근거                                                                                                           |
| ------------------------------- | ------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------- |
| 설정 저장소                     | JSON 파일                                              | Phase 4 에서 SQLite 도입. Phase 2 만을 위해 DB infra 끌어들이지 않음                                           |
| 설정 화면 진입                  | 메인 창 내부 패널 전환                                 | 별도 윈도우는 Phase 3 단축키와 함께 도입 가치 있을 때 검토                                                     |
| Auto resolution 위치            | Backend (translate_stream 내부)                        | UI 가 detect 결과를 다시 payload 로 보내면 race condition. Backend 가 받은 `Auto` 를 그 자리에서 detect → 결정 |
| OllamaClient endpoint 변경 방식 | 매 요청 인자로 전달                                    | mutable shared state 회피. Settings 가 단일 source of truth                                                    |
| 모호 Chinese 감지 prompt        | label `"Chinese"`                                      | PRD §8.2 명시 — generic Chinese fallback                                                                       |
| 간체/번체 판정                  | 대표 문자 frequency table (간체 전용 + 번체 전용 양쪽) | PRD §8.2 명시 방식                                                                                             |
| 설정 영속화 directory           | Tauri `app_data_dir`                                   | macOS 자동 매핑: `~/Library/Application Support/HyTranslate Mac/`                                              |

## 4. 산출물 체크리스트

### 4.1 Backend

- [ ] `src-tauri/src/language/mod.rs` — `SourceLanguage::Auto` 추가
- [ ] `src-tauri/src/language/detector.rs` — Hangul / CJK ratio + frequency table
- [ ] `src-tauri/src/ollama/prompt.rs` — `Auto` 입력 시 `"Chinese"` fallback 라벨
- [ ] `src-tauri/src/settings/mod.rs` — `Settings` struct + 기본값 + (de)serialize
- [ ] `src-tauri/src/settings/store.rs` — load/save 영속화 (JSON), 메모리 캐시 (RwLock)
- [ ] `src-tauri/src/commands/detect.rs` — `detect_language` 커맨드
- [ ] `src-tauri/src/commands/settings.rs` — `get_settings` / `update_settings` 커맨드
- [ ] `src-tauri/src/commands/mod.rs` — 신규 모듈 등록 + `SettingsStore` manage
- [ ] `src-tauri/src/commands/translate.rs` — `Auto` resolve, endpoint from Settings
- [ ] `src-tauri/src/ollama/client.rs` — `generate_stream(base_url, ...)` 인자화

### 4.2 Frontend

- [ ] `src/features/settings/types.ts` — Settings mirror
- [ ] `src/features/settings/ipc.ts` — `getSettings` / `updateSettings`
- [ ] `src/features/settings/store.ts` — Zustand store + onMount load
- [ ] `src/features/settings/components/settings-panel.tsx` — 폼
- [ ] `src/features/translation/types.ts` — `SourceLanguage` Auto 추가, 기본값 `Auto`
- [ ] `src/features/translation/components/source-language-select.tsx` — Auto 옵션
- [ ] `src/windows/main/main.tsx` — in-memory route + 설정 진입 버튼, theme/model 을 Settings 기준으로 적용
- [ ] `src/i18n/ko.ts` — 신규 키

### 4.3 Tests

- [ ] Rust: `language::detector` 4개 이상 — Korean / Simplified / Traditional / mixed-ambiguous
- [ ] Rust: `settings::store` round-trip (load default → update → reload)
- [ ] Rust: `prompt` Auto fallback 라벨 케이스
- [ ] FE: `settings/store.test.ts` 기본값 / update
- [ ] FE: `settings/ipc.test.ts` invoke 매핑

## 5. 비범위 (명시적 제외)

- DB encryption (PRD §12.2 — v1 OUT)
- 전역 단축키 동작 (Phase 3)
- 메뉴바 / 팝업 윈도우 (Phase 3)
- 이력 저장 / 검색 (Phase 4)
- 모델 자동 추천 / 다운로드 (Phase 5)

## 6. 검증 계획

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
npm run typecheck
npm run lint
npm run format:check
npm test -- --run
```

전부 그린 후 단일 커밋 `feat(phase2): 언어 감지 + 설정 영속화 + 설정 화면 scaffold`.

## 7. 리스크 / 미해결

- 간체/번체 frequency table — 모든 케이스 보장 X. 명백한 샘플 위주로 1차 통과, 사용자 override 가 안전망.
- macOS `app_data_dir` 권한: Tauri 기본 capability 안에서 `path::app_data_dir` 사용 — 별도 권한 추가 없음.
