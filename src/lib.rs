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

// ─── Flourishing Metrics Layer ────────────────────────────────────────────────

/// Net-Positive Flourishing Metric (NPFM) telemetry.
/// Replaces throughput KPIs with human-flourishing indicators.
/// Enforces the Fiduciary Duty Against Busywork.
/// Invariants: INV-5 (Fiduciary Authority), INV-2 (Consent)
pub mod telemetry;

// ─── Embodiment Protocol ──────────────────────────────────────────────────────

/// Embodiment Protocol — spatial (metaverse) and physical (robotic) presence.
/// All proposals are gated by NPFM and human fiduciary approval.
/// Physical proposals additionally require a high SimulationFidelityScore
/// proving metaverse-trained superiority before manufacture is permitted.
/// Invariants: INV-1 (Sovereignty), INV-5 (Fiduciary Authority)
pub mod embodiment;
