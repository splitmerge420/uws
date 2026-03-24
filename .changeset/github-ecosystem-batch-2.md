---
"@splitmerge420/uws": minor
---

**GitHub ecosystem integrations — Batch 2 (20+ additional features)**

### Issue Templates — Upgraded to Structured YAML Forms
- **`bug_report.yml`** — replaces markdown template with a full YAML form: provider dropdown, OS picker, version field, mandatory checklists
- **`feature_request.yml`** — replaces markdown template with structured form: API reference field, provider dropdown, AI agent use-case field
- **`provider_request.yml`** — new form for requesting new provider integrations (API type, auth type, key use cases)
- **`ISSUE_TEMPLATE/config.yml`** — extended with Governance and Contributing Guide links

### Repository Infrastructure
- **`rust-toolchain.toml`** — pins Rust toolchain to `1.85.0` for consistent builds across all environments
- **`.cargo/config.toml`** — workspace-level Cargo flags, cross-compilation target settings, profile tuning
- **`.gitattributes`** — LF line endings, binary file markers, Linguist overrides (skills/ as documentation, pnpm-lock.yaml as generated), export-ignore for CI infra
- **`Cargo.toml`** — adds `rust-version = "1.85.0"` MSRV declaration

### New GitHub Actions Workflows
- **`validate-pr-title.yml`** — enforces Conventional Commits format on PR titles (feat/fix/docs/ci/chore/…)
- **`lock.yml`** — automatically locks closed issues (30 days) and PRs (14 days) to prevent stale comments
- **`assign.yml`** — auto-assigns issues and PRs to maintainers when they open them
- **`msrv.yml`** — verifies the declared MSRV (1.85.0) on every Rust change and weekly
- **`nightly.yml`** — daily CI against nightly Rust; automatically opens a GitHub issue if it fails
- **`wip.yml`** — blocks merging PRs marked as WIP (title or label)
- **`cherry-pick.yml`** — `/backport <branch>` slash command in PR comments for backporting
- **`needs-info.yml`** — auto-closes issues labelled `needs-info` after 21 days with no response
- **`contributors.yml`** — auto-updates `CONTRIBUTORS.md` after every push to main
- **`windows.yml`** — Windows (`x86_64-pc-windows-msvc`) CI on every Rust change and weekly
- **`draft-release.yml`** — creates a GitHub Draft Release automatically when a version tag is pushed

### Community & Governance
- **`SUPPORT.md`** — GitHub Support sidebar file (routes questions to Discussions, bugs to Issues, security to Private Advisory)
- **`CONTRIBUTORS.md`** — contributor list (auto-updated by workflow)
- **`GOVERNANCE.md`** — BDFL model, decision-making process, roadmap milestones, CoC reference
- **`CODE_OF_CONDUCT.md`** — moved to repo root (GitHub discovers it there for the Community Standards checker)

### README
- **7 new badges**: CodeQL status, Codecov coverage %, OpenSSF Best Practices, OpenSSF Scorecard, Latest Release, Docker (GHCR), MSRV

### Makefile additions
- `make hooks` — installs lefthook git hooks
- `make msrv` — runs MSRV check locally
