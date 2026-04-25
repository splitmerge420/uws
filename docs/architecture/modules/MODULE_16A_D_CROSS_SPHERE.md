# Module 16A-D — Cross-Sphere Accountability Modules

Status: Phase 1 skeleton / Phase 1.5 synthesis target
Source context: ADR-145-DPI v3.0, INV-13, INV-14, INV-14a, INV-16 reserved lineage

## Purpose

Module 16A-D provides the module spine for cross-sphere accountability. It converts the Phase 3 constitutional doctrine into Phase 1 skeleton architecture: named modules, boundaries, and future integration points.

## Module 16A — Sphere144 Mapping Registry

Purpose: define authoritative sphere mappings for invariants, provider decisions, and routed actions.

Skeleton responsibilities:
- map invariants to primary and secondary spheres;
- prevent retroactive narrowing of sphere touches;
- expose a lookup surface for governance and routing;
- prepare for Sheldonbrain 144-sphere ontology integration.

Phase 1 output: registry structs / docs.
Phase 1.5 output: validation against canonical Sphere144 taxonomy.

## Module 16B — Multi-Sphere Decision Classifier

Purpose: classify decisions under INV-13 multi-sphere discipline.

Decision classes:
- Class A: single-sphere impact;
- Class B: multi-sphere aligned incentives;
- Class C: multi-sphere conflicting incentives;
- Class D: multi-sphere irreversible commitment.

Phase 1 output: enum + placeholder classifier.
Phase 1.5 output: classifier integrated with provider dispatch and governance preflight.

## Module 16C — Indiana Pattern Detector

Purpose: detect single-sphere optimization that externalizes ecology, civic, community, or health impacts.

Detection signals:
- economic impact presented without environmental analysis;
- zoning / deployment process faster than community engagement cycle;
- water or energy projections absent;
- community opposition dismissed as NIMBYism rather than treated as cross-sphere accounting.

Phase 1 output: anti-pattern doc + detector skeleton.
Phase 1.5 output: scoring model and governance warning output.

## Module 16D — Symbiotic Infrastructure Preference

Purpose: implement the INV-14a preference layer for symbiotic compute infrastructure.

Preference is a tie-breaker only. Quality, cost, latency, consent, and safety remain hard constraints.

Phase 1 output: preference doc + routing hook.
Phase 1.5 output: optional routing bias when multiple providers / regions are equivalent.

## Integration Points

- `src/governance/priority.rs`
- `src/governance/impact.rs`
- `src/governance/attachment_points.rs`
- future `src/governance/sphere.rs`
- future `src/governance/patterns.rs`
- future provider routing / AdaptiveRouter layer

## Acceptance Criteria

- [ ] Module 16A-D has a visible documentation home.
- [ ] Phase 1 baseline references the module family.
- [ ] Rust skeleton homes exist for sphere mapping and anti-pattern detection.
- [ ] No hard enforcement is introduced until Phase 1.5 synthesis.
