// src/universal/janus.rs
// Janus v2 — Constitutional Multi-Agent Router
//
// Implements the Janus v2 protocol as specified in janus/JANUS_V2_SPEC.md.
// Manages query routing between council members, enforces INV-7 (47%
// dominance cap), emits GoldenTrace events at every decision point, and
// handles Kintsugi failure recovery.
//
// Tiers:
//   Tier 1 — single model, latency < 500 ms
//   Tier 2 — 2-3 models with synthesis, latency < 3 000 ms
//   Tier 3 — full council + human sign-off, latency < 30 000 ms
//
// Council Session: 2026-03-20
// Spec: janus/JANUS_V2_SPEC.md

use crate::universal::model_router::{compute_digest_from_str, ModelConfig, ModelRouter};

// ─── Routing tier ─────────────────────────────────────────────────────────────

/// Tier determines how many council members participate in a round.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingTier {
    /// Simple query — single model, ≤ 500 ms target.
    Tier1,
    /// Complex query — 2–3 models, synthesis required, ≤ 3 000 ms target.
    Tier2,
    /// Critical / irreversible — full council + human sign-off, ≤ 30 s target.
    Tier3,
}

impl RoutingTier {
    /// Latency target in milliseconds for this tier.
    pub fn latency_target_ms(&self) -> u64 {
        match self {
            RoutingTier::Tier1 => 500,
            RoutingTier::Tier2 => 3_000,
            RoutingTier::Tier3 => 30_000,
        }
    }

    /// Human sign-off required for this tier?
    pub fn requires_human_signoff(&self) -> bool {
        matches!(self, RoutingTier::Tier3)
    }
}

impl std::fmt::Display for RoutingTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RoutingTier::Tier1 => write!(f, "Tier1"),
            RoutingTier::Tier2 => write!(f, "Tier2"),
            RoutingTier::Tier3 => write!(f, "Tier3"),
        }
    }
}

// ─── Council vote ─────────────────────────────────────────────────────────────

/// A single model's vote in a Tier 2 or Tier 3 consensus round.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelVote {
    /// Model that cast this vote (e.g. "claude", "gemini").
    pub model: String,
    /// The model's chosen answer / recommendation.
    pub answer: String,
    /// Confidence in [0.0, 1.0].
    pub confidence: f64,
    /// Weighted vote value = confidence × model effective_weight.
    pub weighted: f64,
}

// ─── GoldenTrace ─────────────────────────────────────────────────────────────

/// Type of GoldenTrace event emitted by the Janus router.
#[derive(Debug, Clone, PartialEq)]
pub enum GoldenTraceKind {
    /// An action was routed to a model (Tier 1 result or partial Tier 2/3).
    Action,
    /// A council vote was collected (Tier 2/3).
    CouncilVote,
    /// Council reached consensus (Tier 2/3).
    CouncilConsensus,
    /// A human override was recorded (Tier 3).
    HumanOverride,
    /// A Kintsugi repair event — model failure followed by recovery.
    KintsugiRepair,
    /// Periodic heartbeat trace.
    Heartbeat,
}

impl std::fmt::Display for GoldenTraceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GoldenTraceKind::Action           => write!(f, "action"),
            GoldenTraceKind::CouncilVote      => write!(f, "council_vote"),
            GoldenTraceKind::CouncilConsensus => write!(f, "council_consensus"),
            GoldenTraceKind::HumanOverride    => write!(f, "human_override"),
            GoldenTraceKind::KintsugiRepair   => write!(f, "kintsugi_repair"),
            GoldenTraceKind::Heartbeat        => write!(f, "heartbeat"),
        }
    }
}

/// A structured audit event emitted at every Janus decision point.
///
/// All traces are appended to the `JanusRouter`'s internal trace log.
/// In production the log is forwarded to the constitutional `AuditChain`.
#[derive(Debug, Clone)]
pub struct GoldenTrace {
    /// Unique trace identifier — FNV-1a digest of (round_id + event_kind + timestamp).
    pub trace_id: String,
    /// Round this event belongs to.
    pub round_id: String,
    /// Event classification.
    pub kind: GoldenTraceKind,
    /// Model that produced the event (empty for consensus/heartbeat traces).
    pub model: String,
    /// Human-readable summary of the event.
    pub summary: String,
    /// INV-7 compliant at the time of this event?
    pub inv7_ok: bool,
    /// ISO 8601 timestamp (stub — real impl uses system time).
    pub timestamp: String,
}

impl GoldenTrace {
    fn new(
        round_id: &str,
        kind: GoldenTraceKind,
        model: &str,
        summary: &str,
        inv7_ok: bool,
    ) -> Self {
        let raw = format!("{}-{}-{}", round_id, kind, model);
        let trace_id = format!("{:016x}", compute_digest_from_str(&raw));
        Self {
            trace_id,
            round_id: round_id.to_string(),
            kind,
            model: model.to_string(),
            summary: summary.to_string(),
            inv7_ok,
            timestamp: "2026-03-20T00:00:00Z".to_string(),
        }
    }
}

// ─── Kintsugi repair record ───────────────────────────────────────────────────

/// Records a model failure and its repair — "strength through scars".
#[derive(Debug, Clone)]
pub struct KintsugiRepair {
    /// The model that failed.
    pub failed_model: String,
    /// The fallback model that stepped in.
    pub repair_model: String,
    /// Brief description of the failure.
    pub failure_reason: String,
    /// Beauty score: how seamlessly the user experience continued [0.0, 1.0].
    pub beauty_score: f64,
    /// Updated reliability score for the failed model after repair.
    pub new_reliability: f64,
}

// ─── Janus routing result ────────────────────────────────────────────────────

/// The result of a Janus routing round.
#[derive(Debug, Clone)]
pub struct JanusResult {
    /// Round identifier.
    pub round_id: String,
    /// Routing tier used.
    pub tier: RoutingTier,
    /// Primary model that answered (or consensus model for Tier 2/3).
    pub primary_model: String,
    /// Final answer / response from the council.
    pub answer: String,
    /// All votes cast (empty for Tier 1).
    pub votes: Vec<ModelVote>,
    /// INV-7 compliant?
    pub inv7_ok: bool,
    /// Human sign-off obtained (for Tier 3)?
    pub human_signoff: bool,
    /// All GoldenTrace events produced during this round.
    pub traces: Vec<GoldenTrace>,
    /// Kintsugi repair record, if any model failed during this round.
    pub repair: Option<KintsugiRepair>,
}

// ─── Heartbeat ────────────────────────────────────────────────────────────────

/// Periodic health snapshot emitted every 60 seconds (spec §Heartbeat).
#[derive(Debug, Clone)]
pub struct HeartbeatTrace {
    /// Models that responded to the last probe.
    pub models_available: Vec<String>,
    /// Models that responded slowly (> 2 × latency target).
    pub models_degraded: Vec<String>,
    /// Models that did not respond.
    pub models_offline: Vec<String>,
    /// At least 2 council members available for consensus?
    pub consensus_ready: bool,
    /// INV-7 compliant with current available set?
    pub inv7_compliant: bool,
}

// ─── JanusRouter ─────────────────────────────────────────────────────────────

/// Janus v2 multi-agent router.
///
/// # Usage
/// ```
/// use uws::universal::janus::{JanusRouter, RoutingTier};
///
/// let mut router = JanusRouter::new_default();
/// let result = router.route("What is 2 + 2?", RoutingTier::Tier1);
/// assert!(!result.primary_model.is_empty());
/// assert!(result.inv7_ok);
/// ```
pub struct JanusRouter {
    /// Underlying model router (holds per-model weights + reliability).
    pub model_router: ModelRouter,
    /// Accumulated GoldenTrace log — append-only.
    traces: Vec<GoldenTrace>,
    /// Monotonically increasing round counter for stable round IDs.
    round_counter: u64,
    /// INV-7 dominance cap (default: 0.47).
    pub inv7_threshold: f64,
    /// Ghost Seat enabled (Sphere 144)?
    pub ghost_seat_enabled: bool,
}

impl JanusRouter {
    /// Create a JanusRouter with the default five-member council.
    pub fn new_default() -> Self {
        let model_router = ModelRouter::default_council();
        let inv7_threshold = model_router.inv7_threshold;
        Self {
            model_router,
            traces: Vec::new(),
            round_counter: 0,
            inv7_threshold,
            ghost_seat_enabled: true,
        }
    }

    /// Create a JanusRouter from a custom ModelRouter.
    pub fn new(model_router: ModelRouter) -> Self {
        let inv7_threshold = model_router.inv7_threshold;
        Self {
            model_router,
            traces: Vec::new(),
            round_counter: 0,
            inv7_threshold,
            ghost_seat_enabled: true,
        }
    }

    // ── Round-ID generation ──────────────────────────────────────────────────

    fn next_round_id(&mut self, query: &str) -> String {
        self.round_counter += 1;
        let raw = format!("round-{}-{}", self.round_counter, query);
        format!("{:016x}", compute_digest_from_str(&raw))
    }

    // ── Public routing entry point ───────────────────────────────────────────

    /// Route a query to the council using the specified tier.
    ///
    /// - Tier 1: single-model fast path.
    /// - Tier 2: 2–3 models with weighted-average synthesis.
    /// - Tier 3: full council, requires human sign-off stub.
    pub fn route(&mut self, query: &str, tier: RoutingTier) -> JanusResult {
        let round_id = self.next_round_id(query);
        let query_digest = compute_digest_from_str(query);

        match tier {
            RoutingTier::Tier1 => self.route_tier1(&round_id, query, query_digest),
            RoutingTier::Tier2 => self.route_tier2(&round_id, query, query_digest),
            RoutingTier::Tier3 => self.route_tier3(&round_id, query, query_digest),
        }
    }

    // ── Tier 1 ───────────────────────────────────────────────────────────────

    fn route_tier1(&mut self, round_id: &str, query: &str, query_digest: u64) -> JanusResult {
        let (primary, inv7_ok) = self.model_router.select_primary(query_digest);

        // If primary is "none", attempt Kintsugi repair
        let (primary, inv7_ok, repair) = if primary == "none" {
            let repair = KintsugiRepair {
                failed_model: "unknown".into(),
                repair_model: "none".into(),
                failure_reason: "no models available".into(),
                beauty_score: 0.0,
                new_reliability: 0.0,
            };
            ("none".to_string(), false, Some(repair))
        } else {
            (primary, inv7_ok, None)
        };

        let answer = if primary == "none" {
            "[Janus: degraded — no models available]".to_string()
        } else {
            format!("[{}] {}", primary, query)
        };

        let trace = GoldenTrace::new(
            round_id,
            GoldenTraceKind::Action,
            &primary,
            &format!("Tier1 route: {} → {primary}", query),
            inv7_ok,
        );
        self.traces.push(trace.clone());

        JanusResult {
            round_id: round_id.to_string(),
            tier: RoutingTier::Tier1,
            primary_model: primary,
            answer,
            votes: vec![],
            inv7_ok,
            human_signoff: false,
            traces: vec![trace],
            repair,
        }
    }

    // ── Tier 2 ───────────────────────────────────────────────────────────────

    fn route_tier2(&mut self, round_id: &str, query: &str, query_digest: u64) -> JanusResult {
        // Collect votes from all available models (≥ 2 expected for Tier 2).
        let votes = self.collect_votes(round_id, query, query_digest);
        let inv7_ok = self.check_inv7(&votes);

        let mut traces: Vec<GoldenTrace> = votes.iter().map(|v| {
            GoldenTrace::new(
                round_id,
                GoldenTraceKind::CouncilVote,
                &v.model,
                &format!("vote confidence={:.2}", v.confidence),
                inv7_ok,
            )
        }).collect();

        // Weighted-majority synthesis
        let (consensus_model, answer) = self.synthesise(&votes, query);

        let consensus_trace = GoldenTrace::new(
            round_id,
            GoldenTraceKind::CouncilConsensus,
            &consensus_model,
            &format!("Tier2 consensus from {} votes", votes.len()),
            inv7_ok,
        );
        traces.push(consensus_trace.clone());
        self.traces.extend(traces.clone());

        JanusResult {
            round_id: round_id.to_string(),
            tier: RoutingTier::Tier2,
            primary_model: consensus_model,
            answer,
            votes,
            inv7_ok,
            human_signoff: false,
            traces,
            repair: None,
        }
    }

    // ── Tier 3 ───────────────────────────────────────────────────────────────

    fn route_tier3(&mut self, round_id: &str, query: &str, query_digest: u64) -> JanusResult {
        // Full council round
        let mut result = self.route_tier2(round_id, query, query_digest);

        // Human sign-off stub — in production this blocks on a callback
        let override_trace = GoldenTrace::new(
            round_id,
            GoldenTraceKind::HumanOverride,
            "human",
            "Tier3 human sign-off obtained (stub)",
            result.inv7_ok,
        );
        result.traces.push(override_trace.clone());
        self.traces.push(override_trace);
        result.tier = RoutingTier::Tier3;
        result.human_signoff = true;
        result
    }

    // ── Vote collection ───────────────────────────────────────────────────────

    fn collect_votes(&self, _round_id: &str, query: &str, query_digest: u64) -> Vec<ModelVote> {
        // In production this dispatches HTTP requests to each model's API.
        // Here we synthesise plausible deterministic votes for testing.
        let mut votes = Vec::new();
        let available_models: Vec<_> = self.model_router.models()
            .filter(|m| m.available)
            .collect();

        for m in available_models {
            // Deterministic confidence: FNV(model_id + query_digest) → [0.5, 1.0]
            let raw = format!("{}-{:016x}", m.id, query_digest);
            let h = compute_digest_from_str(&raw);
            let confidence = 0.5 + (h % 1000) as f64 / 2000.0;
            let weighted = confidence * m.effective_weight();

            votes.push(ModelVote {
                model: m.id.clone(),
                answer: format!("[{}] {}", m.id, query),
                confidence,
                weighted,
            });
        }
        votes
    }

    // ── Synthesis ─────────────────────────────────────────────────────────────

    fn synthesise(&self, votes: &[ModelVote], _query: &str) -> (String, String) {
        if votes.is_empty() {
            return ("none".to_string(), "[Janus: no council members available]".to_string());
        }
        // Pick the vote with the highest weighted score.
        let winner = votes.iter()
            .max_by(|a, b| a.weighted.partial_cmp(&b.weighted).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap();
        (winner.model.clone(), winner.answer.clone())
    }

    // ── INV-7 check ───────────────────────────────────────────────────────────

    fn check_inv7(&self, votes: &[ModelVote]) -> bool {
        let total: f64 = votes.iter().map(|v| v.weighted).sum();
        if total == 0.0 {
            return false;
        }
        votes.iter().all(|v| v.weighted / total <= self.inv7_threshold)
    }

    // ── Kintsugi ─────────────────────────────────────────────────────────────

    /// Record a model failure and perform Kintsugi repair.
    ///
    /// Marks the failed model unavailable, reduces its reliability score,
    /// and optionally switches to its declared fallback.
    pub fn kintsugi_repair(&mut self, failed_model: &str, reason: &str) -> KintsugiRepair {
        // Reduce reliability by 20% per failure (floor: 0.1)
        let current = self.model_router
            .get(failed_model)
            .map(|m| m.reliability)
            .unwrap_or(1.0);
        let new_reliability = (current * 0.8).max(0.1);
        self.model_router.update_reliability(failed_model, new_reliability);
        self.model_router.mark_unavailable(failed_model);

        let fallback = self.model_router
            .fallback_for(failed_model)
            .unwrap_or_else(|| "none".to_string());

        let beauty = if fallback != "none" { 0.9 } else { 0.2 };

        let repair = KintsugiRepair {
            failed_model: failed_model.to_string(),
            repair_model: fallback.clone(),
            failure_reason: reason.to_string(),
            beauty_score: beauty,
            new_reliability,
        };

        let trace = GoldenTrace::new(
            &format!("kintsugi-{failed_model}"),
            GoldenTraceKind::KintsugiRepair,
            failed_model,
            &format!("failure: {reason} → repaired by {fallback}, reliability → {new_reliability:.2}"),
            true,
        );
        self.traces.push(trace);
        repair
    }

    // ── Heartbeat ─────────────────────────────────────────────────────────────

    /// Emit a heartbeat trace (should be called every 60 s in production).
    pub fn heartbeat(&mut self) -> HeartbeatTrace {
        let available: Vec<String> = self.model_router.models()
            .filter(|m| m.available && m.reliability >= 0.5)
            .map(|m| m.id.clone())
            .collect();

        let degraded: Vec<String> = self.model_router.models()
            .filter(|m| m.available && m.reliability < 0.5)
            .map(|m| m.id.clone())
            .collect();

        let offline: Vec<String> = self.model_router.models()
            .filter(|m| !m.available)
            .map(|m| m.id.clone())
            .collect();

        let consensus_ready = available.len() >= 2;

        // INV-7: each available model's share ≤ threshold
        let total: f64 = self.model_router.total_weight();
        let inv7_compliant = total == 0.0 || self.model_router.models()
            .filter(|m| m.available)
            .all(|m| m.effective_weight() / total <= self.inv7_threshold);

        let hb = HeartbeatTrace {
            models_available: available.clone(),
            models_degraded: degraded.clone(),
            models_offline: offline.clone(),
            consensus_ready,
            inv7_compliant,
        };

        let summary = format!(
            "available={}, degraded={}, offline={}, consensus_ready={}, inv7_compliant={}",
            available.len(), degraded.len(), offline.len(), consensus_ready, inv7_compliant
        );
        let trace = GoldenTrace::new(
            "heartbeat",
            GoldenTraceKind::Heartbeat,
            "",
            &summary,
            inv7_compliant,
        );
        self.traces.push(trace);
        hb
    }

    // ── Trace log access ──────────────────────────────────────────────────────

    /// Return all emitted GoldenTrace events (immutable view).
    pub fn traces(&self) -> &[GoldenTrace] {
        &self.traces
    }

    /// Clear the trace log (for testing / log rotation).
    pub fn clear_traces(&mut self) {
        self.traces.clear();
    }
}

// ─── ModelRouter iterator helper ─────────────────────────────────────────────

impl ModelRouter {
    /// Iterate over all model configs (used by JanusRouter).
    pub fn models(&self) -> impl Iterator<Item = &ModelConfig> {
        self.models.values()
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_router() -> JanusRouter {
        JanusRouter::new_default()
    }

    // ── RoutingTier ──────────────────────────────────────────────────────────

    #[test]
    fn test_tier_latency_targets() {
        assert_eq!(RoutingTier::Tier1.latency_target_ms(), 500);
        assert_eq!(RoutingTier::Tier2.latency_target_ms(), 3_000);
        assert_eq!(RoutingTier::Tier3.latency_target_ms(), 30_000);
    }

    #[test]
    fn test_tier_human_signoff() {
        assert!(!RoutingTier::Tier1.requires_human_signoff());
        assert!(!RoutingTier::Tier2.requires_human_signoff());
        assert!(RoutingTier::Tier3.requires_human_signoff());
    }

    #[test]
    fn test_tier_display() {
        assert_eq!(RoutingTier::Tier1.to_string(), "Tier1");
        assert_eq!(RoutingTier::Tier2.to_string(), "Tier2");
        assert_eq!(RoutingTier::Tier3.to_string(), "Tier3");
    }

    // ── GoldenTrace ──────────────────────────────────────────────────────────

    #[test]
    fn test_golden_trace_kind_display() {
        assert_eq!(GoldenTraceKind::Action.to_string(), "action");
        assert_eq!(GoldenTraceKind::CouncilVote.to_string(), "council_vote");
        assert_eq!(GoldenTraceKind::CouncilConsensus.to_string(), "council_consensus");
        assert_eq!(GoldenTraceKind::HumanOverride.to_string(), "human_override");
        assert_eq!(GoldenTraceKind::KintsugiRepair.to_string(), "kintsugi_repair");
        assert_eq!(GoldenTraceKind::Heartbeat.to_string(), "heartbeat");
    }

    #[test]
    fn test_golden_trace_id_deterministic() {
        let t1 = GoldenTrace::new("round1", GoldenTraceKind::Action, "claude", "test", true);
        let t2 = GoldenTrace::new("round1", GoldenTraceKind::Action, "claude", "test", true);
        assert_eq!(t1.trace_id, t2.trace_id);
    }

    #[test]
    fn test_golden_trace_id_distinct_for_different_inputs() {
        let t1 = GoldenTrace::new("round1", GoldenTraceKind::Action, "claude", "test", true);
        let t2 = GoldenTrace::new("round2", GoldenTraceKind::Action, "gemini", "test", true);
        assert_ne!(t1.trace_id, t2.trace_id);
    }

    // ── Tier 1 routing ───────────────────────────────────────────────────────

    #[test]
    fn test_tier1_returns_result() {
        let mut r = make_router();
        let result = r.route("What is 2 + 2?", RoutingTier::Tier1);
        assert_eq!(result.tier, RoutingTier::Tier1);
        assert!(!result.primary_model.is_empty());
        assert!(!result.answer.is_empty());
        assert!(result.inv7_ok);
        assert!(!result.human_signoff);
        assert!(result.votes.is_empty());
    }

    #[test]
    fn test_tier1_emits_action_trace() {
        let mut r = make_router();
        let result = r.route("hello", RoutingTier::Tier1);
        assert_eq!(result.traces.len(), 1);
        assert_eq!(result.traces[0].kind, GoldenTraceKind::Action);
    }

    #[test]
    fn test_tier1_accumulates_to_global_trace_log() {
        let mut r = make_router();
        r.route("q1", RoutingTier::Tier1);
        r.route("q2", RoutingTier::Tier1);
        assert_eq!(r.traces().len(), 2);
    }

    #[test]
    fn test_tier1_round_ids_unique() {
        let mut r = make_router();
        let r1 = r.route("q1", RoutingTier::Tier1);
        let r2 = r.route("q1", RoutingTier::Tier1); // same query, different round
        assert_ne!(r1.round_id, r2.round_id);
    }

    // ── Tier 2 routing ───────────────────────────────────────────────────────

    #[test]
    fn test_tier2_collects_votes() {
        let mut r = make_router();
        let result = r.route("complex question", RoutingTier::Tier2);
        assert_eq!(result.tier, RoutingTier::Tier2);
        assert!(!result.votes.is_empty(), "Tier2 should collect votes");
    }

    #[test]
    fn test_tier2_emits_vote_and_consensus_traces() {
        let mut r = make_router();
        let result = r.route("complex question", RoutingTier::Tier2);
        let has_vote = result.traces.iter().any(|t| t.kind == GoldenTraceKind::CouncilVote);
        let has_consensus = result.traces.iter().any(|t| t.kind == GoldenTraceKind::CouncilConsensus);
        assert!(has_vote, "should emit CouncilVote traces");
        assert!(has_consensus, "should emit CouncilConsensus trace");
    }

    #[test]
    fn test_tier2_no_human_signoff() {
        let mut r = make_router();
        let result = r.route("complex", RoutingTier::Tier2);
        assert!(!result.human_signoff);
    }

    // ── Tier 3 routing ───────────────────────────────────────────────────────

    #[test]
    fn test_tier3_human_signoff_set() {
        let mut r = make_router();
        let result = r.route("critical", RoutingTier::Tier3);
        assert_eq!(result.tier, RoutingTier::Tier3);
        assert!(result.human_signoff, "Tier3 must obtain human sign-off");
    }

    #[test]
    fn test_tier3_emits_human_override_trace() {
        let mut r = make_router();
        let result = r.route("critical", RoutingTier::Tier3);
        let has_override = result.traces.iter().any(|t| t.kind == GoldenTraceKind::HumanOverride);
        assert!(has_override, "Tier3 must emit HumanOverride trace");
    }

    // ── INV-7 enforcement ────────────────────────────────────────────────────

    #[test]
    fn test_inv7_respected_full_council() {
        let mut r = make_router();
        let result = r.route("test", RoutingTier::Tier2);
        assert!(result.inv7_ok, "full council must be INV-7 compliant");
    }

    #[test]
    fn test_inv7_threshold_is_047() {
        let r = make_router();
        assert!((r.inv7_threshold - 0.47).abs() < 1e-9);
    }

    // ── Kintsugi ─────────────────────────────────────────────────────────────

    #[test]
    fn test_kintsugi_marks_model_unavailable() {
        let mut r = make_router();
        r.kintsugi_repair("claude", "timeout");
        assert!(!r.model_router.get("claude").unwrap().available);
    }

    #[test]
    fn test_kintsugi_reduces_reliability() {
        let mut r = make_router();
        let repair = r.kintsugi_repair("gemini", "http error");
        assert!(repair.new_reliability < 1.0);
        assert!(repair.new_reliability >= 0.1);
    }

    #[test]
    fn test_kintsugi_emits_repair_trace() {
        let mut r = make_router();
        r.kintsugi_repair("claude", "timeout");
        let has_repair = r.traces().iter().any(|t| t.kind == GoldenTraceKind::KintsugiRepair);
        assert!(has_repair);
    }

    #[test]
    fn test_kintsugi_beauty_score_with_fallback() {
        let mut r = make_router();
        // claude's fallback is gemini (available)
        let repair = r.kintsugi_repair("claude", "timeout");
        assert_eq!(repair.repair_model, "gemini");
        assert!(repair.beauty_score > 0.5, "should have high beauty when fallback is available");
    }

    #[test]
    fn test_kintsugi_beauty_score_without_fallback() {
        // Custom router: single model with no fallback
        use crate::universal::model_router::{ModelConfig, ModelRouter};
        let mut r = JanusRouter::new(ModelRouter::new(
            vec![ModelConfig::new("solo", "solo", 1.0, None)],
            0.47,
        ));
        let repair = r.kintsugi_repair("solo", "crash");
        assert_eq!(repair.repair_model, "none");
        assert!(repair.beauty_score < 0.5);
    }

    // ── Heartbeat ─────────────────────────────────────────────────────────────

    #[test]
    fn test_heartbeat_full_council() {
        let mut r = make_router();
        let hb = r.heartbeat();
        assert_eq!(hb.models_available.len(), 5);
        assert!(hb.models_degraded.is_empty());
        assert!(hb.models_offline.is_empty());
        assert!(hb.consensus_ready);
        assert!(hb.inv7_compliant);
    }

    #[test]
    fn test_heartbeat_emits_trace() {
        let mut r = make_router();
        r.heartbeat();
        let has_hb = r.traces().iter().any(|t| t.kind == GoldenTraceKind::Heartbeat);
        assert!(has_hb);
    }

    #[test]
    fn test_heartbeat_after_repair_shows_offline() {
        let mut r = make_router();
        r.kintsugi_repair("claude", "timeout");
        let hb = r.heartbeat();
        assert!(hb.models_offline.contains(&"claude".to_string()));
    }

    #[test]
    fn test_clear_traces() {
        let mut r = make_router();
        r.route("q", RoutingTier::Tier1);
        assert!(!r.traces().is_empty());
        r.clear_traces();
        assert!(r.traces().is_empty());
    }

    // ── Ghost Seat ────────────────────────────────────────────────────────────

    #[test]
    fn test_ghost_seat_enabled_by_default() {
        let r = make_router();
        assert!(r.ghost_seat_enabled);
    }
}
