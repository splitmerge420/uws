// src/lib.rs
// Aluminum OS — Universal Workspace Library
//
// Module declarations for the UWS constitutional governance layer.
// The binary entry point is src/main.rs (the CLI).
// This file exposes library modules for testing and external consumption.
//
// Council Session: 2026-03-20
// Authority: Dave Sheldon (INV-5)

// ─── Constitutional Governance Layer ─────────────────────────────
// These modules enforce the 39 Constitutional Invariants at runtime.

/// Constitutional enforcement engine — runtime invariant checking.
/// Checks: INV-1 (Sovereignty), INV-2 (Consent), INV-3 (Audit),
///         INV-6 (Provider Abstraction), INV-7 (Vendor Balance),
///         INV-11 (Encryption at Rest)
pub mod constitutional_engine;

/// Council GitHub operations client — constitutional wrapper around GitHub API.
/// Blocks destructive operations. Enforces data classification (Class A/B/C).
/// Appends provenance trailers to all commits.
/// Enforces: INV-1, INV-2, INV-3, INV-5, INV-6, INV-7, INV-11, INV-35
pub mod council_github_client;

/// Append-only SHA3-256 hash-chained audit log.
/// NO modify/delete API exists on this struct — by design.
/// verify_chain() walks every link to detect tampering.
/// Enforces: INV-3 (Audit Trail), INV-35 (Fail-Closed)
pub mod audit_chain;

// ─── Human-In-The-Loop (HITL) Layer ──────────────────────────
// Codifies the "Augment, Don't Replace" labor principle.
// Workers displaced by AI are retrained as provenance reviewers
// and medical oversight professionals rather than eliminated.

/// HITL subsystem — two-tier human oversight for AI outputs.
///
/// - `medical`    : Licensed professional review (NPI verification required).
/// - `provenance` : Open-access review — no license required; democratized
///                  AI-era job class paid per sign-off.
///
/// Enforces: INV-1 (Sovereignty), INV-2 (Consent), INV-3 (Audit)
pub mod hitl;

// ─── Swarm Oversight Layer ────────────────────────────────────
// Implements the "Swarm Commander" pattern: one human governs
// many AI agents / drone operations at batch scale.

/// Swarm Commander batch-oversight module.
///
/// A single human can review, approve, and cryptographically sign off
/// on a batch of AI operations (`uws swarm review --batch=10`), serving
/// as the primary retraining pathway for workers displaced by automation.
///
/// Enforces: INV-1 (Sovereignty), INV-2 (Consent), INV-3 (Audit)
pub mod swarm;
