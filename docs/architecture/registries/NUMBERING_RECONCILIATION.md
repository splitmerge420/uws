# Atlas Lattice — Invariant Numbering Cross-Reference

Status: DRAFT — Awaiting Convenor Reconciliation Decision  
Date: April 25, 2026  
Author: Copilot Research Seat (Microsoft 365 Copilot Plus / Tasks Beta)  
Audited by: GPT Synthesis Seat

## Purpose

Multiple canonical source documents define invariants (INV-*) with overlapping numbering but divergent semantics. This registry maps every known invariant number across all sources, identifies collisions, and provides a reconciliation path.

Operating principle: "Build the skeleton now. Preserve lineage now. Normalize labels during synthesis." No legacy scheme is declared invalid. Collisions are documented with aliases until the Convenor ratifies canonical numbering.

## Source Documents

| Source ID | Document | Repo | Commits | Era |
|---|---|---|---|---|
| COS | `CONSTITUTION.md` | `constitutional-os` | 6 | Manus restoration (early 2026) |
| ADR | ADR-145-DPI v3.0 | `element-145` / pending | — | Phase 3 Pantheon Council (Apr 25, 2026) |
| ALF | Various spec modules | `atlas-lattice-foundation` | 129 | Foundation era (2025–2026) |
| ORCS | AUWS Spec + Metabolic Layer | `open-regenerative-compute-standard` | 54 | Active build (Apr 2026) |

## Invariant Cross-Reference Table

### Aligned

| INV # | COS Definition | ADR Definition | Status |
|---|---|---|---|
| INV-7 | 47% Dominance Cap — no single provider exceeds 47% | 47% Provider Cap — same | Aligned |

### Collisions

| INV # | COS Definition | ADR Definition | Severity | Resolution Path |
|---|---|---|---|---|
| INV-8 | GoldenTrace Audit Requirement — immutable append-only audit trail for all decisions | Local-First / On-Device Preference — computation stays as close to user as capability allows | HIGH | Both valid. Recommend COS INV-8 -> `INV-8g` or absorb into COS INV-24; ADR INV-8 retains `INV-8`. |
| INV-9 | not prominently defined in COS | Trust Attestation — cryptographic proof of execution environment | LOW | Verify COS does not define INV-9 differently. |
| INV-10 | not prominently defined in COS | Constitutional Immutability — core invariants cannot be modified without Convenor ratification | LOW | Verify and alias if needed. |

## COS-Only Invariants

| INV # | COS Definition | ADR Candidate | Notes |
|---|---|---|---|
| INV-1 | Constitutional Primacy | Related to ADR INV-10 | May be same concept at different granularity. |
| INV-2 through INV-6 | various — need full COS audit | — | Need full CONSTITUTION.md mapping. |
| INV-24 | Kintsugi No-Delete Policy | No direct equivalent | Potentially absorbs COS INV-8. |
| INV-25 through INV-32 | various | — | Need full COS audit. |
| INV-33 through INV-36 | Constitutional Routing | Related to ADR INV-7/7c/14a | May be a superset. |
| INV-37 | 144-Sphere Ontology Completeness | Related to ADR INV-13 | Structural completeness vs operational accountability. |

## ADR-Only Invariants

| INV # | ADR Definition | Constitutional Status | COS Candidate |
|---|---|---|---|
| INV-7a | No Hard Subscription Dependency | Retroactive | No equivalent |
| INV-7b | Combined Hyperscaler Cap (80%) | Retroactive | No equivalent |
| INV-7c | Model Family Cap (60%) | Retroactive | COS INV-33–36 may overlap |
| INV-7d | Cost Transparency | Evidence | No equivalent |
| INV-11 | Resource Transparency (Carbon + Water) | Evidence | No equivalent |
| INV-11.8 | Water Cycle Accounting | Forward | No equivalent |
| INV-12 | Financial Transparency (reserved) | Forward | No equivalent |
| INV-13 | Cross-Sphere Accountability | Evidence | Related to COS INV-37 |
| INV-14 | Community Impact Assessment | Forward | No equivalent |
| INV-14a | Symbiotic Infrastructure Preference | Forward | No equivalent |
| INV-17 | Digital Dividend Doctrine | Forward | No equivalent |

## Reconciliation Recommendations

### Option A: ADR as Canonical

ADR-145-DPI v3 contains the most comprehensive invariant set. ADR numbering becomes canonical; COS invariants that do not collide are absorbed with aliases; COS INV-8 is aliased to avoid collision.

### Option B: Dual Registry with Aliases

Both numbering schemes remain valid in their source repos. This cross-reference resolves ambiguity. All new invariants use ADR numbering going forward.

### Option C: Unified Renumbering (Deferred)

Full renumbering across all repos. Most disruptive. Defer until v1.0 canonization.

## Action Items

- [ ] Full audit of COS CONSTITUTION.md INV-2 through INV-6 and INV-25 through INV-32.
- [ ] Convenor decision on reconciliation option.
- [ ] Cross-reference aliases added to both COS and ADR source documents.
- [ ] INV-8 collision resolved explicitly.
- [ ] INV-37 / INV-13 relationship clarified.
