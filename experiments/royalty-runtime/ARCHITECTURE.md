# Royalty Runtime Architecture

## Thesis
Royalty Runtime began as an execution-linked royalty concept and evolved into an observability-first system for making software value creation visible, replayable, and eventually routable.

## Core principles
- Attribution before payment
- Store first, verify clearly, interpret later
- Pay packages before people
- Enforcement systems get forked; measurement systems get embedded

## Layers
1. **Lineage Extraction**
   - Build a canonical view of the executing environment
   - Include primary package, runtime, lockfile digest, and resolved dependencies

2. **Deterministic Hashing**
   - Serialize the canonical lineage into a stable form
   - Hash it in Rust to avoid environment drift

3. **Collector / Event Store**
   - Accept execution events
   - Store raw payload JSON plus verification state
   - Preserve replayability for future model upgrades

4. **Weighting Engine**
   - Package-level first
   - v0 policy: primary package gets 40%, remaining 60% split equally among dependencies
   - Important: weighting is a model, not objective truth

5. **Optional Lease / Premium Capability Layer**
   - Explored as a product wedge, not yet the strategic center
   - Premium capability examples: concurrency, latency, throughput

6. **Observability Business Layer**
   - Dependency intelligence
   - Supply-chain transparency
   - Compliance reporting
   - AI agent execution metering
   - Optional future royalty routing

## Why `uws` matters
`uws` is a strong candidate integration point because it sits at a real orchestration boundary for humans and AI agents across multiple ecosystems.

## Honest current state
This is not a finished payment network.
It is the first coherent substrate for:
- execution observability
- versioned attribution
- future economic routing
