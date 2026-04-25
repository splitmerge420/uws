# INV-001 — User Sovereignty

| Field | Value |
|-------|-------|
| **ID** | INV-001 |
| **Slug** | sovereignty |
| **Title** | User Sovereignty |
| **Severity** | Critical |
| **Status** | ✅ Specified |
| **Source** | `src/constitutional_engine.rs`, `src/council_github_client.rs`, `src/lib.rs` |
| **Test fixture** | `src/constitutional_engine.rs` → `test_consent_gating_*` |

---

## Rationale

The user is the ultimate authority over their own data and integrations.
No operation — including AI agent actions — may override an explicit user
refusal or operate outside the scope the user has consented to.

User Sovereignty is an **architectural principle**, not just a policy setting.
It means the system is built so that violations are structurally impossible,
not merely discouraged.

## Invariant Statement

> **Every action taken by uws or any agent acting through uws MUST be
> explicitly authorized by the user, either directly or through a
> standing consent that the user can revoke at any time.**

## Enforcement Mechanism

- `ConstitutionalEngine::check_user_sovereignty()` — evaluates `StateSnapshot.user_consent`.
- `CouncilGitHubClient` — blocks destructive operations (`push --force`, `delete repo`) unconditionally, reflecting the user's standing refusal.
- `ZeroTrustGate` — acts as a pre-flight sovereignty check: if the user has set `UWS_ZERO_TRUST_MODE=deny`, all external integrations are blocked.

## Violation Handling

**Severity: Critical** — violation immediately halts execution with a structured JSON error:

```json
{
  "error": {
    "code": 403,
    "reason": "sovereigntyViolation",
    "message": "Operation blocked: user sovereignty invariant (INV-001)"
  }
}
```

An audit log entry is appended with `decision: BLOCKED`.

## Followups

- [ ] Add `INV-001` tag to all `CouncilGitHubClient` block decisions in audit log entries.
- [ ] Surface sovereignty check result in `uws auth status` output.
