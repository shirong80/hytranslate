# Architecture

## Layering

```
React UI ─▶ feature store (Zustand) ─▶ ipc.ts (invoke/listen wrappers)
                                              │
                                              ▼
                                      Tauri IPC bridge
                                              │
                                              ▼
src-tauri/commands ─▶ ollama / history / settings / language modules
                                  │
                                  ▼
                            rusqlite / reqwest
```

## Frontend rules

- One feature = one directory under `src/features/<feature>/`
- A feature owns its components, Zustand store, IPC wrapper, and types
- **Never cross-import another feature's store** — compose at the component layer via selectors
- `src/lib/` is for cross-cutting code only (IPC base client, generic hooks, utils)
- `src/windows/` contains entry points (`main`, `popup`, `menubar`) — composition only, minimal logic
- Components are presentation-first; business logic belongs in stores or hooks

## Backend rules

- `commands/` is an adapter layer — handlers ≤ 30 lines, just deserialize → call domain → serialize
- Domain modules (`ollama`, `history`, `settings`, `language`) are framework-agnostic — no `tauri::` imports
- DB layer (`db/`) owns the connection pool; expose typed repo functions to domain modules
- Each module owns its error variants; convert into `AppError` at the command boundary
- Background tasks (`tokio::spawn`) must be cancellable via `CancellationToken`

## Forbidden cross-cuts

- ❌ Component directly calls `invoke()` — go through the feature's `ipc.ts`
- ❌ Tauri command calls `rusqlite::Connection::open()` directly — use the repo
- ❌ Feature A imports feature B's store
- ❌ Korean UI string inlined in a component — must come from `src/i18n/ko.ts`
- ❌ Domain module imports `tauri::*` — it must remain unit-testable in isolation
