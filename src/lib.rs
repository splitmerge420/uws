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

/// Local-first personal knowledge graph.
pub mod local_noosphere;

/// SaaS Data Repatriation Sweeper — the CognitiveDust engine.
pub mod cognitive_dust;

// ─── Phase 3: Fusion Engine & Agentic Autonomy ───────────────────────────

/// Janus multi-model AI omni-router.
pub mod janus;

// ─── Phase 4: Extreme Interoperability ───────────────────────────────────

/// FrictionlessCal unified calendar engine.
pub mod frictionless_cal;

/// GitHub REST API provider.
/// Exposes repos, issues, PRs, releases, Actions, search, and more via
/// `uws github <resource> <method>`.  Auth: GITHUB_TOKEN or UWS_GITHUB_TOKEN.
/// Enforces: INV-6 (Provider Abstraction), INV-7 (Vendor Balance)
pub mod github_provider;

// ─── Novel Inventions: 20 New Integrations (Session 2026-03-23) ──────────

/// Slack REST API provider.
pub mod slack_provider;

/// Linear.app GraphQL API provider.
pub mod linear_provider;

/// Notion REST API provider.
pub mod notion_provider;

/// Figma REST API provider.
pub mod figma_provider;

/// Stripe payment infrastructure provider.
pub mod stripe_provider;

/// Cross-provider fan-out semantic search aggregator.
pub mod cross_search;

/// Provider API health monitor + rate-limit sentinel.
pub mod provider_health;

/// LLM context window JSON optimizer and truncator.
pub mod context_compressor;

/// Local encrypted versioned prompt store.
pub mod prompt_vault;

/// Unified cross-provider activity/changelog stream.
pub mod activity_stream;

/// Cross-provider document diff engine (LCS-based).
pub mod diff_engine;

/// YAML-defined multi-step automation pipeline composer.
pub mod workflow_composer;

/// Data provenance and lineage tracker (DAG).
pub mod data_lineage;

/// Multi-identity persona manager (work / personal / freelance).
pub mod persona_manager;
