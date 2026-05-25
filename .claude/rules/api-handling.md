# API Handling (Ollama Client)

## Endpoints used

- `GET /api/tags` — list local models (status + onboarding)
- `POST /api/generate` (stream) — translation
- `POST /api/pull` (stream) — model download progress
- Base URL is the user-configurable `ollama_endpoint`, default `http://localhost:11434`

## Streaming pattern

- Always send `{ "stream": true, ... }` to `/api/generate` and `/api/pull`
- Consume `Response::bytes_stream()` from reqwest
- Accumulate bytes until newline; parse each line as JSON
- Maintain a `String` UTF-8 accumulator for partial codepoints — never emit broken chars to the FE (PRD §8.1 acceptance)
- Emit one Tauri event per parsed chunk

## Prompt builder (PRD §8.3)

Fixed template:

```
Translate the following segment from {source_language} into English.
Output only the translation. Do not add explanations, preambles, quotation marks, or markdown.

{source_text}
```

- `target_language` is hardcoded to English in v1 — do NOT take it as a parameter
- Default options: `temperature: 0.3`, `top_p: 0.9`, `num_predict: 512` (scales with input length, cap visible in code)
- Builder MUST NOT normalize / summarize / strip whitespace from `source_text`
- Unit-test the builder per source language (PRD §8.3 acceptance)

## Reconnect / retry

- Status checks use exponential backoff: 250ms → 500ms → 1s → 2s → 4s → cap 8s
- Stop retrying when the user navigates away or explicitly cancels
- Failed `/api/pull` shows a retry button — never auto-retry destructively

## Model identifiers

```
Hy-MT2 7B:   hf.co/tencent/Hy-MT2-7B-GGUF:Q4_K_M
Hy-MT2 1.8B: hf.co/tencent/Hy-MT2-1.8B-GGUF:Q4_K_M
```

Stored as constants in `src-tauri/src/ollama/models.rs`.
