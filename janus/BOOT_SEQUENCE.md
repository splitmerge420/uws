---
title: "Janus Boot Sequence"
version: "2.0.0"
date: "2026-03-20"
---

# Janus Boot Sequence

## Initialization Order

```
1. LOAD constitutional invariants (INV-1 through INV-30)
   → Source: aluminum-os/kintsugi/policies/constitutional_audit.rego
   → GoldenTrace: action (type: boot_invariants_loaded)

2. INITIALIZE GoldenTrace emitter
   → Hash chain starts with GENESIS block
   → GoldenTrace: action (type: trace_chain_initialized)

3. PROBE available models
   → For each configured model: send heartbeat ping
   → Record available/degraded/offline status
   → GoldenTrace: action (type: model_probe, payload: {status per model})

4. VERIFY INV-7 compliance
   → Check that available models can form valid consensus
   → Minimum: 2 models required for Tier 2+
   → GoldenTrace: invariant_check (invariants: [INV-7])

5. LOAD SHELDONBRAIN context (if available)
   → Retrieve recent golden seams for learning
   → Load sphere routing preferences from past sessions
   → GoldenTrace: action (type: sheldonbrain_context_loaded)

6. EMIT boot complete heartbeat
   → Full status report with model availability
   → GoldenTrace: action (type: boot_complete)

7. BEGIN accepting queries
   → Router active, traces flowing
```

## Failure During Boot

If any step fails:
1. Emit failure trace with step number
2. Attempt golden repair (retry with degraded config)
3. If repair succeeds: emit golden_seam trace, continue boot
4. If repair fails: emit critical trace, enter safe mode (Tier 1 only, single model)

## Safe Mode

In safe mode:
- Only Tier 1 queries accepted
- Single model routing (best available)
- Heartbeat interval reduced to 15 seconds
- Recovery attempts every 60 seconds
- All actions logged with severity: warning

---

*Atlas Lattice Foundation © 2026*