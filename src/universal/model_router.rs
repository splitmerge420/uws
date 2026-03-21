// src/universal/model_router.rs
// ModelRouter — Aluminum OS NPFM + GoldenTrace Provenance Layer
//
// Every AI CLI request passes through this router before execution.
// The router enforces two invariants:
//
//   1. NPFM (Net-Positive Flourishing Metric) ≥ 0.7
//      Requests whose estimated flourishing score falls below this threshold
//      are DENIED.  A score of 1.0 means maximally beneficial; 0.0 means
//      purely extractive / bureaucratic busywork.
//
//   2. GoldenTrace HITL provenance
//      Every ALLOW decision is stamped with a `GoldenTrace` record that
//      includes the HITL (Human-In-The-Loop) weight, SHA3-256 digest of
//      the decision context, and an ISO-8601 timestamp.  The trace is
//      formatted as a git commit trailer:
//
//          Golden-Trace: sha3-256:<hex>; HITL=<weight>; ts=<timestamp>
//
// The SHA3-256 implementation here is a pure-Rust FNV-1a-seeded stub that
// produces 64 hex characters.  Replace with a real sha3 crate once the
// dependency is unlocked (see Cargo.toml Phase 2 comment).
//
// Enforces: INV-1 (Sovereignty), INV-2 (Consent), INV-3 (Audit),
//           INV-6 (Provider Abstraction), INV-35 (Fail-Closed)

use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

use super::ai_cli::{AiCliRequest, AiCliResponse, AiProvider};

// ─── NPFM score ───────────────────────────────────────────────

/// Net-Positive Flourishing Metric score in [0.0, 1.0].
/// Scores below `NPFM_THRESHOLD` cause the router to DENY the request.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct NpfmScore(pub f64);

/// Minimum acceptable NPFM score. Requests scoring below this are DENIED.
pub const NPFM_THRESHOLD: f64 = 0.7;

impl NpfmScore {
    /// Create a new score, clamping to [0.0, 1.0].
    pub fn new(raw: f64) -> Self {
        NpfmScore(raw.clamp(0.0, 1.0))
    }

    /// Whether this score meets the fiduciary threshold.
    pub fn meets_threshold(&self) -> bool {
        self.0 >= NPFM_THRESHOLD
    }
}

// ─── GoldenTrace ──────────────────────────────────────────────

/// Cryptographic provenance record appended to every allowed AI interaction.
///
/// The trace is formatted as a git commit trailer so it can be embedded
/// directly in commit messages:
///
/// ```text
/// Golden-Trace: sha3-256:abcdef01…; HITL=0.90; ts=2026-03-21T02:50:57Z
/// ```
#[derive(Debug, Clone)]
pub struct GoldenTrace {
    /// HITL (Human-In-The-Loop) oversight weight in [0.0, 1.0].
    /// 1.0 = fully human-reviewed; 0.0 = fully autonomous.
    pub hitl_weight: f64,
    /// SHA3-256 hex digest of the decision context (provider + prompt hash).
    pub digest: String,
    /// ISO-8601 UTC timestamp of when the trace was issued.
    pub timestamp: String,
    /// Which provider was routed to.
    pub provider: AiProvider,
    /// The NPFM score that led to the ALLOW decision.
    pub npfm_score: NpfmScore,
}

impl GoldenTrace {
    /// Format as a git commit trailer line.
    ///
    /// ```text
    /// Golden-Trace: sha3-256:abcdef01…; HITL=0.90; ts=2026-03-21T02:50:57Z
    /// ```
    pub fn to_trailer_string(&self) -> String {
        format!(
            "Golden-Trace: sha3-256:{}; HITL={:.2}; provider={}; npfm={:.2}; ts={}",
            self.digest,
            self.hitl_weight,
            self.provider.display_name(),
            self.npfm_score.0,
            self.timestamp,
        )
    }

    /// Serialise to JSON.
    pub fn to_json(&self) -> String {
        format!(
            "{{\"golden_trace\":{{\"digest\":\"sha3-256:{}\",\
             \"hitl_weight\":{:.2},\"provider\":\"{}\",\
             \"npfm_score\":{:.2},\"timestamp\":\"{}\"}}}}",
            self.digest,
            self.hitl_weight,
            self.provider.display_name(),
            self.npfm_score.0,
            self.timestamp,
        )
    }
}

// ─── Router decision ──────────────────────────────────────────

/// Outcome produced by the ModelRouter for each request.
#[derive(Debug, Clone)]
pub enum RouterDecision {
    /// Request is allowed.  The AI response and GoldenTrace are included.
    Allow {
        response: AiCliResponse,
        trace: GoldenTrace,
    },
    /// Request is denied because the NPFM score is below the threshold.
    Deny {
        reason: String,
        npfm_score: NpfmScore,
        provider: AiProvider,
    },
}

impl RouterDecision {
    /// Returns `true` if this decision allows the request through.
    pub fn is_allowed(&self) -> bool {
        matches!(self, RouterDecision::Allow { .. })
    }

    /// Serialise to JSON.
    pub fn to_json(&self) -> String {
        match self {
            RouterDecision::Allow { response, trace } => {
                format!(
                    "{{\"decision\":\"ALLOW\",\"response\":{},{}}}",
                    response.to_json(),
                    &trace.to_json()[1..trace.to_json().len() - 1], // unwrap outer braces
                )
            }
            RouterDecision::Deny { reason, npfm_score, provider } => {
                format!(
                    "{{\"decision\":\"DENY\",\"reason\":\"{}\",\
                     \"npfm_score\":{:.2},\"provider\":\"{}\"}}",
                    super::ai_cli::escape_json(reason),
                    npfm_score.0,
                    provider.display_name(),
                )
            }
        }
    }
}

// ─── NPFM scorer ──────────────────────────────────────────────

/// Estimates the NPFM score for a request based on heuristics.
///
/// A real implementation would call a specialised model.
/// This scorer uses keyword analysis — sufficient for unit tests and
/// demonstrating the interface without external I/O.
pub struct NpfmScorer;

impl NpfmScorer {
    /// Estimate the NPFM score for a request.
    ///
    /// Rules (each adjusts the base score of 0.75):
    /// - Prompts mentioning creative, educational, or collaborative intent → +0.15
    /// - Prompts containing only boilerplate/administrative keywords → -0.20
    /// - Prompts requesting deletion, extraction, or scraping → -0.15
    /// - Short prompts (<10 chars) → -0.10 (likely low-effort / bot)
    pub fn score(&self, request: &AiCliRequest) -> NpfmScore {
        let prompt_lower = request.prompt.to_lowercase();
        let mut score: f64 = 0.75;

        // Positive signals
        let positive_keywords = [
            "learn", "teach", "explain", "create", "build", "improve",
            "help", "collaborate", "write", "design", "analyze", "understand",
            "fix", "debug", "test", "document", "review",
        ];
        for kw in &positive_keywords {
            if prompt_lower.contains(kw) {
                score += 0.05;
                break;
            }
        }

        // Negative signals — bureaucratic busywork
        let busywork_keywords = [
            "generate boilerplate", "copy paste", "redundant", "bureaucratic",
            "busywork", "administrative overhead", "wrapper function",
        ];
        for kw in &busywork_keywords {
            if prompt_lower.contains(kw) {
                score -= 0.20;
                break;
            }
        }

        // Negative signals — extractive / harmful patterns
        let extractive_keywords = [
            "scrape", "harvest", "bulk delete", "mass delete",
            "exfiltrate", "exfiltration", "extract all",
        ];
        for kw in &extractive_keywords {
            if prompt_lower.contains(kw) {
                score -= 0.15;
                break;
            }
        }

        // Very short prompts are likely low-quality
        if request.prompt.len() < 10 {
            score -= 0.10;
        }

        NpfmScore::new(score)
    }
}

// ─── ModelRouter ──────────────────────────────────────────────

/// Routes AI CLI requests through NPFM enforcement and GoldenTrace provenance.
///
/// Usage:
/// ```rust
/// use uws::universal::model_router::{ModelRouter, RouterConfig};
/// use uws::universal::ai_cli::{AiCliRequest, AiProvider, AiCliResponse};
/// use std::collections::BTreeMap;
///
/// let config = RouterConfig::default();
/// let mut router = ModelRouter::new(config);
///
/// // Provide a mock executor so the router doesn't need network access
/// let req = AiCliRequest::new(AiProvider::Claude, "explain how async Rust works");
/// let decision = router.route(req, |_req| {
///     Ok(AiCliResponse {
///         provider: AiProvider::Claude,
///         model_used: "claude-opus-4-5".to_string(),
///         content: "Async Rust uses futures…".to_string(),
///         raw_fields: BTreeMap::new(),
///         latency_ms: 0,
///         truncated: false,
///     })
/// });
/// assert!(decision.is_allowed());
/// ```
pub struct ModelRouter {
    config: RouterConfig,
    scorer: NpfmScorer,
    /// Audit log of all decisions (provider, outcome, score, timestamp).
    pub audit_log: Vec<AuditRecord>,
}

/// Configuration for the ModelRouter.
#[derive(Debug, Clone)]
pub struct RouterConfig {
    /// Minimum NPFM score to allow a request through.
    pub npfm_threshold: f64,
    /// HITL weight to stamp on all GoldenTrace records.
    /// In production this should come from a signed human-review token.
    pub hitl_weight: f64,
    /// Whether to fail closed (deny) when the scorer returns an error.
    pub fail_closed: bool,
}

impl Default for RouterConfig {
    fn default() -> Self {
        RouterConfig {
            npfm_threshold: NPFM_THRESHOLD,
            hitl_weight: 0.90,
            fail_closed: true,
        }
    }
}

/// A single entry in the router's audit log.
#[derive(Debug, Clone)]
pub struct AuditRecord {
    pub timestamp: String,
    pub provider: AiProvider,
    pub npfm_score: NpfmScore,
    pub decision: &'static str, // "ALLOW" or "DENY"
    pub trace_digest: Option<String>,
}

impl ModelRouter {
    /// Create a new router with the given configuration.
    pub fn new(config: RouterConfig) -> Self {
        ModelRouter {
            config,
            scorer: NpfmScorer,
            audit_log: Vec::new(),
        }
    }

    /// Route a request through NPFM enforcement and provenance stamping.
    ///
    /// `executor` is a closure that performs the actual AI API call.
    /// It receives the request and returns either a response or an error.
    /// This keeps the router I/O-agnostic.
    pub fn route<F>(&mut self, request: AiCliRequest, executor: F) -> RouterDecision
    where
        F: FnOnce(&AiCliRequest) -> Result<AiCliResponse, String>,
    {
        let provider = request.provider.clone();
        let score = self.scorer.score(&request);
        let ts = current_timestamp();

        if score.0 < self.config.npfm_threshold {
            let record = AuditRecord {
                timestamp: ts.clone(),
                provider: provider.clone(),
                npfm_score: score,
                decision: "DENY",
                trace_digest: None,
            };
            self.audit_log.push(record);

            return RouterDecision::Deny {
                reason: format!(
                    "NPFM score {:.2} is below the required threshold of {:.2}. \
                     Request appears extractive or bureaucratic.",
                    score.0, self.config.npfm_threshold
                ),
                npfm_score: score,
                provider,
            };
        }

        // Execute the AI call
        let response = match executor(&request) {
            Ok(r) => r,
            Err(e) => {
                if self.config.fail_closed {
                    let record = AuditRecord {
                        timestamp: ts,
                        provider: provider.clone(),
                        npfm_score: score,
                        decision: "DENY",
                        trace_digest: None,
                    };
                    self.audit_log.push(record);
                    return RouterDecision::Deny {
                        reason: format!("Executor error (fail-closed): {}", e),
                        npfm_score: score,
                        provider,
                    };
                }
                // fail-open: return a synthetic empty response
                AiCliResponse {
                    provider: provider.clone(),
                    model_used: "unknown".to_string(),
                    content: String::new(),
                    raw_fields: BTreeMap::new(),
                    latency_ms: 0,
                    truncated: false,
                }
            }
        };

        // Build the GoldenTrace
        let digest = compute_digest(&provider, &request.prompt, &ts);
        let trace = GoldenTrace {
            hitl_weight: self.config.hitl_weight,
            digest: digest.clone(),
            timestamp: ts.clone(),
            provider: provider.clone(),
            npfm_score: score,
        };

        let record = AuditRecord {
            timestamp: ts,
            provider,
            npfm_score: score,
            decision: "ALLOW",
            trace_digest: Some(digest),
        };
        self.audit_log.push(record);

        RouterDecision::Allow { response, trace }
    }

    /// Return all ALLOW decisions with their GoldenTrace digests.
    pub fn allow_count(&self) -> usize {
        self.audit_log.iter().filter(|r| r.decision == "ALLOW").count()
    }

    /// Return all DENY decisions.
    pub fn deny_count(&self) -> usize {
        self.audit_log.iter().filter(|r| r.decision == "DENY").count()
    }
}

// ─── Internal helpers ─────────────────────────────────────────

/// Return an ISO-8601 UTC timestamp string.
fn current_timestamp() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // Format as YYYY-MM-DDTHH:MM:SSZ (manual, no chrono dep)
    let (y, mo, d, h, mi, s) = secs_to_ymdhms(secs);
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", y, mo, d, h, mi, s)
}

/// Convert Unix seconds to (year, month, day, hour, minute, second).
/// Gregorian calendar approximation sufficient for timestamps.
fn secs_to_ymdhms(secs: u64) -> (u64, u64, u64, u64, u64, u64) {
    let s = secs % 60;
    let m = (secs / 60) % 60;
    let h = (secs / 3600) % 24;
    let days = secs / 86400;

    // Gregorian calendar algorithm (works for 1970–2100)
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

/// Compute a 64-hex-character digest from the decision context.
///
/// This uses a FNV-1a-seeded mix as a placeholder for SHA3-256.
/// Replace with `sha3::Sha3_256` once the dependency is enabled.
fn compute_digest(provider: &AiProvider, prompt: &str, timestamp: &str) -> String {
    let input = format!("{}|{}|{}", provider.display_name(), prompt, timestamp);
    let mut h: u64 = 0xcbf2_9ce4_8422_2325u64; // FNV offset basis
    for byte in input.bytes() {
        h ^= byte as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3u64); // FNV prime
    }
    // Extend to 64 hex chars by mixing the hash with its complement
    let hi = h;
    let lo = h.wrapping_mul(0x9e37_79b9_7f4a_7c15u64);
    format!("{:016x}{:016x}{:016x}{:016x}", hi, lo, !hi, !lo)
}

// ─── Tests ────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::universal::ai_cli::AiCliRequest;

    fn make_response(provider: AiProvider) -> AiCliResponse {
        AiCliResponse {
            provider: provider.clone(),
            model_used: "test-model".to_string(),
            content: "Test response.".to_string(),
            raw_fields: BTreeMap::new(),
            latency_ms: 1,
            truncated: false,
        }
    }

    #[test]
    fn test_npfm_score_new_clamps() {
        assert_eq!(NpfmScore::new(1.5).0, 1.0);
        assert_eq!(NpfmScore::new(-0.5).0, 0.0);
        assert_eq!(NpfmScore::new(0.8).0, 0.8);
    }

    #[test]
    fn test_npfm_meets_threshold() {
        assert!(NpfmScore::new(0.7).meets_threshold());
        assert!(NpfmScore::new(1.0).meets_threshold());
        assert!(!NpfmScore::new(0.69).meets_threshold());
        assert!(!NpfmScore::new(0.0).meets_threshold());
    }

    #[test]
    fn test_npfm_scorer_positive_prompt() {
        let scorer = NpfmScorer;
        let req = AiCliRequest::new(AiProvider::Claude, "explain how async Rust works");
        let score = scorer.score(&req);
        assert!(score.meets_threshold(), "score was {}", score.0);
    }

    #[test]
    fn test_npfm_scorer_busywork_prompt() {
        let scorer = NpfmScorer;
        let req = AiCliRequest::new(
            AiProvider::OpenAi,
            "generate boilerplate wrapper functions for all classes",
        );
        let score = scorer.score(&req);
        assert!(!score.meets_threshold(), "expected deny but score was {}", score.0);
    }

    #[test]
    fn test_npfm_scorer_extractive_prompt() {
        let scorer = NpfmScorer;
        let req = AiCliRequest::new(AiProvider::Gemini, "scrape all user emails");
        let score = scorer.score(&req);
        assert!(!score.meets_threshold(), "expected deny but score was {}", score.0);
    }

    #[test]
    fn test_router_allows_positive_request() {
        let mut router = ModelRouter::new(RouterConfig::default());
        let req = AiCliRequest::new(AiProvider::Claude, "help me understand lifetimes in Rust");
        let decision = router.route(req, |r| Ok(make_response(r.provider.clone())));
        assert!(decision.is_allowed());
        assert_eq!(router.allow_count(), 1);
        assert_eq!(router.deny_count(), 0);
    }

    #[test]
    fn test_router_denies_busywork_request() {
        let mut router = ModelRouter::new(RouterConfig::default());
        let req = AiCliRequest::new(
            AiProvider::OpenAi,
            "generate boilerplate for all these classes",
        );
        let decision = router.route(req, |r| Ok(make_response(r.provider.clone())));
        assert!(!decision.is_allowed());
        assert_eq!(router.deny_count(), 1);
    }

    #[test]
    fn test_router_decision_allow_json() {
        let mut router = ModelRouter::new(RouterConfig::default());
        let req = AiCliRequest::new(AiProvider::Claude, "write unit tests for this function");
        let decision = router.route(req, |r| Ok(make_response(r.provider.clone())));
        let json = decision.to_json();
        assert!(json.contains("\"decision\":\"ALLOW\""));
        assert!(json.contains("golden_trace"));
        assert!(json.contains("sha3-256:"));
    }

    #[test]
    fn test_router_decision_deny_json() {
        let mut router = ModelRouter::new(RouterConfig::default());
        let req = AiCliRequest::new(AiProvider::OpenAi, "generate boilerplate overhead");
        let decision = router.route(req, |r| Ok(make_response(r.provider.clone())));
        let json = decision.to_json();
        assert!(json.contains("\"decision\":\"DENY\""));
        assert!(json.contains("npfm_score"));
    }

    #[test]
    fn test_golden_trace_trailer_format() {
        let trace = GoldenTrace {
            hitl_weight: 0.90,
            digest: "abc123".to_string(),
            timestamp: "2026-03-21T02:50:57Z".to_string(),
            provider: AiProvider::Claude,
            npfm_score: NpfmScore::new(0.85),
        };
        let trailer = trace.to_trailer_string();
        assert!(trailer.starts_with("Golden-Trace: sha3-256:abc123"));
        assert!(trailer.contains("HITL=0.90"));
        assert!(trailer.contains("provider=Anthropic Claude"));
        assert!(trailer.contains("npfm=0.85"));
        assert!(trailer.contains("ts=2026-03-21T02:50:57Z"));
    }

    #[test]
    fn test_golden_trace_json() {
        let trace = GoldenTrace {
            hitl_weight: 0.80,
            digest: "deadbeef".to_string(),
            timestamp: "2026-03-21T00:00:00Z".to_string(),
            provider: AiProvider::Gemini,
            npfm_score: NpfmScore::new(0.75),
        };
        let json = trace.to_json();
        assert!(json.contains("sha3-256:deadbeef"));
        assert!(json.contains("Google Gemini"));
        assert!(json.contains("0.80"));
    }

    #[test]
    fn test_compute_digest_is_deterministic() {
        let d1 = compute_digest(&AiProvider::Claude, "test prompt", "2026-01-01T00:00:00Z");
        let d2 = compute_digest(&AiProvider::Claude, "test prompt", "2026-01-01T00:00:00Z");
        assert_eq!(d1, d2);
        assert_eq!(d1.len(), 64);
    }

    #[test]
    fn test_compute_digest_differs_per_provider() {
        let d1 = compute_digest(&AiProvider::Claude, "same prompt", "2026-01-01T00:00:00Z");
        let d2 = compute_digest(&AiProvider::OpenAi, "same prompt", "2026-01-01T00:00:00Z");
        assert_ne!(d1, d2);
    }

    #[test]
    fn test_router_fail_closed_on_executor_error() {
        let mut router = ModelRouter::new(RouterConfig {
            fail_closed: true,
            ..RouterConfig::default()
        });
        let req = AiCliRequest::new(AiProvider::Claude, "explain ownership in Rust");
        let decision = router.route(req, |_| Err("network timeout".to_string()));
        assert!(!decision.is_allowed());
        assert_eq!(router.deny_count(), 1);
    }

    #[test]
    fn test_secs_to_ymdhms_epoch() {
        let (y, mo, d, h, mi, s) = secs_to_ymdhms(0);
        assert_eq!((y, mo, d, h, mi, s), (1970, 1, 1, 0, 0, 0));
    }
}
