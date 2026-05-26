# Phase 1 Core Translation Code Review

대상: `.claude/worktrees/phase1-core-translation`

기준 diff: `main...HEAD` (`e6aad06 feat(phase1): 핵심 번역 루프 구현`)

리뷰 기준: `docs/hytranslate-mac-prd.md` §15.1 Phase 1, §10 command 계약, §14.3 구현 평가

## 요약

- review depth: **deep** (`23 files changed, 2387 insertions, 13 deletions`)
- scope: **on target**. Ollama streaming client, command bridge, main translation UI, cancellation, theme, tests가 Phase 1 목표와 연결되어 있다.
- hard stop: **1 found, 0 fixed, 1 deferred**
- specialists: **security, architecture**를 같은 세션에서 순차 수행. 사용 가능한 sub-agent 도구는 있었지만, 현재 도구 정책상 사용자가 명시적으로 sub-agent 위임을 요청한 경우에만 호출 가능해 병렬 agent는 사용하지 않았다.
- interview: 요청하지 않음. 리뷰에 필요한 범위와 기준은 PRD와 Phase 1 계획서로 충분히 확인 가능했다.

## Findings

### High 1. 30,000자 초과 입력이 기존 요청을 취소하지 않고, stale 결과가 최종 출력될 수 있음

- 위치: `src/features/translation/use-translation-controller.ts:77`
- 관련 위치: `src/features/translation/store.ts:91`

`runTranslation()`은 입력 길이가 `MAIN_INPUT_LIMIT`를 초과하면 `markError({ requestId: 'local', ... })` 후 바로 return한다. 하지만 `markError()`는 현재 store의 `requestId`와 payload `requestId`가 다르면 무시한다. 일반 idle 상태에서는 `requestId`가 `null`이라 초과 입력 에러가 표시되지 않고, 더 심각하게는 이전 번역이 진행 중인 상태에서 입력이 30,000자를 넘으면 기존 `inFlightRef.current` 요청을 취소하기 전에 return한다.

그 결과 이전 요청의 chunk/completed 이벤트가 계속 현재 requestId와 매칭되어, 사용자가 이미 다른 입력을 넣었는데도 이전 입력의 번역이 최종 결과로 표시될 수 있다. 이는 PRD §15.1의 "입력 변경 시 이전 요청이 취소된다"와 Phase 1 계획서의 "이전 결과가 최종 결과로 저장되지 않는다"를 직접 위반한다.

권장 수정:

- 길이 검증 전에 현재 in-flight 요청을 취소하거나, 초과 입력 상태 진입 시 반드시 현재 requestId를 무효화한다.
- local/client-side 에러를 표시할 수 있는 store action을 별도로 두거나 `markError`가 `requestId: null`/local error를 명시적으로 허용하게 한다.
- 회귀 테스트 추가: "기존 요청 진행 중 입력이 limit 초과로 바뀌면 cancelTranslation이 호출되고 stale chunk/completed가 무시된다."

### Medium 1. Ollama stream이 `done: true` 없이 끊겨도 완료로 처리됨

- 위치: `src-tauri/src/ollama/client.rs:111`
- 관련 위치: `src-tauri/src/ollama/client.rs:132`

`bytes_stream()`이 EOF를 반환하면 loop를 빠져나와 `Ok(full_text)`를 반환한다. Ollama streaming 계약상 정상 완료는 `done: true` chunk로 판단해야 한다. 네트워크 중단, Ollama 프로세스 재시작, 프록시/런타임 오류 등으로 stream이 중간에 닫히면 부분 번역이 `translation:completed`로 emit되고 UI에는 정상 완료처럼 duration이 표시된다.

권장 수정:

- `seen_done` 플래그를 두고 `done: true`를 보지 못한 EOF는 `AppError::Internal` 또는 별도 `TranslationFailed` 성격의 error로 처리한다.
- newline 없이 남은 `line_buf` 잔여 데이터가 있는 경우도 incomplete/chunk parse error로 취급한다.
- wiremock 테스트 추가: `done:false` 몇 개만 보낸 뒤 EOF이면 error가 반환되어야 한다.

### Medium 2. 모델 미설치/404 응답이 `ModelMissing`이 아니라 일반 `Internal`로 노출됨

- 위치: `src-tauri/src/ollama/client.rs:101`
- 관련 위치: `src-tauri/src/errors.rs:40`

`response.error_for_status().map_err(AppError::from)?` 이후 `From<reqwest::Error>`는 HTTP status별 분기를 하지 않는다. Ollama가 모델 미설치 또는 unknown model을 404 계열로 응답하면 FE에는 `Internal`만 전달된다. 이미 `AppError::ModelMissing`과 한국어 메시지가 정의되어 있는데 실제 경로에서 사용되지 않아, 모델 준비가 안 된 Phase 1 사용자는 원인을 알기 어렵다.

권장 수정:

- `err.status()`를 확인해 404는 `AppError::ModelMissing { model }`로 매핑한다.
- 400/500 계열 응답 body에 Ollama error message가 있으면 원문 텍스트 없이 안전하게 요약 매핑한다.
- 테스트 추가: mock 404 응답이 `ModelMissing`으로 변환되는지 확인한다.

### Low 1. 빈 입력으로 전환할 때 상태가 idle로 돌아가지 않음

- 위치: `src/features/translation/use-translation-controller.ts:73`
- 관련 위치: `src/features/translation/store.ts:96`

입력이 비면 `clearOutput()`만 호출하는데, 이 action은 `output`, `durationMs`, `error`만 초기화하고 `status`와 `requestId`는 그대로 둔다. 완료 직후 입력을 지우면 내부 상태는 `completed`로 남고, 번역 중 입력을 지우면 cancel event 도착 전/후에 `cancelled`로 남을 수 있다. 현재 UI에서는 큰 표시 문제가 제한적이지만, 이후 Phase에서 이력 저장, 팝업, 상태 indicator가 붙으면 빈 입력인데 완료/취소 상태로 보이는 상태 불일치가 전파될 수 있다.

권장 수정:

- 빈 입력 전환용 action을 추가해 `status: 'idle'`, `requestId: null`, `startedAtMs: null`까지 초기화한다.
- 회귀 테스트 추가: completed/translating 상태에서 sourceText를 빈 문자열로 바꾸면 store가 idle 상태가 된다.

## Security Review

- `open_ollama_download_page`는 FE에 임의 URL open 권한을 주지 않고 백엔드 상수 URL만 `open`에 전달하므로 command injection 위험은 낮다.
- 번역 원문/결과를 로그에 남기지 않고 길이와 metadata만 기록하는 점은 PRD의 로컬/민감정보 원칙과 맞다.
- `model` 값은 FE payload에서 그대로 Ollama HTTP body로 전달된다. shell이나 file path로 사용되지는 않아 직접 injection 위험은 낮지만, Phase 2 설정 저장 또는 모델 선택 UI가 붙을 때 allowlist 검증을 추가하는 것이 좋다.

## Architecture Review

- FE는 `features/translation` 아래 store/ipc/types/components/controller로 경계가 나뉘어 있어 Phase 1 범위에서는 이해하기 쉽다.
- Rust는 command handler와 Ollama client가 분리되어 있고, `CancellationToken` registry도 command layer에 국한되어 있다.
- 가장 큰 구조적 취약점은 client-side 에러와 request-bound 서버 이벤트를 같은 `markError(requestId)` action으로 처리하는 점이다. 이 때문에 High 1 같은 local validation bug가 생겼다. local validation 상태와 request lifecycle 상태를 구분하면 이후 팝업/메인창 병행에도 안전해진다.

## Adversarial Pass

- Confidence 0.86: 사용자가 정상 길이 입력으로 번역을 시작한 뒤, 즉시 30,001자 텍스트를 붙여넣으면 이전 요청이 취소되지 않는다. stale requestId가 유지되므로 이전 번역의 chunk와 completed 이벤트가 현재 UI에 계속 반영된다.
- Confidence 0.74: Ollama가 partial NDJSON만 보낸 뒤 연결을 끊으면 앱은 부분 번역을 완료로 표시한다. 사용자는 불완전한 번역을 정상 결과로 복사할 수 있다.
- Confidence 0.68: 모델이 없는 fresh machine에서 404가 `Internal`로만 표시되어 사용자가 Ollama 실행 문제와 모델 미설치 문제를 구분하지 못한다. Phase 5 온보딩 전이라도 Phase 1 smoke test 실패 분석이 어려워진다.

## Verification

### `bash /Users/shiron/.agents/skills/check/scripts/run-tests.sh`

결과: pass

```text
> hytranslate@0.1.0 test
> vitest run

 RUN  v2.1.9 /Users/shiron/Documents/projects/hytranslate/.claude/worktrees/phase1-core-translation

 ✓ tests/unit/sanity.test.ts (1 test) 1ms
 ✓ src/lib/ipc/errors.test.ts (5 tests) 1ms
 ✓ src/features/translation/ipc.test.ts (3 tests) 2ms
 ✓ src/features/translation/store.test.ts (5 tests) 1ms

 Test Files  4 passed (4)
      Tests  14 passed (14)
```

### `cargo test --manifest-path src-tauri/Cargo.toml`

첫 실행 결과: sandbox의 localhost port bind 제한 때문에 wiremock 테스트 3개가 실패했다.

승인 후 동일 명령 재실행 결과: pass

```text
running 18 tests
test language::tests::prompt_label_per_variant ... ok
test ollama::prompt::tests::korean_prompt_contains_source_label_and_text ... ok
test ollama::prompt::tests::does_not_strip_or_normalize_source_text ... ok
test errors::tests::cancelled_has_no_extra_fields ... ok
test commands::translate::tests::validate_request_id_rejects_non_uuid ... ok
test ollama::prompt::tests::num_predict_scales_with_input_and_caps ... ok
test language::tests::serializes_with_variant_name ... ok
test ollama::prompt::tests::traditional_chinese_uses_traditional_label ... ok
test ollama::prompt::tests::simplified_chinese_uses_simplified_label ... ok
test tests::skeleton_compiles ... ok
test errors::tests::internal_helper_wraps_display ... ok
test ollama::client::tests::line_buffer_handles_split_lines ... ok
test commands::translate::tests::registry_cancel_missing_returns_false ... ok
test errors::tests::serializes_with_kind_discriminator ... ok
test commands::translate::tests::registry_inserts_and_cancels ... ok
test ollama::client::tests::http_error_is_mapped_to_internal ... ok
test ollama::client::tests::cancellation_returns_cancelled_error ... ok
test ollama::client::tests::accumulates_chunks_and_returns_full_text ... ok

test result: ok. 18 passed; 0 failed; 0 ignored
```

### 추가 검증

결과: pass

```text
npm run typecheck
> tsc --noEmit

npm run lint
> eslint .

cargo fmt --manifest-path src-tauri/Cargo.toml -- --check

cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.65s
```

## Sign-off

```text
files changed:    23 (+2387 -13) reviewed, plus this review document added
scope:            on target
review depth:     deep
hard stops:       1 found, 0 fixed, 1 deferred
specialists:      security, architecture
new tests:        0
verification:     check run-tests -> pass; cargo test -> pass after sandbox escalation; typecheck/lint/fmt/clippy -> pass
```
