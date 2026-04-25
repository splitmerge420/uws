# Invariant Specifications — INV-001 through INV-024

This directory contains the canonical specification documents for the first 24
Aluminum OS Constitutional Invariants.  Each file follows a consistent template:

```
# INV-NNN: <Title>
## Statement   — the invariant as a single declarative sentence
## Rationale   — why this invariant exists
## Implementation — file/symbol table mapping the invariant to code
## Dangerous Patterns — code patterns that violate the invariant
## Guard Patterns — code patterns that satisfy the invariant
## Test Vectors — pass/fail scenarios for automated checking
## Constitutional Relations — depends-on / required-by / conflicts-with
```

---

## Index

| ID | Name | Severity | Check type |
|---|---|---|---|
| [INV-001](INV-001.md) | User Sovereignty | critical | advisory |
| [INV-002](INV-002.md) | Consent Gating | critical | guard_check |
| [INV-003](INV-003.md) | Audit Trail | critical | guard_check |
| [INV-004](INV-004.md) | Data Classification | mandatory | advisory |
| [INV-005](INV-005.md) | Constitutional Authority | critical | advisory |
| [INV-006](INV-006.md) | Provider Abstraction | mandatory | guard_check |
| [INV-007](INV-007.md) | Vendor Balance | critical | guard_check |
| [INV-008](INV-008.md) | Cross-Platform Compatibility | mandatory | advisory |
| [INV-009](INV-009.md) | Offline Capability | mandatory | advisory |
| [INV-010](INV-010.md) | Interoperability | mandatory | advisory |
| [INV-011](INV-011.md) | Encryption at Rest | critical | pattern_absence |
| [INV-012](INV-012.md) | Encryption in Transit | critical | pattern_absence |
| [INV-013](INV-013.md) | Post-Quantum Readiness | mandatory | advisory |
| [INV-014](INV-014.md) | Zero-Knowledge Where Possible | advisory | advisory |
| [INV-015](INV-015.md) | Key Rotation | mandatory | advisory |
| [INV-016](INV-016.md) | Data Minimization | mandatory | advisory |
| [INV-017](INV-017.md) | Right to Delete | critical | advisory |
| [INV-018](INV-018.md) | Data Portability | mandatory | advisory |
| [INV-019](INV-019.md) | Jurisdictional Compliance | critical | advisory |
| [INV-020](INV-020.md) | No Silent Sharing | critical | advisory |
| [INV-021](INV-021.md) | Error Boundaries | mandatory | guard_check |
| [INV-022](INV-022.md) | Type Safety | warning | advisory |
| [INV-023](INV-023.md) | Test Coverage | mandatory | advisory |
| [INV-024](INV-024.md) | Graceful Degradation | mandatory | advisory |

---

## Relationship to `toolchain/invariants_registry.py`

The Python registry (`toolchain/invariants_registry.py`) is the **machine-readable**
single source of truth used by the invariant linter, healer, pipeline, and stress test.
These spec documents are the **human-readable** companion that explains the rationale
and provides implementation guidance.

If they diverge, the registry takes precedence for automated checks; the spec documents
take precedence for human intent.

---

## Zero Trust Integration Gate

The Zero Trust gate (`toolchain/zero_trust_gate.py`) enforces that every new external
integration (provider, MCP server, API client) declares:

1. **Scopes / permissions** — the minimum set of OAuth scopes or API permissions required
2. **Data-handling class** — `CLASS_A`, `CLASS_B`, or `CLASS_C` (see INV-004)
3. **Constitutional invariants** — which of the above invariants the integration depends on

Integration manifests live in the `integrations/` directory.  See
[`integrations/README.md`](../../integrations/README.md) for the manifest format.
