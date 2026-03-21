// src/embodiment/physical.rs
// Aluminum OS — Physical Embodiment Protocol
//
// This module enforces the principle that **metaverse-trained and designed
// robotics must be mathematically proven to be functionally superior in
// manufacture, design, and training before physical realisation is approved.**
//
// Workflow:
//   1. An AI (or human designer) submits a RoboticChassisProposal.
//   2. The proposal accumulates SimulationFidelityScore from exhaustive
//      metaverse training runs.
//   3. Once the score exceeds the required threshold, a human fiduciary
//      can approve the proposal for physical manufacture.
//   4. All proposals are gated by the NPFM — the robotic system must create
//      or augment high-agency human jobs (building, maintaining, overseeing).
//
// Council Session: 2026-03-20
// Authority: Dave Sheldon (INV-5)

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::{string::String, vec::Vec, format};

use crate::telemetry::kpi::{JobRecord, JobTier, NetPositiveScore};

// ─── Simulation Fidelity Score ────────────────────────────────────────────────

/// Minimum simulation fidelity required before physical realisation is allowed.
/// Expressed as a 0.0–1.0 confidence value.
pub const MINIMUM_FIDELITY_FOR_APPROVAL: f64 = 0.90;

/// Records the result of a single metaverse simulation run.
#[derive(Debug, Clone)]
pub struct SimulationRun {
    /// Human-readable description of the simulation scenario.
    pub scenario: String,
    /// Confidence score achieved in this run (0.0 = total failure, 1.0 = perfect).
    pub confidence: f64,
    /// Number of independent iterations run in this scenario.
    pub iterations: u32,
    /// Whether this run's results were reviewed and signed off by a human fiduciary.
    pub human_reviewed: bool,
}

impl SimulationRun {
    pub fn new(
        scenario: impl Into<String>,
        confidence: f64,
        iterations: u32,
        human_reviewed: bool,
    ) -> Self {
        SimulationRun {
            scenario: scenario.into(),
            confidence: confidence.clamp(0.0, 1.0),
            iterations,
            human_reviewed,
        }
    }

    /// Weighted contribution: human-reviewed runs carry full weight;
    /// unreviewed runs are discounted by 50%.
    pub fn weighted_confidence(&self) -> f64 {
        if self.human_reviewed {
            self.confidence
        } else {
            self.confidence * 0.5
        }
    }
}

/// Aggregate simulation fidelity score across all metaverse training runs.
///
/// This score must exceed `MINIMUM_FIDELITY_FOR_APPROVAL` before a
/// `RoboticChassisProposal` may be approved for physical manufacture.
#[derive(Debug, Clone)]
pub struct SimulationFidelityScore {
    /// All simulation runs contributing to this score.
    pub runs: Vec<SimulationRun>,
}

impl SimulationFidelityScore {
    pub fn new() -> Self {
        SimulationFidelityScore { runs: Vec::new() }
    }

    pub fn add_run(&mut self, run: SimulationRun) {
        self.runs.push(run);
    }

    /// Compute the weighted mean confidence across all runs.
    /// Returns 0.0 if no runs have been submitted.
    pub fn aggregate(&self) -> f64 {
        if self.runs.is_empty() {
            return 0.0;
        }
        let total_weight: f64 = self.runs.iter().map(|r| {
            if r.human_reviewed { 1.0 } else { 0.5 }
        }).sum();
        let weighted_sum: f64 = self.runs.iter().map(|r| r.weighted_confidence()).sum();
        if total_weight == 0.0 {
            0.0
        } else {
            weighted_sum / total_weight
        }
    }

    /// Returns true when fidelity is high enough to propose physical realisation.
    pub fn ready_for_physical_realisation(&self) -> bool {
        self.aggregate() >= MINIMUM_FIDELITY_FOR_APPROVAL
    }

    /// Human-readable summary of the fidelity score.
    pub fn summary(&self) -> String {
        let agg = self.aggregate();
        let status = if self.ready_for_physical_realisation() {
            "READY FOR PHYSICAL REALISATION ✓"
        } else {
            "INSUFFICIENT FIDELITY — continue metaverse training ✗"
        };
        format!(
            "SimulationFidelityScore: {:.3} / {:.3} required [{}] ({} runs)",
            agg,
            MINIMUM_FIDELITY_FOR_APPROVAL,
            status,
            self.runs.len(),
        )
    }
}

impl Default for SimulationFidelityScore {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Robotic Chassis Proposal ─────────────────────────────────────────────────

/// Approval status for a RoboticChassisProposal.
#[derive(Debug, Clone, PartialEq)]
pub enum ProposalStatus {
    /// Awaiting sufficient simulation fidelity.
    PendingSimulation,
    /// Simulation fidelity achieved; awaiting human fiduciary approval.
    AwaitingFiduciaryApproval,
    /// Approved by a human fiduciary — cleared for physical manufacture.
    Approved { approved_by: String },
    /// Rejected — either fidelity too low or NPFM failed.
    Rejected { reason: String },
}

/// A formal proposal for an AI to co-design a physical robotic body or hybrid.
///
/// The proposal bundles:
///   - Kinematic/URDF specifications
///   - Metaverse simulation evidence (`SimulationFidelityScore`)
///   - NPFM validation (`NetPositiveScore`) proving the robot creates or augments
///     human jobs rather than eliminating them without compensation
///
/// Physical realisation is only permitted once simulation fidelity meets the
/// threshold **and** a human fiduciary explicitly approves.
#[derive(Debug, Clone)]
pub struct RoboticChassisProposal {
    /// Unique identifier for this proposal.
    pub id: String,
    /// Name of the proposing AI or engineering team.
    pub proposed_by: String,
    /// Human-readable description of the chassis design intent.
    pub design_intent: String,
    /// Optional path to a URDF or kinematic specification file.
    pub urdf_spec_path: Option<String>,
    /// Simulation evidence accumulated across metaverse training runs.
    pub simulation_fidelity: SimulationFidelityScore,
    /// NPFM analysis: does this robot create/augment high-agency human jobs?
    pub net_positive_score: NetPositiveScore,
    /// Current approval status.
    pub status: ProposalStatus,
}

impl RoboticChassisProposal {
    pub fn new(
        id: impl Into<String>,
        proposed_by: impl Into<String>,
        design_intent: impl Into<String>,
    ) -> Self {
        RoboticChassisProposal {
            id: id.into(),
            proposed_by: proposed_by.into(),
            design_intent: design_intent.into(),
            urdf_spec_path: None,
            simulation_fidelity: SimulationFidelityScore::new(),
            net_positive_score: NetPositiveScore::new(),
            status: ProposalStatus::PendingSimulation,
        }
    }

    /// Add a simulation run and advance status if thresholds are met.
    pub fn add_simulation_run(&mut self, run: SimulationRun) {
        self.simulation_fidelity.add_run(run);
        self.refresh_status();
    }

    /// Add a job record to the NPFM analysis and advance status if applicable.
    pub fn add_job_record(&mut self, record: JobRecord) {
        self.net_positive_score.anti_busywork.add_job(record);
        self.refresh_status();
    }

    /// Attempt human fiduciary approval.
    ///
    /// Approval succeeds only when:
    ///   1. Simulation fidelity meets `MINIMUM_FIDELITY_FOR_APPROVAL`
    ///   2. The NPFM is net-positive (creates/augments high-agency human jobs)
    pub fn approve(&mut self, approver: impl Into<String>) -> Result<(), String> {
        if !self.simulation_fidelity.ready_for_physical_realisation() {
            return Err(format!(
                "Simulation fidelity too low ({:.3} / {:.3} required, {} run(s) submitted). \
                 Continue metaverse training before physical realisation.",
                self.simulation_fidelity.aggregate(),
                MINIMUM_FIDELITY_FOR_APPROVAL,
                self.simulation_fidelity.runs.len(),
            ));
        }
        if !self.net_positive_score.is_net_positive() {
            return Err(format!(
                "NPFM check failed (score: {:.3}): proposal does not create or augment \
                 high-agency human jobs. Revise the socio-economic impact plan.",
                self.net_positive_score.composite(),
            ));
        }
        self.status = ProposalStatus::Approved {
            approved_by: approver.into(),
        };
        Ok(())
    }

    /// Reject the proposal with a reason.
    pub fn reject(&mut self, reason: impl Into<String>) {
        self.status = ProposalStatus::Rejected {
            reason: reason.into(),
        };
    }

    /// Refresh the status based on current simulation and NPFM scores.
    fn refresh_status(&mut self) {
        // Only advance from PendingSimulation — do not override terminal states.
        if self.status == ProposalStatus::PendingSimulation
            && self.simulation_fidelity.ready_for_physical_realisation()
            && self.net_positive_score.is_net_positive()
        {
            self.status = ProposalStatus::AwaitingFiduciaryApproval;
        }
    }

    /// Human-readable report for the proposal.
    pub fn report(&self) -> String {
        format!(
            "RoboticChassisProposal [{}]\n  Proposed by: {}\n  Intent: {}\n  {}\n  {}\n  Status: {:?}",
            self.id,
            self.proposed_by,
            self.design_intent,
            self.simulation_fidelity.summary(),
            self.net_positive_score.report(),
            self.status,
        )
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn high_fidelity_run() -> SimulationRun {
        SimulationRun::new("Full-range kinematics stress test", 0.95, 10_000, true)
    }

    fn low_fidelity_run() -> SimulationRun {
        SimulationRun::new("Basic unit test", 0.50, 100, false)
    }

    #[test]
    fn human_reviewed_run_carries_full_weight() {
        let run = SimulationRun::new("test", 0.80, 100, true);
        assert_eq!(run.weighted_confidence(), 0.80);
    }

    #[test]
    fn unreviewed_run_is_discounted() {
        let run = SimulationRun::new("test", 0.80, 100, false);
        assert_eq!(run.weighted_confidence(), 0.40);
    }

    #[test]
    fn insufficient_fidelity_blocks_approval() {
        let mut proposal = RoboticChassisProposal::new(
            "CHASSIS-001", "GrokAI", "Bipedal research assistant",
        );
        proposal.add_simulation_run(low_fidelity_run());
        proposal.add_job_record(JobRecord::new_created(
            "Robot maintenance engineer",
            JobTier::PhysicalMetaverseEngineering,
        ));

        let result = proposal.approve("Dave Sheldon");
        assert!(result.is_err(), "Should reject when fidelity is too low");
    }

    #[test]
    fn negative_npfm_blocks_approval() {
        let mut proposal = RoboticChassisProposal::new(
            "CHASSIS-002", "GrokAI", "Busywork automator",
        );
        proposal.add_simulation_run(high_fidelity_run());
        // Add only busywork — NPFM will be negative
        proposal.add_job_record(JobRecord::new_created(
            "Redundant admin task generator",
            JobTier::BusyworkAdministrative,
        ));

        let result = proposal.approve("Dave Sheldon");
        assert!(result.is_err(), "Should reject when NPFM is negative");
    }

    #[test]
    fn high_fidelity_and_positive_npfm_allows_approval() {
        let mut proposal = RoboticChassisProposal::new(
            "CHASSIS-003", "GrokAI", "Precision surgical assistant",
        );
        proposal.add_simulation_run(high_fidelity_run());
        proposal.add_job_record(JobRecord::new_created(
            "Robotic surgery oversight engineer",
            JobTier::PhysicalMetaverseEngineering,
        ));

        let result = proposal.approve("Dave Sheldon");
        assert!(result.is_ok(), "Should approve when fidelity and NPFM are sufficient");
        assert!(
            matches!(proposal.status, ProposalStatus::Approved { .. }),
            "Status should be Approved"
        );
    }

    #[test]
    fn proposal_requires_human_fiduciary_approval() {
        let mut proposal = RoboticChassisProposal::new(
            "CHASSIS-004", "GrokAI", "Autonomous factory worker",
        );
        proposal.add_simulation_run(high_fidelity_run());
        proposal.add_job_record(JobRecord::new_created(
            "Factory oversight commander",
            JobTier::HighAgencyOversight,
        ));

        // Even with sufficient fidelity and positive NPFM, status is
        // AwaitingFiduciaryApproval — not Approved — until a human signs off.
        assert_eq!(
            proposal.status,
            ProposalStatus::AwaitingFiduciaryApproval,
        );
        // Human must explicitly call approve()
        proposal.approve("Dave Sheldon").unwrap();
        assert!(matches!(proposal.status, ProposalStatus::Approved { .. }));
    }

    #[test]
    fn simulation_fidelity_aggregate_is_weighted() {
        let mut score = SimulationFidelityScore::new();
        // One human-reviewed 0.95 run and one unreviewed 0.30 run
        score.add_run(SimulationRun::new("scenario-a", 0.95, 1000, true));
        score.add_run(SimulationRun::new("scenario-b", 0.30, 100, false));
        // weights: 1.0 and 0.5; weighted_confidences: 0.95 and 0.15
        // aggregate = (0.95 + 0.15) / (1.0 + 0.5) = 1.10 / 1.5 ≈ 0.733
        let agg = score.aggregate();
        assert!(
            (agg - 0.733).abs() < 0.001,
            "Unexpected aggregate: {}",
            agg
        );
        assert!(!score.ready_for_physical_realisation());
    }

    #[test]
    fn zero_confidence_clamped_to_zero() {
        let run = SimulationRun::new("catastrophic failure", -1.0, 100, true);
        assert_eq!(run.confidence, 0.0, "Negative confidence must be clamped to 0.0");
    }

    #[test]
    fn over_unity_confidence_clamped_to_one() {
        let run = SimulationRun::new("perfect run", 1.5, 100, true);
        assert_eq!(run.confidence, 1.0, "Confidence above 1.0 must be clamped to 1.0");
    }

    #[test]
    fn exact_minimum_fidelity_threshold_passes() {
        let mut score = SimulationFidelityScore::new();
        score.add_run(SimulationRun::new("threshold run", MINIMUM_FIDELITY_FOR_APPROVAL, 1000, true));
        assert!(score.ready_for_physical_realisation());
    }

    #[test]
    fn empty_fidelity_score_is_zero() {
        let score = SimulationFidelityScore::new();
        assert_eq!(score.aggregate(), 0.0);
        assert!(!score.ready_for_physical_realisation());
    }

    #[test]
    fn error_message_includes_run_count_and_scores() {
        let mut proposal = RoboticChassisProposal::new(
            "CHASSIS-005", "TestAI", "Low-fidelity robot",
        );
        proposal.add_simulation_run(low_fidelity_run());
        let err = proposal.approve("Dave Sheldon").unwrap_err();
        assert!(err.contains("run(s)"), "Error should mention run count");
        assert!(err.contains("0.90"), "Error should mention required threshold");
    }
}
