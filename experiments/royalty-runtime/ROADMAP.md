# Royalty Runtime Roadmap

## Chosen direction
This project is being developed as **infrastructure**.

That means the near-term goal is not to enforce payment. The near-term goal is to become the most trustworthy execution observability substrate for software and agent workflows.

## Phase 0 — Session capture (done)
- architecture captured
- observability pivot captured
- initial scaffold written
- `uws` identified as a strong integration point

## Phase 1 — Truth layer
- canonical lineage schema
- deterministic hashing
- execution event collector
- replayability and verification
- package-level attribution map

## Phase 2 — `uws` integration
- non-blocking async telemetry hook
- execution event emission from real workspace workflows
- lease / premium path ideas kept experimental and separate
- zero disruption to the core CLI UX

## Phase 3 — Visibility layer
- dashboard for events, lineage hashes, and attribution maps
- dependency hot spots
- package criticality views
- agent execution volume views

## Phase 4 — Intelligence layer
- dependency risk scoring
- maintainer underfunding alerts
- supply-chain reporting
- provenance exports

## Phase 5 — Optional economic layer
- opt-in royalty routing
- package treasury registry
- attribution model versioning and simulation
- enterprise reporting

## Rules of engagement
- do not break `uws`
- do not lead with DRM
- do not overclaim objectivity in weighting
- always keep raw events replayable
- machine-trust before pretty UX
