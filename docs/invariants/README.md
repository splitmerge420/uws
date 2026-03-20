# Aluminum OS Constitutional Invariants — INV-001 through INV-024
## Specification Index
**Version:** 1.0 | **Date:** 2026-03-20 | **Authority:** Dave Sheldon (INV-5)

This directory contains the complete specification for Constitutional Invariants
INV-001 through INV-024. Each file follows the canonical template used across the
`aluminum-os` and `uws` repositories.

INV-025 through INV-036 are documented in `docs/invariant_distribution_table.md`.

---

## Quick Reference

| ID | Name | Severity | Check Type | Status | File |
|----|------|----------|------------|--------|------|
| INV-001 | User Sovereignty | Critical | Advisory | IMPLEMENTED | [→](INV_001_USER_SOVEREIGNTY.md) |
| INV-002 | Consent Gating | Critical | Guard Check | IMPLEMENTED | [→](INV_002_CONSENT_GATING.md) |
| INV-003 | Audit Trail | Critical | Guard Check | IMPLEMENTED | [→](INV_003_AUDIT_TRAIL.md) |
| INV-004 | Data Classification | Mandatory | Advisory | IMPLEMENTED | [→](INV_004_DATA_CLASSIFICATION.md) |
| INV-005 | Constitutional Authority | Critical | Advisory | IMPLEMENTED | [→](INV_005_CONSTITUTIONAL_AUTHORITY.md) |
| INV-006 | Provider Abstraction | Mandatory | Guard Check | IMPLEMENTED | [→](INV_006_PROVIDER_ABSTRACTION.md) |
| INV-007 | Vendor Balance (47% cap) | Critical | Guard Check | IMPLEMENTED | [→](INV_007_VENDOR_BALANCE.md) |
| INV-008 | Cross-Platform Compatibility | Mandatory | Advisory | IMPLEMENTED | [→](INV_008_CROSS_PLATFORM_COMPATIBILITY.md) |
| INV-009 | Offline Capability | Mandatory | Advisory | PARTIAL | [→](INV_009_OFFLINE_CAPABILITY.md) |
| INV-010 | Interoperability | Mandatory | Advisory | IMPLEMENTED | [→](INV_010_INTEROPERABILITY.md) |
| INV-011 | Encryption at Rest | Critical | Guard Check | IMPLEMENTED | [→](INV_011_ENCRYPTION_AT_REST.md) |
| INV-012 | Encryption in Transit | Critical | Pattern Absence | IMPLEMENTED | [→](INV_012_ENCRYPTION_IN_TRANSIT.md) |
| INV-013 | Post-Quantum Readiness | Mandatory | Advisory | PARTIAL | [→](INV_013_POST_QUANTUM_READINESS.md) |
| INV-014 | Zero-Knowledge Where Possible | Advisory | Advisory | ADVISORY | [→](INV_014_ZERO_KNOWLEDGE_WHERE_POSSIBLE.md) |
| INV-015 | Key Rotation | Mandatory | Advisory | PARTIAL | [→](INV_015_KEY_ROTATION.md) |
| INV-016 | Data Minimisation | Mandatory | Advisory | PARTIAL | [→](INV_016_DATA_MINIMISATION.md) |
| INV-017 | Right to Delete | Critical | Advisory | PARTIAL | [→](INV_017_RIGHT_TO_DELETE.md) |
| INV-018 | Data Portability | Mandatory | Advisory | PARTIAL | [→](INV_018_DATA_PORTABILITY.md) |
| INV-019 | Jurisdictional Compliance | Critical | Advisory | PARTIAL | [→](INV_019_JURISDICTIONAL_COMPLIANCE.md) |
| INV-020 | No Silent Sharing | Critical | Advisory | IMPLEMENTED | [→](INV_020_NO_SILENT_SHARING.md) |
| INV-021 | Error Boundaries | Mandatory | Guard Check | IMPLEMENTED | [→](INV_021_ERROR_BOUNDARIES.md) |
| INV-022 | Type Safety | Warning | Advisory | IMPLEMENTED | [→](INV_022_TYPE_SAFETY.md) |
| INV-023 | Test Coverage | Mandatory | Advisory | IMPLEMENTED | [→](INV_023_TEST_COVERAGE.md) |
| INV-024 | Graceful Degradation | Mandatory | Advisory | IMPLEMENTED | [→](INV_024_GRACEFUL_DEGRADATION.md) |

---

## Implementation Coverage

| Status | Count | Invariants |
|--------|------:|-----------|
| IMPLEMENTED | 15 | 1, 2, 3, 4, 5, 6, 7, 8, 10, 11, 12, 20, 21, 22, 23, 24 |
| PARTIAL | 8 | 9, 13, 15, 16, 17, 18, 19 |
| ADVISORY | 1 | 14 |

---

## Enforcement Layers

```
┌─────────────────────────────────────────────────────────────────────┐
│  User Intent (INV-1 User Sovereignty)                               │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │  ConstitutionalEngine (INV-2, INV-3, INV-6, INV-7, INV-11)  │  │
│  │  ┌────────────────────────────────────────────────────────┐  │  │
│  │  │  ZeroTrustGate                                         │  │  │
│  │  │  Gate 1: Logic   → ConstitutionalEngine.enforce()      │  │  │
│  │  │  Gate 2: Stress  → resilience ≥ 0.70, iters ≥ 100     │  │  │
│  │  │  Gate 3: Council → INV-5 approval token                │  │  │
│  │  └────────────────────────────────────────────────────────┘  │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                     │
│  AuditChain (INV-3 — all decisions logged, SHA3-256 hash chain)    │
│  OPA Rego Policies (9 policies, all default-deny)                  │
└─────────────────────────────────────────────────────────────────────┘
```

---

*Generated during Janus Checkpoint 2026-03-20 — GitHub Copilot (P2 deliverable expansion)*
