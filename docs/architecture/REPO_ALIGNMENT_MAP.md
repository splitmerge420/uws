# Atlas Lattice Repo Alignment Map

Status: Phase 1 alignment spine
Owner: Atlas Lattice / Aluminum OS council workflow

## Purpose

This document aligns the main Atlas Lattice repositories into a coherent build topology. The goal is to prevent public/private repo fragmentation, preserve prior work as harvestable strata, and make it clear which repository owns which layer of the system.


---

## Tier 0 — Core system repos

### atlaslattice/atlas-lattice-foundation

Role: foundation / constitutional / public institutional anchor.

Primary responsibilities:
- institutional identity;
- governance principles;
- public constitutional framing;
- high-level doctrine;
- neutral substrate / foundation narrative;
- public-facing trust artifacts.

Public/private posture: public.

Canonical status: Tier 0 canonical foundation repo.

### atlaslattice/aluminum-os

Role: Aluminum OS umbrella / product and operating-system identity layer.

Primary responsibilities:
- OS-level vision;
- roadmap;
- public product narrative;
- demos and deployment plans;
- integration map across app, agent, workspace, and device layers;
- bridge between foundation doctrine and runnable UWS execution.

Public/private posture: public-facing when stable.

Canonical status: Tier 0 canonical OS umbrella repo.

### atlaslattice/uws

Role: Universal Workspace / command kernel / execution layer.

Primary responsibilities:
- schema-driven JSON-first command surface;
- provider dispatch;
- GitHub / Microsoft / Apple / Android / Google / Notion / agent integrations;
- governance skeletons;
- provider skeletons;
- Phase 1 baseline docs;
- module build scaffolds;
- PR cleanup and execution convergence.

Public/private posture: public or controlled-public depending on security review.

Canonical status: Tier 0 canonical execution repo.

---

## Tier 1 — Domain/module source repos

These repos should be treated as source material for future module extraction, not necessarily as independent products unless later promoted.

Examples visible through the connected GitHub account:

- `atlaslattice/healthcare-ai`
- `atlaslattice/banking-revolution`
- `atlaslattice/fintech-disruption`
- `atlaslattice/climate-ai-solutions`
- `atlaslattice/energy-grid-ai`
- `atlaslattice/agricultural-ai`
- `atlaslattice/water-management-ai`
- `atlaslattice/mental-health-ai`
- `atlaslattice/military-ai-ethics`
- `atlaslattice/disability-tech-access`
- `atlaslattice/surveillance-capitalism`
- `atlaslattice/ai-labor-exploitation`

Recommended status labels:

- module-source;
- needs-harvest;
- private-source;
- public-extract-candidate;
- sensitive-review-required.

Harvest policy:

1. Extract concepts into Tier 0 docs first.
2. Convert into compiling stubs only after ownership is clear.
3. Do not copy sensitive private material into public repos without review.
4. Preserve provenance: source repo, file path, date, agent/source if known.

---

## Tier 2 — Archive / prior art / evidence repos

These repos preserve reasoning, evidence, prior attempts, or background research. They may be private or public.

Examples:

- `atlaslattice/noosphere-archive`
- `atlaslattice/banking-revolution-archive`
- AI ethics / transparency / accountability repos;
- labor, rights, safety, governance, and bias analysis repos.

Recommended status labels:

- archive-source;
- evidence-source;
- prior-art;
- citation-source;
- deprecated-after-harvest.

Harvest policy:

- Treat these repos as evidence libraries, not current execution targets.
- Use them to support ADRs, whitepapers, and module docs.
- Avoid turning every archive into an active code dependency.

---

## Canonical layering

```text
atlas-lattice-foundation
  -> constitutional and institutional anchor

aluminum-os
  -> OS/product umbrella and public roadmap

uws
  -> runnable universal workspace kernel and provider command surface

private/domain repos
  -> source material and domain-specific module inputs

archive repos
  -> evidence, prior art, and historical strata
```

---

## Cross-repo doctrine

### 1. Closed or inactive does not mean discarded

Old branches, closed PRs, and private repos can remain valuable. They should be treated as harvest strata.

### 2. Public repos should not blindly absorb private material

Before moving content from private to public, check:

- privacy;
- security;
- legal sensitivity;
- personal data;
- third-party confidential data;
- whether claims require external verification.

### 3. UWS is the execution kernel, not the entire project

UWS should contain runnable scaffolding, provider surfaces, governance hooks, and CLI execution. It should reference but not swallow every domain repo.

### 4. Aluminum OS is the umbrella

Aluminum OS should explain how UWS, foundation doctrine, provider interoperability, and domain modules fit into one OS-level roadmap.

### 5. Atlas Lattice Foundation is the public trust anchor

The foundation repo should remain cleaner, more stable, and less implementation-heavy than UWS.

---

## Immediate alignment tasks

- [ ] Add this alignment map to `uws`.
- [ ] Add a short pointer from `aluminum-os` to this map.
- [ ] Add a short pointer from `atlas-lattice-foundation` to this map.
- [ ] Create a private-repo harvest queue before copying private content into public repos.
- [ ] Define public/private release gates.
- [ ] Add provenance metadata format for extracted material.
- [ ] Create a cross-repo README standard.

---

## Agent instruction

When working across repos, always identify:

- source repo;
- target repo;
- reason for movement;
- sensitivity level;
- whether content is canonical, module-source, archive-source, duplicate, deprecated, or needs-harvest.

Do not flatten the project into a single repo. Preserve the layered architecture.
