---
"@splitmerge420/uws": minor
---

**30+ GitHub ecosystem integrations**

Every GitHub product and feature that benefits open-source adoption has been wired in:

### Supply Chain & Security (GitHub Advanced Security)
- **`.github/workflows/codeql.yml`** — CodeQL SAST analysis for Rust + Python, results uploaded to GitHub Security tab
- **`.github/workflows/dependency-review.yml`** — blocks PRs that introduce vulnerable/license-incompatible dependencies
- **`.github/workflows/audit.yml`** — daily `cargo audit` advisory scan, SARIF uploaded to Security tab
- **`.github/workflows/scorecard.yml`** — OpenSSF Scorecard supply chain security scoring, published to scorecard.dev + GitHub Security tab
- **`CITATION.cff`** — GitHub shows "Cite this repository" panel in sidebar

### Dependency Automation
- **`.github/dependabot.yml`** — automated dependency update PRs for GitHub Actions, Cargo, and npm (weekly, grouped)

### Release Experience
- **`.github/release.yml`** — GitHub auto-generated release notes with labelled categories (🚀 Features, 🐛 Fixes, 🔐 Security, etc.)

### Developer Experience
- **`Makefile`** — `make build`, `make test`, `make lint`, `make fmt`, `make check`, `make mcp`, `make docs`, `make docker`, `make install` — works in Codespaces and locally
- **`.github/actions/setup-rust/action.yml`** — reusable composite action (DRY Rust + sccache + cache setup)
- **`.github/workflows/reusable-rust.yml`** — reusable `workflow_call` for Rust setup
- **`.github/copilot-instructions.md`** — GitHub Copilot Workspace instructions injected into every chat session

### Documentation & Discoverability
- **`mkdocs.yml`** — MkDocs Material documentation site configuration
- **`docs/index.md`** — GitHub Pages landing page
- **`.github/workflows/pages.yml`** — GitHub Pages automatic deployment on every push to main

### Container Registry (GitHub Packages / GHCR)
- **`Dockerfile`** — multi-stage build: Rust binary + Python MCP server in one image
- **`.dockerignore`** — minimal context for fast Docker builds
- **`.github/workflows/docker.yml`** — builds and pushes multi-arch image (`linux/amd64`, `linux/arm64`) to `ghcr.io/splitmerge420/uws` with attestations

### Benchmarks
- **`benches/README.md`** — benchmark scaffolding guide
- **`.github/workflows/bench.yml`** — Criterion benchmark workflow, results tracked in gh-pages

### Community & Engagement
- **`.github/workflows/greet.yml`** — welcome first-time contributors and issue reporters
- **`.github/workflows/pr-size.yml`** — PR size labeler (XS/S/M/L/XL)
- **`.github/ISSUE_TEMPLATE/config.yml`** — community links, Security advisory link, Discussions link; disables blank issues
- **`.github/DISCUSSION_TEMPLATE/ideas.yml`** — structured GitHub Discussions form for feature proposals
- **`.github/DISCUSSION_TEMPLATE/q-and-a.yml`** — structured Q&A discussion form
- **`.github/workflows/auto-milestone.yml`** — auto-assigns milestones based on area labels

### Observability
- **`.github/workflows/links.yml`** — weekly broken link checker across all Markdown files
- **`.github/workflows/typos.yml`** — spell checker (`typos`) on every push/PR
- **`_typos.toml`** — allowlist for domain-specific terms (Kintsugi, Noosphere, Dilithium, etc.)
- **`.github/workflows/summary.yml`** — enriched GitHub Actions job summaries
- **`codecov.yml`** — Codecov precision configuration (patch + project thresholds)

### Governance & Maintenance
- **`.github/CODEOWNERS`** — updated from `jpoehnelt` to `splitmerge420` across all core files
- **`.github/labeler.yml`** — extended with `area: microsoft`, `area: apple`, `area: android`, `area: ci`, `area: security`, `area: aluminum`
- **`.github/PULL_REQUEST_TEMPLATE.md`** — expanded with type-of-change selector and security section
