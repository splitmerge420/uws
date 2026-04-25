# Aluminum / UWS Module Registry

Status: Phase 1 skeleton registry
Source of truth: `docs/architecture/PHASE_1_BASELINE.md` + `docs/architecture/REPO_ALIGNMENT_MAP.md`

## Purpose

This registry gives agents and humans a shared map of the module system. It is intentionally allowed to be incomplete at the implementation level while complete at the architectural level.

Phase 1 means every major module family has a named home. Phase 1.5 means synthesis, wiring, and enforcement after the homes exist.

---

## Status Legend

- **planned** — named but not yet scaffolded.
- **doc-skeleton** — documented home exists.
- **code-skeleton** — compiling code home exists, no live behavior.
- **prototype** — partial behavior exists, not production-ready.
- **canonical** — stable owner for that layer.
- **needs-harvest** — source material exists elsewhere and must be reconciled.

---

## Core Execution Modules

| Module | Name | Owner Repo | Status | Notes |
|---|---|---|---|---|
| Core-0 | Phase 1 Architecture Baseline | `uws` | canonical | Defines skeleton vs synthesis model. |
| Core-1 | Provider Dispatch | `uws` | prototype | PR #20 lineage. |
| Core-2 | Swarm / GoldenTrace / ABF | `uws` | prototype | PR #21 lineage. |
| Core-3 | Zero Trust / INV Gate | `uws` | prototype | PR #22 lineage. |
| Core-4 | Kintsugi Weave CI | `uws` | prototype | PR #24 lineage. |

---

## Governance / Constitutional Modules

| Module | Name | Owner Repo | Status | Notes |
|---|---|---|---|---|
| G-1 | House 12 Governance Runtime Skeleton | `uws` | code-skeleton | `src/governance/`. |
| G-2 | ConsentKernel | `uws` | planned | Future consent enforcement. |
| G-3 | PreFlightGate | `uws` | code-skeleton | `src/governance/preflight.rs`. |
| G-4 | PriorityEngine | `uws` | code-skeleton | `src/governance/priority.rs`. |
| G-5 | MetabolicImpact | `uws` | code-skeleton | `src/governance/impact.rs`. |
| G-6 | Cross-Sphere Accountability / INV-13 | `atlas-lattice-foundation` + `uws` | doc-skeleton | ADR-145 extraction target. |

---

## Interoperability Modules

| Module | Name | Owner Repo | Status | Notes |
|---|---|---|---|---|
| 16A | AI Model Provider Interoperability | `uws` | code-skeleton | OpenAI, Anthropic, Gemini, DeepSeek. |
| 16B | Cloud / Hyperscaler Interoperability | `uws` | code-skeleton | AWS, Azure, GCP. |
| 16C | Productivity / OS Surface Interoperability | `uws` | planned | M365, Google Workspace, Apple, Android/Chrome, GitHub, Notion. |
| 16D | Protocol / Agent Interoperability | `uws` | planned | MCP, A2A, GitHub loops, Copilot, Manus, Notion AI. |

---

## Provenance / Economics Modules

| Module | Name | Owner Repo | Status | Notes |
|---|---|---|---|---|
| 17 | Digital Dividend / Marketplace Kernel | `uws` + private domain repos | planned | INV-17; skeleton only until legal/safety review. |
| 18 | FinancialContextKernel | `uws` | doc-skeleton | `docs/architecture/modules/MODULE_18_FCK.md`. |
| IP-1 | Regenerative IP / Provenance Engine | `uws` | prototype | PR #25/#29 lineage. |
| IP-2 | Royalty Oracle | `uws` | prototype | PR #18 lineage. |

---

## Domain Module Sources

These modules may draw from private repos, but should not expose private content without review.

| Domain | Candidate Source Repos | Public Target |
|---|---|---|
| Healthcare | `healthcare-ai`, `mental-health-ai`, `pharmaceutical-ai` | `aluminum-os` roadmap + `uws` stubs |
| Finance | `banking-revolution`, `fintech-disruption`, `insurance-ai-bias`, `credit-scoring-injustice` | Module 18 / Module 17 |
| Energy / Water / Climate | `energy-grid-ai`, `water-management-ai`, `climate-ai-solutions` | ADR / MetabolicLayer |
| Agriculture | `agricultural-ai`, `food-delivery-exploitation`, `waste-management-ai` | Symbiotic infrastructure docs |
| Labor / Rights | `ai-labor-exploitation`, `gig-economy-exploitation`, `digital-labor-rights` | Foundation doctrine / NPFM |
| Governance / Safety | `ai-governance-framework`, `ai-safety-research`, `ai-risk-assessment` | Foundation + Zero Trust |

---

## Public / Private Boundary Rule

Private repos are source material, not automatic public inputs. Before extraction into public repos, agents must record:

- source repo;
- source path;
- sensitivity level;
- target repo;
- target module;
- reason for extraction;
- whether claims require verification.

---

## Next Registry Tasks

- [ ] Add Module 17 Digital Dividend skeleton doc.
- [ ] Add workspace provider skeletons.
- [ ] Add protocol provider skeletons.
- [ ] Add `src/ledger/` skeleton.
- [ ] Add `src/financial_context/` skeleton.
- [ ] Add ADR-145 extraction docs for INV-13 / INV-17 / Indiana Pattern.
- [ ] Reconcile Claude module naming with this registry.
- [ ] Reconcile Microsoft 365 Copilot Phase 3 artifacts with this registry.
