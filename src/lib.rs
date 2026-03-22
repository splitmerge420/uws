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

/// Universal AI CLI abstraction layer + Janus v2 multi-agent protocol.
/// Provides provider-agnostic adapters for GitHub Copilot, Claude, Gemini, OpenAI,
/// Grok, and DeepSeek, the ModelRouter (NPFM ≥ 0.7 + GoldenTrace HITL provenance),
/// and JanusRouter (tiered routing, INV-7 council consensus, Kintsugi repair).
/// Enforces: INV-6 (Provider Abstraction), INV-7 (Vendor Balance), INV-1 (Sovereignty)
pub mod universal;

/// GitHub as a first-class uws provider.
/// Exposes GitHub Issues, Pull Requests, Actions, Releases, Code Search,
/// GitHub Models (AI inference), and Notifications through the standard
/// uws command grammar: `uws github <resource> <method> [--params] [--json]`.
/// Authentication: GITHUB_TOKEN (injected automatically in GitHub Actions,
/// forwarded by the `gh` CLI extension, or set via `uws github auth`).
pub mod github_provider;
