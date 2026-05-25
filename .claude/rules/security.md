# Security & Privacy

Non-negotiable per PRD §12. Enforced in code review.

## Network discipline

- **Translation requests** go only to the configured `ollama_endpoint` — default `http://localhost:11434`. Reject any other host at the HTTP layer
- **Allowed network use**: user-approved model pull, user-clicked Ollama installer link, user-approved update check (v1 ships without an update check)
- **Forbidden**: telemetry, crash reporting to remote, analytics, default-on update checks

## Logging

- Never log `source_text` or `translated_text` at `info`
- Permitted fields: lengths, source language, model name, `durationMs`, `requestId`, `error.kind`
- Logs are local files only (`~/Library/Application Support/HyTranslate Mac/logs/`)

## Data at rest

- DB path: `~/Library/Application Support/HyTranslate Mac/hytranslate.sqlite`
- v1 does NOT encrypt the DB (PRD §12.2)
- User can disable history (Settings) and wipe all records (Settings → 전체 이력 삭제)

## Tauri capabilities

- `tauri.conf.json` allowlist is the minimum needed: `shell-open` (for the Ollama install link), `fs` (app-data dir only), `clipboard`
- Never enable `shell-execute` with arbitrary commands
- CSP: deny `http://` except `localhost:11434`; deny `unsafe-inline` for scripts

## Input handling

- Hard cap on `source_text` BEFORE sending: 30,000 main / 5,000 popup
- Output rendered as text, not HTML — no XSS surface
- No file paths from `source_text` interpreted by the OS layer

## Code review red flags

- ❌ Any `reqwest::get(url)` where `url` is not the configured Ollama endpoint
- ❌ Any new capability in `tauri.conf.json` without PR justification
- ❌ Any new dependency that opens a socket or does telemetry — must be flagged in the PR description
- ❌ Any log line that interpolates `source_text` / `translated_text`
