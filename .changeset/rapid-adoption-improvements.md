---
"@splitmerge420/uws": minor
---

**Rapid adoption improvements (contrarian audit)**

From a zero-friction adoption standpoint, multiple critical barriers have been addressed:

### Install experience
- Add `install.sh` — curl-pipe installer that downloads platform-native pre-built binaries from GitHub Releases (Linux x86_64/ARM64, macOS Intel/Apple Silicon, Windows). No Rust required for users.

### Branding correctness
- `dist-workspace.toml`: npm-scope → `@splitmerge420`, npm-package → `uws` (was pointing at `@googleworkspace/cli`)
- `package.json`: name, repository URL, author, license (`MIT`), keywords, publishConfig registry all updated to match the actual project

### README quality signals
- Fix license badge: was incorrectly showing Apache 2.0 (Cargo.toml says MIT)
- Add CI status badge pointing to the real workflow URL
- Add `?style=social` to Stars badge for higher click-through
- Add curl install and npm install as the primary install methods
- Add GitHub MCP section showing how to wire `uws` into Copilot, Claude Desktop, and VS Code

### GitHub ecosystem wiring
- Add `.github/FUNDING.yml` — enables the GitHub Sponsors sidebar button
- Add `.github/ISSUE_TEMPLATE/bug_report.md` + `feature_request.md` — structured community engagement
- Add `SECURITY.md` — GitHub surfaces this in the Security tab; required for serious open-source credibility
- Add `.devcontainer/devcontainer.json` — Codespaces one-click dev environment (Rust + Python + Node pre-configured)
- Add `mcp_server/mcp.json` — standard MCP configuration for any MCP-compatible tool (Claude Desktop, VS Code MCP extension, Cursor)
