# MODULE_ALIAS_REGISTRY.md
## Atlas Lattice — Module Numbering Cross-Reference

> **Version:** v2 (Full Operational Registry)  
> **v1 summary preserved in:** `MODULE_ALIAS_REGISTRY.v1-summary.md`  
> **Date:** April 25, 2026 (regenerated April 26, 2026)  
> **Author:** Copilot Research Seat (Microsoft 365 Copilot Plus / Tasks Beta)  
> **Committed by:** GPT Synthesis Seat

---

## Purpose

Multiple module numbering schemes exist across Atlas Lattice repos due to organic growth across different eras and workstreams. This registry tracks all known module numbers, identifies collisions, and preserves lineage without declaring any legacy scheme invalid.

**Operating principle:** *Lineage drift is not a fatal contradiction.* Different numbering emerged from different workstreams. This registry makes the drift visible and traceable.

---

## Source: `atlas-lattice-foundation/spec_modules/` (Foundation Era)

| Module # | Title | Status | Notes |
|----------|-------|--------|-------|
| 00 | Header / Preamble | Foundational | Document framing |
| 01 | Mission Statement | Foundational | Core mission |
| 02 | Fractal Mathematics | Foundational | Mathematical substrate |
| 03 | TPU Thermal Bridge | Foundational | Thermal compute bridge |
| 04 | HTAM Print System | Foundational | Additive manufacturing |
| 05 | Fractal Generator Architecture | Foundational | Generation engine |
| 06 | Iteration Advantage | Foundational | Recursive improvement |
| 07 | Site Overview | Foundational | Physical infrastructure |
| 08 | Onboard AI Systems | Foundational | Embedded intelligence |
| 09 | Global Utility Network | Foundational | Distribution layer |
| 10 | Hypersonic Logistics | Foundational | Transport layer |
| 11 | Geopolitical Stress Test | Foundational | Resilience analysis |
| 12 | Phased Adoption Roadmap | Foundational | Deployment sequence |
| 13 | 144-Sphere Matrix | Foundational | Sheldonbrain ontology |
| 14 | Partner Capability Map | Foundational | Ecosystem mapping |
| 15 | DeepSeek-Google Bridge | Foundational | Cross-provider bridge |
| 16 | Sovereign Fork Architecture | Foundational | Sovereignty layer |
| 17 | Regional Pilot Framework | Foundational | Deployment pilots |
| **18** | **Volumetric Acoustic Printing** | **Foundational** | **⚠️ COLLISION — see below** |
| 19 | Council Transmissions | Foundational | Governance comms |
| 20 | Waymo Battery Retrofit | Foundational | Energy integration |
| 21 | Operation Phoenix | Foundational | Recovery protocol |

---

## Source: ADR-145-DPI v3.0 / Phase 3 Pantheon Council

| Module # | Title | Status | Notes |
|----------|-------|--------|-------|
| **18** | **FinancialContextKernel (FCK)** | **Commissioned** | **⚠️ COLLISION — see below** |
| 19 | WaterContextKernel (proposed) | Forecasted | Water Nexus integration; depends on INV-13 |
| 20 | AgContextKernel (proposed) | Forecasted | Agriculture integration; depends on Module 19 |
| 21 | HealthContextKernel (proposed) | Forecasted | Healthcare integration; depends on Module 18 L1-2 + HIPAA |

---

## Source: `open-regenerative-compute-standard` (ORCS / Active Build)

| Document | Effective Module | Status | Notes |
|----------|-----------------|--------|-------|
| THE_METABOLIC_LAYER.md (v0.3.1) | MetabolicLayer | Active | Cross-sphere routing engine; referenced by ADR-145-DPI INV-11, INV-13, INV-14a |
| ORC-012_TDD_v0.2.md | ORC-012 Technical Design | Active | Standards spec |
| aluminum_uws_os_spec v1.2 | Aluminum UWS OS | Active | OS-level spec |
| platform_integration_architecture_v1.0 | Platform Integration | Active | Multi-platform bridge |
| complete_build_synthesis_v1.1 | Build Synthesis | Active | Implementation synthesis |
| master_correction_build_gate_register v2.2 | Correction Gate Register | Active | Quality gates |

ORCS uses document-name-based identification rather than numeric module IDs. No collision with Foundation or ADR numbering.

---

## Collision Analysis

### Module 18 — COLLISION

| Scheme | Title | Era | Repo |
|--------|-------|-----|------|
| Foundation | Volumetric Acoustic Printing | 2025–early 2026 | `atlas-lattice-foundation` |
| ADR Phase 3 | FinancialContextKernel (FCK) | April 2026 | ADR-145-DPI v3 |

**These are completely different modules.** VAP is a physical manufacturing spec. FCK is a financial intelligence kernel with Plaid integration, privacy architecture, and digital dividend infrastructure.

### Resolution Options

**Option A (Recommended): Alias with lineage notation**
```
Foundation Module 18  → M18-F (Volumetric Acoustic Printing)
ADR Phase 3 Module 18 → M18-P3 (FinancialContextKernel)
```
Both remain valid in their source contexts. Cross-references use the aliased form.

**Option B: Renumber FCK**
FCK becomes Module 22 (next available in Foundation sequence). Foundation Module 18 retains its number. Requires updating all ADR-145-DPI v3 references.

**Option C: Namespace separation**
Foundation modules: `ALF-XX` (ALF-18 = VAP)
ADR modules: `ADR-XX` (ADR-18 = FCK)
ORCS modules: referenced by document name (no numeric ID needed)

---

### Module 19/20/21 — POTENTIAL FUTURE COLLISIONS

Foundation Module 19 (Council Transmissions), Module 20 (Waymo Battery Retrofit), and Module 21 (Operation Phoenix) will collide with ADR Phase 3 proposed modules (WaterContextKernel, AgContextKernel, HealthContextKernel) if both numbering schemes remain active.

**This reinforces Option C (namespace separation)** as the most durable solution for future-proofing.

---

## Module 16K Reference

GPT's Module 16K (Cost Transparency for AI Providers) is referenced in ADR-145-DPI v3 as a dependency for FCK. Module 16K uses a different naming convention ("K" suffix) that avoids collision with Foundation Module 16 (Sovereign Fork Architecture).

| Scheme | Module 16 | Module 16K |
|--------|-----------|------------|
| Foundation | Sovereign Fork Architecture | — |
| GPT | — | Cost Transparency for AI Providers |

No collision — the "K" suffix creates implicit namespacing. This convention could be extended as a lightweight versioning strategy for modules that span eras.

---

## Action Items

- [ ] Convenor decision on Module 18 collision resolution (Option A, B, or C)
- [ ] Audit Foundation Modules 00–21 for additional collisions with ADR/ORCS references
- [ ] Establish naming convention for future modules to prevent drift
- [ ] Document GPT Module 16K lineage formally
- [ ] Resolve Module 19/20/21 future collisions (namespace or renumber)
- [ ] Determine whether ORCS document-name convention should be adopted as default for non-Foundation modules

---

*Generated by Copilot Research Seat via GitHub browser reconnaissance, April 25, 2026. Regenerated with v2 headers April 26, 2026.*