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

// ─── Swarm Commander ──────────────────────────────────────────────
// Implements `uws swarm review --batch=<n>`: batch PR/dependency review
// with a single NPFM dry-run check and cryptographic sign-off.
// CLI entry point: handle_swarm_command(args)

/// Swarm Commander — batch PR / dependency review with NPFM gating.
/// Exposes: `run_swarm_review`, `handle_swarm_command`, `format_review_result`
/// Enforces: INV-2 (Consent Gating), INV-3 (Audit Trail)
pub mod swarm;

// ─── Provenance Ledger ────────────────────────────────────────────
// Maps the philosophical ProvenanceTrailer to the concrete GoldenTrace
// git commit trailer.  Primary API: append_golden_trace_to_commit()

/// Provenance ledger — ProvenanceTrailer → GoldenTrace mapping.
/// Exposes: `append_golden_trace_to_commit`, `GoldenTrace`, `ProvenanceTrailer`
/// Enforces: INV-3 (Audit Trail), INV-5 (Provenance)
pub mod ledger;

// ─── Telemetry / Systemic Linter ──────────────────────────────────
// Static analysis pass that flags extractive / busywork anti-patterns and
// reports NPFM score degradations.  Primary API: scan_for_busywork()

/// Systemic linter — scan source files for busywork anti-patterns.
/// Exposes: `scan_for_busywork`, `collect_warnings`, `LintWarning`
/// Enforces: INV-1 (Anti-Busywork / Human Flourishing)
pub mod telemetry;
