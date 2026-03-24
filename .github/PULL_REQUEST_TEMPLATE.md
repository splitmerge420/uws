## Description

Please include a summary of the change and which issue is fixed. If adding a new feature or command, please include the output of running it with `--dry-run` to prove the JSON request body matches the Discovery Document schema.

**Related issue:** Fixes #

**Dry Run Output (if applicable):**
```json
// Paste --dry-run output here
```

## Type of Change

<!-- Mark the relevant option with [x] -->

- [ ] 🐛 Bug fix (non-breaking change that fixes an issue)
- [ ] 🚀 New feature (non-breaking change that adds functionality)
- [ ] 💥 Breaking change (fix or feature that would cause existing functionality to change)
- [ ] 📖 Documentation update
- [ ] 🔐 Security fix
- [ ] 🤖 AI agent / MCP integration
- [ ] 🏗 Refactor / internal improvement

## Checklist

- [ ] My code follows the `AGENTS.md` guidelines (no generated `google-*` crates).
- [ ] I have run `cargo fmt --all` (or `make fmt`) to format the code.
- [ ] I have run `cargo clippy -- -D warnings` (or `make lint`) and resolved all warnings.
- [ ] I have added or updated tests that prove my fix is effective or that my feature works.
- [ ] All existing tests pass (`make test` or `cargo test`).
- [ ] I have provided a Changeset file (e.g. via `pnpm changeset`) in the `.changeset/` directory.
- [ ] I have updated documentation if this change affects user-facing behaviour.

## Security Considerations

<!-- Does this change touch authentication, credential storage, URL construction, or file paths? -->
<!-- If yes, describe the security implications and mitigations below. -->

- [ ] This change does not affect security-sensitive code paths.
- [ ] OR: I have reviewed the change against `SECURITY.md` and considered relevant threats.

## Screenshots / Demo (if applicable)

<!-- For UI or output format changes, include before/after screenshots or command output. -->
