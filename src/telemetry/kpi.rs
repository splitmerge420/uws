// src/telemetry/kpi.rs
// Aluminum OS — Net-Positive Flourishing Metric (NPFM) System
//
// Replaces throughput-centric KPIs (RPS, latency) with Regenerative KPIs
// that measure real human value: jobs created, knowledge expanded, and
// provenance payouts triggered.
//
// Core principle: "Throughput is a False Idol."
// Raw speed or volume is not an indicator of human flourishing.
// Every AI action must be net-positive across these dimensions or be blocked.
//
// Author: GitHub Copilot (builder)
// Council Session: 2026-03-21
// Invariants Enforced: INV-5 (Human Flourishing), INV-2 (Consent/Augmentation)

#![allow(dead_code)]

// ─── Net-Positive Flourishing Score ──────────────────────────────────────────

/// The primary KPI for every AI action in Aluminum OS.
///
/// A `NetPositiveScore` is computed before any batch operation is executed.
/// If the score is negative the operation is **blocked** and requires
/// a Tier-1 human override before it can proceed.
///
/// Scoring rules
/// -------------
/// * `jobs_created`              — each new oversight/provenance role adds +10 pts
/// * `human_knowledge_expanded`  — each ontological addition adds +5 pts
/// * `provenance_payouts_triggered` — each payout event adds +8 pts
/// * `throughput_penalty`        — deducted when raw throughput is the *only*
///   measurable gain and no human-flourishing dimension is served
///
/// Net score = jobs_component + knowledge_component + payout_component
///             - throughput_penalty
#[derive(Debug, Clone, PartialEq)]
pub struct NetPositiveScore {
    /// New human jobs created or protected by this operation.
    /// Each job adds +10 to the raw score.
    pub jobs_created: u32,

    /// Ontological additions or expansions of human knowledge.
    /// Each addition adds +5 to the raw score.
    pub human_knowledge_expanded: u32,

    /// Provenance payout events triggered (IP royalties, HITL rewards, etc.).
    /// Each event adds +8 to the raw score.
    pub provenance_payouts_triggered: u32,

    /// Penalty applied when high throughput negatively impacts the score by
    /// displacing human value without a regenerative replacement path.
    /// Prevents AI runaway optimisation for raw speed at the expense of people.
    pub throughput_penalty: i32,
}

impl NetPositiveScore {
    /// Construct a new score from its constituent dimensions.
    pub fn new(
        jobs_created: u32,
        human_knowledge_expanded: u32,
        provenance_payouts_triggered: u32,
        throughput_penalty: i32,
    ) -> Self {
        Self {
            jobs_created,
            human_knowledge_expanded,
            provenance_payouts_triggered,
            throughput_penalty,
        }
    }

    /// Calculate the final numeric score.
    ///
    /// Returns a signed integer so that negative scores (harmful operations)
    /// are clearly distinguishable from neutral (0) or positive outcomes.
    pub fn calculate(&self) -> i64 {
        let jobs_component = (self.jobs_created as i64) * 10;
        let knowledge_component = (self.human_knowledge_expanded as i64) * 5;
        let payout_component = (self.provenance_payouts_triggered as i64) * 8;
        let penalty = self.throughput_penalty as i64;

        jobs_component + knowledge_component + payout_component - penalty
    }

    /// Returns `true` when this operation is net-positive for human flourishing.
    pub fn is_net_positive(&self) -> bool {
        self.calculate() > 0
    }

    /// Returns `true` when this operation is net-neutral or better.
    pub fn is_acceptable(&self) -> bool {
        self.calculate() >= 0
    }

    /// Human-readable summary of the score breakdown.
    pub fn summary(&self) -> String {
        format!(
            "NetPositiveScore {{ \
             jobs_created: {} (+{}pts), \
             human_knowledge_expanded: {} (+{}pts), \
             provenance_payouts_triggered: {} (+{}pts), \
             throughput_penalty: -{}, \
             total: {} }}",
            self.jobs_created,
            (self.jobs_created as i64) * 10,
            self.human_knowledge_expanded,
            (self.human_knowledge_expanded as i64) * 5,
            self.provenance_payouts_triggered,
            (self.provenance_payouts_triggered as i64) * 8,
            self.throughput_penalty,
            self.calculate(),
        )
    }
}

impl Default for NetPositiveScore {
    fn default() -> Self {
        Self::new(0, 0, 0, 0)
    }
}

// ─── Regenerative KPI Registry ───────────────────────────────────────────────

/// The complete set of Regenerative KPIs tracked by Aluminum OS.
///
/// Deliberately excludes throughput-only metrics such as requests-per-second
/// or raw latency, which do not measure human flourishing.
#[derive(Debug, Clone)]
pub struct RegenerativeKpiRegistry {
    /// Cumulative score across all operations in this session.
    pub session_score: NetPositiveScore,

    /// Total operations that were allowed (score >= 0).
    pub operations_allowed: u64,

    /// Total operations that were blocked (score < 0).
    pub operations_blocked: u64,

    /// Total Tier-1 human overrides that were invoked.
    pub tier1_overrides: u64,
}

impl RegenerativeKpiRegistry {
    pub fn new() -> Self {
        Self {
            session_score: NetPositiveScore::default(),
            operations_allowed: 0,
            operations_blocked: 0,
            tier1_overrides: 0,
        }
    }

    /// Record an operation outcome, updating counters accordingly.
    pub fn record(&mut self, score: &NetPositiveScore, was_overridden: bool) {
        if score.is_acceptable() {
            self.operations_allowed += 1;
        } else {
            self.operations_blocked += 1;
        }
        if was_overridden {
            self.tier1_overrides += 1;
        }
        // Accumulate dimensions
        self.session_score.jobs_created += score.jobs_created;
        self.session_score.human_knowledge_expanded += score.human_knowledge_expanded;
        self.session_score.provenance_payouts_triggered += score.provenance_payouts_triggered;
        self.session_score.throughput_penalty += score.throughput_penalty;
    }

    /// Human-readable registry summary.
    pub fn summary(&self) -> String {
        format!(
            "RegenerativeKpiRegistry {{\n  \
             session_score: {},\n  \
             operations_allowed: {},\n  \
             operations_blocked: {},\n  \
             tier1_overrides: {}\n}}",
            self.session_score.summary(),
            self.operations_allowed,
            self.operations_blocked,
            self.tier1_overrides,
        )
    }
}

impl Default for RegenerativeKpiRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positive_score_when_jobs_created() {
        let score = NetPositiveScore::new(2, 0, 0, 0);
        assert_eq!(score.calculate(), 20);
        assert!(score.is_net_positive());
        assert!(score.is_acceptable());
    }

    #[test]
    fn positive_score_with_all_dimensions() {
        let score = NetPositiveScore::new(1, 2, 3, 0);
        // 10 + 10 + 24 = 44
        assert_eq!(score.calculate(), 44);
        assert!(score.is_net_positive());
    }

    #[test]
    fn throughput_penalty_reduces_score() {
        let score = NetPositiveScore::new(0, 0, 0, 15);
        assert_eq!(score.calculate(), -15);
        assert!(!score.is_net_positive());
        assert!(!score.is_acceptable());
    }

    #[test]
    fn penalty_can_cancel_positive_dims() {
        // 1 job (+10) but high throughput penalty (-20) => net -10
        let score = NetPositiveScore::new(1, 0, 0, 20);
        assert_eq!(score.calculate(), -10);
        assert!(!score.is_net_positive());
    }

    #[test]
    fn zero_score_is_acceptable_but_not_positive() {
        let score = NetPositiveScore::new(1, 0, 0, 10);
        assert_eq!(score.calculate(), 0);
        assert!(!score.is_net_positive());
        assert!(score.is_acceptable());
    }

    #[test]
    fn default_score_is_zero() {
        let score = NetPositiveScore::default();
        assert_eq!(score.calculate(), 0);
        assert!(score.is_acceptable());
    }

    #[test]
    fn registry_tracks_allowed_and_blocked() {
        let mut registry = RegenerativeKpiRegistry::new();
        let good = NetPositiveScore::new(1, 1, 1, 0);
        let bad = NetPositiveScore::new(0, 0, 0, 5);
        registry.record(&good, false);
        registry.record(&bad, true);
        assert_eq!(registry.operations_allowed, 1);
        assert_eq!(registry.operations_blocked, 1);
        assert_eq!(registry.tier1_overrides, 1);
    }

    #[test]
    fn summary_contains_total() {
        let score = NetPositiveScore::new(1, 2, 3, 1);
        let summary = score.summary();
        assert!(summary.contains("total:"));
    }
}
