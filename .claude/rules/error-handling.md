# Error Handling

## Source of truth

`AppError` in `src-tauri/src/errors.rs` is the canonical taxonomy. Frontend mirrors the discriminated union in `src/lib/ipc/errors.ts` and updates in lockstep.

## UI display rules (PRD §7.1, §11)

- Errors are inline within the affected region — **never modal alerts**
- Show: short Korean message + 1–2 action buttons (e.g., "다시 시도", "Ollama 상태 보기")
- Standard messages live in `src/i18n/ko.ts`, keyed by `AppError.kind`

## Standard mappings (from PRD §11)

| `AppError.kind`       | Korean message (excerpt)                                 | Actions                   |
|-----------------------|----------------------------------------------------------|---------------------------|
| `OllamaUnavailable`   | "Ollama가 설치되어 있지 않습니다…"                        | 공식 다운로드 / 다시 확인  |
| `OllamaNotRunning`    | "Ollama가 실행 중이 아닙니다…"                             | 자동 실행 / 다시 연결      |
| `ModelMissing`        | "선택한 번역 모델이 아직 다운로드되지 않았습니다."          | 추천 모델 다운로드 / 다른 모델 |
| `InputTooLong`        | "현재 화면에서는 최대 {limit}자까지 번역할 수 있습니다."    | (없음)                     |
| `TranslationFailed`   | "번역 중 문제가 발생했습니다…"                             | 다시 시도 / 상태 보기      |
| `PermissionRequired`  | "전역 단축키를 사용하려면 macOS 권한 설정이 필요합니다."    | System Settings 열기 / 나중에 |

## Rust patterns

- Never `unwrap()` outside tests; never `panic!` in a command
- `?` propagates `Result<T, AppError>` via `From` impls
- Wrap external errors with context (`tracing::error!` + return `AppError::Internal`)
- Cancellation is **not** an error — emit `translation:cancelled` separately

## Frontend patterns

- IPC errors caught in feature `ipc.ts`, normalized to `AppError`, stored in feature store
- Components read `error` from the store and render `<InlineError error={...} />`
- Retry actions call the same IPC function — no parallel "retry" code path
