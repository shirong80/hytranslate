# Phase 2 Language Detection & Settings Code Review

대상: `.claude/worktrees/phase2-language-settings`

리뷰 기준:

- `docs/hytranslate-mac-prd.md` §15.2 Phase 2
- `docs/hytranslate-mac-prd.md` §8.2, §8.3, §9.2, §10.3
- 기준 diff: Phase 2 자체 변경은 `worktree-phase1-core-translation...HEAD`

## 요약

- review depth: **deep**
- Phase 2 diff: `28 files changed, 1329 insertions, 56 deletions`
- full branch diff from `main`: `37 files changed, 3827 insertions, 16 deletions`
- scope: **on target**. 언어 감지, `Auto` resolution, 수동 override, prompt builder fallback, settings 저장, settings 화면 scaffold가 Phase 2 목표와 연결되어 있다.
- hard stops: **1 found, 0 fixed, 1 deferred**
- specialists: **security, architecture**를 같은 세션에서 순차 수행. 사용 가능한 sub-agent 도구는 있었지만, 현재 도구 정책상 사용자가 명시적으로 sub-agent 위임을 요청한 경우에만 호출 가능해 병렬 agent는 사용하지 않았다.
- interview: 요청하지 않음. 리뷰에 필요한 기준은 PRD와 Phase 2 계획서로 충분했다.

## Findings

### High 1. Concurrent settings saves can leave memory and disk with different values

- 위치: `src-tauri/src/settings/store.rs:59`
- 관련 위치: `src-tauri/src/settings/store.rs:82`

`SettingsStore::update()`는 메모리 값을 write lock으로 갱신한 뒤 lock을 놓고 `save_to_disk()`를 수행한다. 이 때문에 두 `update_settings` 호출이 겹치면 저장 작업이 같은 `settings.json.tmp`를 공유하고, 메모리와 디스크 상태가 서로 다른 값으로 끝날 수 있다.

가능한 순서:

1. A가 메모리를 A 값으로 바꾸고 `settings.json.tmp`에 쓰기 시작한다.
2. B가 메모리를 B 값으로 바꾸고 같은 tmp path를 다시 생성/쓰기/rename한다.
3. A의 rename이 실패하거나 늦게 성공하면서 rollback 또는 overwrite가 발생한다.
4. 커맨드 응답, 메모리 상태, 재시작 후 로드되는 디스크 상태가 서로 달라질 수 있다.

이 문제는 Phase 2 완료 기준인 "기본 설정이 앱 재시작 후 유지된다"를 깨뜨릴 수 있다. FE 버튼은 `saving` 상태로 disabled 처리되지만 React state 반영 전 double-click, devtools/IPC 직접 호출, 이후 Phase의 여러 window/popup에서 동일 command 호출이 가능하므로 backend store가 자체적으로 직렬화해야 한다.

권장 수정:

- 메모리 갱신과 disk save 전체를 하나의 write lock 안에서 수행한다.
- 또는 `SettingsStore`에 별도 `Mutex`를 두어 save operation을 직렬화하고, tmp 파일명도 operation-local unique path를 사용한다.
- rollback은 "현재 update가 마지막 writer"인지 확인할 수 있을 때만 수행한다. 지금처럼 이전 snapshot으로 무조건 rollback하면 다른 성공 update를 되돌릴 수 있다.
- 회귀 테스트 추가: 두 update를 병렬 실행한 뒤 `store.get()`과 새 `SettingsStore::load(path).get()`이 항상 같은 값인지 검증한다.

## Security Review

- `ollama_endpoint`는 저장 시 `update_settings`에서 loopback allowlist를 적용하고, 번역 시 `OllamaClient::generate_stream`에서도 재검증한다. 설정 파일을 직접 조작해도 non-loopback outbound request는 차단된다.
- `open_ollama_download_page`는 백엔드 고정 URL만 `open`에 전달하므로 임의 command/URL injection 위험은 낮다.
- 번역 원문/결과는 로그에 남기지 않고 metadata만 기록한다.
- 설정 파일은 사용자 앱 데이터 디렉터리에 JSON으로 저장된다. v1 범위에서 DB encryption은 제외되어 있어 PRD와 충돌하지 않는다.

## Architecture Review

- Backend `Auto` resolution을 `translate_stream` 내부에서 수행해 UI 감지 결과 재전송 race를 피한 점은 적절하다.
- settings와 translation store를 직접 import로 결합하지 않고 `main.tsx`에서 active model을 주입하는 구조는 현재 Phase 범위에서는 충분히 단순하다.
- 다만 settings persistence는 상태 관리의 single source of truth 역할을 하므로, 저장 경로는 command 호출자가 순서를 보장한다고 가정하면 안 된다. Phase 3 이후 여러 surface가 settings를 만질 수 있어 backend 레벨 직렬화가 필요하다.

## Adversarial Pass

- Confidence 0.82: 설정 저장 버튼을 빠르게 두 번 누르거나 여러 window에서 `update_settings`를 동시에 호출하면, shared tmp file과 lock 범위 문제로 memory/disk mismatch를 만들 수 있다. 이후 번역은 메모리의 endpoint/model을 쓰지만 재시작 후에는 다른 값이 로드된다.
- Confidence 0.66: 악의적 local client가 non-loopback endpoint를 `settings.json`에 직접 써도 번역 시점에서 `NetworkBlocked`로 막힌다. 이 경로는 차단되어 있어 finding으로 올리지 않는다.

## Verification

### `bash /Users/shiron/.agents/skills/check/scripts/run-tests.sh`

결과: pass

```text
> hytranslate@0.1.0 test
> vitest run

 RUN  v2.1.9 /Users/shiron/Documents/projects/hytranslate/.claude/worktrees/phase2-language-settings

 ✓ tests/unit/sanity.test.ts (1 test) 1ms
 ✓ src/lib/ipc/errors.test.ts (5 tests) 1ms
 ✓ src/features/settings/ipc.test.ts (2 tests) 2ms
 ✓ src/features/translation/ipc.test.ts (3 tests) 2ms
 ✓ src/features/settings/store.test.ts (5 tests) 2ms
 ✓ src/features/translation/store.test.ts (7 tests) 2ms

 Test Files  6 passed (6)
      Tests  23 passed (23)
```

### `cargo test --manifest-path src-tauri/Cargo.toml`

결과: pass

```text
running 44 tests
test language::detector::tests::korean_with_hanja_still_detected_as_korean ... ok
test language::detector::tests::empty_input_is_auto_with_zero_confidence ... ok
test language::detector::tests::ambiguous_cjk_without_markers_is_auto ... ok
test language::detector::tests::no_table_collision_between_simplified_and_traditional ... ok
test language::detector::tests::non_cjk_input_is_auto ... ok
test errors::tests::cancelled_has_no_extra_fields ... ok
test errors::tests::internal_helper_wraps_display ... ok
test commands::translate::tests::validate_request_id_rejects_non_uuid ... ok
test language::detector::tests::pure_korean_detected_as_korean ... ok
test language::detector::tests::simplified_chinese_detected ... ok
test language::detector::tests::traditional_chinese_detected ... ok
test language::tests::prompt_label_per_variant ... ok
test language::tests::serializes_with_variant_name ... ok
test errors::tests::serializes_with_kind_discriminator ... ok
test commands::translate::tests::registry_cancel_missing_returns_false ... ok
test ollama::client::tests::line_buffer_handles_split_lines ... ok
test language::detector::tests::serializes_with_camel_case_fields ... ok
test commands::translate::tests::registry_inserts_and_cancels ... ok
test commands::detect::tests::detect_language_returns_korean_for_hangul ... ok
test ollama::prompt::tests::auto_falls_back_to_generic_chinese_label ... ok
test ollama::endpoint::tests::malformed_urls_rejected ... ok
test ollama::prompt::tests::does_not_strip_or_normalize_source_text ... ok
test ollama::prompt::tests::korean_prompt_contains_source_label_and_text ... ok
test ollama::prompt::tests::simplified_chinese_uses_simplified_label ... ok
test ollama::prompt::tests::num_predict_scales_with_input_and_caps ... ok
test ollama::prompt::tests::traditional_chinese_uses_traditional_label ... ok
test ollama::endpoint::tests::non_http_schemes_rejected ... ok
test ollama::endpoint::tests::localhost_variants_allowed ... ok
test ollama::endpoint::tests::non_loopback_hosts_rejected ... ok
test settings::tests::defaults_match_prd_section_9_2 ... ok
test tests::skeleton_compiles ... ok
test settings::tests::serializes_with_camel_case_keys ... ok
test settings::store::tests::corrupt_json_falls_back_to_defaults_without_overwriting ... ok
test ollama::client::tests::non_loopback_endpoint_returns_network_blocked ... ok
test ollama::client::tests::http_404_maps_to_model_missing ... ok
test ollama::client::tests::stream_with_trailing_partial_line_returns_error ... ok
test ollama::client::tests::stream_without_done_true_returns_error ... ok
test ollama::client::tests::cancellation_returns_cancelled_error ... ok
test ollama::client::tests::http_error_is_mapped_to_internal ... ok
test ollama::client::tests::accumulates_chunks_and_returns_full_text ... ok
test commands::settings::tests::update_rejects_non_loopback_endpoint ... ok
test settings::store::tests::load_creates_default_file_when_missing ... ok
test commands::settings::tests::update_accepts_loopback_with_custom_port ... ok
test settings::store::tests::update_persists_to_disk_and_round_trips ... ok

test result: ok. 44 passed; 0 failed; 0 ignored
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
Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.02s
```

## Sign-off

```text
files changed:    28 (+1329 -56) in Phase 2 diff
scope:            on target
review depth:     deep
hard stops:       1 found, 0 fixed, 1 deferred
specialists:      security, architecture
new tests:        0
verification:     check run-tests -> pass; cargo test -> pass; typecheck/lint/fmt/clippy -> pass
```
