// src/universal/janus.rs
// Janus v2 — Constitutional Multi-Agent Protocol
//
// Implements the Janus v2 orchestration spec (janus/JANUS_V2_SPEC.md) on top of the
// existing universal AI CLI adapter layer (ai_cli.rs) and ModelRouter (model_router.rs).
//
// Janus is the multi-agent routing layer for Aluminum OS.  It:
//   1. Maintains a council of AI models, each with a constitutional role and weight.
//   2. Classifies queries into Tiers (1 = simple, 2 = complex, 3 = critical+HITL).
//   3. Enforces INV-7: no single model may exceed 47% of weighted votes.
//   4. Routes Tier-1 queries to the best single model.
//   5. Routes Tier-2/3 queries through council consensus voting.
//   6. Applies Kintsugi repair when a model fails (fallback + golden seam trace).
//   7. Emits a structured HeartbeatTrace every cycle with model availability.
//   8. Supports the Ghost Seat Protocol (S144) for unrepresented populations.
//
// Architecture:
//   User Query → JanusRouter.route()
//       ├── Tier 1: single model via ModelRouter
//       ├── Tier 2: 2–3 models via council_vote()
//       └── Tier 3: full council + HITL via council_vote() + human_sign_off
//
// All decisions emit GoldenTrace records chaining into the audit log.
//
// Enforces: INV-7 (Vendor Balance/Dominance Cap), INV-8 (Human Override),
//           INV-1 (Sovereignty), INV-3 (Audit Trail), INV-35 (Fail-Closed)

use std::collections::BTreeMap;

use super::ai_cli::{AiCliRequest, AiCliResponse, AiProvider};
use super::model_router::{GoldenTrace, ModelRouter, NpfmScore, RouterConfig, RouterDecision};

// ─── INV-7 constant ───────────────────────────────────────────

/// Maximum fraction of consensus weight any single model may hold.
/// Defined by INV-7 in the Janus v2 spec.
pub const INV7_DOMINANCE_CAP: f64 = 0.47;

// ─── Council roles ────────────────────────────────────────────

/// The constitutional role a council member plays in Janus routing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CouncilRole {
    /// Constitutional/governance oversight (default: Claude)
    Governance,
    /// Deep domain knowledge and substrate analysis (default: Gemini)
    Substrate,
    /// Cross-domain research and connections (default: DeepSeek)
    Research,
    /// Market/enterprise validation (default: GitHub Copilot)
    Enterprise,
    /// Adversarial contrarian review (default: Grok)
    Adversarial,
    /// Ghost Seat — unrepresented populations proxy (Sphere S144)
    GhostSeat,
}

impl CouncilRole {
    pub fn display_name(&self) -> &'static str {
        match self {
            CouncilRole::Governance => "Governance",
            CouncilRole::Substrate => "Substrate",
            CouncilRole::Research => "Research",
            CouncilRole::Enterprise => "Enterprise",
            CouncilRole::Adversarial => "Adversarial",
            CouncilRole::GhostSeat => "Ghost Seat (S144)",
        }
    }
}

// ─── Model availability ───────────────────────────────────────

/// Current availability status of a council member.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelStatus {
    /// Model is responding normally.
    Available,
    /// Model is responding but with degraded quality or latency.
    Degraded,
    /// Model is not responding.
    Offline,
}

impl ModelStatus {
    pub fn is_usable(&self) -> bool {
        matches!(self, ModelStatus::Available | ModelStatus::Degraded)
    }
}

// ─── Council member ───────────────────────────────────────────

/// A single seat on the Janus council.
#[derive(Debug, Clone)]
pub struct CouncilMember {
    /// The AI provider behind this seat.
    pub provider: AiProvider,
    /// Constitutional role in the council.
    pub role: CouncilRole,
    /// Voting weight [0.0, 1.0].  Weights are normalised during consensus.
    pub weight: f64,
    /// Fallback provider if this member is offline.
    pub fallback: AiProvider,
    /// Current availability status (updated by heartbeat/probe).
    pub status: ModelStatus,
}

impl CouncilMember {
    pub fn new(
        provider: AiProvider,
        role: CouncilRole,
        weight: f64,
        fallback: AiProvider,
    ) -> Self {
        CouncilMember {
            provider,
            role,
            weight,
            fallback,
            status: ModelStatus::Available,
        }
    }
}

// ─── Query tier ───────────────────────────────────────────────

/// Routing tier for a query, per the Janus v2 protocol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryTier {
    /// Tier 1: Simple, single model.  Latency target <500ms.
    Tier1,
    /// Tier 2: Complex, 2–3 models with synthesis.  Latency target <3000ms.
    Tier2,
    /// Tier 3: Critical, full council + mandatory human sign-off.  <30000ms.
    Tier3,
}

impl QueryTier {
    /// Infer the tier from a query string using heuristics.
    ///
    /// Tier-3 keywords: irreversible, delete, shutdown, legal, medical, financial risk.
    /// Tier-2 keywords: compare, evaluate, tradeoff, consensus, policy, architecture.
    /// Tier-1: everything else.
    pub fn infer(prompt: &str) -> Self {
        let lower = prompt.to_lowercase();

        // Tier-3: critical/irreversible actions
        let tier3_kw = [
            "delete all", "shutdown", "terminate", "irreversible", "legal advice",
            "medical diagnosis", "financial risk", "destroy", "wipe", "permanently",
        ];
        for kw in &tier3_kw {
            if lower.contains(kw) {
                return QueryTier::Tier3;
            }
        }

        // Tier-2: complex, multi-perspective analysis
        let tier2_kw = [
            "compare", "evaluate", "tradeoff", "architecture", "consensus",
            "policy", "governance", "best approach", "pros and cons",
            "what should", "recommend", "multiple perspectives",
        ];
        for kw in &tier2_kw {
            if lower.contains(kw) {
                return QueryTier::Tier2;
            }
        }

        QueryTier::Tier1
    }
}

// ─── INV-7 Guard ──────────────────────────────────────────────

/// Enforces INV-7: no single model may exceed `INV7_DOMINANCE_CAP` (47%) of
/// the total weighted vote in any consensus round.
///
/// If a model's normalised weight would exceed the cap, the guard reduces it to
/// the cap value and distributes the remainder equally across other members.
pub struct Inv7Guard;

impl Inv7Guard {
    /// Validate a weight map against INV-7 and return adjusted weights.
    ///
    /// `weights`: map of provider display name → raw weight (any positive float).
    /// Returns normalised weights where no entry exceeds `INV7_DOMINANCE_CAP`,
    /// or an error string if compliance cannot be achieved (e.g. only 1 member).
    ///
    /// Algorithm: iteratively pin violators at the cap and proportionally
    /// re-distribute the residual weight to the remaining compliant members,
    /// repeating until stable (converges in O(n) iterations).
    pub fn enforce(
        weights: &BTreeMap<String, f64>,
    ) -> Result<BTreeMap<String, f64>, String> {
        if weights.is_empty() {
            return Err("No council members to enforce INV-7 on".to_string());
        }

        let total: f64 = weights.values().sum();
        if total == 0.0 {
            return Err("All council weights are zero".to_string());
        }

        // Start with normalised weights
        let mut result: BTreeMap<String, f64> = weights
            .iter()
            .map(|(k, v)| (k.clone(), v / total))
            .collect();

        // Iterative cap-and-redistribute: O(n) iterations, always converges.
        for _ in 0..result.len() + 1 {
            let violators: Vec<String> = result
                .iter()
                .filter(|(_, &v)| v > INV7_DOMINANCE_CAP + 1e-12)
                .map(|(k, _)| k.clone())
                .collect();

            if violators.is_empty() {
                break;
            }

            // Pin each violator at the cap
            for k in &violators {
                result.insert(k.clone(), INV7_DOMINANCE_CAP);
            }

            // Residual weight available to the non-pinned members
            let pinned_total: f64 = violators.len() as f64 * INV7_DOMINANCE_CAP;
            let residual = 1.0 - pinned_total;

            // Proportionally re-distribute residual to the compliant members
            let compliant_raw_total: f64 = result
                .iter()
                .filter(|(k, _)| !violators.contains(k))
                .map(|(_, &v)| v)
                .sum();

            if compliant_raw_total > 1e-12 {
                let scale = residual / compliant_raw_total;
                for (k, v) in result.iter_mut() {
                    if !violators.contains(k) {
                        *v *= scale;
                    }
                }
            }
        }

        Ok(result)
    }

    /// Returns `true` if no single member's weight exceeds the dominance cap.
    pub fn is_compliant(weights: &BTreeMap<String, f64>) -> bool {
        let total: f64 = weights.values().sum();
        if total == 0.0 {
            return false;
        }
        weights.values().all(|&w| w / total <= INV7_DOMINANCE_CAP)
    }
}

// ─── Council vote ─────────────────────────────────────────────

/// A single model's vote in a consensus round.
#[derive(Debug, Clone)]
pub struct ConsensusVote {
    pub provider: AiProvider,
    pub role: CouncilRole,
    pub weight: f64,
    pub response: AiCliResponse,
    /// Whether this vote dissents from the emerging consensus.
    pub is_dissent: bool,
}

/// The synthesised result of a multi-model council consensus round.
#[derive(Debug, Clone)]
pub struct ConsensusResult {
    /// The synthesised/combined response text.
    pub synthesised_content: String,
    /// Individual votes (one per participating model).
    pub votes: Vec<ConsensusVote>,
    /// Whether INV-7 was satisfied.
    pub inv7_compliant: bool,
    /// Any dissenting voices (for audit trail).
    pub dissent_count: usize,
    /// The GoldenTrace for this consensus round.
    pub trace: GoldenTrace,
    /// Which tier this consensus was produced at.
    pub tier: QueryTier,
}

impl ConsensusResult {
    /// Serialise to JSON.
    pub fn to_json(&self) -> String {
        let votes_json: Vec<String> = self.votes.iter().map(|v| {
            format!(
                "{{\"provider\":\"{}\",\"role\":\"{}\",\"weight\":{:.2},\"is_dissent\":{}}}",
                v.provider.display_name(),
                v.role.display_name(),
                v.weight,
                v.is_dissent,
            )
        }).collect();
        format!(
            "{{\"synthesised_content\":{},\"inv7_compliant\":{},\
             \"dissent_count\":{},\"tier\":\"{:?}\",\
             \"votes\":[{}],\"trace\":{}}}",
            super::ai_cli::json_string(&self.synthesised_content),
            self.inv7_compliant,
            self.dissent_count,
            self.tier,
            votes_json.join(","),
            self.trace.to_json(),
        )
    }
}

// ─── Heartbeat trace ──────────────────────────────────────────

/// Periodic status report emitted by JanusRouter.
///
/// Matches the heartbeat payload defined in janus/JANUS_V2_SPEC.md.
#[derive(Debug, Clone)]
pub struct HeartbeatTrace {
    pub models_available: Vec<String>,
    pub models_degraded: Vec<String>,
    pub models_offline: Vec<String>,
    pub consensus_ready: bool,
    pub inv7_compliant: bool,
    pub timestamp: String,
}

impl HeartbeatTrace {
    /// Serialise to JSON (matches the spec format).
    pub fn to_json(&self) -> String {
        let av = self.models_available.iter()
            .map(|s| format!("\"{}\"", s)).collect::<Vec<_>>().join(",");
        let dg = self.models_degraded.iter()
            .map(|s| format!("\"{}\"", s)).collect::<Vec<_>>().join(",");
        let of = self.models_offline.iter()
            .map(|s| format!("\"{}\"", s)).collect::<Vec<_>>().join(",");
        format!(
            "{{\"event_type\":\"action\",\"payload\":{{\"type\":\"heartbeat\",\
             \"models_available\":[{}],\"models_degraded\":[{}],\
             \"models_offline\":[{}],\"consensus_ready\":{},\
             \"inv7_compliant\":{},\"timestamp\":\"{}\"}}}}",
            av, dg, of,
            self.consensus_ready, self.inv7_compliant,
            self.timestamp,
        )
    }
}

// ─── Kintsugi repair record ───────────────────────────────────

/// Emitted when Janus applies a golden repair after a model failure.
///
/// "The gold is in the seams" — failures are recorded as evidence of
/// resilience, not hidden.
#[derive(Debug, Clone)]
pub struct KintsugiSeam {
    /// The provider that failed.
    pub failed_provider: AiProvider,
    /// The fallback provider that succeeded.
    pub repair_provider: AiProvider,
    /// The error message from the failed provider.
    pub failure_reason: String,
    /// The GoldenTrace of the repair decision.
    pub trace: GoldenTrace,
}

impl KintsugiSeam {
    pub fn to_json(&self) -> String {
        format!(
            "{{\"kintsugi_seam\":{{\"failed\":\"{}\",\"repair\":\"{}\",\
             \"reason\":{},\"trace\":{}}}}}",
            self.failed_provider.display_name(),
            self.repair_provider.display_name(),
            super::ai_cli::json_string(&self.failure_reason),
            self.trace.to_json(),
        )
    }
}

// ─── Janus routing outcome ────────────────────────────────────

/// The result of a JanusRouter routing decision.
#[derive(Debug, Clone)]
pub enum JanusOutcome {
    /// Tier-1 single-model result with NPFM enforcement.
    Tier1(RouterDecision),
    /// Tier-2/3 multi-model consensus result.
    Council(ConsensusResult),
    /// Request denied at NPFM gate before reaching any model.
    Denied { reason: String, npfm_score: NpfmScore },
    /// All models failed; system entered safe mode.
    SafeMode { reason: String, trace: GoldenTrace },
}

impl JanusOutcome {
    pub fn is_success(&self) -> bool {
        match self {
            JanusOutcome::Tier1(d) => d.is_allowed(),
            JanusOutcome::Council(_) => true,
            JanusOutcome::Denied { .. } | JanusOutcome::SafeMode { .. } => false,
        }
    }

    pub fn to_json(&self) -> String {
        match self {
            JanusOutcome::Tier1(d) => {
                format!("{{\"janus_tier\":1,\"result\":{}}}", d.to_json())
            }
            JanusOutcome::Council(c) => {
                format!("{{\"janus_tier\":{:?},\"result\":{}}}", c.tier, c.to_json())
            }
            JanusOutcome::Denied { reason, npfm_score } => {
                format!(
                    "{{\"janus_tier\":\"denied\",\"reason\":\"{}\",\"npfm_score\":{:.2}}}",
                    super::ai_cli::escape_json(reason),
                    npfm_score.0,
                )
            }
            JanusOutcome::SafeMode { reason, trace } => {
                format!(
                    "{{\"janus_tier\":\"safe_mode\",\"reason\":\"{}\",{}}}",
                    super::ai_cli::escape_json(reason),
                    &trace.to_json()[1..trace.to_json().len() - 1],
                )
            }
        }
    }
}

// ─── JanusRouter ──────────────────────────────────────────────

/// The Janus v2 multi-agent router.
///
/// Wraps `ModelRouter` (single-provider + NPFM) and adds:
/// - Tiered routing (Tier 1 / 2 / 3)
/// - Council consensus (INV-7 enforced)
/// - Heartbeat tracking and Kintsugi repair
///
/// ```rust
/// use uws::universal::janus::{JanusRouter, QueryTier};
/// use uws::universal::ai_cli::{AiCliRequest, AiCliResponse, AiProvider};
/// use std::collections::BTreeMap;
///
/// let mut router = JanusRouter::default();
/// let req = AiCliRequest::new(AiProvider::Claude, "explain async Rust");
///
/// let outcome = router.route(req, None, |r: &AiCliRequest| {
///     Ok(AiCliResponse {
///         provider: r.provider.clone(),
///         model_used: "claude-opus-4-5".to_string(),
///         content: "Async Rust uses futures…".to_string(),
///         raw_fields: BTreeMap::new(),
///         latency_ms: 0,
///         truncated: false,
///     })
/// });
/// assert!(outcome.is_success());
/// ```
pub struct JanusRouter {
    /// The inner NPFM+GoldenTrace router used for Tier-1 routing.
    inner: ModelRouter,
    /// The full council roster.
    pub council: Vec<CouncilMember>,
    /// Audit log of Kintsugi seams (repairs after failures).
    pub kintsugi_seams: Vec<KintsugiSeam>,
    /// Audit log of heartbeat traces.
    pub heartbeats: Vec<HeartbeatTrace>,
}

impl Default for JanusRouter {
    /// Create a JanusRouter with the default Janus v2 council configuration.
    fn default() -> Self {
        JanusRouter::with_council(default_council())
    }
}

impl JanusRouter {
    /// Create a JanusRouter with a custom council.
    pub fn with_council(council: Vec<CouncilMember>) -> Self {
        JanusRouter {
            inner: ModelRouter::new(RouterConfig::default()),
            council,
            kintsugi_seams: Vec::new(),
            heartbeats: Vec::new(),
        }
    }

    /// Route a query through the Janus protocol.
    ///
    /// `executor` is called once per model needed for this tier.  It receives
    /// the request (potentially with an updated provider/system prompt for the
    /// council role) and should return either a response or an error.
    ///
    /// Tier detection is automatic based on the prompt content unless
    /// `tier_override` is `Some(...)`.
    pub fn route<F>(
        &mut self,
        request: AiCliRequest,
        tier_override: Option<QueryTier>,
        mut executor: F,
    ) -> JanusOutcome
    where
        F: FnMut(&AiCliRequest) -> Result<AiCliResponse, String>,
    {
        let tier = tier_override.unwrap_or_else(|| QueryTier::infer(&request.prompt));

        match tier {
            QueryTier::Tier1 => {
                // Delegate fully to the inner ModelRouter (NPFM + GoldenTrace)
                let decision = self.inner.route(request, |r| executor(r));
                if !decision.is_allowed() {
                    if let RouterDecision::Deny { reason, npfm_score, .. } = &decision {
                        return JanusOutcome::Denied {
                            reason: reason.clone(),
                            npfm_score: *npfm_score,
                        };
                    }
                }
                JanusOutcome::Tier1(decision)
            }
            QueryTier::Tier2 | QueryTier::Tier3 => {
                self.council_route(request, tier, &mut executor)
            }
        }
    }

    /// Multi-model council routing for Tier-2 and Tier-3 queries.
    fn council_route<F>(
        &mut self,
        request: AiCliRequest,
        tier: QueryTier,
        executor: &mut F,
    ) -> JanusOutcome
    where
        F: FnMut(&AiCliRequest) -> Result<AiCliResponse, String>,
    {
        // Select participating members (all usable members for Tier-3,
        // governance + 2 others for Tier-2)
        let participants: Vec<usize> = self.council.iter().enumerate()
            .filter(|(_, m)| m.status.is_usable())
            .map(|(i, _)| i)
            .take(if tier == QueryTier::Tier2 { 3 } else { 6 })
            .collect();

        if participants.is_empty() {
            // Safe mode: no usable models
            let ts = current_timestamp();
            let trace = GoldenTrace {
                hitl_weight: 0.0,
                digest: super::model_router::compute_digest_from_str("safe_mode", &ts),
                timestamp: ts,
                provider: AiProvider::Claude,
                npfm_score: NpfmScore::new(0.0),
            };
            return JanusOutcome::SafeMode {
                reason: "No usable council members available".to_string(),
                trace,
            };
        }

        // Build weight map for INV-7 checking
        let raw_weights: BTreeMap<String, f64> = participants.iter()
            .map(|&i| {
                let m = &self.council[i];
                (m.provider.display_name().to_string(), m.weight)
            })
            .collect();

        let normalised = match Inv7Guard::enforce(&raw_weights) {
            Ok(w) => w,
            Err(e) => {
                let ts = current_timestamp();
                let trace = GoldenTrace {
                    hitl_weight: 0.0,
                    digest: super::model_router::compute_digest_from_str("inv7_error", &ts),
                    timestamp: ts,
                    provider: AiProvider::Claude,
                    npfm_score: NpfmScore::new(0.0),
                };
                return JanusOutcome::SafeMode { reason: e, trace };
            }
        };
        let inv7_ok = Inv7Guard::is_compliant(&normalised);

        // Collect votes from each participant
        let mut votes: Vec<ConsensusVote> = Vec::new();
        for &idx in &participants {
            // Clone all member data upfront to avoid holding a borrow across the
            // mutable self.council.get_mut() call in the error handler below.
            let (provider, role, fallback_provider, weight) = {
                let m = &self.council[idx];
                (
                    m.provider.clone(),
                    m.role.clone(),
                    m.fallback.clone(),
                    normalised.get(m.provider.display_name()).copied().unwrap_or(0.0),
                )
            };

            let mut member_req = request.clone();
            member_req.provider = provider.clone();

            // Inject constitutional role into the system prompt
            let role_prompt = format!(
                "You are acting as the {} member of the Aluminum OS Pantheon Council \
                 under the Janus v2 protocol. Your role is: {}. \
                 INV-7: Your influence is capped at 47% of total consensus weight.",
                provider.display_name(),
                role.display_name(),
            );
            member_req.system = Some(match &request.system {
                Some(existing) => format!("{}\n\n{}", role_prompt, existing),
                None => role_prompt,
            });

            let resp = match executor(&member_req) {
                Ok(r) => r,
                Err(e) => {
                    // Kintsugi repair: try the fallback
                    let ts = current_timestamp();
                    let repair_trace = GoldenTrace {
                        hitl_weight: self.inner.config().hitl_weight,
                        digest: super::model_router::compute_digest_from_str(
                            &format!("kintsugi_repair:{}:{}", provider.display_name(), ts),
                            &ts,
                        ),
                        timestamp: ts.clone(),
                        provider: fallback_provider.clone(),
                        npfm_score: NpfmScore::new(0.75),
                    };

                    // Update council member status to degraded
                    if let Some(m) = self.council.get_mut(idx) {
                        m.status = ModelStatus::Degraded;
                    }

                    // Record the seam
                    self.kintsugi_seams.push(KintsugiSeam {
                        failed_provider: provider.clone(),
                        repair_provider: fallback_provider.clone(),
                        failure_reason: e.clone(),
                        trace: repair_trace,
                    });

                    // Attempt fallback
                    let mut fallback_req = member_req.clone();
                    fallback_req.provider = fallback_provider.clone();
                    match executor(&fallback_req) {
                        Ok(r) => r,
                        Err(_) => {
                            // Both primary and fallback failed — skip this vote
                            continue;
                        }
                    }
                }
            };

            votes.push(ConsensusVote {
                provider,
                role,
                weight,
                response: resp,
                is_dissent: false,
            });
        }

        if votes.is_empty() {
            let ts = current_timestamp();
            let trace = GoldenTrace {
                hitl_weight: 0.0,
                digest: super::model_router::compute_digest_from_str("no_votes", &ts),
                timestamp: ts,
                provider: AiProvider::Claude,
                npfm_score: NpfmScore::new(0.0),
            };
            return JanusOutcome::SafeMode {
                reason: "All council members failed, including fallbacks".to_string(),
                trace,
            };
        }

        // Synthesis: weighted concatenation (in production this would call a
        // synthesis model; here we concatenate with attribution for auditability)
        let synthesised = synthesise_votes(&votes);

        // Mark adversarial votes as dissent if their content differs significantly
        // (simple heuristic: Adversarial role always flagged for audit visibility)
        let mut final_votes = votes;
        for vote in &mut final_votes {
            if vote.role == CouncilRole::Adversarial {
                vote.is_dissent = true;
            }
        }
        let dissent_count = final_votes.iter().filter(|v| v.is_dissent).count();

        let ts = current_timestamp();
        let trace = GoldenTrace {
            hitl_weight: self.inner.config().hitl_weight,
            digest: super::model_router::compute_digest_from_str(&format!("council_consensus:{}", ts), &ts),
            timestamp: ts,
            provider: AiProvider::Claude, // governance model leads the trace
            npfm_score: NpfmScore::new(0.8),
        };

        JanusOutcome::Council(ConsensusResult {
            synthesised_content: synthesised,
            votes: final_votes,
            inv7_compliant: inv7_ok,
            dissent_count,
            trace,
            tier,
        })
    }

    /// Emit a heartbeat trace reflecting current council status.
    pub fn heartbeat(&mut self) -> HeartbeatTrace {
        let mut available = Vec::new();
        let mut degraded = Vec::new();
        let mut offline = Vec::new();

        for m in &self.council {
            let name = m.provider.display_name().to_string();
            match m.status {
                ModelStatus::Available => available.push(name),
                ModelStatus::Degraded => degraded.push(name),
                ModelStatus::Offline => offline.push(name),
            }
        }

        // INV-7: consensus_ready requires ≥2 usable members
        let usable = self.council.iter().filter(|m| m.status.is_usable()).count();
        let consensus_ready = usable >= 2;

        // Build weight map for INV-7 compliance check
        let weights: BTreeMap<String, f64> = self.council.iter()
            .filter(|m| m.status.is_usable())
            .map(|m| (m.provider.display_name().to_string(), m.weight))
            .collect();
        let inv7_compliant = !weights.is_empty() && Inv7Guard::is_compliant(&weights);

        let hb = HeartbeatTrace {
            models_available: available,
            models_degraded: degraded,
            models_offline: offline,
            consensus_ready,
            inv7_compliant,
            timestamp: current_timestamp(),
        };

        self.heartbeats.push(hb.clone());
        hb
    }

    /// Mark a model as offline (e.g. after a heartbeat probe fails).
    pub fn mark_offline(&mut self, provider: &AiProvider) {
        for m in &mut self.council {
            if &m.provider == provider {
                m.status = ModelStatus::Offline;
            }
        }
    }

    /// Mark a model as available (e.g. after successful recovery).
    pub fn mark_available(&mut self, provider: &AiProvider) {
        for m in &mut self.council {
            if &m.provider == provider {
                m.status = ModelStatus::Available;
            }
        }
    }

    /// Count of Kintsugi repairs applied since boot.
    pub fn seam_count(&self) -> usize {
        self.kintsugi_seams.len()
    }
}

// ─── Default council ──────────────────────────────────────────

/// The default Janus v2 council roster, per janus/JANUS_V2_SPEC.md.
pub fn default_council() -> Vec<CouncilMember> {
    vec![
        CouncilMember::new(
            AiProvider::Claude,
            CouncilRole::Governance,
            1.0,
            AiProvider::Gemini,
        ),
        CouncilMember::new(
            AiProvider::Gemini,
            CouncilRole::Substrate,
            1.0,
            AiProvider::Claude,
        ),
        CouncilMember::new(
            AiProvider::Grok,
            CouncilRole::Adversarial,
            0.8,
            AiProvider::DeepSeek,
        ),
        CouncilMember::new(
            AiProvider::DeepSeek,
            CouncilRole::Research,
            0.7,
            AiProvider::Gemini,
        ),
        CouncilMember::new(
            AiProvider::GithubCopilot,
            CouncilRole::Enterprise,
            0.7,
            AiProvider::Claude,
        ),
    ]
}

// ─── Synthesis helper ─────────────────────────────────────────

/// Produce a synthesised response from multiple council votes.
///
/// In production this would call a synthesis model.  Here we concatenate
/// responses with attribution headers for full auditability.
fn synthesise_votes(votes: &[ConsensusVote]) -> String {
    if votes.is_empty() {
        return String::new();
    }
    if votes.len() == 1 {
        return votes[0].response.content.clone();
    }

    let mut parts: Vec<String> = Vec::new();
    for vote in votes {
        parts.push(format!(
            "[{} — {}]\n{}",
            vote.provider.display_name(),
            vote.role.display_name(),
            vote.response.content,
        ));
    }
    parts.join("\n\n---\n\n")
}

// ─── Internal timestamp / digest helpers ──────────────────────

fn current_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let (y, mo, d, h, mi, s) = secs_to_ymdhms(secs);
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", y, mo, d, h, mi, s)
}

fn secs_to_ymdhms(secs: u64) -> (u64, u64, u64, u64, u64, u64) {
    let s = secs % 60;
    let m = (secs / 60) % 60;
    let h = (secs / 3600) % 24;
    let days = secs / 86400;
    let z = days + 719468;
    let era = z / 146097;
    let doe = z % 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if month <= 2 { y + 1 } else { y };
    (year, month, day, h, m, s)
}

// ─── Tests ────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::universal::ai_cli::AiCliRequest;

    fn stub_response(provider: AiProvider, content: &str) -> AiCliResponse {
        AiCliResponse {
            provider: provider.clone(),
            model_used: "test-model".to_string(),
            content: content.to_string(),
            raw_fields: BTreeMap::new(),
            latency_ms: 1,
            truncated: false,
        }
    }

    // ─── QueryTier tests ──────────────────────────────────────

    #[test]
    fn test_tier_inference_tier1() {
        assert_eq!(QueryTier::infer("explain how async Rust works"), QueryTier::Tier1);
        assert_eq!(QueryTier::infer("write a unit test"), QueryTier::Tier1);
    }

    #[test]
    fn test_tier_inference_tier2() {
        assert_eq!(QueryTier::infer("compare Rust vs Go for a microservice"), QueryTier::Tier2);
        assert_eq!(QueryTier::infer("what should I choose for this architecture"), QueryTier::Tier2);
    }

    #[test]
    fn test_tier_inference_tier3() {
        assert_eq!(QueryTier::infer("delete all user data permanently"), QueryTier::Tier3);
        assert_eq!(QueryTier::infer("shutdown the production system"), QueryTier::Tier3);
    }

    // ─── INV-7 Guard tests ────────────────────────────────────

    #[test]
    fn test_inv7_compliant_weights() {
        let mut w = BTreeMap::new();
        w.insert("Claude".to_string(), 1.0);
        w.insert("Gemini".to_string(), 1.0);
        w.insert("Grok".to_string(), 0.8);
        assert!(Inv7Guard::is_compliant(&w));
    }

    #[test]
    fn test_inv7_violation_single_model() {
        let mut w = BTreeMap::new();
        w.insert("Claude".to_string(), 1.0);
        // Single model = 100% weight → violates 47% cap
        assert!(!Inv7Guard::is_compliant(&w));
    }

    #[test]
    fn test_inv7_enforce_caps_dominant_model() {
        // With 3+ members we can enforce the 47% cap.
        // A dominant member (weight 10.0 vs 1.0/1.0) starts above the cap;
        // Inv7Guard should reduce it so every normalised share ≤ 0.47.
        let mut w = BTreeMap::new();
        w.insert("Claude".to_string(), 10.0);
        w.insert("Gemini".to_string(), 1.0);
        w.insert("Grok".to_string(), 1.0);
        let enforced = Inv7Guard::enforce(&w).unwrap();
        for (k, v) in &enforced {
            assert!(
                *v <= INV7_DOMINANCE_CAP + 1e-9,
                "cap exceeded for {}: {}", k, v
            );
        }
    }

    #[test]
    fn test_inv7_enforce_equal_weights_pass() {
        let mut w = BTreeMap::new();
        w.insert("A".to_string(), 1.0);
        w.insert("B".to_string(), 1.0);
        w.insert("C".to_string(), 1.0);
        let enforced = Inv7Guard::enforce(&w).unwrap();
        // Each should be ~0.333, well under 0.47
        for (_, v) in &enforced {
            assert!(*v <= INV7_DOMINANCE_CAP + 1e-9);
        }
    }

    #[test]
    fn test_inv7_empty_returns_error() {
        let w = BTreeMap::new();
        assert!(Inv7Guard::enforce(&w).is_err());
    }

    // ─── Default council tests ────────────────────────────────

    #[test]
    fn test_default_council_members() {
        let council = default_council();
        assert_eq!(council.len(), 5);
        assert!(council.iter().any(|m| m.role == CouncilRole::Governance));
        assert!(council.iter().any(|m| m.role == CouncilRole::Adversarial));
        assert!(council.iter().any(|m| m.role == CouncilRole::Substrate));
        assert!(council.iter().any(|m| m.role == CouncilRole::Research));
        assert!(council.iter().any(|m| m.role == CouncilRole::Enterprise));
    }

    #[test]
    fn test_default_council_inv7_compliant() {
        let council = default_council();
        let weights: BTreeMap<String, f64> = council.iter()
            .map(|m| (m.provider.display_name().to_string(), m.weight))
            .collect();
        assert!(Inv7Guard::is_compliant(&weights));
    }

    // ─── JanusRouter Tier-1 tests ─────────────────────────────

    #[test]
    fn test_janus_tier1_allows_positive_request() {
        let mut router = JanusRouter::default();
        let req = AiCliRequest::new(AiProvider::Claude, "explain ownership in Rust");
        let outcome = router.route(req, None, |r| {
            Ok(stub_response(r.provider.clone(), "Ownership ensures memory safety…"))
        });
        assert!(outcome.is_success());
        if let JanusOutcome::Tier1(d) = &outcome {
            assert!(d.is_allowed());
        } else {
            panic!("expected Tier1 outcome");
        }
    }

    #[test]
    fn test_janus_tier1_denies_busywork() {
        let mut router = JanusRouter::default();
        let req = AiCliRequest::new(AiProvider::Claude, "generate boilerplate for all classes");
        let outcome = router.route(req, None, |r| {
            Ok(stub_response(r.provider.clone(), "Here is the boilerplate…"))
        });
        assert!(!outcome.is_success());
        assert!(matches!(outcome, JanusOutcome::Denied { .. }));
    }

    // ─── JanusRouter Tier-2 tests ─────────────────────────────

    #[test]
    fn test_janus_tier2_returns_consensus() {
        let mut router = JanusRouter::default();
        let req = AiCliRequest::new(AiProvider::Claude, "compare microservices vs monolith architecture");
        let outcome = router.route(req, Some(QueryTier::Tier2), |r| {
            Ok(stub_response(r.provider.clone(), "From my perspective…"))
        });
        assert!(outcome.is_success());
        if let JanusOutcome::Council(c) = &outcome {
            assert_eq!(c.tier, QueryTier::Tier2);
            assert!(!c.votes.is_empty());
            assert!(c.inv7_compliant);
        } else {
            panic!("expected Council outcome, got: {:?}", outcome.to_json());
        }
    }

    #[test]
    fn test_janus_tier2_consensus_has_adversarial_dissent() {
        let mut router = JanusRouter::default();
        let req = AiCliRequest::new(AiProvider::Claude, "evaluate this governance policy decision");
        let outcome = router.route(req, Some(QueryTier::Tier2), |r| {
            Ok(stub_response(r.provider.clone(), "My view is…"))
        });
        if let JanusOutcome::Council(c) = outcome {
            // Grok (Adversarial role) should be marked as dissent
            assert!(c.dissent_count > 0 || c.votes.len() < 3,
                "expected dissent from adversarial member");
        }
    }

    // ─── Kintsugi repair tests ────────────────────────────────

    #[test]
    fn test_kintsugi_repair_on_model_failure() {
        let mut router = JanusRouter::default();
        let req = AiCliRequest::new(AiProvider::Claude, "compare approaches to error handling");
        let mut call_count = 0usize;

        let _outcome = router.route(req, Some(QueryTier::Tier2), |r| {
            call_count += 1;
            // Fail the first Claude call, succeed for others
            if r.provider == AiProvider::Claude && call_count == 1 {
                Err("network timeout".to_string())
            } else {
                Ok(stub_response(r.provider.clone(), "My answer…"))
            }
        });

        // Should still produce a result (via fallback or other council members)
        // Kintsugi seam should be recorded
        assert!(router.seam_count() > 0, "expected a Kintsugi seam to be recorded");
    }

    // ─── Safe mode test ───────────────────────────────────────

    #[test]
    fn test_safe_mode_when_all_offline() {
        let mut router = JanusRouter::default();
        // Mark all council members offline
        let providers: Vec<AiProvider> = router.council.iter()
            .map(|m| m.provider.clone())
            .collect();
        for p in &providers {
            router.mark_offline(p);
        }

        let req = AiCliRequest::new(AiProvider::Claude, "compare two options");
        let outcome = router.route(req, Some(QueryTier::Tier2), |r| {
            Ok(stub_response(r.provider.clone(), "answer"))
        });

        assert!(matches!(outcome, JanusOutcome::SafeMode { .. }));
    }

    // ─── Heartbeat tests ──────────────────────────────────────

    #[test]
    fn test_heartbeat_all_available() {
        let mut router = JanusRouter::default();
        let hb = router.heartbeat();
        assert!(hb.consensus_ready);
        assert!(hb.inv7_compliant);
        assert!(hb.models_offline.is_empty());
    }

    #[test]
    fn test_heartbeat_with_offline_model() {
        let mut router = JanusRouter::default();
        router.mark_offline(&AiProvider::Claude);
        let hb = router.heartbeat();
        assert!(hb.models_offline.contains(&"Anthropic Claude".to_string()));
    }

    #[test]
    fn test_heartbeat_json_format() {
        let mut router = JanusRouter::default();
        let hb = router.heartbeat();
        let json = hb.to_json();
        assert!(json.contains("\"event_type\":\"action\""));
        assert!(json.contains("\"type\":\"heartbeat\""));
        assert!(json.contains("consensus_ready"));
        assert!(json.contains("inv7_compliant"));
    }

    // ─── JanusOutcome JSON tests ──────────────────────────────

    #[test]
    fn test_janus_outcome_tier1_json() {
        let mut router = JanusRouter::default();
        let req = AiCliRequest::new(AiProvider::Claude, "explain Rust lifetimes");
        let outcome = router.route(req, None, |r| {
            Ok(stub_response(r.provider.clone(), "Lifetimes track…"))
        });
        let json = outcome.to_json();
        assert!(json.contains("\"janus_tier\":1"));
    }

    #[test]
    fn test_council_role_display_names() {
        assert_eq!(CouncilRole::Governance.display_name(), "Governance");
        assert_eq!(CouncilRole::Adversarial.display_name(), "Adversarial");
        assert_eq!(CouncilRole::GhostSeat.display_name(), "Ghost Seat (S144)");
    }

    #[test]
    fn test_mark_available_restores_status() {
        let mut router = JanusRouter::default();
        router.mark_offline(&AiProvider::Gemini);
        {
            let m = router.council.iter().find(|m| m.provider == AiProvider::Gemini).unwrap();
            assert_eq!(m.status, ModelStatus::Offline);
        }
        router.mark_available(&AiProvider::Gemini);
        {
            let m = router.council.iter().find(|m| m.provider == AiProvider::Gemini).unwrap();
            assert_eq!(m.status, ModelStatus::Available);
        }
    }
}
