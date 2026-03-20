# INV-006 — Provider Abstraction
**Category:** Architecture | **Severity:** Mandatory | **Check Type:** Guard Check

> "All cloud/AI provider calls must go through an abstraction layer."

---

## Statement

No module may call a cloud or AI provider SDK directly. All provider interactions must
route through a provider-neutral abstraction layer that can be swapped, mocked, and
monitored without changing calling code.

## Rationale

Direct provider coupling creates fragility: a single API change, outage, or terms-of-
service update can break the entire system. Abstraction enables INV-7 (vendor balance),
testability, and audit-logging of all external calls.

## Dangerous Patterns (Linter Detects)

```python
from openai import ...          # Direct OpenAI import
from anthropic import ...       # Direct Anthropic import
from google import genai        # Direct Gemini import
import openai                   # Direct OpenAI module
```

## Guard Patterns (Must Be Present)

`provider`, `multi_provider`, `fallback`, `ProviderRouter`,
`MultiProviderLLM`, `vendor_balance`

## Implementation

| Layer | Implementation |
|-------|----------------|
| `ConstitutionalEngine` | `check_provider_abstraction()` — fails when `provider_abstracted = false` |
| `ZeroTrustGate` | `run_logic_gate(provider_abstracted=...)` — rejects components without abstraction |
| `provider_abstraction.rego` | `default allow = false` — blocks direct-provider calls |
| Linter | Flags direct imports of known provider SDKs |

## Abstraction Contract

Every provider call must:
1. Accept a `ProviderConfig` or equivalent that can specify primary + fallback
2. Return a typed result (not raw provider response)
3. Log the provider used in the audit trail (INV-3)
4. Respect the 47% dominance cap (INV-7)

## Test Vectors

| Scenario | Expected Outcome |
|----------|-----------------|
| `provider_abstracted = false` in logic gate | Gate rejects component |
| Direct `from openai import` in Python | Linter flags INV-6 violation |
| `ProviderRouter` wrapping OpenAI + Anthropic | Passes INV-6 and INV-7 |

## Constitutional Relations

- **Enables:** INV-7 (Vendor Balance) — abstraction makes swapping possible
- **Depends on:** INV-3 (Audit Trail) — provider calls must be logged
- **Supported by:** `vendor_balance.rego`

## Status

`IMPLEMENTED` — `ConstitutionalEngine.check_provider_abstraction()`,
`ZeroTrustGate.run_logic_gate()`, `provider_abstraction.rego`,
`invariant_linter.py`.
