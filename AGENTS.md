# HyTranslate Mac Agent Harness

This is the Codex-facing entrypoint for this repository. It is derived from
`.claude/CLAUDE.md` plus `.claude/rules/`, `.claude/agents/`, and
`.claude/skills/`. Treat those files as detailed reference material, but follow
this file for Codex execution mechanics.

## Source Of Truth

- Product behavior: read `docs/hytranslate-mac-prd.md` before any non-trivial
  product, UX, architecture, IPC, privacy, or release work.
- Detailed rules: read the relevant file under `.claude/rules/` when touching
  that area. This file summarizes the rules; the rule files contain the full
  checklist.
- Claude-specific subagent calls in `.claude/CLAUDE.md` are not executable in
  Codex. Convert them into direct Codex actions: read the referenced file, use
  available Codex skills/tools, run the relevant commands, and report results.
- If instructions conflict, obey system/developer/user instructions first, then
  this `AGENTS.md`, then the PRD for product decisions, then `.claude/rules/`
  and `.claude/CLAUDE.md` for details.

## Communication And Language

- User-facing progress updates, questions, and final reports are Korean by
  default unless the user asks for another language.
- Code identifiers, filenames, Rust/TypeScript symbols, test descriptions, and
  log messages are English.
- User-facing UI strings are Korean and centralized in `src/i18n/ko.ts`.
- Code comments are Korean only when the why is non-obvious; otherwise do not
  add comments.
- Commit messages use English Conventional Commits. PR titles/bodies are Korean
  unless the user asks otherwise.

## Project Boundary

HyTranslate Mac is a macOS-only local translation desktop app. It translates
Korean, Simplified Chinese, and Traditional Chinese into English by calling
Ollama on the user's Mac with Tencent Hy-MT2 GGUF models.

Non-negotiables:

- Translation requests stay on the configured Ollama endpoint, default
  `http://localhost:11434`.
- No default-on telemetry, remote logging, cloud sync, accounts, or network
  translation.
- Do not log raw `source_text` or `translated_text` at `info` level.
- v1 output language is English only.
- v1 input limits are 30,000 characters in the main window and 5,000 characters
  in the floating popup.
- Default global hotkey is `Cmd+Shift+T`; translation debounce is 500 ms, with
  `Cmd+Enter` bypassing debounce.
- DB encryption, Ollama bundling, telemetry, cloud sync, and accounts are out of
  v1 unless the user explicitly reopens the decision.

## Project Map

- `src/windows/`: Tauri window entrypoints (`main`, `popup`, `menubar`).
  Keep these composition-focused.
- `src/features/<feature>/`: feature-owned components, Zustand store, IPC
  wrapper, and types. Do not cross-import another feature's store.
- `src/components/`: reusable presentational components.
- `src/lib/`: cross-cutting IPC client, shared hooks, utilities, and helpers.
- `src/i18n/ko.ts`: Korean UI copy and error messages.
- `src/styles/`: global CSS and Tailwind layers.
- `src-tauri/src/commands/`: `#[tauri::command]` adapters only.
- `src-tauri/src/{ollama,history,settings,language,environment}`: domain
  modules. Keep these framework-agnostic; no `tauri::` imports.
- `src-tauri/src/db/`: SQLite pool, migrations, and repo functions.
- `src-tauri/src/events.rs` and `src/lib/ipc/events.ts`: shared event names.
- `src-tauri/capabilities/` and `src-tauri/tauri.conf.json`: Tauri permission
  and CSP surface.
- `tests/e2e/`: Playwright flows.
- `evals/translation-quality.md`: translation quality eval set and thresholds.
- `docs/plans/`: implementation plans and durable decision records.

## Core Commands

Run from the repository root unless noted.

```bash
npm run dev
npm run tauri:dev
npm run build
npm run lint
npm run typecheck
npm run test
npm run test:e2e
npm run format:check
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
cargo build --manifest-path src-tauri/Cargo.toml
```

Use `npm run tauri:build` only when a signed/release app build is explicitly
needed; it is slower than the normal verification loop.

## Architecture Rules

Frontend flow:

```text
React UI -> feature Zustand store -> feature ipc.ts -> Tauri IPC
```

Backend flow:

```text
commands -> domain modules -> db / reqwest / filesystem as needed
```

Keep these boundaries:

- Components do not call `invoke()` directly; use the feature `ipc.ts`.
- Commands should deserialize, call domain code, and serialize. Keep handlers
  short and adapter-focused.
- Domain modules must not import `tauri::*`.
- DB access goes through repo modules, never raw `rusqlite::Connection` in a
  command.
- Long-running work must be cancellable with `CancellationToken`.

## Frontend Rules

- React function components only. Type props as named `interface` declarations.
- Hooks stay at the top level and clean up subscriptions/listeners.
- Zustand stores use selectors: `useStore(s => s.value)`, never bare
  `useStore()`.
- Use discriminated unions for UI state machines.
- Avoid `any`; use `@ts-expect-error <reason>` only when unavoidable.
- Use Tailwind for layout and styling. Static inline styles and `!important` are
  forbidden unless there is a documented technical reason.
- Use `lucide-react` icons; do not use emoji icons in UI.
- Keep Korean UI strings in `src/i18n/ko.ts`, not inline in components.
- Error UI is inline in the affected region, not modal alerts.
- Core actions must be keyboard reachable and VoiceOver friendly.

## Backend Rules

- Rust edition is 2021. Format with `cargo fmt`; pass clippy with `-D warnings`.
- Avoid `unwrap()`/`expect()` outside tests and one-time startup.
- Commands return `Result<T, AppError>` and never `panic!`.
- `AppError` in `src-tauri/src/errors.rs` is the canonical error taxonomy.
- Use one shared `reqwest::Client` and one DB pool from app state.
- Ollama streaming must preserve UTF-8 boundaries and emit ordered Tauri events.
- Migrations are forward-only and versioned; FTS5 stays synced through triggers.
- Logs may include lengths, model IDs, request IDs, durations, and error kinds,
  but not raw source or translated text.

## IPC Contract

- Command names are snake_case and match Rust function names.
- Request structs use `#[serde(rename_all = "camelCase")]`.
- Long-running commands require frontend-generated UUID v4 `requestId`.
- Event names follow `domain:action`, such as `translation:chunk`.
- Event names are defined once in both `src/lib/ipc/events.ts` and
  `src-tauri/src/events.rs`; do not inline magic strings.
- `translate_stream` emits `translation:started`, zero or more
  `translation:chunk`, and exactly one terminal event:
  `translation:completed`, `translation:cancelled`, or `translation:error`.
- Cancellation emits `translation:cancelled` and must not save to history.

Read `.claude/rules/tauri-ipc.md` before changing commands, events, streaming,
or cancellation behavior.

## Security And Privacy

- Treat every webview IPC call as untrusted input.
- Validate command inputs on the Rust side even if the frontend validates them.
- Tauri capabilities must stay minimal and window-scoped where possible.
- Never add `shell-execute` or broad filesystem scopes without explicit user
  approval and PR justification.
- CSP should deny external `http://` except the local Ollama endpoint and should
  not permit script `unsafe-inline`.
- External links should open in the system browser, not inside the app webview.
- New dependencies that open sockets, collect telemetry, or expand Tauri
  permissions must be called out before merging.

## Verification Matrix

After code changes, do a Codex quality gate:

1. Inspect `git diff --name-only` and `git diff`.
2. Read the relevant `.claude/rules/*.md` files for the changed surface.
3. Run targeted verification:
   - `.ts`, `.tsx`, `.css`: `npm run lint`, `npm run typecheck`, and
     `npm run test`.
   - `.rs`: `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`,
     `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings`, and
     `cargo test --manifest-path src-tauri/Cargo.toml`.
   - Shared IPC contract changes: run both frontend and Rust checks.
   - UI/Tauri flow changes: consider `npm run build`; use Playwright only when
     the app flow is affected or the user asks for E2E verification.
   - Markdown/config-only changes: run the narrowest relevant formatter/checker,
     such as `npx prettier --check AGENTS.md` for this file.
4. Summarize what passed, what failed, and any verification not run.

When available and appropriate, use the installed `check` skill for final diff
review. If the skill is unavailable, perform the same review manually.

## Specialized Workflows

- New or unfamiliar external library/API: consult official docs before coding.
  Prefer project MCP docs if available; otherwise browse official sources.
- Commit/push/PR requests: follow
  `.claude/skills/git-commit-push-pr/SKILL.md`. Commit only when the user asks;
  push/PR only when requested. Never force-push shared branches.
- Tauri/security review requests: follow
  `.claude/skills/tauri-code-review/SKILL.md` and its references. Focus first
  on IPC input validation and capability/scope minimization.
- GitHub release, release notes, changelog, version tag, or publish requests:
  follow `.claude/skills/github-release/SKILL.md`. Release notes must be based
  on code that is actually in the tagged commit, not roadmap intent.
- Claude `quality-gate` equivalent: use `.claude/agents/quality-gate.md` as the
  checklist, but execute it directly in Codex.
- Claude `library-docs-fetcher` equivalent: use
  `.claude/agents/library-docs-fetcher.md` as the checklist, but execute it with
  Codex tools and official documentation.

## Stop Conditions

For long-running or iterative agent work, stop and report instead of retrying
indefinitely when any of these occurs:

- Two consecutive checkpoints show no new progress.
- The same error, stack trace, or failing assertion repeats three times.
- A declared token, time, or command budget is exhausted.
- External state blocks progress: missing credentials, unavailable network,
  dependency registry failure, merge conflict, or required user approval.

## Before Finishing

- Confirm the work matches the user's newest request.
- Check the diff for unrelated changes and do not revert user work.
- Report changed files, verification results, and residual risk in Korean.
