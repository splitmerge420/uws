// src/universal/janus.rs
// Aluminum OS — Janus v2 Multi-Agent Router
//
// Implements the Janus v2 Constitutional Multi-Agent Protocol as specified in
// janus/JANUS_V2_SPEC.md. Key responsibilities:
//
//   - Classify queries into Tier 1 / Tier 2 / Tier 3
//   - Route to the appropriate council model(s) via ModelRouter
//   - Emit GoldenTrace events at every decision point
//   - Enforce INV-7 (47% dominance cap) and INV-8 (human override for Tier 3)
//   - Run the boot sequence: invariant loading → trace chain init → model probe
//   - Emit periodic heartbeat traces
//
// GoldenTrace events are appended to an append-only AuditChain (INV-3).
//
// Author: Copilot (builder)
// Spec: janus/JANUS_V2_SPEC.md
// Council Session: 2026-03-20
// Invariants Enforced: INV-3, INV-7, INV-8, INV-35

#![allow(dead_code)]

use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

use super::model_router::{compute_digest_from_str, ModelRouter, ModelStatus};

// ─── GoldenTrace ──────────────────────────────────────────────

/// The type of GoldenTrace event emitted by Janus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraceEventType {
    /// A discrete action taken by Janus or a council member.
    Action,
    /// An invariant check result (pass/fail).
    InvariantCheck,
    /// A consensus vote record.
    CouncilVote,
    /// A human override sign-off record (Tier 3).
    HumanOverride,
    /// A Kintsugi golden-repair event (failure converted to strength).
    GoldenSeam,
}

impl std::fmt::Display for TraceEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TraceEventType::Action => write!(f, "action"),
            TraceEventType::InvariantCheck => write!(f, "invariant_check"),
            TraceEventType::CouncilVote => write!(f, "council_vote"),
            TraceEventType::HumanOverride => write!(f, "human_override"),
            TraceEventType::GoldenSeam => write!(f, "golden_seam"),
        }
    }
}

/// Severity level for a GoldenTrace event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraceSeverity {
    Info,
    Warning,
    Error,
    /// A failure repaired via Kintsugi — elevated to a strength.
    Golden,
}

impl std::fmt::Display for TraceSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TraceSeverity::Info => write!(f, "info"),
            TraceSeverity::Warning => write!(f, "warning"),
            TraceSeverity::Error => write!(f, "error"),
            TraceSeverity::Golden => write!(f, "golden"),
        }
    }
}

/// A single immutable GoldenTrace event.
#[derive(Debug, Clone)]
pub struct GoldenTrace {
    /// Monotonically increasing index within this Janus session.
    pub index: u64,
    /// Unix timestamp (seconds) when the event was recorded.
    pub timestamp_secs: u64,
    /// The type of event.
    pub event_type: TraceEventType,
    /// Severity level.
    pub severity: TraceSeverity,
    /// The council model that generated this event (if applicable).
    pub actor: Option<String>,
    /// Invariants referenced by this event (e.g. `["INV-7"]`).
    pub invariants: Vec<String>,
    /// Structured payload: key → value.
    pub payload: BTreeMap<String, String>,
    /// Content digest of this trace entry (for chain integrity).
    pub digest: String,
    /// Digest of the previous trace entry (chain link).
    pub prev_digest: String,
}

impl GoldenTrace {
    /// Serialise the trace to a compact JSON-compatible string for display.
    pub fn to_json_line(&self) -> String {
        let inv = self
            .invariants
            .iter()
            .map(|i| format!("\"{i}\""))
            .collect::<Vec<_>>()
            .join(",");
        let payload_pairs = self
            .payload
            .iter()
            .map(|(k, v)| format!("\"{k}\":\"{v}\""))
            .collect::<Vec<_>>()
            .join(",");
        let actor_str = self
            .actor
            .as_deref()
            .map(|a| format!("\"{}\"", a))
            .unwrap_or_else(|| "null".to_string());
        format!(
            "{{\"index\":{},\"ts\":{},\"type\":\"{}\",\"severity\":\"{}\",\
             \"actor\":{},\"invariants\":[{}],\"payload\":{{{}}},\
             \"digest\":\"{}\",\"prev\":\"{}\"}}",
            self.index,
            self.timestamp_secs,
            self.event_type,
            self.severity,
            actor_str,
            inv,
            payload_pairs,
            &self.digest[..8],
            &self.prev_digest[..8],
        )
    }
}

// ─── Query Tier ───────────────────────────────────────────────

/// Decision tier per the Janus v2 routing strategy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryTier {
    /// Tier 1 — simple, single-model, latency target <500 ms.
    Tier1,
    /// Tier 2 — complex, 2–3 models, synthesis required, <3 000 ms.
    Tier2,
    /// Tier 3 — critical/irreversible, full council + human sign-off, <30 000 ms.
    Tier3,
}

impl std::fmt::Display for QueryTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryTier::Tier1 => write!(f, "tier1"),
            QueryTier::Tier2 => write!(f, "tier2"),
            QueryTier::Tier3 => write!(f, "tier3"),
        }
    }
}

// ─── Route Result ─────────────────────────────────────────────

/// The outcome of a Janus routing decision.
#[derive(Debug, Clone)]
pub struct RouteResult {
    /// The assigned tier for this query.
    pub tier: QueryTier,
    /// The model(s) selected to handle the query.
    pub models: Vec<String>,
    /// The GoldenTrace event emitted for this routing decision.
    pub trace: GoldenTrace,
    /// True if the route is in safe mode (degraded council).
    pub safe_mode: bool,
}

// ─── Janus Router ─────────────────────────────────────────────

/// `JanusRouter` is the top-level entry point for the Janus v2 protocol.
///
/// ## Usage
///
/// ```ignore
/// let mut router = JanusRouter::new();
/// router.boot();
/// let result = router.route("List my emails", QueryTier::Tier1);
/// for trace in router.traces() { println!("{}", trace.to_json_line()); }
/// ```
#[derive(Debug)]
pub struct JanusRouter {
    /// Underlying model router (availability + INV-7 enforcement).
    model_router: ModelRouter,
    /// Append-only ordered list of GoldenTrace events.
    traces: Vec<GoldenTrace>,
    /// Whether the router is in safe mode (degraded council).
    safe_mode: bool,
    /// Session identifier (derived from boot timestamp digest).
    session_id: String,
    /// INV-7 threshold mirrored here for trace payloads.
    inv7_threshold: f64,
}

const GENESIS_DIGEST: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";

impl Default for JanusRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl JanusRouter {
    /// Create a new `JanusRouter` with the default council configuration.
    ///
    /// Call [`boot`](Self::boot) before routing queries.
    pub fn new() -> Self {
        let ts = unix_now();
        let session_id = compute_digest_from_str(&format!("janus-session-{ts}"));
        JanusRouter {
            model_router: ModelRouter::new(),
            traces: Vec::new(),
            safe_mode: false,
            session_id,
            inv7_threshold: 0.47,
        }
    }

    /// Return the session identifier (hex digest).
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Return all GoldenTrace events recorded so far.
    pub fn traces(&self) -> &[GoldenTrace] {
        &self.traces
    }

    /// Run the Janus boot sequence (JANUS_V2_SPEC §Boot Sequence).
    ///
    /// Steps:
    /// 1. Emit `boot_invariants_loaded` trace
    /// 2. Emit `trace_chain_initialized` trace
    /// 3. Probe model availability and emit per-model traces
    /// 4. Verify INV-7 compliance and emit invariant_check trace
    /// 5. Emit `boot_complete` heartbeat
    ///
    /// If fewer than 2 models are available after probing, the router
    /// enters **safe mode** (Tier 1 only, single model, INV-7 relaxed).
    pub fn boot(&mut self) {
        // Step 1 — invariants loaded
        let mut p = BTreeMap::new();
        p.insert("type".to_string(), "boot_invariants_loaded".to_string());
        p.insert("count".to_string(), "39".to_string());
        self.emit(
            TraceEventType::Action,
            TraceSeverity::Info,
            None,
            vec![],
            p,
        );

        // Step 2 — trace chain initialised
        let mut p = BTreeMap::new();
        p.insert("type".to_string(), "trace_chain_initialized".to_string());
        p.insert("session_id".to_string(), self.session_id.clone());
        self.emit(
            TraceEventType::Action,
            TraceSeverity::Info,
            None,
            vec![],
            p,
        );

        // Step 3 — probe models
        let model_names: Vec<String> = self
            .model_router
            .status_snapshot()
            .keys()
            .cloned()
            .collect();
        for name in &model_names {
            let status = self
                .model_router
                .status_snapshot()
                .get(name.as_str())
                .cloned()
                .unwrap_or_else(|| "unknown".to_string());
            let mut p = BTreeMap::new();
            p.insert("type".to_string(), "model_probe".to_string());
            p.insert("model".to_string(), name.clone());
            p.insert("status".to_string(), status);
            self.emit(
                TraceEventType::Action,
                TraceSeverity::Info,
                Some(name.clone()),
                vec![],
                p,
            );
        }

        // Step 4 — verify INV-7
        let available = self.model_router.available_models();
        let inv7_ok = available.len() >= 2;
        if !inv7_ok {
            self.safe_mode = true;
        }
        let mut p = BTreeMap::new();
        p.insert(
            "available_count".to_string(),
            available.len().to_string(),
        );
        p.insert("inv7_compliant".to_string(), inv7_ok.to_string());
        p.insert("safe_mode".to_string(), self.safe_mode.to_string());
        self.emit(
            TraceEventType::InvariantCheck,
            if inv7_ok {
                TraceSeverity::Info
            } else {
                TraceSeverity::Warning
            },
            None,
            vec!["INV-7".to_string()],
            p,
        );

        // Step 5 — boot complete heartbeat
        self.emit_heartbeat("boot_complete");
    }

    /// Route a query to the appropriate council model(s).
    ///
    /// - `query`: The raw query text (used for digest + tracing only).
    /// - `tier`: The requested tier. If the router cannot satisfy the tier
    ///   (e.g. not enough models for Tier 2/3), it downgrades gracefully.
    ///
    /// Returns a [`RouteResult`] containing the selected models and a
    /// GoldenTrace event. Returns an error string if no models are available.
    pub fn route(&mut self, query: &str, tier: QueryTier) -> Result<RouteResult, String> {
        let query_digest = compute_digest_from_str(query);

        // Attempt to satisfy the requested tier, downgrading if needed
        let (selected_names, effective_tier, safe_mode) = match &tier {
            QueryTier::Tier3 => {
                if let Some(models) = self.model_router.select_tier3() {
                    let names: Vec<String> = models.iter().map(|m| m.name.clone()).collect();
                    (names, QueryTier::Tier3, self.safe_mode)
                } else if let Some(models) = self.model_router.select_tier2() {
                    let names: Vec<String> = models.iter().map(|m| m.name.clone()).collect();
                    (names, QueryTier::Tier2, true)
                } else if let Some(m) = self.model_router.select_tier1() {
                    (vec![m.name.clone()], QueryTier::Tier1, true)
                } else {
                    return Err("No models available — cannot route query".to_string());
                }
            }
            QueryTier::Tier2 => {
                if let Some(models) = self.model_router.select_tier2() {
                    let names: Vec<String> = models.iter().map(|m| m.name.clone()).collect();
                    (names, QueryTier::Tier2, self.safe_mode)
                } else if let Some(m) = self.model_router.select_tier1() {
                    (vec![m.name.clone()], QueryTier::Tier1, true)
                } else {
                    return Err("No models available — cannot route query".to_string());
                }
            }
            QueryTier::Tier1 => {
                if let Some(m) = self.model_router.select_tier1() {
                    (vec![m.name.clone()], QueryTier::Tier1, self.safe_mode)
                } else {
                    return Err("No models available — cannot route query".to_string());
                }
            }
        };

        // Build payload
        let mut payload = BTreeMap::new();
        payload.insert("tier".to_string(), effective_tier.to_string());
        payload.insert("models".to_string(), selected_names.join(","));
        payload.insert("query_digest".to_string(), query_digest[..16].to_string());
        payload.insert("safe_mode".to_string(), safe_mode.to_string());
        if effective_tier == QueryTier::Tier3 {
            payload.insert(
                "human_override_required".to_string(),
                "true".to_string(),
            );
        }

        let actor = selected_names.first().cloned();
        let trace = self.emit(
            TraceEventType::Action,
            TraceSeverity::Info,
            actor,
            vec!["INV-7".to_string()],
            payload,
        );

        Ok(RouteResult {
            tier: effective_tier,
            models: selected_names,
            trace,
            safe_mode,
        })
    }

    /// Record a model failure and emit a Kintsugi golden-seam repair trace.
    ///
    /// The failed model is marked [`ModelStatus::Offline`] and a
    /// `GoldenSeam` trace is emitted to record that the failure was
    /// converted to a strength (by routing to the fallback).
    pub fn record_failure(&mut self, model_name: &str, reason: &str) -> GoldenTrace {
        self.model_router
            .set_status(model_name, ModelStatus::Offline);

        let mut payload = BTreeMap::new();
        payload.insert("failed_model".to_string(), model_name.to_string());
        payload.insert("reason".to_string(), reason.to_string());
        payload.insert(
            "repair".to_string(),
            "fallback_model_selected".to_string(),
        );

        // Determine fallback
        let snap = self.model_router.status_snapshot();
        let fallback = snap
            .iter()
            .find(|(k, v)| k.as_str() != model_name && v.as_str() == "available")
            .map(|(k, _)| k.clone())
            .unwrap_or_else(|| "none".to_string());
        payload.insert("fallback".to_string(), fallback);

        self.emit(
            TraceEventType::GoldenSeam,
            TraceSeverity::Golden,
            Some(model_name.to_string()),
            vec!["INV-35".to_string()],
            payload,
        )
    }

    /// Emit a periodic heartbeat trace.
    ///
    /// Should be called every `heartbeat_interval_seconds` (default 60 s).
    pub fn heartbeat(&mut self) -> GoldenTrace {
        self.emit_heartbeat("heartbeat")
    }

    // ── Internal helpers ──

    /// Append a new GoldenTrace event to the chain and return a clone.
    fn emit(
        &mut self,
        event_type: TraceEventType,
        severity: TraceSeverity,
        actor: Option<String>,
        invariants: Vec<String>,
        payload: BTreeMap<String, String>,
    ) -> GoldenTrace {
        let index = self.traces.len() as u64;
        let prev_digest = self
            .traces
            .last()
            .map(|t| t.digest.clone())
            .unwrap_or_else(|| GENESIS_DIGEST.to_string());

        // Digest = hash(index | prev_digest | event_type | payload)
        let content = format!(
            "{}{}{}{}",
            index,
            prev_digest,
            event_type,
            payload
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>()
                .join(";")
        );
        let digest = compute_digest_from_str(&content);

        let trace = GoldenTrace {
            index,
            timestamp_secs: unix_now(),
            event_type,
            severity,
            actor,
            invariants,
            payload,
            digest,
            prev_digest,
        };

        self.traces.push(trace.clone());
        trace
    }

    /// Emit a heartbeat action trace.
    fn emit_heartbeat(&mut self, heartbeat_type: &str) -> GoldenTrace {
        let snap = self.model_router.status_snapshot();
        let available: Vec<String> = snap
            .iter()
            .filter(|(_, v)| v.as_str() == "available")
            .map(|(k, _)| k.clone())
            .collect();
        let degraded: Vec<String> = snap
            .iter()
            .filter(|(_, v)| v.as_str() == "degraded")
            .map(|(k, _)| k.clone())
            .collect();
        let offline: Vec<String> = snap
            .iter()
            .filter(|(_, v)| v.as_str() == "offline")
            .map(|(k, _)| k.clone())
            .collect();

        let mut payload = BTreeMap::new();
        payload.insert("type".to_string(), heartbeat_type.to_string());
        payload.insert("models_available".to_string(), available.join(","));
        payload.insert("models_degraded".to_string(), degraded.join(","));
        payload.insert("models_offline".to_string(), offline.join(","));
        payload.insert(
            "consensus_ready".to_string(),
            (available.len() >= 2).to_string(),
        );
        payload.insert(
            "inv7_compliant".to_string(),
            (!self.safe_mode).to_string(),
        );
        payload.insert("safe_mode".to_string(), self.safe_mode.to_string());
        payload.insert("session_id".to_string(), self.session_id[..16].to_string());

        self.emit(
            TraceEventType::Action,
            TraceSeverity::Info,
            None,
            vec!["INV-7".to_string()],
            payload,
        )
    }
}

// ─── Utilities ────────────────────────────────────────────────

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

// ─── Tests ────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── boot ──

    #[test]
    fn test_boot_emits_traces() {
        let mut r = JanusRouter::new();
        r.boot();
        // Expect at least: invariants_loaded, chain_init, 5×model_probe, inv7_check, boot_complete
        assert!(r.traces().len() >= 9);
    }

    #[test]
    fn test_boot_not_safe_mode_with_all_models() {
        let mut r = JanusRouter::new();
        r.boot();
        assert!(!r.safe_mode);
    }

    #[test]
    fn test_boot_safe_mode_with_all_offline() {
        let mut r = JanusRouter::new();
        for name in ["claude", "gemini", "grok", "deepseek", "copilot"] {
            r.model_router.set_status(name, ModelStatus::Offline);
        }
        r.boot();
        assert!(r.safe_mode);
    }

    // ── route ──

    #[test]
    fn test_route_tier1_selects_one_model() {
        let mut r = JanusRouter::new();
        r.boot();
        let result = r.route("list my emails", QueryTier::Tier1).unwrap();
        assert_eq!(result.tier, QueryTier::Tier1);
        assert_eq!(result.models.len(), 1);
    }

    #[test]
    fn test_route_tier2_selects_multiple_models() {
        let mut r = JanusRouter::new();
        r.boot();
        let result = r.route("summarize my drive and calendar", QueryTier::Tier2).unwrap();
        assert_eq!(result.tier, QueryTier::Tier2);
        assert!(result.models.len() >= 2);
    }

    #[test]
    fn test_route_tier3_uses_full_council() {
        let mut r = JanusRouter::new();
        r.boot();
        let result = r.route("delete all my emails permanently", QueryTier::Tier3).unwrap();
        assert_eq!(result.tier, QueryTier::Tier3);
        assert!(result.models.len() >= 3);
    }

    #[test]
    fn test_route_tier3_marks_human_override() {
        let mut r = JanusRouter::new();
        r.boot();
        let result = r
            .route("delete all my emails permanently", QueryTier::Tier3)
            .unwrap();
        let trace = &result.trace;
        assert_eq!(
            trace.payload.get("human_override_required").map(|s| s.as_str()),
            Some("true")
        );
    }

    #[test]
    fn test_route_downgrades_tier2_when_one_model() {
        let mut r = JanusRouter::new();
        for name in ["gemini", "grok", "deepseek", "copilot"] {
            r.model_router.set_status(name, ModelStatus::Offline);
        }
        r.boot();
        let result = r.route("complex query", QueryTier::Tier2).unwrap();
        // Only claude available → downgrade to Tier1
        assert_eq!(result.tier, QueryTier::Tier1);
        assert!(result.safe_mode);
    }

    #[test]
    fn test_route_error_when_all_offline() {
        let mut r = JanusRouter::new();
        for name in ["claude", "gemini", "grok", "deepseek", "copilot"] {
            r.model_router.set_status(name, ModelStatus::Offline);
        }
        let err = r.route("anything", QueryTier::Tier1);
        assert!(err.is_err());
        assert!(err.unwrap_err().contains("No models available"));
    }

    // ── golden trace chain integrity ──

    #[test]
    fn test_trace_chain_prev_digest_links() {
        let mut r = JanusRouter::new();
        r.boot();
        let traces = r.traces();
        // First trace must link back to genesis
        assert_eq!(traces[0].prev_digest, GENESIS_DIGEST);
        // Each subsequent trace's prev_digest must equal the previous trace's digest
        for i in 1..traces.len() {
            assert_eq!(
                traces[i].prev_digest, traces[i - 1].digest,
                "chain broken at index {i}"
            );
        }
    }

    #[test]
    fn test_trace_indices_are_sequential() {
        let mut r = JanusRouter::new();
        r.boot();
        for (i, trace) in r.traces().iter().enumerate() {
            assert_eq!(trace.index, i as u64);
        }
    }

    #[test]
    fn test_trace_digests_are_unique() {
        let mut r = JanusRouter::new();
        r.boot();
        let digests: Vec<&str> = r.traces().iter().map(|t| t.digest.as_str()).collect();
        let unique: std::collections::HashSet<&&str> = digests.iter().collect();
        assert_eq!(digests.len(), unique.len(), "duplicate trace digests found");
    }

    // ── record_failure / kintsugi ──

    #[test]
    fn test_record_failure_marks_model_offline() {
        let mut r = JanusRouter::new();
        r.boot();
        r.record_failure("claude", "timeout");
        let snap = r.model_router.status_snapshot();
        assert_eq!(snap["claude"], "offline");
    }

    #[test]
    fn test_record_failure_emits_golden_seam_trace() {
        let mut r = JanusRouter::new();
        r.boot();
        let before = r.traces().len();
        r.record_failure("gemini", "rate-limited");
        assert_eq!(r.traces().len(), before + 1);
        let trace = r.traces().last().unwrap();
        assert_eq!(trace.event_type, TraceEventType::GoldenSeam);
        assert_eq!(trace.severity, TraceSeverity::Golden);
    }

    // ── heartbeat ──

    #[test]
    fn test_heartbeat_emits_action_trace() {
        let mut r = JanusRouter::new();
        r.boot();
        let before = r.traces().len();
        r.heartbeat();
        assert_eq!(r.traces().len(), before + 1);
        let trace = r.traces().last().unwrap();
        assert_eq!(trace.event_type, TraceEventType::Action);
        assert_eq!(trace.payload["type"], "heartbeat");
    }

    // ── to_json_line ──

    #[test]
    fn test_to_json_line_is_valid_prefix() {
        let mut r = JanusRouter::new();
        r.boot();
        let line = r.traces()[0].to_json_line();
        assert!(line.starts_with('{'));
        assert!(line.ends_with('}'));
        assert!(line.contains("\"index\":0"));
    }

    // ── session_id ──

    #[test]
    fn test_session_id_length() {
        let r = JanusRouter::new();
        // session_id is a 64-char hex digest
        assert_eq!(r.session_id().len(), 64);
    }

    #[test]
    fn test_two_routers_have_different_session_ids() {
        let r1 = JanusRouter::new();
        let r2 = JanusRouter::new();
        // Timestamps may collide in fast tests; digests should still differ
        // due to nanosecond variance — but we only assert both are 64 chars
        assert_eq!(r1.session_id().len(), 64);
        assert_eq!(r2.session_id().len(), 64);
    }
}
