# Contributing to uws

Thank you for contributing to the Universal Workspace CLI. This project is building the command surface of the AI-native OS — your contributions matter.

---

## Ways to Contribute

- **Add a new provider driver** (e.g., Notion, Slack, Linear, Zoom)
- **Improve an existing ecosystem module** (Microsoft, Apple, Android, Chrome)
- **Add or improve AI agent SKILL.md files**
- **Write tests** for existing modules
- **Improve documentation** (README, ALUMINUM.md, AGENTS.md)
- **Report bugs** via GitHub Issues

---

## Development Setup

```bash
git clone https://github.com/splitmerge420/uws
cd uws
cargo build
cargo test
cargo clippy -- -D warnings
```

---

## Adding a New Provider Driver

Every provider implements the `ProviderDriver` trait:

```rust
pub trait ProviderDriver: Send + Sync {
    /// Short identifier, e.g. "notion", "slack"
    fn name(&self) -> &str;

    /// Authenticate and return a bearer token
    async fn authenticate(&self) -> Result<String>;

    /// Execute a resource/method call and return JSON
    async fn execute(
        &self,
        resource: &str,
        method: &str,
        params: Option<&str>,
        body: Option<&str>,
        token: &str,
    ) -> Result<serde_json::Value>;

    /// List all resources this driver supports
    fn list_resources(&self) -> Vec<ResourceDescriptor>;
}
```

Steps:

1. Create `src/<provider>.rs` implementing `ProviderDriver`
2. Register the service aliases in `src/services.rs`
3. Add auth env vars to `.env.example`
4. Create `skills/<provider>-<service>/SKILL.md` following the pattern in `skills/uws-core/SKILL.md`
5. Write tests in `src/<provider>.rs`
6. Add a changeset at `.changeset/<descriptive-name>.md`

---

## Adding an AI Agent Skill

Skills are Markdown files that tell AI agents (Claude, Manus, Gemini) how to use a specific service.

Structure:

```
skills/
  <provider>-<service>/
    SKILL.md          # Required: instructions for AI agents
    examples/         # Optional: example command outputs
```

A good `SKILL.md` includes:
- Service overview (one paragraph)
- Prerequisites (auth setup)
- Common commands with real examples
- Output example (JSON snippet)
- AI agent notes (gotchas, best practices)

---

## Code Quality Standards

| Requirement | Tool |
|---|---|
| No warnings | `cargo clippy -- -D warnings` |
| Tests pass | `cargo test` |
| Path inputs validated | `validate::validate_safe_output_dir()` |
| URL segments encoded | `helpers::encode_path_segment()` |
| JSON output on success | All commands must output valid JSON |
| Changeset included | `.changeset/<name>.md` |

---

## Changeset Format

Every PR must include a changeset:

```markdown
---
"uws": patch
---

Fix: correct Microsoft Graph token refresh on 401 response
```

Use `patch` for fixes, `minor` for new features, `major` for breaking changes.

---

## The Aluminum OS Roadmap

If you want to contribute to the larger Aluminum OS vision (provider abstraction layer, identity substrate, agent runtime), read [ALUMINUM.md](ALUMINUM.md) first. The most impactful contributions right now are:

1. **Phase 2**: Completing the Microsoft Graph backend (`src/ms_graph.rs`)
2. **Phase 3**: Completing the Apple CalDAV/CardDAV/CloudKit backend (`src/apple.rs`)
3. **Phase 4**: The `alum` provider abstraction layer and `--provider` flag

---

## Code of Conduct

Be excellent to each other. This project is open to contributors of all backgrounds and experience levels.

---

## License

By contributing, you agree that your contributions will be licensed under the Apache License 2.0.

---

*Questions? Open a GitHub Discussion at [github.com/splitmerge420/uws/discussions](https://github.com/splitmerge420/uws/discussions)*
