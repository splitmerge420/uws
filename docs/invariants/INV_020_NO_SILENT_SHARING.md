# INV-020 — No Silent Sharing
**Category:** Privacy | **Severity:** Critical | **Check Type:** Advisory

> "No data may be shared with third parties without explicit, informed, revocable consent."

---

## Statement

Aluminum OS must never transmit user data to a third party without:

1. **Explicit consent** — user actively opted in (not pre-checked boxes)
2. **Informed consent** — user understands exactly what data, with whom, and why
3. **Revocable consent** — user can withdraw consent at any time
4. **Audited consent** — the consent decision is recorded in the audit chain (INV-3)

Silent telemetry, hidden analytics, and background data sharing are unconditionally
prohibited, regardless of commercial justification.

## Sharing Categories

| Category | Requires INV-20 Consent | Notes |
|----------|------------------------|-------|
| AI provider API calls | Yes — per provider, per data type | Logged per call |
| GitHub API | Yes — authorised by user OAuth | Scope-limited token |
| Health data to EHR | Yes — HIPAA BAA required | INV-32 applies |
| Audit logs to council | Yes — user must be notified | Pseudonymised if needed |
| Telemetry / analytics | **Always prohibited** | No opt-in available |

## Silent Sharing Detection

The linter scans for outbound HTTP/HTTPS calls in paths not covered by a provider
abstraction layer (INV-6). Any direct outbound call that isn't wrapped in a `provider`
or `ProviderRouter` is flagged as a potential INV-20 violation.

## Implementation

| Layer | Implementation |
|-------|----------------|
| `src/executor.rs` | All outbound calls are user-initiated; no background calls |
| `src/auth.rs` | OAuth scopes are minimised; no scopes granted silently |
| `acp_governance.py` | `GovernanceContext.sharing_consent` checked before any export |
| INV-2 enforcement | No operation proceeds without consent — sharing included |

## Constitutional Relations

- **Depends on:** INV-2 (Consent Gating) — sharing requires consent
- **Depends on:** INV-3 (Audit Trail) — sharing is audited
- **Strengthened by:** INV-14 (Zero-Knowledge) — ZK eliminates sharing entirely
- **Required by:** INV-19 (Jurisdictional Compliance) — GDPR Article 6, CCPA

## Status

`IMPLEMENTED` — architectural principle enforced by consent-gated executor.
No background calls; all API calls are user-initiated via explicit CLI commands.
