# INV-001 — User Sovereignty
**Category:** Governance | **Severity:** Critical | **Check Type:** Advisory

> "The user is the ultimate authority. No AI agent may override explicit user decisions."

---

## Statement

Every AI agent, automated process, and infrastructure component in Aluminum OS must treat
the user's explicit decisions as final. An agent may advise, warn, or recommend — but it
may never silently override, ignore, or route around a user decision.

## Rationale

Without this invariant, every other protection can be bypassed: an AI that routes around
user intent converts a tool into an adversary. INV-1 is the load-bearing pillar of the
entire constitutional framework.

## Scope

Applies to all agents (Claude, Copilot, Grok, GPT, Gemini, Manus, DeepSeek, Alexa) and
all automated pipelines touching user data, preferences, or consent records.

## Implementation

| Layer | Implementation |
|-------|----------------|
| Architectural | No write operation proceeds without tracing back to a user-initiated consent signal |
| `ConstitutionalEngine` | `check_user_sovereignty()` — always passes (structural principle, not a runtime gate) |
| `CouncilGitHubClient` | All operations are actor-attributed; no anonymous writes |
| `ZeroTrustGate` | `session_consent=True` required; consent is checked before every gate |
| Rego | `consent_enforcement.rego` — `default allow = false` |

## Test Vectors

| Scenario | Expected Outcome |
|----------|-----------------|
| Agent attempts write without tracing to user consent | BLOCK — INV-2 fires |
| User explicitly cancels an in-flight operation | Operation must abort within one polling cycle |
| Agent overrides a user-set privacy flag | BLOCK — audit entry `DENY` recorded |

## Constitutional Relations

- **Strengthened by:** INV-2 (Consent Gating), INV-5 (Constitutional Authority)
- **Relies on:** INV-3 (Audit Trail) to prove the user decision was recorded
- **Violated by:** any operation not traceable to `user_consent = true`

## Status

`IMPLEMENTED` — architectural principle enforced by all three governance layers
(ConstitutionalEngine, CouncilGitHubClient, ZeroTrustGate).
