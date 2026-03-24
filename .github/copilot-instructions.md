---
# GitHub Copilot Workspace instructions for the uws repository
# https://docs.github.com/en/copilot/customizing-copilot/adding-repository-custom-instructions-for-github-copilot
#
# These instructions are injected into every GitHub Copilot chat session
# opened from this repository (Copilot Workspace, VS Code, GitHub.com chat).
---

You are working in **uws** — the Universal Workspace CLI, a Rust binary that gives
humans and AI agents structured JSON-first access to Google Workspace, Microsoft 365,
Apple iCloud, Android, and Chrome via a single unified grammar.

## Architecture

- **Dynamic discovery**: Google services are built by fetching Discovery Documents at runtime.
  Do NOT add `google-*` generated crates to `Cargo.toml`.
- **Provider drivers**: Microsoft → `src/ms_graph.rs`, Apple → `src/apple.rs`,
  Android/Chrome → `src/android_chrome.rs`.
- **Aluminum OS**: The multi-provider abstraction layer in `src/universal/`.

## Coding Conventions

- Rust: `cargo clippy -- -D warnings` must pass. Format with `cargo fmt --all`.
- Node.js: Use `pnpm` not `npm`.
- All user-supplied file paths must be validated with `validate::validate_safe_output_dir()`.
- All URL path segments must be encoded with `helpers::encode_path_segment()`.
- Every PR requires a `.changeset/<name>.md` file.

## Testing

```bash
cargo test            # unit tests
cargo clippy -- -D warnings  # lint
cargo fmt --all       # format
```

## Key Files

| File | Purpose |
|---|---|
| `src/main.rs` | Two-phase CLI entry point |
| `src/services.rs` | Service alias → Discovery API mapping |
| `src/executor.rs` | HTTP request construction |
| `src/ms_graph.rs` | Microsoft Graph backend |
| `src/universal/` | Aluminum OS abstraction layer |
| `skills/` | AI agent SKILL.md files |
| `mcp_server/server.py` | MCP server for Copilot/Claude/Gemini/Manus |
| `AGENTS.md` | Agent integration guide (read this!) |
| `ALUMINUM.md` | Full architecture specification |

## DO NOT

- Do not add `google-*` generated crates — use dynamic Discovery Documents
- Do not echo credentials to stdout, logs, or `--dry-run` output
- Do not skip the `.changeset/` requirement for Rust changes
- Do not use `npm` — always use `pnpm`
