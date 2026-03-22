// src/universal/mod.rs
// Universal AI CLI Abstraction Layer — Aluminum OS
//
// Exposes a provider-agnostic interface for routing AI CLI requests through
// every major AI provider (GitHub Copilot, Claude, Gemini, OpenAI, Grok, DeepSeek)
// with NPFM enforcement and GoldenTrace HITL provenance stamping, plus the
// Janus v2 multi-agent orchestration protocol for tiered council consensus.
//
// Module structure:
//   ai_cli        — provider enum, uniform request/response types, per-provider adapters
//   model_router  — NPFM scoring, GoldenTrace generation, single-provider routing
//   janus         — Janus v2: tiered routing, council consensus, INV-7, Kintsugi repair

pub mod ai_cli;
pub mod janus;
pub mod model_router;

// Re-export the most-used types for ergonomic `use uws::universal::*`
pub use ai_cli::{AiCliAdapter, AiCliRequest, AiCliResponse, AiProvider, adapter_for};
pub use janus::{
    ConsensusResult, ConsensusVote, CouncilMember, CouncilRole, HeartbeatTrace,
    Inv7Guard, JanusOutcome, JanusRouter, KintsugiSeam, ModelStatus, QueryTier,
    INV7_DOMINANCE_CAP, default_council,
};
pub use model_router::{
    GoldenTrace, ModelRouter, NpfmScore, RouterConfig, RouterDecision, NPFM_THRESHOLD,
};
