# INV-024 — Graceful Degradation
**Category:** Engineering | **Severity:** Mandatory | **Check Type:** Advisory

> "Systems must degrade gracefully under failure rather than crash entirely."

---

## Statement

When a component, provider, or subsystem fails, Aluminum OS must:

1. Detect the failure with a typed error (INV-21)
2. Activate a fallback path where one exists (INV-7)
3. Return a meaningful response to the user (not a raw stack trace)
4. Record the degradation event in the audit chain (INV-3)
5. Continue serving non-failing components unaffected

A system that crashes entirely because one provider is down has violated INV-24.

## Degradation Levels

| Level | Description | Response |
|-------|-------------|---------|
| L1 — Provider failover | Primary provider down | Switch to fallback (INV-7); no user-visible impact |
| L2 — Service degraded | Fallback provider slow | Warn user; continue with reduced capability |
| L3 — Feature unavailable | Both providers down | Return typed `FeatureUnavailableError`; other features unaffected |
| L4 — Offline mode | No network | Return cached data; log staleness warning (INV-9) |
| L5 — Audit chain failure | Disk full / corruption | BLOCK ALL WRITES; alert immediately (INV-35 Fail-Closed) |

## GoldenTrace Fallback Pattern

The "Kintsugi-like" approach used throughout this codebase: when the primary
implementation is unavailable, fall back to a structurally correct alternative with
a clear comment explaining the trade-off.

```rust
// audit_chain.rs — GoldenTrace fallback
#[cfg(feature = "sha3")]
fn portable_sha3_256(input: &str) -> String {
    // Real SHA3-256 — production path
    let mut hasher = Sha3_256::new(); ...
}

/// GoldenTrace fallback — FNV-1a, NOT cryptographically secure.
/// Used only when sha3 crate is unavailable (no_std / embedded).
#[cfg(not(feature = "sha3"))]
fn portable_sha3_256(input: &str) -> String {
    // FNV-1a simulation — structural correctness preserved
}
```

## Implementation

| Layer | Implementation |
|-------|----------------|
| `src/error.rs` | `GwsError` variants encode degradation level |
| `ZeroTrustGate` | Each gate is independent; one gate failure doesn't skip others |
| `AuditChain` | Never fails silently — `ChainError` if disk/memory issue |
| `ConstitutionalEngine` | `strict_mode = false` for non-production environments |
| `pqc_provider.py` | GoldenTrace fallback from ML-DSA to HMAC-SHA3-256 |

## Constitutional Relations

- **Depends on:** INV-21 (Error Boundaries) — graceful degradation requires typed errors
- **Depends on:** INV-7 (Vendor Balance) — fallback providers enable Level 1 degradation
- **Requires:** INV-9 (Offline Capability) — Level 4 degradation
- **Constrained by:** INV-35 (Fail-Closed) — Level 5 blocks writes; never silently allows

## Status

`IMPLEMENTED` — GoldenTrace fallback pattern in `audit_chain.rs`, `pqc_provider.py`;
typed error hierarchy in `error.rs`, `council_github_client.rs`, `zero_trust_registry`;
`ConstitutionalEngine` strict/non-strict modes.
