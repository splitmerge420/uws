# INV-003 — Audit Trail

| Field | Value |
|-------|-------|
| **ID** | INV-003 |
| **Slug** | audit-trail |
| **Title** | Immutable Audit Trail |
| **Severity** | Mandatory |
| **Status** | ✅ Specified |
| **Source** | `src/audit_chain.rs`, `src/zero_trust.rs`, `src/lib.rs` |
| **Test fixture** | `src/audit_chain.rs` → `test_*` (8 tests), `src/zero_trust.rs` → `test_audit_log_written_on_block` |

---

## Rationale

AI agents operating at scale can make thousands of decisions per hour.
Without an immutable audit trail, accountability is impossible — there
is no way to trace why a decision was made, who authorized it, or whether
the system behaved as intended.

The `AuditChain` in `src/audit_chain.rs` provides SHA3-256 hash-chained
audit entries that are append-only and tamper-evident: `verify_chain()`
detects any modification of historical entries.

The `ZeroTrustGate` in `src/zero_trust.rs` appends a structured log line
to `GatePolicy::audit_log_path` whenever it blocks an integration.

## Invariant Statement

> **Every decision (allow or block) made by the Zero Trust Gate, the
> Constitutional Engine, or the Council GitHub Client MUST be recorded
> in an immutable, hash-chained audit log that can be verified for
> tampering at any time.**

## Enforcement Mechanism

- `AuditChain::append()` — the only way to add entries; no modify/delete API exists.
- `AuditChain::verify_chain()` — walks every entry and verifies hash links.
- `ZeroTrustGate::write_audit_log()` — appends a plain-text audit line on block.
- `CouncilGitHubClient` — writes to the `AuditChain` for every operation.

## Data Model

Each `AuditEntry` contains:

```
index           Sequential (monotonically increasing)
timestamp       ISO 8601 wall clock
actor           Who initiated the action
action          Operation name
resource        What was affected
decision        ALLOW | DENY | WARN | BLOCKED
invariants_checked  Which INVs were evaluated
evidence        Supporting context
entry_hash      SHA3-256 of this entry's content
previous_hash   SHA3-256 of the prior entry (chain link)
```

## Test Fixtures

```rust
// src/audit_chain.rs
#[test]
fn test_verify_valid_chain() { ... }   // Happy path: chain validates
#[test]
fn test_detect_tampering() { ... }    // Tampered entry detected

// src/zero_trust.rs
#[test]
fn test_audit_log_written_on_block() { ... }   // Gate writes audit on block
#[test]
fn test_audit_log_not_written_on_allow() { ... } // Gate does NOT write on allow
```

## Followups

- [ ] Switch `AuditChain` hash from FNV-1a simulation to real SHA3-256 (`sha3` crate).
- [ ] Add `AuditChain` write to `ZeroTrustGate::check()` (currently writes to flat log only).
- [ ] Expose `uws audit verify` CLI command to check chain integrity.
