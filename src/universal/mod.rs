// src/universal/mod.rs
// Universal AI CLI Abstraction Layer — Aluminum OS
//
// Exposes a provider-agnostic interface for routing AI CLI requests through
// every major AI provider (GitHub Copilot, Claude, Gemini, OpenAI) with
// NPFM enforcement and GoldenTrace HITL provenance stamping.
//
// Module structure:
//   ai_cli        — provider enum, uniform request/response types, per-provider adapters
//   model_router  — NPFM scoring, GoldenTrace generation, request routing
//
// Quick start:
//
// ```rust
// use uws::universal::{
//     ai_cli::{AiCliRequest, AiProvider, adapter_for},
//     model_router::{ModelRouter, RouterConfig},
// };
//
// let mut router = ModelRouter::new(RouterConfig::default());
// let req = AiCliRequest::new(AiProvider::Claude, "help me write better tests");
// let decision = router.route(req, |r| {
//     let adapter = adapter_for(&r.provider);
//     // In production: send adapter.build_request_body(r) via HTTP and parse response.
//     // Here we return a stub.
//     Ok(uws::universal::ai_cli::AiCliResponse {
//         provider: r.provider.clone(),
//         model_used: "claude-opus-4-5".to_string(),
//         content: "Use table-driven tests…".to_string(),
//         raw_fields: std::collections::BTreeMap::new(),
//         latency_ms: 0,
//         truncated: false,
//     })
// });
// assert!(decision.is_allowed());
// ```

pub mod ai_cli;
pub mod model_router;

// Re-export the most-used types for ergonomic `use uws::universal::*`
pub use ai_cli::{AiCliAdapter, AiCliRequest, AiCliResponse, AiProvider, adapter_for};
pub use model_router::{
    GoldenTrace, ModelRouter, NpfmScore, RouterConfig, RouterDecision, NPFM_THRESHOLD,
};
