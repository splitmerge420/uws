# INV-003 — Audit Trail
**Category:** Governance | **Severity:** Critical | **Check Type:** Guard Check

> "Every governance decision, data access, and state change must be logged immutably."

---

## Statement

Every action taken by any agent or automated process must produce an immutable, tamper-
detectable audit record. Destructive operations (`delete`, `remove`, `destroy`, `drop`,
`purge`) require audit logging as a hard prerequisite — they are blocked if no audit is
active.

## Rationale

Without an audit trail, governance is unenforceable. Post-hoc accountability requires
that a complete, unaltered record of all decisions exists. Hash-chaining ensures no entry
can be deleted or modified after the fact.

## Scope

All destructive operations, all governance decisions, all consent records, all security
events, and all inter-agent communications.

## Dangerous Patterns (Linter Detects)

```python
def delete(...):          # Python destructive function without audit guard
def remove(...):
def purge(...):
fn delete(...):           # Rust destructive function without audit guard
async function destroy(): # JS/TS destructive function without audit guard
```

## Guard Patterns (Must Be Present)

`audit`, `log`, `logger`, `logging`, `record`, `trace`, `AuditTrail`, `AuditChain`

## Implementation

| Layer | Implementation |
|-------|----------------|
| `audit_chain.rs` | SHA3-256 hash-chained append-only log; `verify_chain()` detects tampering |
| `ConstitutionalEngine` | `check_audit_trail()` — blocks destructive ops when `audit_enabled = false` |
| `ZeroTrustGate` | Every allow/deny decision appended to internal `AuditChain` |
| `acp_governance.py` | `AuditChain.append()` + `verify_chain()` |
| Rego | `audit_requirements.rego` — `default allow = false` |

## Hash Chain Specification

```
entry_hash = SHA3-256(
    index | timestamp | actor | action | resource |
    decision | invariants_checked | previous_hash
)
```

The genesis hash is 64 zero hex digits. `verify_chain()` walks every link.

## Test Vectors

| Scenario | Expected Outcome |
|----------|-----------------|
| `delete` operation, `audit_enabled = false` | `ConstitutionalEngine` blocks it |
| Tamper with `entry_hash` at index 0 | `verify_chain()` returns `Err(IntegrityViolation)` |
| 100 entries appended then verified | `verify_chain()` returns `Ok(true)` |
| ZeroTrustGate full pipeline | ≥4 audit entries; integrity holds |

## Constitutional Relations

- **Required by:** INV-2 (Consent Gating) — consent decisions must be audited
- **Required by:** INV-5 (Constitutional Authority) — Dave approvals must be audited
- **Strengthened by:** INV-12 (Audit Immutability) — the log itself must be tamper-proof

## Status

`IMPLEMENTED` — `audit_chain.rs` (Rust), `acp_governance.py` (Python),
`ZeroTrustGate` (both languages), `audit_requirements.rego`.
