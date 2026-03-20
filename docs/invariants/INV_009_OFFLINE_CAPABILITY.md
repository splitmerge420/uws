# INV-009 — Offline Capability
**Category:** Engineering | **Severity:** Mandatory | **Check Type:** Advisory

> "Critical operations must function without network connectivity."

---

## Statement

The following operations must complete successfully when the device has no network
connection:

- Reading previously cached data
- Running invariant checks (linter, healer)
- Verifying audit chain integrity
- Running the stress test engine
- Accessing the OPA policy engine (policies are local files)

Network-required operations (GitHub API, cloud provider calls) must fail gracefully with
a clear offline error rather than hanging or crashing.

## Rationale

Dave operates across ChromeOS, iOS, and environments with intermittent connectivity.
A system that fails silently when offline violates INV-24 (Graceful Degradation) and
INV-1 (User Sovereignty — the user cannot make decisions about data they cannot access).

## Implementation

| Layer | Implementation |
|-------|----------------|
| Discovery cache | `src/discovery.rs` — 24-hour TTL cache of API discovery documents |
| Audit chain | `audit_chain.rs` — fully in-memory, no network required |
| OPA policies | `toolchain/policies/*.rego` — local files, evaluated by `opa_rego_engine.py` |
| Invariant linter | `invariant_linter.py` — file-system scan only |
| Kintsugi healer | `kintsugi_healer.py` — local analysis, network optional |
| `GwsError` | Returns typed offline error; never hangs on timeout |

## Graceful Offline Behaviour

```
Network required operation → OfflineError { operation, requires_network: true }
Cached data available      → Return cached data + staleness warning
No cache, no network       → Return empty result + clear error message
```

## Constitutional Relations

- **Depends on:** INV-8 (Cross-Platform) — offline must work on all platforms
- **Requires:** INV-24 (Graceful Degradation) — offline is a failure mode, must degrade gracefully
- **Enables:** INV-1 (User Sovereignty) — user can access their data offline

## Status

`ADVISORY` — architectural principle. Key offline paths (audit chain, policies, linter)
are implemented. Network-gated operations return typed errors.
