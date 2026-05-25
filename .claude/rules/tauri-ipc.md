# Tauri IPC Contract

## Command naming

- snake_case, verb-first: `translate_stream`, `cancel_translation`, `detect_language`, `pull_model`, `get_ollama_status`
- Rust fn name matches command name 1:1
- Frontend wrapper in `src/features/<feature>/ipc.ts` exposes a typed function

## Request / response shape

- Request payloads are serde structs with `#[serde(rename_all = "camelCase")]`
- Response: `Result<T, AppError>` — Tauri auto-rejects the FE promise on `Err`
- Long-running commands return immediately after spawning the worker; results stream via events

## Event naming

- Pattern: `domain:action` (e.g., `translation:started`, `translation:chunk`, `translation:completed`, `translation:cancelled`, `translation:error`, `model-pull:progress`)
- Defined once in `src/lib/ipc/events.ts` AND `src-tauri/src/events.rs` — never inline magic strings
- Every event payload carries the `requestId` it belongs to (UUID v4 string)

## Cancellation contract

- Frontend generates `requestId` (UUID v4) before invoking `translate_stream`
- Backend registers a `CancellationToken` keyed by `requestId` in a `DashMap` (or `Arc<Mutex<HashMap>>`)
- `cancel_translation { requestId }` flips the token
- Worker checks the token before each chunk emit and before the DB save
- On cancel: emit `translation:cancelled`; do NOT save to DB

## Streaming contract (`translate_stream`)

Request:

```ts
{
  sourceText: string;
  sourceLanguage: 'Korean' | 'ChineseSimplified' | 'ChineseTraditional' | 'Auto';
  model: string;        // e.g. "hf.co/tencent/Hy-MT2-7B-GGUF:Q4_K_M"
  requestId: string;    // UUID v4
}
```

Events (in order):

1. `translation:started` — `{ requestId, model, startedAt }`
2. `translation:chunk` (0..N) — `{ requestId, delta: string }` — UTF-8 safe (no partial codepoints)
3. Exactly one terminal:
   - `translation:completed` — `{ requestId, fullText, durationMs }`
   - `translation:cancelled` — `{ requestId }`
   - `translation:error` — `{ requestId, error: AppError }`

## AppError serialization shape

```ts
type AppError =
  | { kind: 'OllamaUnavailable' }
  | { kind: 'OllamaNotRunning' }
  | { kind: 'ModelMissing'; model: string }
  | { kind: 'InputTooLong'; limit: number }
  | { kind: 'Cancelled' }
  | { kind: 'NetworkBlocked' }
  | { kind: 'Internal'; message: string };
```

Frontend `ipc.ts` MUST narrow on `kind` before accessing other fields.

## Other commands (PRD §10)

- `detect_language { text } → { language, confidence }`
- `get_ollama_status` → `{ installed, running, endpoint, models[] }`
- `pull_model { model }` — events: `model-pull:started | progress | completed | error`
- History commands: `list_translation_records`, `search_translation_records`, `get_translation_record`, `delete_translation_record`, `delete_all_translation_records`, `toggle_favorite`, `set_tags`, `export_history_csv`, `export_history_json`
