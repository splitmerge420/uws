# Janus Checkpoint v2 Environmental Continuity Protocol

> **UWS Operational Mirror** — Constitutional spine maintained in [aluminum-os/janus/JANUS_CHECKPOINT_V2_SPEC.md](https://github.com/aluminum-os/janus/blob/main/JANUS_CHECKPOINT_V2_SPEC.md)

**Atlas Lattice Foundation** | Janus v2 Specification

---

## Overview

The Janus Checkpoint v2 Environmental Continuity Protocol defines the operational framework for maintaining system state coherence across distributed lattice nodes. This specification establishes four distinct system layers that coordinate checkpoint creation, state validation, and recovery procedures.

---

## System Layers

### 1. BOOT Layer

The BOOT layer handles initial system initialization and checkpoint recovery from persistent storage.

**Responsibilities:**
- Load persisted checkpoint state from storage
- Validate checkpoint integrity and signatures
- Initialize core system components
- Establish baseline operational state

**Key Operations:**
- `bootstrap()` - Initialize system from checkpoint
- `validate_checkpoint()` - Verify checkpoint authenticity
- `restore_state()` - Reconstruct operational state

---

### 2. WARM Layer

The WARM layer manages steady-state operations and incremental checkpoint updates.

**Responsibilities:**
- Monitor system health metrics
- Collect operational telemetry
- Generate incremental checkpoints
- Maintain state consistency

**Key Operations:**
- `collect_metrics()` - Gather system telemetry
- `create_checkpoint()` - Generate incremental checkpoint
- `validate_state()` - Verify operational consistency

---

### 3. HOT Layer

The HOT layer handles high-frequency state updates and real-time synchronization.

**Responsibilities:**
- Process rapid state changes
- Coordinate distributed updates
- Maintain consensus across nodes
- Handle transient state conflicts

**Key Operations:**
- `sync_state()` - Synchronize state across nodes
- `resolve_conflict()` - Handle state divergence
- `broadcast_update()` - Propagate state changes

---

### 4. HEARTBEAT Layer

The HEARTBEAT layer provides continuous liveness monitoring and failure detection.

**Responsibilities:**
- Emit periodic liveness signals
- Detect node failures
- Trigger recovery procedures
- Maintain node registry

**Key Operations:**
- `emit_heartbeat()` - Send liveness signal
- `detect_failure()` - Identify failed nodes
- `initiate_recovery()` - Begin recovery sequence

---

## Data Schemas

### PulseReport

The PulseReport schema encapsulates health and state information emitted by nodes.

```json
{
  "pulse_id": "string (UUID v4)",
  "node_id": "string (identifier)",
  "timestamp": "integer (Unix milliseconds)",
  "layer": "enum (BOOT|WARM|HOT|HEARTBEAT)",
  "status": "enum (healthy|degraded|critical)",
  "metrics": {
    "cpu_usage": "float (0-100)",
    "memory_usage": "float (0-100)",
    "queue_depth": "integer",
    "checkpoint_age_ms": "integer"
  },
  "state_hash": "string (SHA-256 hex)",
  "signature": "string (base64 encoded)"
}
```

### TaskQueueItem

The TaskQueueItem schema represents work units in the distributed task queue.

```json
{
  "task_id": "string (UUID v4)",
  "queue_id": "string (identifier)",
  "priority": "integer (0-255)",
  "created_at": "integer (Unix milliseconds)",
  "scheduled_for": "integer (Unix milliseconds)",
  "payload": "object (task-specific data)",
  "retry_count": "integer",
  "max_retries": "integer",
  "status": "enum (pending|processing|completed|failed)",
  "assigned_node": "string (node identifier or null)",
  "result": "object (optional, populated on completion)"
}
```

---

## Failure Modes

| Failure Mode | Layer | Detection | Recovery | RTO | RPO |
|---|---|---|---|---|---|
| Node Crash | HEARTBEAT | Missed heartbeats (3x interval) | Failover to replica | 30s | 5s |
| State Divergence | HOT | Hash mismatch in sync | Consensus resolution | 10s | 1s |
| Checkpoint Corruption | BOOT | Signature validation failure | Rollback to previous | 60s | 30s |
| Queue Overflow | WARM | Queue depth threshold exceeded | Backpressure + spillover | 5s | 0s |
| Network Partition | HOT | Heartbeat timeout + quorum loss | Partition isolation | 45s | 10s |
| Storage Failure | BOOT | I/O error on checkpoint read | Attempt replica storage | 120s | 60s |
| Consensus Timeout | HOT | No quorum achieved | Abort operation, retry | 15s | 2s |
| Memory Exhaustion | WARM | Memory usage > 95% | Trigger GC + checkpoint | 20s | 5s |

---

## Notion Pointer Map

Central documentation and operational runbooks are maintained in Notion:

| Component | Notion ID | Purpose |
|---|---|---|
| **Hub** | `3290c1de-73d9-8189-991d-c47dbda016e0` | Central coordination and overview |
| **Boot** | `3290c1de-73d9-817b-990e-e23fe9b48ab3` | BOOT layer procedures and troubleshooting |
| **Pulse** | `3290c1de-73d9-81e8-a4e1-c24cca262026` | PulseReport schema and monitoring |
| **Queue** | `3290c1de-73d9-81c8-a68b-c28cd36ac863` | TaskQueueItem management and operations |

---

## Implementation Notes

1. **Checkpoint Frequency**: WARM layer should generate checkpoints every 30 seconds under normal conditions
2. **Heartbeat Interval**: HEARTBEAT layer should emit signals every 5 seconds
3. **State Hash Algorithm**: Use SHA-256 for all state hashing
4. **Signature Scheme**: Use Ed25519 for checkpoint signatures
5. **Quorum Size**: Require 2/3 + 1 nodes for consensus decisions

---

## Version History

- **v2.0** (2026-03-20) - Initial specification for Atlas Lattice Foundation

---

**Janus v2 Spec (uws mirror)** — Constitutional Scribe — Atlas Lattice Foundation — 2026-03-20