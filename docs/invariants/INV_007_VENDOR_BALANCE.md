# INV-007 — Vendor Balance (47% Dominance Cap)
**Category:** Architecture | **Severity:** Critical | **Check Type:** Guard Check

> "No single-vendor dependency. Every external API call must have a fallback provider."

---

## Statement

No single vendor may account for more than 47% of active provider load. Every external
API integration must declare at least one fallback provider. Operations that lack a
fallback provider are blocked at the logic gate of the Zero Trust registry.

## The 47% Cap

The 47% dominance threshold is derived from the 144-Sphere routing model: no single
provider may hold majority routing. At 47% a single provider has significant influence
but cannot unilaterally determine system behaviour without at least two other providers
failing simultaneously.

## Dangerous Patterns (Linter Detects)

```python
openai.ChatCompletion.create(...)   # Direct single-vendor call
anthropic.Anthropic().messages.create(...)
google.genai.GenerativeModel(...)
xai.Grok(...)
```

## Guard Patterns (Must Be Present)

`fallback`, `alternative`, `vendor_balance`, `multi_provider`,
`PROVIDERS =`, `provider_chain`

## Implementation

| Layer | Implementation |
|-------|----------------|
| `ConstitutionalEngine` | `check_vendor_balance()` — fails when `has_fallback = false` |
| `ZeroTrustGate` | `run_logic_gate(has_fallback=...)` — hard gate; rejects without fallback |
| `vendor_balance.rego` | `default allow = false`; checks `has_fallback` and `provider_count` |
| Stress tests | `stress_test_inv7_vendor_balance_enforcement` × 1000 iterations |

## Stress Test Results

```
test stress_test_inv7_vendor_balance_enforcement ... ok  (1000 iterations)
test stress_test_inv7_vendor_balance_with_fallback ... ok (1000 iterations)
```

Both tests run in the CI suite under `cargo test --lib`.

## Provider Diversity Requirements

| Providers Configured | Status |
|----------------------|--------|
| 0 | BLOCKED |
| 1 (no fallback) | BLOCKED |
| 2+ with primary + fallback | ALLOWED |

## Test Vectors

| Scenario | Expected Outcome |
|----------|-----------------|
| `has_fallback = false`, strict mode | `ConstitutionalEngine` → `Err(violations)` |
| `has_fallback = true`, strict mode | `Ok(checks)` |
| Logic gate, `has_fallback = false` | `LogicGateFailed` (Python) / `GateError::LogicGateFailed` (Rust) |
| 1000 × no fallback | All 1000 blocked, 0 pass-throughs |

## Constitutional Relations

- **Depends on:** INV-6 (Provider Abstraction) — abstraction makes fallback routing possible
- **Prevents:** single-provider outage from taking down the system
- **Audited by:** INV-3 — all provider-selection decisions are logged

## Status

`IMPLEMENTED` — `ConstitutionalEngine.check_vendor_balance()`,
`ZeroTrustGate.run_logic_gate()`, `vendor_balance.rego`,
stress-tested × 1000 in `constitutional_engine.rs`.
