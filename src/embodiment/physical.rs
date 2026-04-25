// src/embodiment/physical.rs
// Aluminum OS — Physical Embodiment Gate
//
// `RoboticChassisProposal` describes a robotic or physical-world deployment.
// Approval requires:
//   1. SimulationFidelityScore ≥ 0.90
//   2. A strictly positive NPFM
//
// Status transitions:
//   PendingSimulation → AwaitingFiduciaryApproval → Approved
//
// Simulation runs reviewed by a human carry full weight.
// Unreviewed (automated-only) runs are discounted 50%.

#![allow(dead_code)]

use crate::telemetry::kpi::NetPositiveScore;

/// Lifecycle status of a robotic chassis proposal.
#[derive(Debug, Clone, PartialEq)]
pub enum ChassisStatus {
    /// Waiting for simulation results to be submitted.
    PendingSimulation,
    /// Simulation passed the fidelity gate; now awaiting human fiduciary sign-off.
    AwaitingFiduciaryApproval,
    /// Fully approved — simulation fidelity + NPFM gate both passed.
    Approved,
}

/// A single simulation run contributing to the fidelity score.
#[derive(Debug, Clone)]
pub struct SimulationRun {
    /// Identifier for this run (e.g. "sim-run-003").
    pub run_id: String,
    /// Raw fidelity score produced by the simulator, in [0.0, 1.0].
    pub raw_score: f64,
    /// When `true`, a human engineer has reviewed and endorsed this run.
    /// Unreviewed runs are discounted 50% per the Embodiment Protocol.
    pub human_reviewed: bool,
}

impl SimulationRun {
    /// The effective fidelity contribution of this run.
    ///
    /// Human-reviewed: `raw_score × 1.0`
    /// Unreviewed:     `raw_score × 0.5`
    pub fn effective_score(&self) -> f64 {
        if self.human_reviewed {
            self.raw_score.clamp(0.0, 1.0)
        } else {
            (self.raw_score * 0.5).clamp(0.0, 1.0)
        }
    }
}

/// Minimum effective fidelity score required for approval.
pub const MIN_FIDELITY_SCORE: f64 = 0.90;

/// Physical environment the chassis will operate in.
#[derive(Debug, Clone, PartialEq)]
pub enum PhysicalEnvironment {
    IndoorStructured,
    IndoorUnstructured,
    OutdoorUrban,
    OutdoorRural,
    Underwater,
    Aerospace,
    Custom(String),
}

/// Proposal to deploy a robotic chassis in the physical world.
///
/// Call [`RoboticChassisProposal::advance`] to transition through status stages,
/// and [`RoboticChassisProposal::approve`] to attempt final approval.
#[derive(Debug, Clone)]
pub struct RoboticChassisProposal {
    /// Human-readable name for this proposal.
    pub name: String,
    /// General form factor (e.g. "bipedal", "quadruped", "wheeled", "drone").
    pub form_factor: String,
    /// List of required sensor types (e.g. ["lidar", "stereo-camera", "imu"]).
    pub sensor_requirements: Vec<String>,
    /// URI pointing to the URDF or CAD file for this chassis.
    pub urdf_or_cad_uri: String,
    /// Mass budget in kilograms.
    pub mass_budget_kg: f64,
    /// Target operating environment.
    pub environment: PhysicalEnvironment,
    /// Simulation runs submitted for this proposal.
    pub simulation_runs: Vec<SimulationRun>,
    /// Current lifecycle status.
    pub status: ChassisStatus,
}

impl RoboticChassisProposal {
    /// Compute the aggregated simulation fidelity score.
    ///
    /// If no runs have been submitted the score is 0.0.
    /// Otherwise the score is the mean of all effective run scores.
    pub fn simulation_fidelity_score(&self) -> f64 {
        if self.simulation_runs.is_empty() {
            return 0.0;
        }
        let total: f64 = self.simulation_runs.iter().map(|r| r.effective_score()).sum();
        total / self.simulation_runs.len() as f64
    }

    /// Attempt to advance the proposal to the next status stage.
    ///
    /// * `PendingSimulation` → `AwaitingFiduciaryApproval` requires fidelity ≥ 0.90.
    /// * `AwaitingFiduciaryApproval` → `Approved` requires a positive NPFM.
    ///
    /// Returns `Err` with a human-readable message if the gate is not met.
    pub fn advance(&mut self, npfm: &NetPositiveScore) -> Result<ChassisStatus, String> {
        match &self.status {
            ChassisStatus::PendingSimulation => {
                let fidelity = self.simulation_fidelity_score();
                if fidelity < MIN_FIDELITY_SCORE {
                    return Err(format!(
                        "Chassis '{}': simulation fidelity {:.4} is below required {:.2}. \
                         Submit additional human-reviewed simulation runs.",
                        self.name, fidelity, MIN_FIDELITY_SCORE
                    ));
                }
                self.status = ChassisStatus::AwaitingFiduciaryApproval;
                Ok(self.status.clone())
            }
            ChassisStatus::AwaitingFiduciaryApproval => {
                if !npfm.is_positive() {
                    return Err(format!(
                        "Chassis '{}': NPFM score {:.4} is not strictly positive. \
                         Correct the job-tier composition before final fiduciary sign-off.",
                        self.name,
                        npfm.composite()
                    ));
                }
                self.status = ChassisStatus::Approved;
                Ok(self.status.clone())
            }
            ChassisStatus::Approved => {
                Err(format!("Chassis '{}' is already approved.", self.name))
            }
        }
    }

    /// Convenience method: attempt full approval in one call.
    ///
    /// Equivalent to calling [`advance`] twice in sequence.
    /// Returns `Err` at the first blocking gate encountered.
    pub fn approve(&mut self, npfm: &NetPositiveScore) -> Result<(), String> {
        if self.status == ChassisStatus::PendingSimulation {
            self.advance(npfm)?;
        }
        if self.status == ChassisStatus::AwaitingFiduciaryApproval {
            self.advance(npfm)?;
        }
        Ok(())
    }
}

// ─── Unit tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telemetry::kpi::{AntiBusyworkFactor, JobEntry, JobTier, NetPositiveScore};

    fn proposal_with_runs(runs: Vec<SimulationRun>) -> RoboticChassisProposal {
        RoboticChassisProposal {
            name: "test-chassis".into(),
            form_factor: "bipedal".into(),
            sensor_requirements: vec!["lidar".into(), "imu".into()],
            urdf_or_cad_uri: "https://cad.example.com/chassis.urdf".into(),
            mass_budget_kg: 80.0,
            environment: PhysicalEnvironment::IndoorStructured,
            simulation_runs: runs,
            status: ChassisStatus::PendingSimulation,
        }
    }

    fn positive_npfm() -> NetPositiveScore {
        let mut abf = AntiBusyworkFactor::new();
        abf.add(JobEntry {
            label: "robotics-engineering".into(),
            tier: JobTier::PhysicalMetaverseEngineering,
            fte_count: 1.0,
            displaced_to_high_agency: false,
        });
        NetPositiveScore::new(abf, 0.8, 0.7)
    }

    fn non_positive_npfm() -> NetPositiveScore {
        let abf = AntiBusyworkFactor::new();
        NetPositiveScore::new(abf, 0.0, 0.0)
    }

    fn high_fidelity_run(human_reviewed: bool) -> SimulationRun {
        SimulationRun {
            run_id: "run-001".into(),
            raw_score: 0.95,
            human_reviewed,
        }
    }

    // ── SimulationRun effective score ─────────────────────────────────────────

    #[test]
    fn human_reviewed_run_full_weight() {
        let run = high_fidelity_run(true);
        assert!((run.effective_score() - 0.95).abs() < f64::EPSILON);
    }

    #[test]
    fn unreviewed_run_fifty_percent_discount() {
        let run = high_fidelity_run(false);
        assert!((run.effective_score() - 0.475).abs() < f64::EPSILON);
    }

    // ── Fidelity gate (PendingSimulation → AwaitingFiduciaryApproval) ─────────

    #[test]
    fn fidelity_gate_passes_with_human_reviewed_high_score() {
        let mut proposal = proposal_with_runs(vec![high_fidelity_run(true)]);
        let result = proposal.advance(&positive_npfm());
        assert!(result.is_ok());
        assert_eq!(proposal.status, ChassisStatus::AwaitingFiduciaryApproval);
    }

    #[test]
    fn fidelity_gate_blocks_when_unreviewed_run_drops_below_threshold() {
        // raw_score = 0.95; effective = 0.475 < 0.90
        let mut proposal = proposal_with_runs(vec![high_fidelity_run(false)]);
        let result = proposal.advance(&positive_npfm());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("simulation fidelity"));
        assert_eq!(proposal.status, ChassisStatus::PendingSimulation);
    }

    #[test]
    fn fidelity_gate_blocks_when_no_runs_submitted() {
        let mut proposal = proposal_with_runs(vec![]);
        let result = proposal.advance(&positive_npfm());
        assert!(result.is_err());
    }

    // ── NPFM gate (AwaitingFiduciaryApproval → Approved) ──────────────────────

    #[test]
    fn npfm_gate_passes_with_positive_score() {
        let mut proposal = proposal_with_runs(vec![high_fidelity_run(true)]);
        // Move past fidelity gate first
        proposal.advance(&positive_npfm()).unwrap();
        assert_eq!(proposal.status, ChassisStatus::AwaitingFiduciaryApproval);
        // Now exercise NPFM gate
        let result = proposal.advance(&positive_npfm());
        assert!(result.is_ok());
        assert_eq!(proposal.status, ChassisStatus::Approved);
    }

    #[test]
    fn npfm_gate_blocks_with_non_positive_score() {
        let mut proposal = proposal_with_runs(vec![high_fidelity_run(true)]);
        proposal.advance(&positive_npfm()).unwrap();
        let result = proposal.advance(&non_positive_npfm());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("NPFM score"));
        assert_eq!(proposal.status, ChassisStatus::AwaitingFiduciaryApproval);
    }

    // ── Full approve() convenience ────────────────────────────────────────────

    #[test]
    fn full_approve_succeeds_with_good_fidelity_and_positive_npfm() {
        let mut proposal = proposal_with_runs(vec![high_fidelity_run(true)]);
        assert!(proposal.approve(&positive_npfm()).is_ok());
        assert_eq!(proposal.status, ChassisStatus::Approved);
    }

    #[test]
    fn full_approve_fails_at_fidelity_gate_for_unreviewed_runs() {
        let mut proposal = proposal_with_runs(vec![high_fidelity_run(false)]);
        assert!(proposal.approve(&positive_npfm()).is_err());
        assert_eq!(proposal.status, ChassisStatus::PendingSimulation);
    }

    #[test]
    fn full_approve_fails_at_npfm_gate() {
        let mut proposal = proposal_with_runs(vec![high_fidelity_run(true)]);
        assert!(proposal.approve(&non_positive_npfm()).is_err());
        // Proposal advanced through fidelity gate but stuck at fiduciary stage
        assert_eq!(proposal.status, ChassisStatus::AwaitingFiduciaryApproval);
    }

    #[test]
    fn advance_on_already_approved_returns_err() {
        let mut proposal = proposal_with_runs(vec![high_fidelity_run(true)]);
        proposal.approve(&positive_npfm()).unwrap();
        let result = proposal.advance(&positive_npfm());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already approved"));
    }
}
