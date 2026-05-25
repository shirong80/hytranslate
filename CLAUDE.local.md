# Local Configuration

## CLI Tool Preferences (OVERRIDE DEFAULT TOOL BEHAVIOR)

For file/code SEARCH operations, use Bash with these CLI tools
INSTEAD OF the built-in Grep and Glob tools:

- Use `rg` instead of the Grep tool (e.g., `rg "pattern" --type ts`)
- Use `fd` instead of the Glob tool (e.g., `fd -e tsx -t f`)
- Use `eza` instead of ls via Bash (e.g., `eza --tree --level=2`)

For file READING and EDITING, continue using the built-in Read and Edit tools.

### Code Structure Search and Refactoring
- Use `ast-grep` (sg) for AST-based search/refactoring (e.g., `sg --pattern '$FUNC($$$ARGS)' --lang ts`)

### Data Processing
- JSON parsing: Use `jq` pipelines (e.g., `http GET api/endpoint | jq '.data[]'`)
- YAML parsing: Use `yq` pipelines (e.g., `yq '.services' docker-compose.yml`)
- HTTP requests: Use `httpie` (`http`) (e.g., `http --json GET https://api.example.com`)

### GitHub Operations
- Use `gh` CLI in non-interactive mode (e.g., `gh pr list --json number,title,state`)

### Linting and Formatting
- Python: Use `ruff` (e.g., `ruff check --fix .`, `ruff format .`)
- Shell: Use `shellcheck` + `shfmt`

### Diff Display
- Use `difft` (difftastic) when structural diff is needed

### Non-Interactive Principle
- Force non-interactive mode for all external CLI commands (`--yes`, `--quiet`, `--no-input`, etc.)
- Prefer JSON output format when possible (`--format json`, `--json`, `--output json`)
