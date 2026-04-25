# INV-005 — Fail-Closed

| Field | Value |
|-------|-------|
| **ID** | INV-005 |
| **Slug** | fail-closed |
| **Title** | Fail-Closed |
| **Severity** | Critical |
| **Status** | ✅ Specified |
| **Source** | `src/zero_trust.rs`, `src/audit_chain.rs`, `src/council_github_client.rs`, `src/lib.rs` |
| **Test fixture** | `src/zero_trust.rs` → `test_deny_all_gate_blocks_all_kinds`, `src/audit_chain.rs` → `test_detect_tampering` |

---

## Rationale

When a security or governance system encounters an error — a missing
policy, a corrupt config, a network failure — the **safe default is to
block**, not to allow.  "Fail-open" systems degrade gracefully from a UX
perspective but catastrophically from a security perspective.

The Aluminum OS constitutional model is fail-closed at every layer:

- **Zero Trust Gate**: `GatePolicy::deny_all()` is available as an
  explicit lockdown mode.  When `UWS_ZERO_TRUST_MODE=deny`, the gate
  blocks everything rather than failing open.
- **Audit Chain**: `verify_chain()` returns `Err` (not `Ok`) when it
  cannot verify integrity, causing callers to halt rather than proceed
  on a potentially tampered log.
- **Council GitHub Client**: destructive operations (`force-push`,
  `repo-delete`, `branch-delete`) are unconditionally blocked regardless
  of policy settings.

## Invariant Statement

> **When the governance layer cannot determine whether an operation is
> safe, it MUST block the operation (fail-closed), not allow it
> (fail-open). Default policies MUST be the most restrictive option
> available.**

## Enforcement Mechanism

### Zero Trust Gate

`GatePolicy::deny_all()` blocks all `IntegrationKind` variants.  The gate
also blocks by default when `policy.allow_*` fields are `false` — these
default to `false` in `GatePolicy::deny_all()`, not `true`.

The binary reads `UWS_ZERO_TRUST_MODE=deny` to engage lockdown:

```rust
fn gate_policy_from_env() -> GatePolicy {
    match std::env::var("UWS_ZERO_TRUST_MODE").as_deref().unwrap_or("") {
        "deny" => GatePolicy::deny_all(),  // ← fail-closed
        _ => GatePolicy::permissive(),
    }
}
```

### Audit Chain

`AuditChain::verify_chain()` returns `Err(ChainError::IntegrityViolation)`
on hash mismatch — callers that do not handle this error will propagate
the failure upward rather than silently continuing with a potentially
tampered log.

### Council GitHub Client

All destructive GitHub operations are unconditionally blocked by
`CouncilGitHubClient`, regardless of the caller's intent.  There is no
override flag or escape hatch.

## Test Fixtures

```rust
// src/zero_trust.rs
#[test]
fn test_deny_all_gate_blocks_all_kinds() {
    let gate = ZeroTrustGate::new(GatePolicy::deny_all());
    // ProviderAuth, NetworkEgress, FileWrite — all blocked
    assert!(matches!(gate.check(&ctx), GateDecision::Block(_)));
}

// src/audit_chain.rs
#[test]
fn test_detect_tampering() {
    // Modified entry causes verify_chain() to return Err
}
```

## Followups

- [ ] Add `GatePolicy::from_env_strict()` that fails closed on any parse error
  (currently falls back to permissive on unknown env var values).
- [ ] Surface `audit_chain::verify_chain()` result in `uws audit status` command.
- [ ] Add `#[deny(clippy::unwrap_used)]` to governance modules to enforce error propagation.
