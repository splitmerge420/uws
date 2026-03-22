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

// ─── Universal I/O Layer ──────────────────────────────────────────────────
// SaaS-Bypass Stream module: converts proprietary Google/Microsoft/Apple
// documents into the locally-owned UniversalDocument format.
// Makes every SaaS provider a "dumb pipe"; the user's OS is the source of truth.
// Implements the CognitiveDust / Dumb-Pipes vision from COPILOT_HORIZON_MANIFEST.md.
// Enforces: INV-1 (Sovereignty), INV-3 (Audit Trail), INV-6 (Provider Abstraction)

/// Provider-neutral document extraction and Markdown conversion.
/// Accepts raw API response content from any connector and returns a
/// locally-owned `UniversalDocument` (Markdown body + JSON frontmatter).
pub mod universal_io;

// ─── Phase 1: Sovereign Memory ────────────────────────────────────────────
// These modules implement the "data lives on your hardware" guarantee.

/// Local-first personal knowledge graph.
/// Every document, email, event, note, and task that passes through a `uws`
/// provider connector is written here.  Full-text keyword search, tag/type/
/// provider filtering, and JSON export/import — all computed locally.
/// Enforces: INV-1 (Sovereignty), INV-3 (Audit Trail)
pub mod local_noosphere;

/// SaaS Data Repatriation Sweeper — the CognitiveDust engine.
/// Converts documents from Google Drive, OneDrive, and Apple Notes into
/// open-standard Markdown/JSON via `universal_io` connectors and writes
/// them into the `LocalNoosphere`.  SaaS apps become write-once inboxes;
/// the OS is the permanent record.
/// Enforces: INV-1 (Sovereignty), INV-2 (Consent), INV-3 (Audit Trail)
pub mod cognitive_dust;

// ─── Phase 3: Fusion Engine & Agentic Autonomy ───────────────────────────

/// Janus multi-model AI omni-router.
/// Scores available inference providers (Claude, Gemini, Grok, GPT-4, …)
/// against a configurable cost/latency/quality profile and returns an
/// ordered `RouteDecision` — primary model + fallback chain.
/// Enforces: INV-7 (Vendor Balance — no single provider privileged)
pub mod janus;

// ─── Phase 4: Extreme Interoperability ───────────────────────────────────

/// FrictionlessCal unified calendar engine.
/// Merges Google Calendar, Outlook Calendar, and Apple Calendar events into
/// a single conflict-resolved, timezone-normalised `UnifiedTimeline`.
/// Handles cross-provider duplicate detection and configurable merge policy.
/// Enforces: INV-1 (Sovereignty), INV-6 (Provider Abstraction)
pub mod frictionless_cal;
