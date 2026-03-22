# Copilot Instructions for `uws` — Universal Workspace CLI

## What this repo is

`uws` is a Rust CLI that gives you — and GitHub Copilot — structured JSON access to every major productivity ecosystem:
- **Google Workspace**: Gmail, Drive, Calendar, Docs, Sheets, Slides, Tasks, People, Chat, Keep, Meet
- **Microsoft 365**: Outlook Mail, Outlook Calendar, OneDrive, Teams, To Do, OneNote, SharePoint, Planner
- **Apple iCloud**: Calendar (CalDAV), Contacts (CardDAV), Drive (CloudKit), Notes, Reminders
- **GitHub**: Repos, Issues, PRs, Releases, Actions, Search, Users, Stars, Gists, Notifications
- **Android**: Device management, Messages (RCS)
- **Chrome**: Management, Policy, Web Store, ChromeOS devices

All output is clean JSON. All commands follow the same grammar.

---

## When to use `uws` as a tool

When the user asks you to:
- Read or send email → use `uws gmail` or `uws ms-mail`
- Read or create calendar events → use `uws calendar` or `uws ms-calendar`
- List or search files → use `uws drive` or `uws ms-onedrive`
- Work with GitHub repos, issues, PRs, releases, Actions → use `uws github`
- Manage tasks → use `uws tasks` or `uws ms-todo`
- Read or write notes → use `uws keep` or `uws apple-notes`

---

## Command grammar

```
uws <service> <resource> [sub-resource] <method> [flags]
```

### Universal flags

| Flag | Description |
|---|---|
| `--params <JSON>` | URL/query parameters AND path parameters |
| `--json <JSON>` | Request body (POST/PATCH/PUT) |
| `--format json\|table\|yaml\|csv` | Output format (default: json) |
| `--page-all` | Auto-paginate all results |
| `--dry-run` | Preview the request without executing |

---

## GitHub provider (`uws github`)

The GitHub provider is optimised for agent use — PAT auth via `GITHUB_TOKEN`, no OAuth flow needed.

```bash
# Repos
uws github repos list
uws github repos get --params '{"owner":"octocat","repo":"Hello-World"}'
uws github repos create --json '{"name":"my-repo","private":false}'

# Issues
uws github issues list --params '{"owner":"octocat","repo":"Hello-World","state":"open"}'
uws github issues get --params '{"owner":"octocat","repo":"Hello-World","issue_number":42}'
uws github issues create \
  --params '{"owner":"octocat","repo":"Hello-World"}' \
  --json '{"title":"Found a bug","body":"I found a bug","labels":["bug"]}'
uws github issues update \
  --params '{"owner":"octocat","repo":"Hello-World","issue_number":42}' \
  --json '{"state":"closed"}'

# Pull Requests
uws github pulls list --params '{"owner":"octocat","repo":"Hello-World","state":"open"}'
uws github pulls get --params '{"owner":"octocat","repo":"Hello-World","pull_number":1}'
uws github pulls files --params '{"owner":"octocat","repo":"Hello-World","pull_number":1}'

# Releases
uws github releases list --params '{"owner":"octocat","repo":"Hello-World"}'
uws github releases latest --params '{"owner":"octocat","repo":"Hello-World"}'
uws github releases create \
  --params '{"owner":"octocat","repo":"Hello-World"}' \
  --json '{"tag_name":"v1.0.0","name":"v1.0.0","body":"Release notes"}'

# GitHub Actions
uws github actions list --params '{"owner":"octocat","repo":"Hello-World"}'
uws github actions runs --params '{"owner":"octocat","repo":"Hello-World"}'
uws github actions jobs --params '{"owner":"octocat","repo":"Hello-World","run_id":"12345"}'
uws github actions dispatch \
  --params '{"owner":"octocat","repo":"Hello-World","workflow_id":"ci.yml"}' \
  --json '{"ref":"main"}'

# Search
uws github search repos --params '{"q":"language:rust stars:>1000"}'
uws github search issues --params '{"q":"is:open is:issue label:bug"}'
uws github search code --params '{"q":"ProviderDriver repo:splitmerge420/uws"}'

# User / Notifications / Stars
uws github user me
uws github users get --params '{"username":"octocat"}'
uws github notifications list
uws github stars list

# File contents
uws github contents get --params '{"owner":"octocat","repo":"Hello-World","path":"README.md"}'

# Commits
uws github commits list --params '{"owner":"octocat","repo":"Hello-World"}'

# Organizations
uws github orgs list
uws github orgs members --params '{"org":"my-org"}'
```

### GitHub auth setup

```bash
export GITHUB_TOKEN=ghp_xxxxxxxxxxxxxxxxxxxx   # or: GITHUB_TOKEN set by GitHub Actions
# or uws-specific override:
export UWS_GITHUB_TOKEN=ghp_xxxxxxxxxxxxxxxxxxxx
```

---

## Rules for Copilot when using `uws`

1. **Always `--dry-run` first** before any write, create, update, or delete operation.
2. **Always confirm with the user** before: creating issues, merging PRs, creating releases, deleting repositories.
3. **Parse the JSON response** to extract only the fields needed.
4. **Use `--params` for filtering** (`state`, `labels`, `per_page`, `q`) to minimize response size.
5. **Use `--page-all` sparingly** — only when you genuinely need all results, not just the first page.
6. **Check auth** if commands fail: ensure `GITHUB_TOKEN` is set in the environment.
7. **GitHub Actions context**: `GITHUB_TOKEN` is automatically available in all workflow steps.

---

## Architecture notes (for code contributions)

- `src/github_provider.rs` — Pure functions: endpoint catalogue, URL building, request construction. No HTTP in tests.
- `src/ms_graph.rs` — Microsoft 365 service registry (same pattern as github_provider).
- `src/services.rs` — Google Discovery Service registry.
- `src/main.rs` — Early dispatch: `uws github` → `github_provider`, `uws auth` → `auth_commands`, all others → Google Discovery path.
- `src/lib.rs` — Library crate with all testable modules.
- All new providers: follow the `github_provider.rs` pattern (service table → `is_X_service()` → `build_request()` → `handle_X_command()`).
- Tests: pure unit tests in the module file, no network calls.
- `cargo clippy -- -D warnings` must pass before every commit.

---

## Build and test

```bash
cargo build                   # dev build
cargo test --lib              # all unit tests (143+ tests, no network required)
cargo clippy -- -D warnings   # lint
```
