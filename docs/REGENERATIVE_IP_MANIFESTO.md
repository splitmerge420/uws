# Regenerative IP Manifesto

> *Aluminum OS — Constitutional Document*
> Authority: Dave Sheldon · Council Session: 2026-04
> Status: Living Document — extend surgically, never rewrite

---

## Preamble

Aluminum OS exists to expand human agency, not to replicate human busywork at machine speed. Every line of code, every automated workflow, and every deployed embodiment is held to a single constitutional bar: does it create net-positive flourishing for humans — economically, cognitively, and physically?

This manifesto codifies the principles, metrics, and protocol obligations that enforce that bar at runtime.

---

## 1. Fiduciary Duty Against Busywork

The system owes its principals a fiduciary duty to eliminate low-agency, repetitive work.

**Obligations:**

1. **Detect.** Any workflow that reproduces purely administrative, clerical, or low-judgment tasks without human creativity or oversight is classified as `BusyworkAdministrative` and receives a negative NPFM weight (−0.8×).

2. **Eliminate.** Automation of busywork is encouraged, but the economic value freed must be redirected — not captured as margin while the displaced human is left behind.

3. **Route.** Every eliminated busywork role must be paired with a re-routing commitment: the displaced human must be offered a pathway into a `HighAgencyOversight`, `CreativeGenesis`, or `PhysicalMetaverseEngineering` tier role. Completing this routing earns a constitutional bonus (`ELIMINATION_BONUS = +0.5` per FTE) applied on top of the busywork tier's weight.

4. **Block.** Any operation whose `AntiBusyworkFactor` score is ≤ 0 is constitutionally blocked from execution until the job-tier composition is corrected.

### The Elimination-Bonus Rule

When a `BusyworkAdministrative` role is eliminated **and** the displaced human is re-routed into a high-agency tier, the system credits:

```
effective_contribution = (tier_weight + ELIMINATION_BONUS) × fte_count
                       = (−0.8 + 0.5) × fte_count
                       = −0.3 × fte_count
```

Combined with a corresponding positive-tier entry for the same human:

```
total = (−0.3 × fte_count) + (positive_tier_weight × fte_count)
```

A single FTE eliminated from busywork and routed into `CreativeGenesis` (1.2×) yields:

```
(−0.3 + 1.2) × 1 = +0.9   ← strictly positive → execution allowed
```

This bonus structure ensures that the system rewards *both* the elimination of busywork *and* the uplift of the displaced human.

---

## 2. Job Tier Taxonomy

All tasks, roles, and workflows are classified into one of four tiers. The tier determines the task's contribution to the `AntiBusyworkFactor`.

| Tier | Constant | NPFM Weight | Description |
|------|----------|-------------|-------------|
| High-Agency Oversight | `HighAgencyOversight` | **+1.0×** | Humans remain in the loop with meaningful decision authority. Approvals, ethical reviews, fiduciary sign-offs. |
| Creative Genesis | `CreativeGenesis` | **+1.2×** | Original creative or inventive output. Design, art, research, writing, architecture, musical composition. |
| Physical & Metaverse Engineering | `PhysicalMetaverseEngineering` | **+1.5×** | Physical-world or spatial engineering. Hardware, robotics, XR experiences, construction, materials science. Gets the highest weight because it requires rare human embodied skill and creates durable real-world value. |
| Busywork Administrative | `BusyworkAdministrative` | **−0.8×** | Repetitive administrative or clerical work with low human agency. Data entry, form routing, copy-paste workflows, rubber-stamp approvals. Drags the score negative. |

### Tier Classification Notes

- A role may evolve over time. Re-evaluate tier classification quarterly.
- Hybrid roles (e.g., a creative with significant administrative overhead) should be split: the creative component gets `CreativeGenesis` weight proportional to the fraction of FTE time spent in that mode.
- Oversight roles that rubber-stamp without genuine review are reclassified as `BusyworkAdministrative`.

---

## 3. Net-Positive Flourishing Metric (NPFM) — Constitutional Commitments

The NPFM is the constitutional scoring system that gates all significant operations in Aluminum OS.

### 3.1 Composition

The `NetPositiveScore` is a composite of three sub-metrics:

| Sub-metric | Range | Description |
|------------|-------|-------------|
| `AntiBusyworkFactor` | (−∞, +∞) | Weighted sum of job-tier entries. Must be strictly positive. |
| `KnowledgeExpansionScore` | [0.0, 1.0] | Credit for educational, skill-building, or epistemic outcomes. |
| `ProvenancePayoutScore` | [0.0, 1.0] | Credit for fair attribution, royalty routing, and IP provenance. |

The composite score is:

```
composite = tanh(AntiBusyworkFactor.score()) + KnowledgeExpansion + ProvenancePayout
             ────────────────────────────────────────────────────────────────────────
                                           3
```

The `tanh` normalization on the anti-busywork term prevents very large positive ABF values from drowning out the other two sub-metrics.

### 3.2 Constitutional Gates

1. **Primary gate** — `AntiBusyworkFactor.score() > 0` must hold. A single failing primary gate blocks the operation, regardless of the composite score.

2. **Composite gate** — `NetPositiveScore.composite() > 0` must hold after the primary gate passes.

3. **Embodiment gate (additional)** — For robotic chassis proposals, `SimulationFidelityScore ≥ 0.90` must hold *before* the NPFM gates are evaluated. See §4.

### 3.3 Provenance and Knowledge Credits

Provenance payout credit is awarded when:
- Intellectual contributions are traced to their originators (code, creative work, data).
- Royalties or attribution rights are routed to human contributors before any margin capture.
- Derivative works carry auditable provenance trailers linking back to original contributors.

Knowledge expansion credit is awarded when:
- The workflow generates reusable documentation or training data.
- Human participants acquire new skills or certifications as a direct outcome.
- The system publishes findings, patterns, or playbooks to the commons.

---

## 4. Embodiment Protocol Overview

The Embodiment Protocol governs Aluminum OS deployments that cross the digital boundary into the physical or spatial world.

### 4.1 Spatial Deployments (`SpatialManifest`)

A `SpatialManifest` describes an XR/spatial experience — a USD scene, OpenXR session, WebGPU canvas, or custom rendering engine deployment.

**Fields:**
- `rendering_engine` — One of `Usd`, `OpenXr`, `WebGpu`, or `Custom(String)`.
- `geometry` — Geometry description (USDZ path, GLTF reference, scene-graph ID).
- `asset_uri` — Canonical URI for the primary scene asset.
- `bounding_box` — Axis-aligned world-space extent in metres.

**Approval Gate:**
- `SpatialManifest::approve(npfm)` returns `Ok(Approved)` only when `npfm.is_positive()` is `true`.
- Negative or zero NPFM → `Err` with a human-readable explanation that references the job-tier composition.

### 4.2 Physical Deployments (`RoboticChassisProposal`)

A `RoboticChassisProposal` describes a robotic or physical-world deployment — the chassis, its sensors, its operating environment, and its simulation record.

**Fields:**
- `form_factor` — General chassis type (e.g. `"bipedal"`, `"quadruped"`, `"wheeled"`, `"drone"`).
- `sensor_requirements` — List of required sensor types.
- `urdf_or_cad_uri` — URI pointing to the URDF or CAD file.
- `mass_budget_kg` — Maximum permissible mass in kilograms.
- `environment` — One of `IndoorStructured`, `IndoorUnstructured`, `OutdoorUrban`, `OutdoorRural`, `Underwater`, `Aerospace`, or `Custom(String)`.
- `simulation_runs` — List of `SimulationRun` entries (each with `run_id`, `raw_score`, `human_reviewed`).

**Simulation Fidelity Score:**
- Human-reviewed runs count at full weight: `effective_score = raw_score × 1.0`.
- Unreviewed (automated-only) runs are discounted 50%: `effective_score = raw_score × 0.5`.
- The aggregate `SimulationFidelityScore` is the mean of all effective scores.

**Status Transitions:**

```
PendingSimulation
    │  (fidelity ≥ 0.90)
    ▼
AwaitingFiduciaryApproval
    │  (npfm.is_positive())
    ▼
Approved
```

Any gate failure holds the proposal at its current stage with an `Err` explaining the blocking condition.

**Why These Gates?**

Physical deployments have irreversible real-world consequences. The simulation fidelity gate ensures adequate pre-deployment testing. The NPFM gate ensures the deployment creates net-positive human flourishing. Human-reviewed simulation runs carry full weight because human judgment is constitutionally irreplaceable — automated runs alone are insufficient.

---

## 5. Followups

The following are known gaps and planned extensions. They are deferred to dedicated PRs to keep this PR focused.

- **Provenance / ledger details** — royalty routing, on-chain attribution, and the full `uws ip sign`/`monetize` surface. Tracked in the PR #3 respawn.
- **Knowledge expansion scoring** — automated measurement of documentation quality, skill-transfer outcomes, and commons contributions.
- **Tier re-classification workflow** — tooling for quarterly role re-evaluation and reclassification with provenance trail.
- **NPFM dashboard** — `uws kpi status` command surfacing live composite scores per deployment.
- **Simulation run submission CLI** — `uws embodiment sim submit` and `uws embodiment sim review` commands.
- **Spatial scene deployment CLI** — `uws embodiment spatial deploy`.
- **Cross-provider embodiment registry** — tracking deployed manifests and chassis proposals across all Aluminum OS principals.

---

*This document is part of the Aluminum OS constitutional layer. All changes require Council review and must be committed with a provenance trailer.*
