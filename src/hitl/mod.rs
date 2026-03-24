// src/hitl/mod.rs
// Human-In-The-Loop (HITL) Subsystem for Aluminum OS / uws
//
// Provides two tiers of human oversight:
//
//   1. `medical`    — Licensed professional review (Amazon One Medical,
//                     requires NPI verification).  All outputs are gated
//                     behind credential checks before they leave the system.
//
//   2. `provenance` — Open-access provenance validation.  No professional
//                     license required.  Standard humans are hired to review
//                     and sign off on AI outputs, creating a new democratized
//                     job class in the post-AI economy.
//
// Design principle: "Augment human jobs, do not replace them.
//   In the event of replacement, transition workers into AI oversight roles."
//
// Council Session: 2026-03-21
// Authority: Dave Sheldon (INV-5)
// Invariants Enforced: INV-1 (Sovereignty), INV-2 (Consent), INV-3 (Audit)

pub mod medical;
pub mod provenance;
