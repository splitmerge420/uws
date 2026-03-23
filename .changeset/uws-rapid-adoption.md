---
"uws": minor
---

feat: rename all user-facing `gws` references to `uws` for consistent branding

- All user-visible messages, help text, error strings, and binary references now say `uws`
- Config dir migrated from `~/.config/gws` to `~/.config/uws`; existing installs fall back gracefully
- `package.json`: renamed to `@uws-cli/uws`, updated homepage/repo/author/keywords, switched npm registry to npmjs.org
- `dist-workspace.toml`: updated npm scope to `@uws-cli` and npm-package to `uws`
- `flake.nix`: renamed pname/mainProgram/packages from `gws` to `uws`, fixed maintainer email
- CI (`ci.yml`): artifact names and binary paths updated from `gws` to `uws`
- `CODEOWNERS`: updated from upstream author to `@splitmerge420`
- Added `action.yml` GitHub Actions composite action (`uses: splitmerge420/uws@v1`)
- Added `.devcontainer/devcontainer.json` for GitHub Codespaces one-click onboarding
- Added `install.sh` curl-pipe installer supporting Linux and macOS (x86_64 + aarch64)
- Added `.github/workflows/examples/uws-usage.yml` with real workflow examples
- README: expanded Installation section with curl|sh, npm, cargo, GitHub Actions, Nix, and Codespaces methods; added CI and release badges
