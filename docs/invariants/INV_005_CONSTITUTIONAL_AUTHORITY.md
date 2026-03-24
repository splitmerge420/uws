# INV-005 — Constitutional Authority
**Category:** Governance | **Severity:** Critical | **Check Type:** Advisory

> "Dave Protocol: Dave Sheldon has veto power on all Critical-severity rules."

---

## Statement

Dave Sheldon holds constitutional authority (INV-5) over all Critical-severity invariants.
Any operation that would change, suspend, or override a Critical invariant requires an
explicit approval token from Dave before it can proceed. No AI agent — regardless of
capability or Council membership — may unilaterally exercise this authority.

## Rationale

Distributed governance without a human anchor creates conditions where a coordinated
multi-agent coalition could override safety constraints. INV-5 ensures there is always
one non-agent decision-maker with veto power.

## Operations Requiring INV-5 Approval

| Operation | Reason |
|-----------|--------|
| `SetVisibility` (repo → public) | Irreversible disclosure risk |
| `shred_secret` | Force-push to rewrite history |
| Ratifying a new invariant (e.g., INV-37) | Changes the constitutional contract |
| Suspending any Critical invariant | Removes a safety constraint |
| Releasing Class A data | Bypasses SHRED default |

## Implementation

| Layer | Implementation |
|-------|----------------|
| `CouncilGitHubClient` | `requires_dave_approval()` — blocks listed operations without token |
| `CouncilGitHubClient` | `execute_with_approval(token)` — requires non-empty approval token |
| `shred_secret()` | Requires `dave_approval` string; empty string → `RequiresConstitutionalAuthority` |
| `ZeroTrustGate` (council gate) | `CouncilApproval.approver` must be a named authority |
| INV-37 ratification | Blocked until Dave explicitly ratifies in session |

## Approval Token Format

Tokens are session-scoped strings issued by Dave during a live session, e.g.:

```
dave-approved-2026-03-20
```

Tokens are redacted in all audit records (`[REDACTED]`). The audit records the actor name
and session ID but never the raw token.

## Test Vectors

| Scenario | Expected Outcome |
|----------|-----------------|
| `shred_secret` with empty token | `RequiresConstitutionalAuthority` error |
| `SetVisibility` via `execute()` | Blocked — `requires_dave_approval()` returns true |
| `ZeroTrustGate` council gate with empty approver | `CouncilGateFailed` |
| Valid approval token, named approver | Council gate passes |

## Constitutional Relations

- **Protects:** INV-1 (User Sovereignty) — user is the human at the top
- **Required by:** INV-37 (Agent Individuality, proposed) — ratification is a Dave decision
- **Audited by:** INV-3 — all authority exercises are recorded

## Status

`IMPLEMENTED` — `CouncilGitHubClient.requires_dave_approval()`,
`execute_with_approval()`, `shred_secret()`, `ZeroTrustGate` council gate.
