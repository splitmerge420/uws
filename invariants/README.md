# Invariants Registry — uws Constitutional OS

This directory contains the **24 Constitutional Invariants (INV-001 – INV-024)**
that govern every operation in the uws Universal Workspace CLI.

Invariants are the constitutional bedrock of the Aluminum OS governance model.
They are enforced at runtime by:

- `src/zero_trust.rs` — Zero Trust Integration Gate (INV-004, INV-003, INV-005)
- `src/constitutional_engine.rs` — Runtime invariant checking engine
- `src/audit_chain.rs` — Append-only audit log (INV-003, INV-005)
- `src/council_github_client.rs` — Constitutional GitHub wrapper

---

## Invariant Index

| ID | Slug | Severity | Status | Enforcer |
|----|------|----------|--------|----------|
| [INV-001](INV-001-sovereignty.md) | sovereignty | Critical | ✅ Specified | `constitutional_engine.rs` |
| [INV-002](INV-002-consent.md) | consent | Critical | ✅ Specified | `constitutional_engine.rs` |
| [INV-003](INV-003-audit-trail.md) | audit-trail | Mandatory | ✅ Specified | `audit_chain.rs` |
| [INV-004](INV-004-zero-trust.md) | zero-trust | Mandatory | ✅ Specified | `zero_trust.rs` |
| [INV-005](INV-005-fail-closed.md) | fail-closed | Critical | ✅ Specified | `zero_trust.rs`, `audit_chain.rs` |
| [INV-006](INV-006-provider-abstraction.md) | provider-abstraction | Mandatory | 🔲 Draft | `constitutional_engine.rs` |
| [INV-007](INV-007-vendor-balance.md) | vendor-balance | Warning | 🔲 Draft | `constitutional_engine.rs` |
| [INV-008](INV-008-encryption-at-rest.md) | encryption-at-rest | Mandatory | 🔲 Draft | TBD |
| [INV-009](INV-009-data-minimisation.md) | data-minimisation | Advisory | 🔲 Draft | TBD |
| [INV-010](INV-010-ephemeral-tokens.md) | ephemeral-tokens | Mandatory | 🔲 Draft | TBD |
| [INV-011](INV-011-provenance-trail.md) | provenance-trail | Mandatory | 🔲 Draft | `council_github_client.rs` |
| [INV-012](INV-012-human-in-the-loop.md) | human-in-the-loop | Critical | 🔲 Draft | TBD |
| [INV-013](INV-013-rate-limit-respect.md) | rate-limit-respect | Advisory | 🔲 Draft | TBD |
| [INV-014](INV-014-no-silent-data-loss.md) | no-silent-data-loss | Critical | 🔲 Draft | TBD |
| [INV-015](INV-015-dry-run-parity.md) | dry-run-parity | Mandatory | 🔲 Draft | TBD |
| [INV-016](INV-016-output-determinism.md) | output-determinism | Advisory | 🔲 Draft | TBD |
| [INV-017](INV-017-credential-isolation.md) | credential-isolation | Critical | 🔲 Draft | `credential_store.rs` |
| [INV-018](INV-018-input-sanitisation.md) | input-sanitisation | Mandatory | 🔲 Draft | `validate.rs` |
| [INV-019](INV-019-open-source-deps.md) | open-source-deps | Advisory | 🔲 Draft | TBD |
| [INV-020](INV-020-schema-validation.md) | schema-validation | Mandatory | 🔲 Draft | `executor.rs` |
| [INV-021](INV-021-backwards-compat.md) | backwards-compat | Warning | 🔲 Draft | TBD |
| [INV-022](INV-022-reproducible-build.md) | reproducible-build | Mandatory | 🔲 Draft | `Cargo.lock` |
| [INV-023](INV-023-agent-attribution.md) | agent-attribution | Mandatory | 🔲 Draft | TBD |
| [INV-024](INV-024-constitutional-amendment.md) | constitutional-amendment | Critical | 🔲 Draft | TBD |

---

## Severity Scale

| Level | Meaning |
|-------|---------|
| **Critical** | Violation halts execution immediately (fail-closed). |
| **Mandatory** | Violation is logged and reported; operation may be blocked depending on context. |
| **Warning** | Violation is logged; operation proceeds with a warning. |
| **Advisory** | Informational; no automatic blocking. |

---

## Enforcement Mechanisms

1. **Pre-flight gate** — `ZeroTrustGate::check()` in `src/zero_trust.rs` runs before every external integration call.
2. **Runtime engine** — `ConstitutionalEngine::check_all()` in `src/constitutional_engine.rs` evaluates the full invariant set against a `StateSnapshot`.
3. **Audit chain** — `AuditChain::append()` in `src/audit_chain.rs` creates an immutable, hash-chained record of every gate decision.
4. **Static analysis** — `cargo clippy -- -D warnings` enforces code-level invariants at compile time.

---

## Adding a New Invariant

1. Create `invariants/INV-XXX-<slug>.md` following the template in any fully-specified invariant.
2. Update this `README.md` index table.
3. If the invariant requires runtime enforcement, add a check in `src/constitutional_engine.rs` or `src/zero_trust.rs`.
4. Add a test fixture in the relevant `#[cfg(test)]` module.

---

*Source authority: [ALUMINUM.md](../ALUMINUM.md) · [AGENTS.md](../AGENTS.md)*
*Registry version: 0.1.0 — INV-001 through INV-024*
