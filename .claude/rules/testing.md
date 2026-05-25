# Testing

## Required test layers (PRD §14.3)

### 1. Frontend unit (Vitest)

- Zustand stores: state transitions, async actions
- Pure helpers under `src/lib/`
- IPC wrappers with mocked `invoke` / `listen`
- Component tests for behavior — not snapshots

### 2. Rust unit (`cargo test`)

- **Prompt builder** — one test per source language (PRD §8.3 acceptance)
- **Language detection** — Korean / Simplified / Traditional samples (PRD §8.2 acceptance)
- **Ollama client** with mock streaming (`wiremock` or hand-rolled)
- **Cancellation** — spawn translation, cancel, assert no DB write
- **SQLite migrations** — forward apply on empty DB, assert `schema_version`
- **History FTS5 search** — insert N records, query, assert ranking
- **Settings round-trip** persistence

### 3. E2E (Playwright)

- Onboarding happy path
- Translate via main window
- Translate via floating popup (`Cmd+Shift+T`)
- History search + favorite + delete
- Clipboard translation

## Conventions

- Frontend tests: `<file>.test.ts(x)` colocated with source
- Rust tests: `#[cfg(test)] mod tests` in same file for unit; `tests/<name>.rs` for integration
- E2E specs: `tests/e2e/<flow>.spec.ts`
- Test descriptions in English (`it('should ...')`) — easier to read in CI output
- No flaky tests merged — quarantine + open an issue

## Coverage targets

- Frontend statements: ≥70% in `src/lib/` and `src/features/*/store.ts`
- Rust: ≥80% for `prompt`, `language`, `history`, `db/migrations`
- Coverage is a floor, not a goal — tests must exercise behavior, not just lines

## Quality eval (PRD §14)

Maintain `evals/translation-quality.md` with the required sample set (40 Korean, 40 Simplified, 20 Traditional) and the v1 thresholds:

- Overall mean ≥ 4.0
- Critical mistranslation rate ≤ 5%
- Term-preservation failure (legal / academic) ≤ 10%
- Per-language mean ≥ 3.8
