# INV-004 — Zero Trust

| Field | Value |
|-------|-------|
| **ID** | INV-004 |
| **Slug** | zero-trust |
| **Title** | Zero Trust Integration Gate |
| **Severity** | Mandatory |
| **Status** | ✅ Specified |
| **Source** | `src/zero_trust.rs`, `src/main.rs` |
| **Test fixture** | `src/zero_trust.rs` → `test_synthetic_ms_auth_login_blocked_by_deny_all_gate` (and 5 others) |

---

## Rationale

The traditional security model — "trusted inside the perimeter, untrusted
outside" — fails in a world where every CLI invocation can be driven by
an AI agent with compromised instructions.  Zero Trust assumes no call is
safe until verified.

Every external integration in uws (provider auth, network egress, file write
outside the workspace) is a potential exfiltration vector.  The Zero Trust
Integration Gate is a mandatory pre-flight check that runs **before** any
such integration executes.

This is the uws equivalent of the Kintsugi Gate in `bazinga/src/main.rs`.

## Invariant Statement

> **Every external integration call (provider auth, network egress, file
> write outside workspace) MUST pass through `ZeroTrustGate::check()`
> before executing. On gate failure, execution is halted and an audit
> log entry is written.**

## Enforcement Mechanism

`src/zero_trust.rs` — `ZeroTrustGate`:

```rust
pub enum IntegrationKind {
    ProviderAuth,   // OAuth2 / API-key authentication flows
    NetworkEgress,  // Outbound HTTP requests to external APIs
    FileWrite,      // File writes outside the workspace
}

pub fn check(&self, ctx: &GateContext<'_>) -> GateDecision {
    // evaluates GatePolicy, writes audit log on block
}
```

### Wired call sites (as of INV-004 v0.1)

| Call site | Kind | File | Status |
|-----------|------|------|--------|
| `uws ms-auth login` | `ProviderAuth` | `src/main.rs` | ✅ Wired |

### TODO: remaining call sites

| Call site | Kind | File |
|-----------|------|------|
| `uws auth login` | `ProviderAuth` | `src/auth_commands.rs` |
| `uws apple-auth` | `ProviderAuth` | `src/apple.rs` |
| `uws ms-auth exchange` | `NetworkEgress` | `src/ms_graph.rs` |
| `executor::execute_method` (pre-request) | `NetworkEgress` | `src/executor.rs` |
| `--output-dir` write | `FileWrite` | `src/validate.rs` |
| `--upload` path | `FileWrite` | `src/executor.rs` |

## Environment Control

| Variable | Effect |
|----------|--------|
| `UWS_ZERO_TRUST_MODE=deny` | `GatePolicy::deny_all()` — blocks all integrations |
| `UWS_ZERO_TRUST_AUDIT_LOG=<path>` | Appends blocked-operation log lines to `<path>` |
| *(not set)* | `GatePolicy::permissive()` — all integrations allowed |

## Test Fixtures

```rust
// src/zero_trust.rs
#[test]
fn test_synthetic_ms_auth_login_blocked_by_deny_all_gate() {
    let gate = ZeroTrustGate::new(GatePolicy::deny_all());
    let ctx = GateContext {
        kind: IntegrationKind::ProviderAuth,
        resource: "microsoft/oauth2",
        actor: "uws-cli",
    };
    assert!(matches!(gate.check(&ctx), GateDecision::Block(_)));
}
```

Running `UWS_ZERO_TRUST_MODE=deny uws ms-auth login` will exit with code 1
and print a structured JSON error, demonstrating the gate blocking in practice.

## Followups

- [ ] Wire gate at remaining call sites listed above.
- [ ] Load `GatePolicy` from a config file (not just env vars) for persistent lockdown.
- [ ] Add `uws gate status` CLI command to inspect current policy.
- [ ] Integrate `ZeroTrustGate::check()` decision into `AuditChain` (currently uses flat log).
