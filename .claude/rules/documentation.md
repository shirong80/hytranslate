# Documentation & Source-Code Language

## Source-code language policy

| Audience | Language |
|---|---|
| Code identifiers (vars, fns, types, files) | **English** |
| Code comments | **Korean** (project convention) |
| Commit message title | **Korean** |
| User-facing UI strings | **Korean** — centralized in `src/i18n/ko.ts` |
| PRD / docs / READMEs | **Korean** |
| Test descriptions | **English** (`it('should …')`) — easier in CI output |
| `tracing` / log messages | **English** |

## Comment policy

- Default: write no comment. Identifiers should carry meaning
- Write a comment only when the **why** is non-obvious:
  - Hidden constraint (e.g., "Ollama returns a final `done:true` chunk with empty `response`")
  - Subtle invariant (e.g., "Must hold the lock for the full UTF-8 accumulator window")
  - Workaround referencing a specific upstream issue (link it)
- Do NOT restate code, reference the current PR, or list "added by X"

## File headers

- No file header banners
- No `@author` / `@since` tags

## Markdown files

- `README.md`, `AGENTS.md`: keep terse; link to the PRD as source of truth
- Plans / TODOs: under `docs/plans/` (matches `settings.local.json#plansDirectory`)
- Do NOT auto-generate `.md` files unless the user explicitly asks
