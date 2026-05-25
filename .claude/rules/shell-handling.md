# Shell Handling

## Tool preferences (override built-in behavior)

For search and listing, prefer modern CLIs:

- `rg` (ripgrep) instead of `grep` and built-in Grep
- `fd` instead of `find` and built-in Glob
- `eza --tree --level=N` instead of `ls -R`
- `bat --plain --paging=never` only when highlighting helps; otherwise use the Read tool
- `ast-grep` (`sg`) for AST-based search and structural refactors
- `jq` for JSON, `yq` for YAML, `http` (httpie) for HTTP

## Non-interactive principle

- Force non-interactive flags on every external CLI: `--yes`, `--quiet`, `--no-input`, `--non-interactive`
- Prefer JSON output: `--format json`, `--json`, `--output json`
- `gh` always in non-interactive mode

## Project-specific commands

- `npm run tauri:dev` — full app (hot reload across FE + Rust)
- `npm run dev` — frontend-only (no Tauri APIs available; use sparingly)
- `cargo` commands take `--manifest-path src-tauri/Cargo.toml` when run from repo root
- `npx playwright test` only after `npm run tauri:build` or with an app instance running

## Forbidden in Bash

- `cat` / `head` / `tail` to read files Claude can Read directly — use the Read tool
- `sed -i` / `awk` to modify files — use the Edit tool
- `echo > file.txt` to write files — use the Write tool
- Chained `cd` before git commands — git already operates on the working tree
- Long sleeps for polling — use Monitor or background tasks
