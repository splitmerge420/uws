# INV-002 — Consent Gating
**Category:** Governance | **Severity:** Critical | **Check Type:** Guard Check

> "All state-changing operations require explicit consent before execution."

---

## Statement

Every operation that mutates state — writes, deletes, creates, updates, sends, modifies —
must be preceded by an explicit, recorded consent signal from the user. Read-only
operations do not require consent gating, but all write paths do.

## Rationale

Without consent gating, AI agents become autonomous actors that change the world without
permission. A single missed consent check can lead to irreversible data loss, unintended
communications, or privacy violations.

## Scope

All `write | delete | create | update | send | modify` operations in any language, any
service. This is the most widely enforced invariant in the toolchain.

## Dangerous Patterns (Linter Detects)

```python
os.system(...)            # shell execution without consent
subprocess.run(...)       # subprocess without consent
open(..., 'w')            # file write without consent
shutil.rmtree(...)        # directory removal without consent
os.remove(...)            # file deletion without consent
fs.writeFile(...)         # Node.js file write
std::fs::write(...)       # Rust file write
```

## Guard Patterns (Must Be Present)

`consent`, `authorize`, `approve`, `confirm`, `validate_permission`,
`ConsentManager`, `consent_manager`, `auto_consent`

## Implementation

| Layer | Implementation |
|-------|----------------|
| `ConstitutionalEngine` | `check_consent_gating()` — blocks write ops when `user_consent = false` |
| `ZeroTrustGate` | `_require_consent()` called before every gate; `ConsentRequired` raised if false |
| `acp_governance.py` | `GovernanceContext.user_consent` checked before `record()` |
| Rego | `consent_enforcement.rego` — `default allow = false` |
| Linter | `invariant_linter.py` flags dangerous patterns without guard patterns |

## Test Vectors

| Scenario | Expected Outcome |
|----------|-----------------|
| `write` operation, `user_consent = false` | `ConstitutionalEngine.enforce()` → `Err(violations)` |
| `write` operation, `user_consent = true` | Passes consent gate |
| `ZeroTrustGate` with `session_consent=False` | Every gate raises `ConsentRequired` |
| Delete without audit enabled | INV-3 also fires |

## Constitutional Relations

- **Depends on:** INV-1 (User Sovereignty) — consent is the mechanism of sovereignty
- **Strengthened by:** INV-3 (Audit Trail) — consent decisions must be recorded
- **Guards against:** any silent mutation of user data or system state

## Status

`IMPLEMENTED` — enforced in `ConstitutionalEngine`, `ZeroTrustGate`, `acp_governance.py`,
`consent_enforcement.rego`, and `invariant_linter.py`.
