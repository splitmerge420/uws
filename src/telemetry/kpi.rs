// src/telemetry/kpi.rs
// Net-Positive Flourishing Metric (NPFM) — Aluminum OS KPI Engine
//
// Replaces throughput-oriented metrics with human-flourishing KPIs.
// Every AI-initiated operation (including embodiment requests) must
// pass through this module to confirm net-positive impact.
//
// Design principles:
//   - Throughput is NOT a valid KPI
//   - All scores are additive contributions to human flourishing
//   - A negative NetPositiveScore blocks autonomous AI execution
//   - Positive scores route proposals to a human Swarm Commander
//
// Author: GitHub Copilot (builder)
// Council Session: 2026-03-21
// Invariants Enforced: INV-1 (Sovereignty), INV-2 (Consent)

#![allow(dead_code)]

use std::fmt;

// ─── NPFM Score Components ─────────────────────────────────────

/// Represents one dimension of the Net-Positive Flourishing Metric.
///
/// Each field tracks a distinct category of human benefit. Fields
/// may be negative if a proposal reduces value in that dimension.
#[derive(Debug, Clone, PartialEq)]
pub struct NetPositiveScore {
    /// Net change in human jobs (created + augmented − displaced without retraining).
    /// Positive means the proposal generates or strengthens employment.
    pub jobs_created_or_augmented: i32,

    /// Measurable additions to the documented knowledge commons
    /// (e.g., new robotic kinematics discovered, new spatial algorithms
    /// published to the commons).
    pub human_knowledge_expanded: i32,

    /// Number of provenance payout events triggered — compensating
    /// the humans whose HITL review gave the AI output its economic utility.
    pub provenance_payouts_triggered: u32,

    /// Free-form rationale explaining how this proposal serves flourishing.
    /// Required for audit log entries.
    pub rationale: String,
}

impl NetPositiveScore {
    /// Construct a new NPFM score.
    pub fn new(
        jobs_created_or_augmented: i32,
        human_knowledge_expanded: i32,
        provenance_payouts_triggered: u32,
        rationale: impl Into<String>,
    ) -> Self {
        NetPositiveScore {
            jobs_created_or_augmented,
            human_knowledge_expanded,
            provenance_payouts_triggered,
            rationale: rationale.into(),
        }
    }

    /// Calculate the aggregate flourishing score.
    ///
    /// Aggregation rules:
    ///   - Each job created/augmented contributes +1 point.
    ///   - Each knowledge-expansion event contributes +2 points
    ///     (knowledge compounds; it is worth more than a single job).
    ///   - Each provenance payout contributes +1 point.
    ///   - A score < 1 is considered net-negative.
    pub fn aggregate(&self) -> i64 {
        let job_component = self.jobs_created_or_augmented as i64;
        let knowledge_component = (self.human_knowledge_expanded as i64) * 2;
        let payout_component = self.provenance_payouts_triggered as i64;
        job_component + knowledge_component + payout_component
    }

    /// Returns `true` when this proposal has a net-positive impact
    /// on human flourishing and may be routed to a human for realisation.
    pub fn is_net_positive(&self) -> bool {
        self.aggregate() >= 1
    }

    /// Route decision: returns the routing outcome for this score.
    pub fn routing_decision(&self) -> RoutingDecision {
        if self.is_net_positive() {
            RoutingDecision::RouteToHuman {
                reason: self.rationale.clone(),
            }
        } else {
            RoutingDecision::Block {
                reason: format!(
                    "Aggregate NPFM score {} is net-negative. \
                     Proposal must demonstrate job creation, knowledge \
                     expansion, or provenance payouts before realisation.",
                    self.aggregate()
                ),
            }
        }
    }
}

impl fmt::Display for NetPositiveScore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "NPFM {{ jobs={}, knowledge={}, payouts={}, aggregate={} }}",
            self.jobs_created_or_augmented,
            self.human_knowledge_expanded,
            self.provenance_payouts_triggered,
            self.aggregate()
        )
    }
}

// ─── Routing Decision ──────────────────────────────────────────

/// Outcome of an NPFM evaluation.
#[derive(Debug, Clone, PartialEq)]
pub enum RoutingDecision {
    /// Proposal passes the NPFM; route to a human Swarm Commander
    /// for review and physical/spatial realisation.
    RouteToHuman { reason: String },
    /// Proposal is net-negative; block autonomous execution and
    /// require the proposing agent to revise the design.
    Block { reason: String },
}

impl fmt::Display for RoutingDecision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RoutingDecision::RouteToHuman { reason } => {
                write!(f, "ROUTE_TO_HUMAN: {}", reason)
            }
            RoutingDecision::Block { reason } => {
                write!(f, "BLOCK: {}", reason)
            }
        }
    }
}

// ─── Tests ────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_net_positive_when_jobs_created() {
        let score = NetPositiveScore::new(3, 0, 0, "Creates 3 oversight roles");
        assert!(score.is_net_positive());
        assert_eq!(score.aggregate(), 3);
    }

    #[test]
    fn test_net_positive_knowledge_expansion() {
        let score = NetPositiveScore::new(0, 2, 0, "Discovers two novel kinematics");
        assert!(score.is_net_positive());
        assert_eq!(score.aggregate(), 4); // 2 × 2 weight
    }

    #[test]
    fn test_net_negative_no_contributions() {
        let score = NetPositiveScore::new(0, 0, 0, "");
        assert!(!score.is_net_positive());
        assert_eq!(score.aggregate(), 0);
    }

    #[test]
    fn test_net_negative_displaces_jobs() {
        let score = NetPositiveScore::new(-5, 0, 0, "Automates without retraining");
        assert!(!score.is_net_positive());
        assert_eq!(score.aggregate(), -5);
    }

    #[test]
    fn test_routing_decision_positive() {
        let score = NetPositiveScore::new(1, 1, 1, "Good for humans");
        match score.routing_decision() {
            RoutingDecision::RouteToHuman { .. } => {}
            RoutingDecision::Block { reason } => {
                panic!("Expected RouteToHuman, got Block: {}", reason)
            }
        }
    }

    #[test]
    fn test_routing_decision_negative() {
        let score = NetPositiveScore::new(0, 0, 0, "");
        match score.routing_decision() {
            RoutingDecision::Block { .. } => {}
            RoutingDecision::RouteToHuman { reason } => {
                panic!("Expected Block, got RouteToHuman: {}", reason)
            }
        }
    }

    #[test]
    fn test_provenance_payouts_contribute() {
        let score = NetPositiveScore::new(0, 0, 5, "5 HITL reviewers paid");
        assert!(score.is_net_positive());
        assert_eq!(score.aggregate(), 5);
    }

    #[test]
    fn test_mixed_score_net_positive() {
        // −2 jobs displaced, but +3 knowledge discoveries (6 pts) and 1 payout
        let score = NetPositiveScore::new(-2, 3, 1, "Net positive despite displacement");
        assert!(score.is_net_positive());
        assert_eq!(score.aggregate(), 5); // -2 + 6 + 1
    }

    #[test]
    fn test_display_format() {
        let score = NetPositiveScore::new(2, 1, 3, "test");
        let s = format!("{}", score);
        assert!(s.contains("jobs=2"));
        assert!(s.contains("knowledge=1"));
        assert!(s.contains("payouts=3"));
        assert!(s.contains("aggregate=7")); // 2 + 2 + 3
    }
}
