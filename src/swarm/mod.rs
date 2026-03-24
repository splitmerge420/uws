// src/swarm/mod.rs
// Swarm Oversight Subsystem for Aluminum OS / uws
//
// Implements the "Swarm Commander" pattern: a single human overseer can
// review, approve, and cryptographically sign off on a *batch* of AI
// operations or drone actions.
//
//   `uws swarm review --batch=10`
//
// Design principle: "Augment human jobs, do not replace them.
//   In the event of replacement, transition workers into AI oversight roles."
//
// Council Session: 2026-03-21
// Authority: Dave Sheldon (INV-5)
// Invariants Enforced: INV-1 (Sovereignty), INV-2 (Consent), INV-3 (Audit)

pub mod batch_oversight;
