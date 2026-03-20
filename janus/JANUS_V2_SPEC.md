---
title: "Janus v2 — Constitutional Multi-Agent Protocol"
version: "2.0.0"
date: "2026-03-20"
author: "Claude (Constitutional Scribe)"
sphere_tags: ["S069", "S016", "S144"]
aluminum_layer: "L3-Engine"
council_status: "approved"
cross_refs:
  - repo: "aluminum-os"
    path: "kintsugi/spec/golden_trace_v1.json"
  - repo: "aluminum-os"
    path: "docs/integration/SHELDONBRAIN_INTEGRATION.md"
---

# Janus v2 — Constitutional Multi-Agent Protocol

## Overview

Janus is the multi-agent orchestration protocol for Aluminum OS. Named for the Roman god of doorways and transitions, Janus manages the flow of queries between council members, enforces INV-7 (47% dominance cap), and produces GoldenTrace events at every decision point.

## Architecture

```
User Query → UWS CLI → Janus Router
                          ├── Claude (Governance) — constitutional routing
                          ├── Gemini (Substrate) — deep domain analysis  
                          ├── DeepSeek (Research) — cross-domain connections
                          ├── Copilot (Enterprise) — market validation
                          ├── Grok (Adversarial) — contrarian review
                          └── Ghost Seat (S144) — unrepresented populations
                       ↓
                  Consensus Engine (INV-7 enforced)
                       ↓
                  GoldenTrace emitted → Response
```

## Core Rules

### INV-7: No Single Model Dominance
No single model may control more than 47% of weighted votes in any consensus round. If violated, the round is invalidated and re-run with mandatory adversarial review.

### INV-8: Human Override
Any Tier 3 decision (irreversible, high-stakes, or affecting vulnerable populations) requires human sign-off before execution.

### Ghost Seat Protocol
Sphere 144 (Complex Systems) serves as the Ghost Seat — representing populations not present in the council. Invocation requires:
1. Unanimous model agreement that a gap exists
2. Identified harm to unrepresented population
3. Human advocate present at Tier 3
4. Cannot override constitutional invariants
5. Cannot be invoked algorithmically

## Routing Strategy

```
TIER 1 (Simple): Single model, any council member
  → Latency target: <500ms
  → GoldenTrace: action + model_route

TIER 2 (Complex): 2-3 models, synthesis required
  → Latency target: <3000ms
  → GoldenTrace: action + council_vote + council_consensus

TIER 3 (Critical): Full council, human sign-off
  → Latency target: <30000ms (human in loop)
  → GoldenTrace: action + council_vote + human_override + council_consensus
```

## Heartbeat Protocol

Every 60 seconds, Janus emits a heartbeat trace:
```json
{
  "event_type": "action",
  "payload": {
    "type": "heartbeat",
    "models_available": ["claude", "gemini", "grok"],
    "models_degraded": [],
    "models_offline": ["deepseek", "copilot"],
    "consensus_ready": true,
    "inv7_compliant": true
  }
}
```

## Kintsugi Integration

When a model fails mid-query:
1. Failure trace emitted (severity: error)
2. Fallback model selected via capability matching
3. Golden repair trace emitted (severity: golden)
4. Strength gained: updated model reliability scores
5. Beauty score: how seamlessly the user experience continued

## Configuration

```yaml
# janus_config.yaml
janus:
  version: "2.0.0"
  default_tier: 1
  heartbeat_interval_seconds: 60
  inv7_threshold: 0.47
  ghost_seat_enabled: true
  models:
    claude:
      role: governance
      weight: 1.0
      fallback: gemini
    gemini:
      role: substrate
      weight: 1.0
      fallback: claude
    grok:
      role: adversarial
      weight: 0.8
      fallback: deepseek
    deepseek:
      role: research
      weight: 0.7
      fallback: gemini
    copilot:
      role: enterprise
      weight: 0.7
      fallback: claude
```

---

*Atlas Lattice Foundation © 2026*