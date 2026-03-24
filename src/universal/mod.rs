// src/universal/mod.rs
// Aluminum OS — Universal Sub-modules
//
// This crate hosts cross-cutting concerns that sit above individual
// provider drivers (ms_graph, apple, android_chrome) but below the
// constitutional governance layer.
//
// Council approval: 2026-03-20 (Janus v2 spec adopted)

/// Janus v2 — Constitutional Multi-Agent Router
///
/// Manages query routing between council members (Claude, Gemini, Grok,
/// DeepSeek, Copilot, Ghost Seat), enforces INV-7 (47% dominance cap),
/// emits GoldenTrace events at every decision point, and handles Kintsugi
/// failure recovery.
///
/// Exposes: `JanusRouter`, `RoutingTier`, `ModelVote`, `GoldenTrace`,
///          `HeartbeatTrace`, `KintsugiRepair`.
/// Enforces: INV-7 (Vendor Balance), INV-8 (Human Override), INV-3 (Audit)
pub mod janus;

/// Model Router — routing decisions and content-digest helpers
///
/// Low-level routing logic: selects the best council member for a given
/// query, computes request digests for deduplication and caching, and
/// maintains per-model reliability scores updated by Kintsugi.
///
/// Exposes: `ModelRouter`, `compute_digest`, `compute_digest_from_str`.
pub mod model_router;
