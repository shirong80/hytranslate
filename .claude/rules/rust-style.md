# Rust Style

## General

- Edition 2021; format with `cargo fmt`; pass `cargo clippy -- -D warnings`
- Avoid `unwrap()` / `expect()` outside tests and one-time startup
- Prefer `?` for error propagation; convert at module boundaries via `From` impls
- No `unsafe` without an inline justification comment

## Async / Tokio

- All `#[tauri::command]` handlers are `async fn`
- Long-running work runs on the Tokio runtime; never block the executor with sync I/O
- `tokio::spawn` for background tasks; hold a `JoinHandle` if cancellation is needed
- For request cancellation, pass `tokio_util::sync::CancellationToken` into the worker and check it before each chunk emit

## Error handling

- One `AppError` enum in `src-tauri/src/errors.rs`, deriving `thiserror::Error` + serde `Serialize`
- Variants per failure domain: `OllamaUnavailable`, `OllamaNotRunning`, `ModelMissing { model }`, `InputTooLong { limit }`, `Cancelled`, `Db(String)`, `Internal(String)`
- Implement `From<reqwest::Error>`, `From<rusqlite::Error>` for ergonomic `?`
- Never `panic!` inside a command; return `AppError::Internal` instead

## Modules

- One module per domain; expose a narrow `pub` surface
- Domain modules MUST NOT import `tauri::` — they stay framework-agnostic for unit testing
- Glue between Tauri and domains lives in `commands/`

## Persistence (rusqlite)

- Single connection pool (e.g., `r2d2_sqlite` or `tokio_rusqlite`) constructed at startup
- Migrations are numbered SQL files run on startup; check `PRAGMA user_version`
- FTS5 virtual table stays in sync via triggers on `translation_records`
- Use `prepare_cached` for hot queries
- All SQL goes through repo functions returning strongly-typed structs

## HTTP (reqwest)

- One shared `reqwest::Client` constructed at startup
- Streaming via `Response::bytes_stream()` + `futures_util::StreamExt`
- Maintain a `String` UTF-8 accumulator for partial codepoints — never emit broken chars (PRD §8.1 acceptance)

## Logging (tracing)

- `info!` for lifecycle events only
- Never log `source_text` / `translated_text` at `info` — `debug!` with explicit feature flag at most
- Permitted fields: lengths, hashes, request IDs, error.kind — never raw contents
