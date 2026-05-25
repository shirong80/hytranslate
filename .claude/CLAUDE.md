# HyTranslate Mac

macOS-only local translation desktop app. Korean / Simplified Chinese / Traditional Chinese → English, executed entirely on the user's Mac via Ollama running Tencent Hy-MT2 GGUF models.

**Source of truth**: @docs/hytranslate-mac-prd.md — read this before any non-trivial work.

## Tech Stack

- **Desktop shell**: Tauri 2
- **Frontend**: React 18 + TypeScript 5 + Tailwind CSS 3 + Zustand
- **Backend**: Rust (edition 2021) + Tokio + reqwest + serde + rusqlite (FTS5)
- **Model runtime**: Ollama HTTP API (`http://localhost:11434` by default)
- **Models**: Tencent Hy-MT2 7B / 1.8B GGUF (Q4_K_M)
- **Build**: Vite (frontend) + cargo (Rust) via the `tauri` CLI
- **Tests**: Vitest (FE unit), `cargo test` (BE unit), Playwright (E2E)
- **Lint / format**: ESLint + Prettier (FE), rustfmt + clippy (BE)
- **Target OS**: macOS 13 Ventura+ (Apple Silicon primary; Intel supported with perf warning)

## Commands

```bash
# Full app
npm run tauri:dev        # tauri dev — hot reload across FE + Rust
npm run tauri:build      # tauri build — signed DMG (release)

# Frontend only
npm run dev              # vite dev server (no Tauri APIs)
npm run build            # vite build
npm run lint             # eslint
npm run typecheck        # tsc --noEmit
npm run test             # vitest
npm run test:e2e         # playwright test

# Rust (inside src-tauri/, or via --manifest-path)
cargo fmt --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

## Project Structure

```
hytranslate/
├── src/                          # React frontend
│   ├── windows/                  # main, popup, menubar — each has its own entry
│   ├── components/               # reusable presentational components
│   ├── features/                 # feature slices
│   │   └── <feature>/
│   │       ├── components/
│   │       ├── store.ts          # Zustand store
│   │       ├── ipc.ts            # invoke()/listen() wrappers
│   │       └── types.ts
│   ├── lib/                      # cross-cutting: ipc client, hooks, utils
│   ├── i18n/                     # Korean strings (v1: ko only)
│   ├── styles/                   # globals, tailwind layers
│   └── main.tsx
├── src-tauri/                    # Rust backend
│   └── src/
│       ├── commands/             # #[tauri::command] adapters only
│       ├── ollama/               # streaming client + prompt builder
│       ├── language/             # detection
│       ├── history/              # TranslationRecord repo + FTS5 search
│       ├── settings/             # Settings persistence
│       ├── db/                   # rusqlite pool, migrations, schema version
│       ├── shortcuts/            # global-shortcut plugin glue
│       ├── menubar/              # tray icon + popover bridge
│       ├── errors.rs             # AppError enum + serde mapping
│       ├── lib.rs
│       └── main.rs
├── docs/
│   └── hytranslate-mac-prd.md    # PRD — source of truth
├── evals/
│   └── translation-quality.md    # quality eval set (created during dev)
├── tests/e2e/                    # Playwright specs
├── tauri.conf.json
├── package.json
├── tsconfig.json
├── vite.config.ts
└── tailwind.config.ts
```

## Path Aliases (Vite + tsconfig)

```
@/*            → src/*
@components/*  → src/components/*
@features/*    → src/features/*
@windows/*     → src/windows/*
@lib/*         → src/lib/*
@i18n/*        → src/i18n/*
@styles/*      → src/styles/*
```

## Golden Rule (Agent I/O language)

**Governs the Claude Code Agent's I/O — NOT project source code.**

- **User input**: Korean or English accepted
- **Internal execution**: thinking, tool calls, subagent invocations, skill usage — English
- **In-flight user interaction**: follow-up questions and confirmations — English
- **Final output to user**: result reports, briefings, summaries — Korean
- **Source code / comments / UI strings**: see `.claude/rules/documentation.md`

## Key Conventions

### Frontend

- React function components only; no classes
- Hooks-first; custom hooks in `src/lib/hooks/` or feature-local `hooks/`
- Zustand: one store per feature, selector functions in components (`useStore(s => s.x)`), never `useStore()` bare
- Tailwind for layout/spacing; SCSS only when Tailwind cannot express it
- Tauri IPC: `invoke('command_name', payload)` for request/response; `listen('event:name', cb)` for streams
- Korean UI strings centralized in `src/i18n/ko.ts` — no inline literals in components
- Never cross-import another feature's store; compose at the component layer

### Backend (Rust)

- Async via Tokio; never block in `#[tauri::command]` handlers
- All commands return `Result<T, AppError>`; `AppError` serializes to a stable JSON shape
- Streaming uses Tauri events keyed by `requestId`
- Cancellation: each in-flight request registers a `CancellationToken`; `cancel_translation` fires it
- DB access only through repo modules — never raw `rusqlite::Connection` in commands
- Migrations forward-only, version-numbered, checked on startup
- Never log raw `source_text` or `translated_text` at `info` level

### Tauri IPC contract

- Command names: snake_case, matching Rust fn names
- Event names: `domain:action` (e.g., `translation:chunk`, `model-pull:progress`)
- `requestId` (UUID v4) required on every long-running command
- See @.claude/rules/tauri-ipc.md for the full contract

## v1 Roadmap (PRD §15)

Implement in order. Each phase has a hard completion bar in the PRD — do not advance until met.

1. **Phase 1 — Core translation loop**: Tauri scaffold, main window, Ollama streaming, cancellation, theme
2. **Phase 2 — Language detection & settings**: detection, manual override, prompt builder, settings persistence
3. **Phase 3 — macOS integration**: global shortcut, floating popup, menubar popover, clipboard, autostart, Dock hide
4. **Phase 4 — History & search**: SQLite schema + migration, FTS5, history UI, favorite/tag/delete, CSV/JSON export
5. **Phase 5 — Onboarding & model lifecycle**: env detection, Ollama install/run check, model recommend + pull progress

## Locked Decisions (PRD §18 — do NOT relitigate without user)

| Item | Decision |
|---|---|
| v1 input languages | Korean, Simplified Chinese, Traditional Chinese |
| v1 output language | English only |
| Main window input limit | 30,000 chars |
| Floating popup input limit | 5,000 chars |
| Translation debounce | 500ms; previous request cancelled on edit; `Cmd+Enter` skips debounce |
| Default global hotkey | `Cmd+Shift+T` |
| History save default | ON |
| Auto-copy after translation default | OFF |
| DB encryption | OUT of v1 |
| Telemetry / cloud sync / accounts | OUT of v1 |
| Ollama bundling | NOT bundled — link to official installer |
| Network during translation | NONE (localhost only) |
| UI language | Korean |
| macOS minimum | 13 Ventura |

## Privacy Posture (non-negotiable, PRD §12)

- Translation requests never leave `localhost:11434`
- No default-on telemetry, no remote logging
- Logs must not include `source_text` or `translated_text` at info level
- Network use restricted to: model pull (user-approved), official Ollama installer link (user-clicked), future user-approved update check

## Git Workflow

- **Default**: work directly on the current branch — no worktree
- **Worktree**: only on explicit user request (parallel experiments, risky refactors)
- Commits: Conventional Commits (`feat`, `fix`, `refactor`, `docs`, `style`, `test`, `chore`)
- PRs: Korean title; Korean or English body; reference PRD sections where relevant

## Agent Workflows

### Before using an external library

Invoke `library-docs-fetcher` before adding any new external library or non-trivial use of an existing one.

```
Agent(subagent_type="library-docs-fetcher", prompt="Fetch documentation for <lib> regarding <topic>")
```

Examples:

- `"Fetch documentation for tauri-plugin-global-shortcut regarding Cmd+Shift+T registration on macOS"`
- `"Fetch documentation for rusqlite regarding FTS5 virtual tables and migration"`
- `"Fetch documentation for zustand regarding shallow selectors and persist middleware"`
- `"Fetch documentation for Ollama HTTP API regarding /api/generate streaming chunks"`

### After every code change

Invoke `quality-gate` after Write/Edit completes on code files.

- **Trigger**: any `.ts`, `.tsx`, `.rs`, `.css` change
- **Skip**: `.md`, `.json`, `.toml` config, lockfiles

## Rules

See `.claude/rules/` for stack-specific guidelines:

- @.claude/rules/architecture.md — project structure, layering
- @.claude/rules/code-style.md — React / TypeScript style
- @.claude/rules/rust-style.md — Rust style + Tokio patterns
- @.claude/rules/tauri-ipc.md — command / event contract
- @.claude/rules/styling.md — Tailwind, themes, macOS feel
- @.claude/rules/error-handling.md — AppError, inline UI errors
- @.claude/rules/api-handling.md — Ollama client + streaming
- @.claude/rules/testing.md — Vitest + cargo test + Playwright
- @.claude/rules/security.md — privacy, network discipline
- @.claude/rules/accessibility.md — keyboard, screen reader
- @.claude/rules/documentation.md — comments, source-code language
- @.claude/rules/shell-handling.md — non-interactive CLI discipline
