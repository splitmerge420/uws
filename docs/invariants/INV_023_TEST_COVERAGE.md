# INV-023 — Test Coverage
**Category:** Engineering | **Severity:** Mandatory | **Check Type:** Advisory

> "All critical paths must have unit tests. Target 80%+ coverage on governance code."

---

## Statement

Every critical code path — consent gating, audit chain operations, invariant checks,
zero trust gates — must have dedicated unit tests. Test coverage for governance modules
must meet or exceed 80%. Tests must be deterministic, fast, and independent of external
services.

## Coverage Targets

| Module | Target | Current (approx.) |
|--------|--------|-------------------|
| `constitutional_engine.rs` | 90% | ~95% (9 tests) |
| `audit_chain.rs` | 90% | ~90% (10 tests) |
| `council_github_client.rs` | 80% | ~85% (11 tests) |
| `zero_trust_registry.rs` | 90% | ~95% (20 tests) |
| `toolchain/zero_trust_registry.py` | 80% | ~90% (18 tests) |
| `toolchain/acp_governance.py` | 80% | ~80% (4 tests) |
| `toolchain/invariants_registry.py` | 80% | ~90% (3 tests) |

## Test Categories

| Category | Examples |
|----------|---------|
| Happy path | Valid consent, fallback, approval → integration succeeds |
| Gate rejection | No fallback, low resilience, empty token → typed error |
| Stress tests | 100–1000 iterations of enforcement rules |
| Audit integrity | Chain intact after N operations |
| Heartbeat | 60-tick simulation with all invariant IDs present |

## Current Test Count

```
Rust library:   50 tests (cargo test --lib)
Python:         46 tests (pytest tests/test_integration_governance.py)
Total:          96 tests
```

## Test Determinism Requirements

- No calls to external APIs in unit tests
- No random seeds without fixed values in tests
- No `time.sleep()` or `asyncio.sleep()` in unit tests
- All test fixtures defined in the test file itself

## Constitutional Relations

- **Required by:** ZeroTrustGate — stress evidence requires `iterations ≥ 100`
- **Strengthens:** INV-36 (Technical Invariant Enforcement) — tests are the enforcement mechanism
- **Supported by:** INV-22 (Type Safety) — typed interfaces are easier to test

## Status

`IMPLEMENTED` — 96 tests passing across Rust and Python. All constitutional invariant
checks have dedicated test vectors. Stress tests validate INV-7 × 1000 and INV-2 × 200.
