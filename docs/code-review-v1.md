# HyTranslate Mac v1 Code Review

검토 기준: `docs/hytranslate-mac-prd.md` Phase 1~5.  
검토 방식: 정적 코드 리뷰. 테스트/빌드/앱 실행은 수행하지 않음.  
범위: `src-tauri/`, `src/`, `tests/`, `evals/`, `docs/`, `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`.

## 1. Executive Summary

총평: 핵심 골격은 PRD 방향과 대체로 맞다. Tauri 2, React/TS/Zustand, Rust command bridge, Ollama streaming, SQLite FTS5, 온보딩, 모델 pull, 설정 저장, 단축키/메뉴바/팝업의 주요 모듈은 존재한다. 그러나 v1 완료 선언으로 보기에는 몇 가지 핵심 요구가 비어 있다. 특히 자동 언어 감지 결과 UI 반영, 입력 변경 즉시 취소, 메뉴바 메뉴/DB 기반 최근 이력, 클립보드 비텍스트 처리, 필수 E2E/품질 평가 충족 증거가 부족하다.

매칭 통계: ✅ 69 / ⚠️ 22 / ❌ 9 / ❓ 5.  
이슈 통계: Critical 2 / Major 8 / Minor 4.

Top 권고:

1. 입력 변경 즉시 `cancel_translation`을 호출하도록 controller lifecycle을 고쳐 stale 번역 저장을 차단한다.
2. `detect_language` 결과를 FE 상태와 UI badge에 연결하고, override와 Auto fallback 정책을 명확히 분리한다.
3. 메뉴바 요구사항을 완성한다: compact popover의 최근 5개 DB 이력, 클립보드 오류 표시, 메인/이력/설정/종료 메뉴.
4. 클립보드 번역을 command/feature 경계로 승격해 빈/이미지/파일 clipboard를 inline 오류로 구분한다.
5. PRD §14.3 테스트 10종과 §14.1/14.2 품질 평가셋을 실제 검증 결과로 채운다.

## 2. PRD ↔ 구현 매칭표

상태 표기: ✅ 완료 / ⚠️ 부분 / ❌ 미구현 / ❓ 확인필요.

### Section 4.1 v1 포함 항목

| PRD 항목 | 상태 | 근거 |
|---|---:|---|
| Tauri 2 macOS 앱 | ✅ | `src-tauri/Cargo.toml:18`, `src-tauri/tauri.conf.json:1` |
| React/TypeScript/Tailwind | ✅ | `package.json:23-31`, `tailwind.config.ts`, `src/windows/main/main.tsx:1-13` |
| Rust 백엔드 | ✅ | `src-tauri/src/lib.rs:1-25` |
| Ollama HTTP API 연동 | ✅ | `src-tauri/src/ollama/client.rs:25-27`, `src-tauri/src/ollama/client.rs:116-147` |
| Hy-MT2 7B/1.8B GGUF | ✅ | `src-tauri/src/commands/onboarding.rs:25`, `src/features/settings/components/settings-panel.tsx:12-13` |
| 실시간 streaming 번역 | ✅ | `src-tauri/src/ollama/client.rs:132-186`, `src-tauri/src/commands/translate.rs:199-232` |
| 한/중 간체/번체 자동 감지 | ⚠️ | detector/command는 있음: `src-tauri/src/language/detector.rs:34-101`, `src-tauri/src/commands/detect.rs:16-18`; FE 표시/호출 연결 부족 |
| 수동 언어 override | ✅ | `src/features/translation/components/source-language-select.tsx:17-32`, `src/features/translation/use-translation-controller.ts:109-114` |
| 전역 단축키 호출 | ✅ | `src-tauri/src/shortcuts/mod.rs:20-37`, `src-tauri/src/commands/mod.rs:101-105` |
| 플로팅 번역 팝업 | ⚠️ | 창/토글은 있음: `src-tauri/tauri.conf.json:25-40`, `src-tauri/src/commands/popup.rs:26-38`; 활성 화면/재오픈 focus/80% 높이 제한 불충분 |
| 메뉴바 모드 | ⚠️ | tray/popover는 있음: `src-tauri/src/menubar/mod.rs:28-60`; PRD 메뉴 항목 부재 |
| 클립보드 번역 | ⚠️ | menubar `readText`만 있음: `src/windows/menubar/menubar.tsx:53-60`; popup 버튼/비텍스트 오류 없음 |
| SQLite 이력 저장 | ✅ | `src-tauri/src/db/migrations/0001_init.sql:8-20`, `src-tauri/src/commands/translate.rs:255-304` |
| 이력 검색/favorite/tag | ✅ | `src-tauri/src/history/mod.rs:135-201`, `src/features/history/components/history-panel.tsx:121-156` |
| 첫 실행 온보딩 | ✅ | `src/windows/main/main.tsx:51-60`, `src/features/onboarding/components/onboarding-screen.tsx:39-81` |
| Ollama 설치 감지/공식 안내 | ✅ | `src-tauri/src/commands/onboarding.rs:134-156`, `src/features/onboarding/components/onboarding-screen.tsx:364-372` |
| 모델 다운로드 진행률 | ✅ | `src-tauri/src/commands/onboarding.rs:207-244`, `src/features/onboarding/components/onboarding-screen.tsx:532-580` |
| Ollama 상태 확인/재연결 | ⚠️ | status/try start는 있음: `src-tauri/src/commands/onboarding.rs:163-205`; exponential backoff는 없음 |
| 한국어 UI | ✅ | `src/i18n/ko.ts:1-185` |
| light/dark/system 테마 | ✅ | `src/features/settings/components/settings-panel.tsx:149-160`, `src/lib/theme.ts` |
| 시작 시 자동 실행 | ✅ | `src-tauri/src/commands/system.rs:35-54`, `src/features/settings/components/settings-panel.tsx:134-140` |
| 기본 품질 평가셋 | ❌ | `evals/translation-quality.md:16-20`는 빈 템플릿 |

### Section 8 기능 요구

| 요구 | 상태 | 근거 |
|---|---:|---|
| `/api/generate`, `stream:true`, chunk 전달 | ✅ | `src-tauri/src/ollama/client.rs:132-147`, `src-tauri/src/commands/translate.rs:199-232` |
| 500ms 디바운스 | ✅ | `src/features/translation/use-translation-controller.ts:9`, `src/features/translation/use-translation-controller.ts:137-141` |
| 입력 변경 시 진행 요청 즉시 취소 | ❌ | 새 non-empty 입력은 timer만 재설정하고 즉시 cancel하지 않음: `src/features/translation/use-translation-controller.ts:128-149` |
| Cmd+Enter 즉시 번역 | ✅ | `src/features/translation/components/translation-panel.tsx:37-45`, `src/windows/popup/popup.tsx:89-93` |
| partial UTF-8 안전 | ⚠️ | line buffer는 newline 단위로 보존하나 `String::from_utf8_lossy` 사용: `src-tauri/src/ollama/client.rs:300-308`; 실제 split codepoint 회귀 테스트 부족 |
| duration_ms 기록 | ✅ | `src-tauri/src/commands/translate.rs:255-275`, `src-tauri/src/history/mod.rs:27` |
| 언어 감지 알고리즘 | ✅ | `src-tauri/src/language/detector.rs:34-101` |
| 감지 결과 UI 표시 | ❌ | UI는 수동 select만 렌더: `src/features/translation/components/source-language-select.tsx:17-32` |
| Prompt builder 원문 보존/English 고정 | ✅ | `src-tauri/src/ollama/prompt.rs:6-14`, `src-tauri/src/ollama/prompt.rs:66-70` |
| temperature/top_p/num_predict | ✅ | `src-tauri/src/ollama/client.rs:136-140`, `src-tauri/src/ollama/prompt.rs:17-22` |
| Ollama endpoint 기본값 | ✅ | `src-tauri/src/settings/mod.rs:46`, `src/features/settings/types.ts:27` |
| `/api/tags` 상태 확인 | ✅ | `src-tauri/src/ollama/client.rs:204-214`, `src-tauri/src/commands/onboarding.rs:163-184` |
| Ollama 자동 실행 시도 | ✅ | `src-tauri/src/commands/onboarding.rs:192-205` |
| exponential backoff 재연결 | ❌ | 800ms 단일 대기만 있음: `src/features/onboarding/store.ts:118-125` |
| 모델 다운로드 사용자 승인 | ✅ | 버튼 클릭 후 pull: `src/features/onboarding/components/onboarding-screen.tsx:458-466` |
| 단축키 변경/등록 실패 거부 | ✅ | `src-tauri/src/commands/settings.rs:32-60`, `src-tauri/src/shortcuts/mod.rs:48-101` |
| 클립보드 텍스트 읽기/결과 복사 | ⚠️ | 읽기: `src/windows/menubar/menubar.tsx:53-60`; 쓰기: `src/lib/clipboard.ts:7-16`; 비텍스트 오류 없음 |
| 자동 복사 기본 OFF | ✅ | `src-tauri/src/settings/mod.rs:42`, `src/lib/hooks/use-auto-copy-translation.ts:15-32` |
| 이력 저장 ON/OFF | ✅ | `src-tauri/src/settings/mod.rs:43`, `src-tauri/src/commands/translate.rs:294-299` |
| 취소/오류 저장 금지 | ⚠️ | backend 경로는 맞지만 즉시 취소 지연으로 stale completed 저장 가능: `src-tauri/src/commands/translate.rs:255-304`, `src/features/translation/use-translation-controller.ts:128-149` |

### Section 9 데이터 모델 필드 전수

| 모델/필드 | 상태 | 근거 |
|---|---:|---|
| TranslationRecord.id | ✅ | `src-tauri/src/db/migrations/0001_init.sql:10`, `src-tauri/src/history/mod.rs:22` |
| source_text | ✅ | `src-tauri/src/db/migrations/0001_init.sql:11`, `src-tauri/src/history/mod.rs:23` |
| source_language | ✅ | `src-tauri/src/db/migrations/0001_init.sql:12`, `src-tauri/src/history/mod.rs:24` |
| translated_text | ✅ | `src-tauri/src/db/migrations/0001_init.sql:13`, `src-tauri/src/history/mod.rs:25` |
| model | ✅ | `src-tauri/src/db/migrations/0001_init.sql:14`, `src-tauri/src/history/mod.rs:26` |
| duration_ms | ✅ | `src-tauri/src/db/migrations/0001_init.sql:15`, `src-tauri/src/history/mod.rs:27` |
| created_at | ✅ | `src-tauri/src/db/migrations/0001_init.sql:16`, `src-tauri/src/history/mod.rs:28` |
| is_favorite | ✅ | `src-tauri/src/db/migrations/0001_init.sql:17`, `src-tauri/src/history/mod.rs:29` |
| tags_json | ✅ | DB는 `tags_json`, API는 `tags`: `src-tauri/src/db/migrations/0001_init.sql:18`, `src-tauri/src/history/mod.rs:30`, `src-tauri/src/history/mod.rs:303-315` |
| Settings 9개 필드 | ✅ | Rust: `src-tauri/src/settings/mod.rs:18-28`; TS: `src/features/settings/types.ts:6-15` |
| Settings 기본값 | ✅ | `src-tauri/src/settings/mod.rs:37-50`, `src/features/settings/types.ts:20-30` |
| ModelInstallState 6개 필드 | ❌ | 별도 저장 모델/스키마 없음. 상태는 `/api/tags` 응답과 FE 메모리로만 관리: `src-tauri/src/commands/onboarding.rs:68-78`, `src/features/onboarding/store.ts:25-38` |
| SQLite FTS5 | ✅ | `src-tauri/src/db/migrations/0001_init.sql:25-31`, `src-tauri/src/history/mod.rs:135-201` |
| DB migration/schema version | ✅ | `src-tauri/src/db/mod.rs:19-23`, `src-tauri/src/db/mod.rs:57-78` |
| DB 경로 | ⚠️ | `app_data_dir()/hytranslate.sqlite` 사용: `src-tauri/src/commands/mod.rs:69-92`; PRD의 `~/Library/Application Support/HyTranslate Mac/hytranslate.sqlite`와 정확히 일치하는지 확인 필요 |

### Section 10 command/event 계약

| 계약 | 상태 | 근거 |
|---|---:|---|
| `translate_stream` | ✅ | `src-tauri/src/commands/translate.rs:95-153`, `src/features/translation/ipc.ts:29-31` |
| translation 5 events | ✅ | `src-tauri/src/events.rs:1-6`, `src/lib/ipc/events.ts:1-5` |
| `cancel_translation` | ✅ | `src-tauri/src/commands/translate.rs:155-162`, `src/features/translation/ipc.ts:33-35` |
| `detect_language` | ✅ | `src-tauri/src/commands/detect.rs:16-18` |
| `get_ollama_status` | ✅ | `src-tauri/src/commands/onboarding.rs:163-184` |
| `pull_model` | ✅ | `src-tauri/src/commands/onboarding.rs:207-244` |
| model pull 4 events | ✅ | `src-tauri/src/events.rs:8-12`, `src/features/onboarding/ipc.ts:59-81` |
| history `list_translation_records` | ✅ | `src-tauri/src/commands/history.rs:78-90` |
| `search_translation_records` | ✅ | `src-tauri/src/commands/history.rs:92-104` |
| `get_translation_record` | ✅ | `src-tauri/src/commands/history.rs:106-112` |
| `delete_translation_record` | ✅ | `src-tauri/src/commands/history.rs:114-120` |
| `delete_all_translation_records` | ✅ | `src-tauri/src/commands/history.rs:122-127` |
| `toggle_favorite` | ✅ | `src-tauri/src/commands/history.rs:129-135` |
| `set_tags` | ✅ | `src-tauri/src/commands/history.rs:137-143` |
| `export_history_csv` | ✅ | `src-tauri/src/commands/history.rs:145-162` |
| `export_history_json` | ✅ | `src-tauri/src/commands/history.rs:164-182` |

### Section 15 Phase 1~5 완료 기준

| Phase | 상태 | 근거 |
|---|---:|---|
| Phase 1: streaming 번역/취소/시간 표시 | ⚠️ | streaming/시간은 있음: `src-tauri/src/commands/translate.rs:199-275`; 입력 변경 즉시 취소 미충족 |
| Phase 2: 감지/override/prompt/settings persistence | ⚠️ | override/prompt/settings는 완료, 감지 결과 UI 미연결 |
| Phase 3: 단축키/팝업/메뉴바/클립보드/autostart/Dock | ⚠️ | 핵심 모듈은 있으나 메뉴 항목, clipboard 오류, popup sizing/focus 일부 미흡 |
| Phase 4: 이력/검색/favorite/tag/export/save off | ✅ | `src-tauri/src/history/mod.rs:76-287`, `src/features/history/components/history-panel.tsx:70-156` |
| Phase 5: 온보딩/Ollama/model lifecycle | ⚠️ | flow는 있음; exponential backoff/품질 검증/일부 완료 조건은 증거 부족 |

### Section 19 Definition of Done

PRD 원문은 14개 bullet로 보인다. 요청문은 13개라고 되어 있어, 아래는 PRD 원문 기준 전수 매핑이다.

| DoD | 상태 | 근거 |
|---|---:|---|
| macOS 13 이상 설치/실행 | ❓ | `minimumSystemVersion`: `src-tauri/tauri.conf.json:64`; 실제 설치/실행 미검증 |
| 첫 실행 온보딩으로 Ollama/model 확인 | ✅ | `src/windows/main/main.tsx:51-60`, `src/features/onboarding/components/onboarding-screen.tsx:287-470` |
| Hy-MT2로 한/중→영 번역 | ⚠️ | prompt/model wired: `src-tauri/src/ollama/prompt.rs:6-14`; 실제 품질/런타임 미검증 |
| streaming 결과 점진 표시 | ✅ | `src-tauri/src/commands/translate.rs:219-230`, `src/features/translation/store.ts:93-96` |
| 입력 변경 시 이전 요청 취소 | ❌ | 즉시 취소 아님: `src/features/translation/use-translation-controller.ts:128-149` |
| 전역 단축키로 팝업 열기 | ✅ | `src-tauri/src/shortcuts/mod.rs:20-37` |
| 메뉴바에서 번역 | ⚠️ | popover 번역은 있음: `src/windows/menubar/menubar.tsx:31-118`; 메뉴 명령 부재 |
| 클립보드 번역 | ⚠️ | text read만 있음: `src/windows/menubar/menubar.tsx:53-60`; 비텍스트/빈 오류 미구현 |
| 이력 SQLite 저장/검색 | ✅ | `src-tauri/src/db/migrations/0001_init.sql:8-45`, `src-tauri/src/history/mod.rs:135-201` |
| 이력 저장 off/전체 삭제 | ✅ | `src-tauri/src/commands/translate.rs:294-299`, `src-tauri/src/commands/history.rs:122-127` |
| 주요 오류 inline 표시 | ⚠️ | 일부 inline: `src/features/translation/components/translation-panel.tsx:151-156`; clipboard/menu flows는 부족 |
| 원문/결과 외부 전송 없음 | ✅ | endpoint allowlist: `src-tauri/src/ollama/endpoint.rs:6-15`, `src-tauri/src/ollama/client.rs:128-130` |
| 필수 unit/integration/E2E 통과 | ❌ | E2E는 sanity뿐: `tests/e2e/sanity.spec.ts:1-5`; 실행 증거 없음 |
| 품질 평가셋 기준 만족 | ❌ | 평가셋 비어 있음: `evals/translation-quality.md:16-20` |

## 3. 이슈 상세

### Critical 1. 필수 E2E/품질 평가가 v1 DoD를 충족하지 못함

위치: `tests/e2e/sanity.spec.ts:1-5`, `evals/translation-quality.md:16-20`  
현상: Playwright E2E는 `expect(true).toBe(true)`뿐이고, 평가셋은 빈 표다. PRD §14.1/14.2/14.3과 §19 DoD를 충족했다는 증거가 없다.  
원인 추정: 기능 구현 단계 후 검증 산출물이 채워지지 않음.  
영향도: v1 완료 기준 미충족. 실제 사용 흐름, 모델 품질, 설치/온보딩/메뉴바/팝업 통합 회귀를 탐지할 수 없다.  
권장 개선안: PRD §14.3 10종 테스트를 실제 assertion으로 작성하고, `evals/translation-quality.md` 100개 샘플과 채점 결과를 채운다.  
재현/검증법: `npm run test:e2e` 테스트 목록이 주요 사용자 흐름을 포함하는지, 평가셋에 100개 샘플과 점수 통계가 있는지 확인한다.

### Critical 2. 입력 변경 즉시 취소 미구현으로 stale 번역이 저장될 수 있음

위치: `src/features/translation/use-translation-controller.ts:128-149`, `src/features/translation/use-translation-controller.ts:87-126`, `src-tauri/src/commands/translate.rs:255-304`  
현상: 사용자가 입력을 바꾸면 기존 요청을 즉시 취소해야 하지만, 현재 effect는 새 500ms timer를 예약할 뿐 non-empty 입력 변경 시 `cancelInFlight()`를 즉시 호출하지 않는다. 기존 요청은 다음 `runTranslation()` 시점까지 계속 진행하며, 그 사이 완료되면 backend가 이력에 저장한다.  
원인 추정: "새 번역 시작 전 취소"와 "입력 변경 즉시 취소" 요구를 동일하게 취급.  
영향도: PRD §8.1/§19 위반, stale 결과 저장, 최근 번역/DB 이력 오염.  
권장 개선안: `sourceText/sourceLanguage/model` 변경 effect에서 기존 in-flight를 즉시 cancel하고, debounce는 새 요청 시작만 지연한다. 완료 이벤트가 온 뒤에도 현재 입력 snapshot과 request snapshot을 대조해 저장 전 방어한다.  
재현/검증법: 긴 번역 시작 후 500ms 이내 입력을 변경한다. 이전 request의 `translation:completed` 또는 DB insert가 발생하지 않아야 한다.

### Major 1. 자동 언어 감지 결과가 UI에 표시되지 않고 FE translate flow에 연결되지 않음

위치: `src-tauri/src/commands/detect.rs:16-18`, `src-tauri/src/language/detector.rs:34-101`, `src/features/translation/components/source-language-select.tsx:17-32`, `src/features/translation/use-translation-controller.ts:109-114`  
현상: detector와 command는 있지만 FE controller가 `detect_language`를 호출하지 않는다. UI도 "감지된 입력 언어 표시"가 아니라 Auto/manual select만 보여준다. backend는 `Auto`일 때 내부 resolve만 수행하고 결과를 event로 보내지 않는다.  
원인 추정: backend-side Auto fallback을 구현하면서 감지 결과 표시 요구가 빠짐.  
영향도: PRD §7.2/§8.2/§15 Phase 2 완료 기준 미충족. 사용자는 실제 감지 결과와 confidence를 알 수 없다.  
권장 개선안: FE controller에서 debounce 전/번역 전 `detect_language` 호출 또는 backend `translation:started` payload에 resolved language를 포함한다. UI badge와 override 상태를 분리한다.  
재현/검증법: 한글/간체/번체 입력 시 UI badge가 각각 Korean/ChineseSimplified/ChineseTraditional로 변하는지 확인한다.

### Major 2. 메뉴바 요구사항 중 메뉴 명령과 DB 기반 최근 5개가 빠짐

위치: `src-tauri/src/menubar/mod.rs:28-60`, `src/windows/menubar/menubar.tsx:120-145`, `src/features/translation/store.ts:23-24`  
현상: tray 클릭 popover는 있지만 PRD가 요구한 메뉴 "메인 창 열기, 이력 열기, 설정, 종료"가 없다. 최근 5개도 SQLite 이력이 아니라 해당 menubar webview의 in-memory `recent`에 의존한다.  
원인 추정: compact 번역 UI와 OS tray menu 요구를 혼동.  
영향도: PRD §6.5/§15 Phase 3/§19 메뉴바 DoD 부분 미충족. 앱 재시작 또는 다른 window에서 번역한 최근 이력이 메뉴바에 나타나지 않을 수 있다.  
권장 개선안: Tauri tray menu를 추가하고, menubar popover 진입 시 `list_translation_records({limit:5})`를 조회한다.  
재현/검증법: 메인 창에서 번역 후 메뉴바를 열어 최근 5개가 DB 기준으로 표시되는지, tray/menu에서 이력/설정/종료가 가능한지 확인한다.

### Major 3. 클립보드 번역이 빈/이미지/파일 clipboard를 구분하지 않고 inline 오류도 없음

위치: `src/windows/menubar/menubar.tsx:53-60`, `src-tauri/capabilities/default.json:12-13`, `src/windows/popup/popup.tsx:114-205`  
현상: menubar에서 `readText()` 결과가 truthy일 때만 입력을 채우고, 실패/빈 값은 무시한다. 이미지 또는 파일 clipboard에 대해 "지원하지 않음" inline 오류를 표시하지 않는다. popup에는 PRD가 허용한 "팝업 또는 메뉴바에서 클립보드 번역" 중 popup 경로 버튼이 없다.  
원인 추정: 클립보드 기능을 UI helper로만 처리하고 domain command/error로 만들지 않음.  
영향도: PRD §6.4/§8.6/§12 클립보드 정책 검증 불가. 사용자가 동작 실패 원인을 알 수 없다.  
권장 개선안: `translate_clipboard` 또는 clipboard feature store를 만들고, text/empty/non-text/file-image unsupported를 명시적 AppError로 표현한다. popup/menubar 모두 같은 로직을 사용한다.  
재현/검증법: 빈 clipboard, 이미지 복사, Finder 파일 복사 후 클립보드 번역 실행 시 번역 요청이 나가지 않고 inline 오류가 떠야 한다.

### Major 4. popup focus와 sizing 요구가 부분 구현

위치: `src-tauri/tauri.conf.json:25-40`, `src-tauri/src/commands/popup.rs:19-38`, `src/windows/popup/popup.tsx:42-46`, `src/windows/popup/popup.tsx:114-205`  
현상: popup width 480은 설정되어 있으나 화면 높이 80% 제한/내용 길이에 따른 세로 확장 로직이 없다. textarea focus는 component mount 시 1회만 수행되어, 숨긴 뒤 다시 열 때 보장되지 않는다. backend도 primary display center만 사용한다.  
원인 추정: Tauri window show/focus와 webview 내부 input focus 이벤트를 분리하지 않음.  
영향도: PRD §6.3/§7.3 미충족. 단축키 사용 시 바로 입력하지 못하거나 작은/큰 화면에서 UX가 깨질 수 있다.  
권장 개선안: `popup:opened` 이벤트를 FE가 listen해서 textarea focus를 재수행하고, current monitor 기준 80% max height 및 content resize 정책을 적용한다.  
재현/검증법: 팝업을 닫고 다시 연 뒤 첫 키 입력이 textarea에 들어가는지, 긴 결과에서 window가 화면 80%를 넘지 않는지 확인한다.

### Major 5. Ollama 재연결 backoff가 PRD와 다름

위치: `src-tauri/src/commands/onboarding.rs:187-205`, `src/features/onboarding/store.ts:118-125`  
현상: PRD §8.4는 exponential backoff 재연결을 요구하지만 구현은 `try_start_ollama` 후 800ms 단일 sleep과 status 재조회다.  
원인 추정: 온보딩 happy path 중심 구현.  
영향도: Ollama GUI가 늦게 뜨는 환경에서 불필요한 실패 안내가 발생한다.  
권장 개선안: 제한된 횟수의 exponential backoff와 cancel 가능한 status polling을 구현한다.  
재현/검증법: Ollama 종료 상태에서 앱의 자동 실행을 눌러 `/api/tags`가 늦게 열릴 때도 최종 running 상태로 전환되는지 확인한다.

### Major 6. ModelInstallState 영속 모델이 없음

위치: `docs/hytranslate-mac-prd.md` §9.3, `src-tauri/src/commands/onboarding.rs:68-78`, `src/features/onboarding/store.ts:25-38`  
현상: PRD §9.3의 `model_id`, `display_name`, `ollama_name`, `installed`, `recommended`, `last_checked_at`를 저장하는 구조나 SQLite schema가 없다. 현재는 `/api/tags` 모델명 배열과 FE 메모리 `installedSinceStart`만 있다.  
원인 추정: 런타임 상태로 충분하다고 판단했으나 PRD 데이터 모델을 누락.  
영향도: 모델 설치 상태의 last checked/recommended 표시, 재시작 후 상태 추적, PRD 데이터 모델 일치성 미충족.  
권장 개선안: PRD 의도가 영속 모델인지 확인한 뒤, 필요하면 settings/DB schema에 ModelInstallState를 추가하거나 PRD를 명확히 수정한다.  
재현/검증법: 앱 재시작 후 추천/설치/last_checked_at 상태가 PRD 필드 단위로 조회 가능한지 확인한다.

### Major 7. DB 경로가 PRD 지정 경로와 정확히 일치하는지 보장되지 않음

위치: `src-tauri/src/commands/mod.rs:69-92`, `src-tauri/tauri.conf.json:3-4`  
현상: PRD는 `~/Library/Application Support/HyTranslate Mac/hytranslate.sqlite`를 지정한다. 구현은 Tauri `app_data_dir()` 아래 `hytranslate.sqlite`를 사용한다. 코드 주석은 bundle id 기반 경로를 언급한다.  
원인 추정: Tauri default data dir 사용.  
영향도: PRD §9.4의 운영/개인정보 안내와 실제 저장 위치가 어긋날 수 있다.  
권장 개선안: 런타임에서 실제 경로를 확인하고, PRD 경로가 hard requirement라면 product name 기반 경로를 명시적으로 구성한다.  
재현/검증법: macOS 앱 실행 후 실제 DB 위치를 확인한다.

### Major 8. 입력 상태 모델이 PRD 상태와 다름

위치: `src/features/translation/types.ts:10-16`, `src/features/translation/store.ts:77-90`, `src/features/translation/use-translation-controller.ts:128-149`  
현상: PRD §7.2 상태는 `typing`, `detecting`을 포함하지만 TS 상태는 `debouncing`이 있고 `typing/detecting`이 없다. 실제로 debounce 중에도 상태 변경이 없다.  
원인 추정: UI 구현에서 세밀한 state machine을 단순화.  
영향도: 감지 중/입력 중 UI 표시, 테스트 가능성, 오류/취소 상태 전이가 PRD와 어긋남.  
권장 개선안: translation state machine을 PRD 상태명에 맞추거나 PRD 변경을 명시한다.  
재현/검증법: 입력 직후 500ms 동안 status indicator가 typing/detecting으로 표현되는지 확인한다.

### Minor 1. `TranslationState.recent` 주석이 현재 구현과 다름

위치: `src/features/translation/store.ts:23-24`, `src/windows/menubar/menubar.tsx:120-145`  
현상: 주석은 Phase 4에서 SQLite+FTS5로 대체된다고 하지만 실제 메뉴바는 여전히 `recent`를 표시한다.  
권장 개선안: DB 기반 최근 이력으로 전환하거나 주석을 현재 책임에 맞게 수정한다.

### Minor 2. CSV/JSON export는 원문/결과를 파일에 저장하므로 개인정보 안내가 더 필요함

위치: `src-tauri/src/commands/history.rs:145-182`, `src-tauri/src/commands/history.rs:202-277`  
현상: 사용자가 저장 경로를 선택하지만 export UI에 민감 데이터 포함 안내는 없다.  
권장 개선안: export 버튼 주변에 로컬 파일로 원문/결과가 저장된다는 inline 안내를 추가한다.

### Minor 3. endpoint allowlist가 `https://localhost`도 허용함

위치: `src-tauri/src/ollama/endpoint.rs:10-14`, `src-tauri/src/ollama/endpoint.rs:21-28`  
현상: 외부 전송은 아니지만 PRD 기본/일반 Ollama endpoint는 `http://localhost:11434`다. HTTPS loopback 허용 의도는 명확하지 않다.  
권장 개선안: 로컬 proxy/advanced use를 의도한 것인지 문서화한다. 아니면 `http`로 제한한다.

### Minor 4. clipboard write 실패가 silent 처리됨

위치: `src/features/translation/components/translation-panel.tsx:47-55`, `src/lib/hooks/use-auto-copy-translation.ts:21-31`, `src/windows/popup/popup.tsx:64-72`  
현상: 복사 실패가 사용자에게 표시되지 않는다.  
권장 개선안: 복사 버튼 주변에 짧은 inline 오류를 표시한다.

## 4. 아키텍처·코드 품질

command/event 계약은 전반적으로 안정적이다. Rust event constants와 TS constants가 1:1로 유지된다 (`src-tauri/src/events.rs:1-12`, `src/lib/ipc/events.ts:1-10`). history 9종 command도 모두 등록되어 있다 (`src-tauri/src/commands/mod.rs:129-137`).

Rust↔TS 경계는 camelCase serde와 typed TS mirror를 쓰는 방향이 좋다. 다만 `detect_language`는 command가 존재해도 product flow에서 사용되지 않아 계약은 있지만 사용자 가치로 연결되지 않는다.

Zustand 상태 설계는 feature별 store 분리가 명확하다. translation/settings/history/onboarding이 분리되어 있고 active model 주입도 import cycle을 피한다 (`src/windows/main/main.tsx:40-42`). 반면 translation state machine은 PRD의 `typing/detecting`이 빠지고, in-flight 취소 ref가 debounce 시작과 결합되어 즉시 취소 요구를 놓친다.

디바운스 500ms와 Cmd+Enter 즉시 번역은 구현되어 있다 (`src/features/translation/use-translation-controller.ts:9`, `src/features/translation/use-translation-controller.ts:151-157`). 취소는 "새 요청 시작 전"에는 수행하지만 "입력 변경 즉시"는 아니다.

prompt builder는 안전하다. 원문을 정규화하지 않고, target은 English로 고정하며, `temperature=0.3`, `top_p=0.9`, 동적 `num_predict`가 적용된다 (`src-tauri/src/ollama/prompt.rs:6-22`, `src-tauri/src/ollama/client.rs:136-140`).

FTS5와 migration은 PRD 요구를 충족한다. external-content FTS와 trigger, `PRAGMA user_version` 기반 migration이 있다 (`src-tauri/src/db/migrations/0001_init.sql:25-45`, `src-tauri/src/db/mod.rs:57-78`). 단, ModelInstallState schema는 없다.

## 5. Tauri 2 + macOS 통합

global-shortcut plugin은 사용 중이며 초기 등록과 설정 변경 swap/rollback이 있다 (`src-tauri/src/shortcuts/mod.rs:20-37`, `src-tauri/src/shortcuts/mod.rs:48-101`). 권한 안내는 설정/온보딩 CTA로 처리한다.

메뉴바 popover는 tray icon 좌클릭으로 열리며 blur 시 hide된다 (`src-tauri/src/menubar/mod.rs:28-70`). 그러나 PRD 메뉴 명령(메인 창, 이력, 설정, 종료)은 구현되지 않았다.

플로팅 팝업은 480px width로 구성되어 있다 (`src-tauri/tauri.conf.json:25-40`). 화면 80% 제한과 활성 화면 중앙 배치는 확인 필요/부분 구현이다 (`src-tauri/src/commands/popup.rs:19-23`).

클립보드는 text read/write capability와 plugin을 사용한다 (`src-tauri/capabilities/default.json:12-13`, `src/lib/clipboard.ts:7-16`). 이미지/파일 차단 메시지와 자동복사 기본 OFF는 각각 부분/완료다.

autostart와 Dock 숨김은 Tauri plugin/activation policy로 구현되어 있다 (`src-tauri/src/commands/system.rs:17-54`). 테마 System/Light/Dark도 설정과 window별 적용이 있다 (`src/features/settings/components/settings-panel.tsx:149-160`, `src/windows/main/main.tsx:36-38`).

DB 경로는 PRD 지정 경로와 정확히 일치하는지 확인 필요다. 현재는 `app_data_dir()` 기반이다 (`src-tauri/src/commands/mod.rs:69-92`).

## 6. 보안·개인정보

번역 네트워크는 loopback allowlist로 제한되어 있다 (`src-tauri/src/ollama/endpoint.rs:6-15`, `src-tauri/src/ollama/client.rs:128-130`). settings 저장 시 non-loopback endpoint도 거부한다 (`src-tauri/src/commands/settings.rs:28-31`).

telemetry/analytics 의존성이나 호출은 코드에서 확인되지 않았다. 외부 URL open은 Ollama 공식 다운로드와 macOS settings deep link로 제한되어 있다 (`src-tauri/src/commands/mod.rs:29-50`).

로그는 `request_id`, `model`, `source_language`, `source_len`, `duration_ms` 정도를 남기며 원문/결과를 직접 로그하지 않는다 (`src-tauri/src/commands/translate.rs:185-193`, `src-tauri/src/commands/translate.rs:276-280`). 다만 model name과 endpoint 관련 오류 메시지는 로그/에러로 노출될 수 있다.

로컬 저장 안내는 온보딩에 포함되어 있다 (`src/i18n/ko.ts:183-185`, `src/features/onboarding/components/onboarding-screen.tsx:618-635`). v1 DB 암호화 제외도 PRD와 맞다.

모델 다운로드는 사용자의 버튼 클릭 후 시작되며, backend는 지원 모델만 허용한다 (`src/features/onboarding/components/onboarding-screen.tsx:458-466`, `src-tauri/src/commands/onboarding.rs:129-132`, `src-tauri/src/commands/onboarding.rs:215-220`).

보안 강조: 가장 큰 개인정보 리스크는 외부 송신이 아니라 stale 번역 저장과 export 파일 생성 안내 부족이다. 입력 변경 즉시 취소가 되지 않으면 사용자가 더 이상 의도하지 않은 원문/결과가 SQLite에 남을 수 있다.

## 7. 테스트 커버리지

| PRD §14.3 필수 테스트 | 상태 | 근거 |
|---|---:|---|
| Prompt builder unit test | ✅ | `src-tauri/src/ollama/prompt.rs:29-77` |
| Language detection unit test | ✅ | `src-tauri/src/language/detector.rs:150-207` |
| Ollama client mock streaming test | ✅ | `src-tauri/src/ollama/client.rs:330-361` |
| Translation cancellation test | ⚠️ | client/registry 단위는 있음: `src-tauri/src/ollama/client.rs:363-391`, `src-tauri/src/commands/translate.rs:306-331`; FE 입력 변경 즉시 취소 회귀는 없음 |
| SQLite migration test | ✅ | `src-tauri/src/db/mod.rs:85-153` |
| History search test | ✅ | `src-tauri/src/history/mod.rs:454-476`, `src/features/history/store.test.ts` |
| Clipboard command test | ❌ | clipboard domain command 없음. hook 테스트만 있음: `src/lib/hooks/use-auto-copy-translation.test.tsx` |
| Settings persistence test | ✅ | `src-tauri/src/settings/store.rs:114-209`, `src/features/settings/store.test.ts` |
| Onboarding state transition test | ✅ | `src/features/onboarding/store.test.ts` |
| Playwright 주요 사용자 흐름 E2E | ❌ | `tests/e2e/sanity.spec.ts:1-5` |

테스트 실행은 하지 않았다. 이 리포트는 정적 존재 여부와 코드 근거만 검토했다.

## 8. 개선 제안

단기:

- 입력 변경 즉시 취소와 stale completion 저장 방지.
- 감지 결과 UI badge 연결 및 `detect_language` 통합 테스트 추가.
- clipboard empty/non-text/file-image inline 오류 구현.
- 메뉴바 최근 5개를 DB 조회로 변경.
- popup reopened focus를 `popup:opened` 이벤트 기반으로 보장.

중기:

- PRD 상태(`typing`, `detecting`, `translating`, `cancelled` 등)에 맞춘 translation state machine 정리.
- Tauri tray menu에 메인/이력/설정/종료 action 추가.
- Ollama status reconnect exponential backoff 구현.
- DB 경로를 PRD 지정 경로와 실제 macOS 경로 기준으로 확정.
- ModelInstallState의 저장 필요성 확인 후 schema/setting 추가.

장기:

- 품질 평가 자동화: 100개 eval sample, 모델별 점수, 회귀 추적.
- macOS 통합 E2E/수동 QA checklist: autostart, Dock hidden, tray, popup focus, clipboard 종류별 동작.
- 개인정보 UX 강화: export, history save, auto-copy, DB 위치 안내를 설정/온보딩에서 더 명확히 표시.

v1.1 후보 권고:

- PRD §4.2의 번역 스타일/용어집보다 먼저 v1 안정화 항목을 v1.1 초반에 배치한다.
- 그 다음 사용자 용어집과 번역 스타일을 도입하되, prompt builder test matrix와 품질 평가셋을 선행 조건으로 둔다.

## Verification Evidence

리포트 경로: `docs/code-review-v1.md`.

목차:

1. Executive Summary
2. PRD ↔ 구현 매칭표
3. 이슈 상세
4. 아키텍처·코드 품질
5. Tauri 2 + macOS 통합
6. 보안·개인정보
7. 테스트 커버리지
8. 개선 제안

검토 주요 파일:

- `docs/hytranslate-mac-prd.md`
- `src-tauri/src/commands/{translate,detect,onboarding,history,settings,popup,system}.rs`
- `src-tauri/src/{ollama,language,db,history,settings,shortcuts,menubar}/`
- `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`, `src-tauri/capabilities/default.json`
- `src/features/{translation,settings,history,onboarding}/`
- `src/windows/{main,popup,menubar}/`
- `src/lib/{clipboard,theme,ipc,hooks}/`
- `tests/`, `evals/translation-quality.md`, `package.json`

사용자 추가 확인 필요:

- Tauri `app_data_dir()`의 실제 macOS 경로가 PRD 지정 경로와 같은지.
- macOS에서 global shortcut 권한 부재 시 실제 UX가 inline 오류/안내로 충분한지.
- popup을 숨긴 뒤 다시 열 때 textarea focus가 실제로 유지되는지.
- Ollama 설치/실행/모델 pull 실패 케이스의 런타임 이벤트 순서.
- 품질 평가셋을 누가, 어떤 기준으로 채점 완료했는지.

`git status` 증거는 리포트 작성 후 별도 확인했다. 기대 상태는 `docs/code-review-v1.md` 신규 파일 외 변경 없음이다.
