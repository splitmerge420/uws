---
"uws": minor
---

feat: GitHub provider + full adoption engineering for rapid universal uptake

## What this adds

### GitHub REST API provider (`uws github`)
- Full endpoint catalogue: repos, issues, PRs, releases, Actions, search, users, notifications, stars, gists, contents, commits, orgs, labels, milestones
- PAT auth via `GITHUB_TOKEN` / `UWS_GITHUB_TOKEN` — no OAuth flow, works in GitHub Actions automatically
- 27 pure unit tests; no network calls required
- Early dispatch in `src/main.rs`, registered in `src/lib.rs`
- `--dry-run` support; token never logged

### GitHub platform integration points
- `action.yml` — `uses: splitmerge420/uws@main` GitHub Action
- `gh-extension/gh-uws` — `gh extension install splitmerge420/uws` gh CLI extension
- `.devcontainer/devcontainer.json` — Codespaces ready with Rust, rust-analyzer, GitHub CLI, and Copilot
- `.github/copilot-instructions.md` — Copilot reads this and auto-suggests uws commands
- `copilot-extension.json` — Copilot Extension skillset manifest
- `.github/ISSUE_TEMPLATE/` — bug_report.yml + feature_request.yml + config.yml

### Developer experience
- `install.sh` — `curl -fsSL .../install.sh | sh` one-line installer for Linux/macOS
- `skills/github/SKILL.md` — AI agent skill for GitHub provider (Manus/Claude/Gemini)
- `.env.example` — updated with GitHub, Microsoft 365, Apple, and AI agent tokens
- `README.md` — 6 install paths, GitHub benefits table, GitHub provider command reference, CI badge

### Clippy fixes
- `src/github_provider.rs` — fix `redundant_closure`, `needless_lifetimes`, `manual_strip`
- `src/local_noosphere.rs` — fix `redundant_closure`; add `#[allow(clippy::too_many_arguments)]`
- `src/janus.rs` — add `#[allow(clippy::too_many_arguments)]`
