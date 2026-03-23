# Royalty Runtime

**Execution is compensation.**

If it runs, it pays.

This repository is a shareable scaffold for the architecture developed in today's working session:
- canonical lineage extraction
- deterministic lineage hashing
- collector ingestion and verification
- CLI developer loop
- package-level attribution weighting
- lease-aware premium execution concept
- observability-first pivot

## Repository layout

- `runtime-core/` — Rust core for canonical hashing, weighting, and lease-aware engine concepts
- `royalty-sdk/` — TypeScript SDK + CLI for tracing, hashing, and emitting lineage events
- `collector/` — Rust Axum + SQLx collector for storing execution events

## Current status

This is a **v0 scaffold**, not production-ready software.

What is solid:
- lineage schema and hashing shape
- collector event model
- CLI command surface
- package-level attribution model
- observability-first business pivot

What still needs real engineering:
- full dependency extraction across ecosystems
- hardened lease issuance / validation
- idempotency, signatures, replay resistance
- maintainer identity / payout routing
- real dashboard and analytics layer
