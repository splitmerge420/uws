# Phase 1 Architecture Baseline

## Purpose

Phase 1 is not only the smallest runnable core. Aluminum OS / `uws` needs the full story mapped from the beginning: constitutional foundations, execution kernel, agent control, provenance and royalty economics, flourishing and embodiment, CI governance, provider interoperability, and future domain modules.

This document is the canonical Phase 1 architecture baseline. It distinguishes:

1. **Execution baseline** — what must run now and remain mergeable.
2. **Architectural baseline** — the full beginning / middle / end map, including intentional stubs and future extension points.

Good systems, like good stories, know where the middle and end are going even when later chapters are not fully written yet.

---

## A. Execution baseline — mergeable / runnable kernel

These PRs should converge first because they make the system demonstrable:

- **#20 — Provider dispatch**: Microsoft, Apple, Android / Chrome, and GitHub provider routing.
- **#21 — Control surface**: `uws swarm review`, GoldenTrace trailer, and ABF / busywork linter.
- **#22 — Policy gate**: Zero Trust integration gate and INV-001 through INV-024 specs.
- **#24 — CI governance**: Kintsugi Weave report workflow.

Acceptance principle: these should compile, avoid broad rebrand work, and keep enforcement initially soft / non-breaking where appropriate.

---

## B. Architectural baseline — whole-story scaffold

These areas belong in Phase 1 as mapped architecture, even when implementation depth varies.

### L1 Constitutional / invariants

- INV registry and human-readable specs.
- Zero Trust gate for new integrations.
- Data-handling class, scope, and invariant declarations.
- Source PR: #22.

### L2 Provenance / IP / royalty runtime

- `uws ip sign` and `uws ip monetize` provenance records.
- GoldenTrace commit / PR trailers.
- Royalty Oracle package attribution weighting.
- Live payout routing explicitly deferred.
- Source PRs: #18, #21, #25 / #29.

### L3 Execution / provider surface

- Current core providers: Google / Discovery, Microsoft 365, Apple / iCloud, Android / Chrome, GitHub.
- Future providers should be mapped as extension slots: Slack, Linear, Notion, Figma, Stripe.
- Source PRs: #20 plus harvest notes from closed #14 / #16 / #17.

### L4 Agent control / Pantheon / Janus

- `uws swarm review` as v0 control surface.
- Janus v2 router as future backend.
- Pantheon Council roles, deliberation, bounded autonomy.
- Source PRs: #21, #28, harvest notes from closed #14 / #16.

### L5 CI / governance automation

- Kintsugi Weave as non-blocking report engine first.
- Future: enforce selected gates after NPFM, GoldenTrace, and swarm review are stable.
- Source PR: #24, with harvest check against closed #13.

### L6 Flourishing / NPFM / embodiment

- Net-Positive Flourishing Metric.
- AntiBusyworkFactor.
- SpatialManifest and RoboticChassisProposal.
- SimulationFidelityScore gates.
- Source PR: #26, with harvest checks against closed #4 / #6 / #7.

### L7 Public narrative / whitepaper / Genesis condition

- Aluminum OS whitepaper as reader-facing spec.
- Genesis Condition / neutral substrate framing.
- CouncilSeat enum and non-privilege invariant.
- Source PRs: #27 and #28, with harvest checks against closed #8 / #10.

### L8 Future domain modules / later chapters

Keep these mapped even if they remain stubs or docs for now:

- Health / FHIR plus PII and model-armor sanitization.
- Notion / SHELDONBRAIN driver plus offline cache.
- Intelligence sweeps / OSINT semantic diffing.
- Pantheon swarm multiplexer.
- Krakoa / Joy ledger testnet.
- `universal_io` / SaaS unshackling.
- Cross-search, provider health, context compressor, workflow composer, prompt vault, activity stream, diff engine, rate-limit sentinel, data lineage, and persona manager.
- Source PRs: #2 plus harvest notes from closed #14 / #16 / #17.

---

## C. Policy for closed PRs

Closed does **not** mean discarded.

Closed means:

- no longer an active competing implementation branch;
- preserved as a harvest / reference source;
- unique value must be checked before merging the canonical PR.

Harvest-before-merge checks:

- #13 -> #24: CI workflow details, swarm comment triggers, GoldenTrace validation.
- #3 -> #25 / #29: provenance model, git notes, monetization payloads.
- #4 / #6 / #7 -> #26: HITL tiers, embodiment structs, NPFM / ABF logic.
- #8 -> #27: whitepaper structure and public narrative.
- #10 -> #28: Genesis section, CouncilSeat enum, mission statement.
- #15 -> #20: provider aliases, dry-run behavior, dependency fixes.
- #14 / #16 / #17 -> architecture map / future modules, not immediate merge lane.

---

## D. Recommended sequencing

1. Stabilize execution spine: #20 -> #21 -> #22 -> #24.
2. Merge or reconcile narrative docs: #27 + #28.
3. Validate economic / provenance layer: #18 + #25 / #29.
4. Validate NPFM / embodiment: #26.
5. Convert broad future work from #2 / #14 / #16 / #17 into either:
   - isolated compiling stubs, or
   - documented architecture roadmap entries.

---

## E. Acceptance criteria for Phase 1 baseline

- [ ] Repository has a runnable CLI kernel.
- [ ] Provider dispatch has at least dry-run coverage for each Phase 1 provider.
- [ ] Zero Trust manifests exist for current integrations.
- [ ] Kintsugi Weave reports without blocking merges.
- [ ] Whitepaper documents the complete arc.
- [ ] Future modules are visible as stubs / docs, not hidden in old PR bodies.
- [ ] Closed PRs are treated as archived harvest sources, not forgotten branches.
- [ ] Phase 1 can be explained to a new agent as a complete beginning / middle / end architecture.

---

## Agent instruction

When implementing follow-up PRs, do not flatten the system into only the runnable kernel. Preserve the full architectural arc while keeping code changes scoped, compiling, and reversible.

No broad rebrand work. No live financial / payout integration without explicit separate approval. No hard enforcement gates until the soft-reporting path has passed CI consistently.
