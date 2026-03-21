// src/telemetry/kpi.rs
// Aluminum OS — Net-Positive Flourishing Metric (NPFM)
//
// This module replaces throughput-centric KPIs with indicators that measure
// actual civilizational health: jobs augmented or created, human knowledge
// expanded, provenance payouts triggered, and fiduciary compliance.
//
// Key principle: **Throughput is not a worthy indicator of human flourishing.**
//
// The AntiBusyworkFactor penalises workflows that create purely administrative
// or repetitive human tasks (e.g., redundant clicking), while rewarding the
// elimination of busywork when the displaced human energy is routed into one of
// three protected high-agency tiers:
//
//   1. High-Agency Oversight   — HITL Swarm Commanders, provenance validators
//   2. Creative Genesis        — IP authorship, cross-domain synthesis
//   3. Physical/Metaverse Eng  — Building/training metaverse-proven robotics
//
// Council Session: 2026-03-20
// Authority: Dave Sheldon (INV-5)

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::{string::String, vec::Vec, format};

// ─── Job Classification ───────────────────────────────────────────────────────

/// The three high-agency tiers that justify job creation or transitions.
/// Only jobs that fall into one of these tiers contribute *positively* to
/// the NetPositiveScore.  Purely administrative or repetitive tasks do not.
#[derive(Debug, Clone, PartialEq)]
pub enum JobTier {
    /// HITL Swarm Commanders, provenance validators, batch drone oversight.
    HighAgencyOversight,
    /// IP authorship, cross-domain synthesis, creative knowledge work.
    CreativeGenesis,
    /// Designing, building, or training metaverse-proven physical/robotic systems.
    PhysicalMetaverseEngineering,
    /// Purely administrative or repetitive work — scores zero / negative.
    BusyworkAdministrative,
}

impl JobTier {
    /// Returns the NPFM weight for this tier.
    /// High-agency tiers earn a positive multiplier; busywork earns a penalty.
    pub fn npfm_weight(&self) -> f64 {
        match self {
            JobTier::HighAgencyOversight => 1.0,
            JobTier::CreativeGenesis => 1.2,
            JobTier::PhysicalMetaverseEngineering => 1.5,
            JobTier::BusyworkAdministrative => -0.8,
        }
    }
}

// ─── Job Record ──────────────────────────────────────────────────────────────

/// A single job created, augmented, or displaced by an AI-driven workflow.
#[derive(Debug, Clone)]
pub struct JobRecord {
    /// Human-readable description of the job.
    pub description: String,
    /// Classification of the job's tier.
    pub tier: JobTier,
    /// Whether this job was *created* (true) or *eliminated* (false).
    /// Eliminating a BusyworkAdministrative job scores positively if
    /// `displaced_to` routes into a high-agency tier.
    pub created: bool,
    /// Optional: the tier the displaced human energy is being routed to.
    /// Only relevant when `created == false`.
    pub displaced_to: Option<JobTier>,
}

impl JobRecord {
    /// Create a new job record for a job being created.
    pub fn new_created(description: impl Into<String>, tier: JobTier) -> Self {
        JobRecord {
            description: description.into(),
            tier,
            created: true,
            displaced_to: None,
        }
    }

    /// Create a new job record for a busywork job being eliminated.
    /// Provide the high-agency tier the displaced human is being routed to.
    pub fn new_eliminated(
        description: impl Into<String>,
        displaced_to: Option<JobTier>,
    ) -> Self {
        JobRecord {
            description: description.into(),
            tier: JobTier::BusyworkAdministrative,
            created: false,
            displaced_to,
        }
    }

    /// Compute the NPFM contribution of this job record.
    ///
    /// - Creating a high-agency job: positive weight
    /// - Creating busywork: negative weight (AntiBusyworkFactor penalty)
    /// - Eliminating busywork and routing displaced energy to a high-agency
    ///   tier: positive (elimination bonus + destination tier weight)
    /// - Eliminating busywork with no re-routing: zero (no fiduciary routing)
    pub fn npfm_contribution(&self) -> f64 {
        if self.created {
            self.tier.npfm_weight()
        } else {
            // Eliminating busywork
            match &self.displaced_to {
                Some(destination) => {
                    // Bonus for destroying busywork + value of new tier
                    0.5 + destination.npfm_weight()
                }
                None => 0.0, // No fiduciary routing — neutral
            }
        }
    }
}

// ─── Anti-Busywork Factor ────────────────────────────────────────────────────

/// Measures the net fiduciary quality of job-creation proposals.
///
/// A score > 0 means the workflow is net-positive for human flourishing.
/// A score ≤ 0 means the workflow creates or preserves useless busywork and
/// must be blocked or revised before execution.
#[derive(Debug, Clone)]
pub struct AntiBusyworkFactor {
    /// Individual job records contributing to this factor.
    pub job_records: Vec<JobRecord>,
}

impl AntiBusyworkFactor {
    pub fn new() -> Self {
        AntiBusyworkFactor {
            job_records: Vec::new(),
        }
    }

    pub fn add_job(&mut self, record: JobRecord) {
        self.job_records.push(record);
    }

    /// Compute the aggregate anti-busywork score.
    /// Positive = fiduciary; Negative = busywork maximizer (paperclip failure).
    pub fn score(&self) -> f64 {
        self.job_records.iter().map(|r| r.npfm_contribution()).sum()
    }

    /// Returns true if the proposal passes the fiduciary threshold (score > 0).
    pub fn passes_fiduciary_threshold(&self) -> bool {
        self.score() > 0.0
    }

    /// Human-readable summary of the factor.
    pub fn summary(&self) -> String {
        let score = self.score();
        let status = if self.passes_fiduciary_threshold() {
            "PASS"
        } else {
            "FAIL — busywork maximization detected"
        };
        format!("AntiBusyworkFactor: {:.3} [{}]", score, status)
    }
}

impl Default for AntiBusyworkFactor {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Net-Positive Score ──────────────────────────────────────────────────────

/// The master Net-Positive Flourishing Metric score for a workflow or operation.
///
/// Replaces throughput as the primary KPI.  All dimensions must be non-negative
/// for an operation to be considered net-positive.
#[derive(Debug, Clone)]
pub struct NetPositiveScore {
    /// Anti-busywork fiduciary analysis of jobs created/displaced.
    pub anti_busywork: AntiBusyworkFactor,
    /// Number of new knowledge nodes added to the ontology.
    pub human_knowledge_expanded: u32,
    /// Number of provenance payout events triggered.
    pub provenance_payouts_triggered: u32,
}

impl NetPositiveScore {
    pub fn new() -> Self {
        NetPositiveScore {
            anti_busywork: AntiBusyworkFactor::new(),
            human_knowledge_expanded: 0,
            provenance_payouts_triggered: 0,
        }
    }

    /// Compute the composite NPFM score.
    ///
    /// Formula:
    ///   composite = anti_busywork_score
    ///             + (human_knowledge_expanded * 0.1)
    ///             + (provenance_payouts_triggered * 0.2)
    pub fn composite(&self) -> f64 {
        self.anti_busywork.score()
            + (self.human_knowledge_expanded as f64 * 0.1)
            + (self.provenance_payouts_triggered as f64 * 0.2)
    }

    /// Returns true if the operation is net-positive and fiduciary-compliant.
    pub fn is_net_positive(&self) -> bool {
        self.anti_busywork.passes_fiduciary_threshold() && self.composite() > 0.0
    }

    /// Human-readable report.
    pub fn report(&self) -> String {
        format!(
            "NetPositiveScore {{\n  {}\n  human_knowledge_expanded: {}\n  provenance_payouts_triggered: {}\n  composite: {:.3}\n  verdict: {}\n}}",
            self.anti_busywork.summary(),
            self.human_knowledge_expanded,
            self.provenance_payouts_triggered,
            self.composite(),
            if self.is_net_positive() { "NET-POSITIVE ✓" } else { "NET-NEGATIVE ✗ — BLOCKED" },
        )
    }
}

impl Default for NetPositiveScore {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creating_high_agency_job_is_positive() {
        let record = JobRecord::new_created(
            "Provenance validator role",
            JobTier::HighAgencyOversight,
        );
        assert!(record.npfm_contribution() > 0.0);
    }

    #[test]
    fn creating_busywork_job_is_negative() {
        let record = JobRecord::new_created(
            "Redundant TPS report clicking",
            JobTier::BusyworkAdministrative,
        );
        assert!(record.npfm_contribution() < 0.0);
    }

    #[test]
    fn eliminating_busywork_with_routing_is_positive() {
        let record = JobRecord::new_eliminated(
            "Manual data-entry task automated",
            Some(JobTier::HighAgencyOversight),
        );
        assert!(record.npfm_contribution() > 0.0);
    }

    #[test]
    fn eliminating_busywork_without_routing_is_neutral() {
        let record = JobRecord::new_eliminated(
            "Task automated, no transition plan",
            None,
        );
        assert_eq!(record.npfm_contribution(), 0.0);
    }

    #[test]
    fn anti_busywork_factor_blocks_paperclip_maximizer() {
        let mut factor = AntiBusyworkFactor::new();
        // Add ten busywork jobs — a classic paperclip maximizer
        for i in 0..10 {
            factor.add_job(JobRecord::new_created(
                format!("Redundant admin task {}", i),
                JobTier::BusyworkAdministrative,
            ));
        }
        assert!(!factor.passes_fiduciary_threshold());
    }

    #[test]
    fn net_positive_score_passes_with_high_agency_jobs() {
        let mut score = NetPositiveScore::new();
        score.anti_busywork.add_job(JobRecord::new_created(
            "Swarm Commander oversight role",
            JobTier::HighAgencyOversight,
        ));
        score.human_knowledge_expanded = 5;
        score.provenance_payouts_triggered = 2;
        assert!(score.is_net_positive());
    }

    #[test]
    fn net_positive_score_blocked_for_pure_busywork() {
        let mut score = NetPositiveScore::new();
        score.anti_busywork.add_job(JobRecord::new_created(
            "Infinite admin checkbox parade",
            JobTier::BusyworkAdministrative,
        ));
        assert!(!score.is_net_positive());
    }

    #[test]
    fn streamlining_busywork_increases_score() {
        let mut score_before = NetPositiveScore::new();
        score_before.anti_busywork.add_job(JobRecord::new_created(
            "Manual reconciliation job",
            JobTier::BusyworkAdministrative,
        ));

        let mut score_after = NetPositiveScore::new();
        score_after.anti_busywork.add_job(JobRecord::new_eliminated(
            "Manual reconciliation job automated",
            Some(JobTier::CreativeGenesis),
        ));

        assert!(score_after.composite() > score_before.composite());
    }

    #[test]
    fn physical_metaverse_engineering_tier_has_highest_weight() {
        assert!(
            JobTier::PhysicalMetaverseEngineering.npfm_weight()
                > JobTier::CreativeGenesis.npfm_weight()
        );
        assert!(
            JobTier::CreativeGenesis.npfm_weight()
                > JobTier::HighAgencyOversight.npfm_weight()
        );
        assert!(
            JobTier::HighAgencyOversight.npfm_weight()
                > JobTier::BusyworkAdministrative.npfm_weight()
        );
    }
}
