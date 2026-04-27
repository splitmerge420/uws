# VWB Canonical Specification v1.0

Status: Canonical anchor draft / Phase 1 regulatory-facing baseline
Date: 2026-04-26
Source: GPT synthesis from Convenor direction, Claude critique, Gemini prototype review, and ORCS/VWB proof-bundle analysis

## 1. Purpose

This document defines the conservative v1.0 canonical form of the Volumetric Water Benefit (VWB) calculation for ORCS-style civic battery / regenerative compute proposals.

The purpose is to create a stable, auditable baseline before adding expanded quality, civic, or provenance multipliers.

VWB v1.0 is intended for:

- transparent scenario comparison;
- early public-policy discussion;
- proof-of-works prototypes;
- engineering review;
- later regulatory or legal review.

VWB v1.0 is not legal certification, regulatory approval, or proof of compliance by itself.

## 2. Core Principle

Do not overclaim.

VWB v1.0 should measure a narrow volumetric claim first:

```text
How much net water benefit does a node plausibly create after accounting for recapture, substitution, conveyance efficiency, and facility consumption?
```

Quality, civic resilience, PFAS gates, multi-model arbitration, and Last Starfighter provenance may be layered later, but they must not be hidden inside the v1.0 volumetric equation.

## 3. Canonical VWB v1.0 Equation

```text
VWB_net = (Wr * eta_t * lambda_r) + (Ws * alpha * beta * Sf * lambda_a) - Wc
```

Where:

| Symbol | Meaning | Unit | Notes |
|---|---|---|---|
| `Wr` | Reclaimed / recaptured water available for beneficial reuse | MGD | Supply-side recapture baseline |
| `eta_t` | Treatment usability factor | unitless 0-1 | Percent of reclaimed water usable for intended substitution |
| `lambda_r` | Recapture conveyance efficiency | unitless 0-1 | May later decompose into distribution, pumping, storage, emitter, evaporation factors |
| `Ws` | Water withdrawal or demand being substituted | MGD | Demand-side substitution baseline |
| `alpha` | Adoption / contracted-acreage participation factor | unitless 0-1 | Fraction of target substitution actually enrolled |
| `beta` | Substitution effectiveness | unitless 0-1 | Fraction of baseline demand actually displaced |
| `Sf` | Seasonal factor | unitless 0-1 | Fraction of year/season in which substitution applies |
| `lambda_a` | Agricultural / demand-side conveyance efficiency | unitless 0-1 | Efficiency of delivering substitute service to the demand-side use |
| `Wc` | Facility water consumption burden | MGD | Node/facility water consumed or lost |

## 4. Water Positivity Index

```text
WPI_facility = VWB_net / Wc
```

Interpretation:

| WPI | Meaning |
|---|---|
| `< 0` | Net negative water impact |
| `0 to 1.0` | Partial offset but not water-positive |
| `1.0` | Break-even water-positive threshold |
| `> 1.0` | Net water-positive by facility accounting |
| `> 1.2` | Suggested early target buffer, not regulatory standard |

## 5. Validation Gates

Every VWB v1.0 calculation should include:

- formula version;
- node ID;
- measurement period;
- basin / watershed identifier;
- units for every water quantity;
- source URI or dataset reference for every baseline;
- calculation timestamp;
- authoring / review seat;
- audit status.

Minimum audit statuses:

```text
draft
pending-source-verification
source-supported
engineering-reviewed
legal-reviewed
rejected
superseded
```

## 6. Required Input Constraints

All unitless factors must satisfy:

```text
0 <= factor <= 1
```

All water quantities must be non-negative unless explicitly modeling a deficit case.

`Wc` must be greater than zero to calculate `WPI_facility`.

Node IDs should be provider-neutral in canonical examples.

Preferred:

```text
IN-SB-NODE-0
```

Avoid provider-specific examples such as:

```text
IN-SB-AWS-0
```

unless a specific provider has formally entered the scenario.

## 7. Explicitly Deferred from v1.0

The following concepts are important but deferred from the canonical v1.0 volumetric equation:

- water quality benefit adjustment (`Qc`);
- PFAS / contaminant hard gates;
- WQBA metrics;
- civic resilience index (`WPI_civic`);
- provenance multipliers (`Pm`);
- Last Starfighter compensation tiers;
- multi-model confidence scoring;
- Meta-Arbitration Layer weighting;
- legal service-substitution interpretation under Indiana law.

These should be separate modules or appendices, not hidden parameters in the core v1.0 formula.

## 8. Expanded Formula Handling

If an expanded formula is used, it must be labeled as experimental or v1.x / v2 candidate.

For example:

```text
VWB_candidate = [(Wr * eta_t * Qc * lambda_r) + (Ws * alpha * beta * Sf * lambda_a * delta)] * Pm - Wc
```

This is not VWB v1.0 unless separately ratified.

Candidate variables such as `Qc`, `delta`, and `Pm` require independent definitions, evidence standards, and governance gates.

## 9. Relationship to Governance / Epistemic Layers

VWB v1.0 belongs to the measurement layer.

It should not depend on a single AI model, vendor, or reasoning seat.

- Epistemic layer: multiple models may critique calculations and assumptions.
- Governance layer: Scribe / Convenor / human review decides what becomes externally authoritative.

Majority voting is not sufficient for later production arbitration. Reliability weighting and disagreement surfaces belong in a future Meta-Arbitration Layer, but that work is Phase 2 and must be implemented neutrally.

## 10. Minimum Example Payload

```json
{
  "formula_version": "VWB-1.0",
  "node_id": "IN-SB-NODE-0",
  "measurement_period": "scenario",
  "basin_id": "TBD",
  "inputs": {
    "Wr_mgd": 5.0,
    "eta_t": 0.90,
    "lambda_r": 0.98,
    "Ws_mgd": 3.0,
    "alpha": 1.0,
    "beta": 0.95,
    "Sf": 1.0,
    "lambda_a": 0.95,
    "Wc_mgd": 2.5
  },
  "outputs": {
    "VWB_net_mgd": 5.6375,
    "WPI_facility": 2.255
  },
  "audit_status": "pending-source-verification"
}
```

## 11. Acceptance Criteria for Code

A reference implementation must:

- expose the formula version;
- validate all factors are between 0 and 1;
- reject negative water quantities by default;
- reject `WPI_facility` calculation when `Wc <= 0`;
- return structured JSON;
- preserve provenance metadata;
- include unit tests for normal, boundary, and invalid cases;
- avoid provider-specific hardcoding.

## 12. Short Form

VWB v1.0 is the narrow volumetric anchor.

Do the boring math first. Add quality, civic, provenance, and arbitration layers only after the baseline is stable, cited, and reviewable.
