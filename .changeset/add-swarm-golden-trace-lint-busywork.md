---
"uws": minor
---

Add `uws swarm review`, GoldenTrace provenance trailer, and `uws lint busywork` (ABF v0 ruleset).

## What was newly built

- **`src/swarm.rs`** — `uws swarm review <pr-ref>` CLI subcommand. Stubbed with a clear description of what it would do; wires into main.rs dispatch. TODO: replace stub with Janus v2 multi-agent router once backend is available.
- **`src/golden_trace.rs`** — GoldenTrace provenance trailer writer. Formats `GoldenTrace: <hash> <timestamp> <invariant-set-version>` trailers and appends them to commit messages / PR descriptions.
- **`src/lint_busywork.rs`** — `uws lint busywork` command with a v0 AntiBusyworkFactor (ABF) starter ruleset (ABF-001 through ABF-005).
- **`tests/lint_busywork_integration.rs`** — Integration test fixture; verifies all 5 ABF rules fire on synthetic inputs.

## Wired from existing

- Main dispatch pattern in `src/main.rs` (same early-exit `if first_arg == "..."` pattern used by `auth`, `schema`, `generate-skills`).
- `sha2` / `chrono` crates (newly added to Cargo.toml, used by GoldenTrace).

## Stubbed

- `uws swarm review` backend — prints what would happen; marked with `// TODO: replace with Janus v2 call`.

## Build fixes (pre-existing)

- Added all missing `[dependencies]` to `Cargo.toml` (tokio, clap + `string` feature, reqwest 0.12, serde_json, anyhow, etc.) that were previously commented out as "Phase 2".
- Added missing dev-dependencies: `tempfile`, `serial_test`.

## Followups

- Wire `handle_review` in `src/swarm.rs` to the Janus v2 multi-agent router (blocked on PR #14 / swarm backend stabilisation).
- Add GoldenTrace integration points: commit-message formatter, PR description writer, audit-chain entry.
- Expand ABF ruleset beyond v0: add rules for duplicate PR descriptions, auto-closed-and-reopened pattern, etc.
- Add `uws swarm status` and `uws swarm list-agents` subcommands once backend exists.
- Consider exposing `golden_trace` from the public library API for use by external tooling.
