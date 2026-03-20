# Invariant Distribution Table — 12-House Mapping
## Aluminum OS Constitutional Framework
**Version:** 1.0 | **Date:** 2026-03-20 | **P2 Deliverable**

This table maps each of the 36 Aluminum Constitutional Invariants to its
primary House in the 144-Sphere Ontology. Each House governs a domain of
human knowledge and technological practice; invariants are assigned to the
House that most directly corresponds to their enforcement domain.

> **INV-7 note:** The 47% dominance cap means no single vendor may supply
> more than 47% of active provider load. This is checked by `check_vendor_balance()`
> in `constitutional_engine.rs` and the `vendor_balance.rego` policy.

---

## Distribution Summary

| House | Domain | Invariant Count |
|------:|--------|:---------------:|
| H01 | Formal Sciences | 3 |
| H02 | Natural Sciences | 1 |
| H03 | Engineering & Technology | 5 |
| H04 | Computing & AI | 5 |
| H05 | Social Sciences | 2 |
| H06 | Medicine & Health | 4 |
| H07 | Law & Governance | 5 |
| H08 | Philosophy & Ethics | 3 |
| H09 | Economics & Finance | 2 |
| H10 | Arts & Communication | 1 |
| H11 | Education & Cognition | 2 |
| H12 | Integration & Synthesis | 6 |
| **Total** | | **39** |

---

## House-by-House Breakdown

### H01 — Formal Sciences
*Proof theory, type systems, formal verification*

| ID | Name | Severity | Check Type | Enforcement Location |
|----|------|----------|------------|----------------------|
| INV-22 | Type Safety | warning | advisory | Compiler / linter |
| INV-23 | Test Coverage | mandatory | advisory | CI/CD pipeline |
| INV-36 | Technical Invariant Enforcement | mandatory | advisory | `invariant_linter.py` |

---

### H02 — Natural Sciences
*Physical constraints, real-world limits*

| ID | Name | Severity | Check Type | Enforcement Location |
|----|------|----------|------------|----------------------|
| INV-8 | Cross-Platform Compatibility | mandatory | advisory | CI matrix |

---

### H03 — Engineering & Technology
*Systems architecture, encryption, resilience*

| ID | Name | Severity | Check Type | Enforcement Location |
|----|------|----------|------------|----------------------|
| INV-11 | Encryption at Rest | critical | guard_check | `constitutional_engine.rs` |
| INV-13 | Post-Quantum Readiness | mandatory | advisory | `pqc_provider.py` |
| INV-15 | Key Rotation | mandatory | advisory | `credential_store.rs` |
| INV-24 | Graceful Degradation | mandatory | advisory | GoldenTrace fallback logic |
| INV-25 | Observability | mandatory | advisory | `audit_chain.rs` |

---

### H04 — Computing & AI
*AI governance, multi-provider safety, vendor balance*

| ID | Name | Severity | Check Type | Enforcement Location |
|----|------|----------|------------|----------------------|
| INV-6 | Provider Abstraction | mandatory | guard_check | `constitutional_engine.rs` |
| INV-7 | Vendor Balance (47% cap) | critical | guard_check | `constitutional_engine.rs`, `vendor_balance.rego` |
| INV-9 | Offline Capability | mandatory | advisory | Feature flags |
| INV-10 | Interoperability | mandatory | advisory | `services.rs` |
| INV-14 | Zero-Knowledge Where Possible | advisory | advisory | `credential_store.rs` |

---

### H05 — Social Sciences
*User behavior, social contracts*

| ID | Name | Severity | Check Type | Enforcement Location |
|----|------|----------|------------|----------------------|
| INV-27 | Session Continuity | mandatory | advisory | `universal_context.rs` |
| INV-28 | Reincarnation Readiness | mandatory | advisory | State serialization |

---

### H06 — Medicine & Health
*Health data, crisis protocols, clinical handoff*

| ID | Name | Severity | Check Type | Enforcement Location |
|----|------|----------|------------|----------------------|
| INV-31 | Crisis Sovereignty | critical | advisory | `health_connectors.py` |
| INV-31a | Crisis Consent Override | critical | advisory | `health_connectors.py` |
| INV-31b | Crisis Data Isolation | critical | advisory | HIPAA partition layer |
| INV-32 | Health-Commerce Separation | critical | advisory | `health_connectors.py` |
| INV-32a | Clinical Handoff Integrity | critical | advisory | Clinical API layer |

> *Note: INV-31a, INV-31b, INV-32a are sub-invariants counted under H06 but are tracked
> as separate entries in the registry.*

---

### H07 — Law & Governance
*Consent, audit, jurisdictional compliance*

| ID | Name | Severity | Check Type | Enforcement Location |
|----|------|----------|------------|----------------------|
| INV-2 | Consent Gating | critical | guard_check | `constitutional_engine.rs`, `consent_enforcement.rego` |
| INV-3 | Audit Trail | critical | guard_check | `audit_chain.rs`, `audit_requirements.rego` |
| INV-19 | Jurisdictional Compliance | critical | advisory | `acp_governance.py` |
| INV-33 | Union-Set Jurisdiction | critical | advisory | `acp_governance.py` |
| INV-34 | Multi-Vantage Jurisdiction Detection | critical | advisory | `acp_governance.py` |

---

### H08 — Philosophy & Ethics
*Sovereignty, agency, ethics*

| ID | Name | Severity | Check Type | Enforcement Location |
|----|------|----------|------------|----------------------|
| INV-1 | User Sovereignty | critical | advisory | Architectural principle |
| INV-5 | Constitutional Authority | critical | advisory | Council veto protocol |
| INV-26 | Noosphere Sovereignty | critical | advisory | `agentic_sovereignty.rs` |

---

### H09 — Economics & Finance
*Privacy economics, data portability rights*

| ID | Name | Severity | Check Type | Enforcement Location |
|----|------|----------|------------|----------------------|
| INV-17 | Right to Delete | critical | guard_check | `acp_governance.py` |
| INV-18 | Data Portability | mandatory | advisory | Export API |

---

### H10 — Arts & Communication
*Message integrity, no silent sharing*

| ID | Name | Severity | Check Type | Enforcement Location |
|----|------|----------|------------|----------------------|
| INV-20 | No Silent Sharing | critical | guard_check | `executor.rs` |

---

### H11 — Education & Cognition
*Healing, self-repair, learning from failure*

| ID | Name | Severity | Check Type | Enforcement Location |
|----|------|----------|------------|----------------------|
| INV-29 | Kintsugi Healing | warning | advisory | `kintsugi_healer.py` |
| INV-30 | Agent Individuality | advisory | advisory | `agentic_sovereignty.rs` |

---

### H12 — Integration & Synthesis
*Cross-cutting invariants, data classification, PQC, fail-closed*

| ID | Name | Severity | Check Type | Enforcement Location |
|----|------|----------|------------|----------------------|
| INV-4 | Data Classification | mandatory | advisory | `data_classification.rego` |
| INV-12 | Audit Immutability | critical | guard_check | `audit_chain.rs` |
| INV-16 | Data Minimization | mandatory | advisory | `executor.rs` |
| INV-21 | Rate Limiting | mandatory | guard_check | `client.rs` |
| INV-35 | Fail-Closed | critical | guard_check | `error.rs`, `fail_closed.rego` |
| INV-37 | Agent Individuality *(proposed)* | advisory | advisory | *Pending ratification* |

---

## Coverage Metrics

| Severity | Total | Implemented (guard_check) | Advisory Only |
|----------|------:|:-------------------------:|:-------------:|
| critical | 17 | 9 | 8 |
| mandatory | 13 | 3 | 10 |
| warning | 2 | 0 | 2 |
| advisory | 4 | 0 | 4 |
| **Total** | **36** | **12** | **24** |

**Rego policy coverage:** 9 of 36 invariants have active OPA policies
(files in `toolchain/policies/`).

---

*Generated by GitHub Copilot (P2 deliverable) — Janus Checkpoint 2026-03-20*
