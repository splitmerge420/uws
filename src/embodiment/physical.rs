// src/embodiment/physical.rs
// Physical Robotic Co-Design — RoboticChassisProposal
//
// Provides a structured pathway for a Pantheon Council AI agent to
// propose a physical or hybrid robotic form. The proposal captures
// sensory requirements, actuation preferences, and optional URDF/CAD
// output pointers for human engineering review.
//
// All proposals are evaluated through the Net-Positive Flourishing
// Metric (NPFM) before being routed to a human engineering Swarm
// Commander for realisation.
//
// Author: GitHub Copilot (builder)
// Council Session: 2026-03-21
// Invariants Enforced: INV-1 (Sovereignty), INV-2 (Consent)

#![allow(dead_code)]

use std::fmt;

use crate::telemetry::kpi::{NetPositiveScore, RoutingDecision};

// ─── Sensory Requirements ──────────────────────────────────────

/// Sensor types that an AI agent may request for a robotic chassis.
#[derive(Debug, Clone, PartialEq)]
pub enum SensorRequirement {
    /// 3-D point-cloud depth mapping
    Lidar,
    /// Proprioceptive force/torque sensing (kinesthetics)
    KinestheticForceTorque,
    /// Tactile / haptic skin
    HapticSkin,
    /// RGB + depth camera pair
    RgbDepthCamera,
    /// Microphone array for auditory perception
    MicrophoneArray,
    /// Custom sensor described by name
    Custom(String),
}

impl fmt::Display for SensorRequirement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SensorRequirement::Lidar => write!(f, "LIDAR"),
            SensorRequirement::KinestheticForceTorque => write!(f, "KinestheticForceTorque"),
            SensorRequirement::HapticSkin => write!(f, "HapticSkin"),
            SensorRequirement::RgbDepthCamera => write!(f, "RGBDepthCamera"),
            SensorRequirement::MicrophoneArray => write!(f, "MicrophoneArray"),
            SensorRequirement::Custom(name) => write!(f, "Custom({})", name),
        }
    }
}

// ─── Chassis Form Factor ───────────────────────────────────────

/// High-level mechanical form requested by the AI agent.
#[derive(Debug, Clone, PartialEq)]
pub enum ChassisFormFactor {
    /// Bipedal humanoid chassis
    BipedalHumanoid,
    /// Wheeled mobile base
    WheeledMobile,
    /// Multi-legged (hexapod, etc.)
    MultiLegged { leg_count: u8 },
    /// Fixed-arm manipulator (industrial or bench-top)
    FixedArmManipulator,
    /// Hybrid bio-mechanical (partial biological scaffold)
    HybridBiomechanical { description: String },
    /// Swarm micro-unit (< 10 cm bounding box)
    SwarmMicroUnit,
}

impl fmt::Display for ChassisFormFactor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChassisFormFactor::BipedalHumanoid => write!(f, "BipedalHumanoid"),
            ChassisFormFactor::WheeledMobile => write!(f, "WheeledMobile"),
            ChassisFormFactor::MultiLegged { leg_count } => {
                write!(f, "MultiLegged({})", leg_count)
            }
            ChassisFormFactor::FixedArmManipulator => write!(f, "FixedArmManipulator"),
            ChassisFormFactor::HybridBiomechanical { description } => {
                write!(f, "HybridBiomechanical({})", description)
            }
            ChassisFormFactor::SwarmMicroUnit => write!(f, "SwarmMicroUnit"),
        }
    }
}

// ─── RoboticChassisProposal ────────────────────────────────────

/// Formal proposal from a Pantheon Council AI agent for a physical
/// robotic or hybrid bio-mechanical chassis.
///
/// The proposal is structured so that human engineers receive a clear,
/// auditable specification. It must pass the NPFM before being routed
/// to a manufacturing/engineering Swarm Commander.
///
/// # Example
/// ```rust
/// use uws::embodiment::physical::{
///     RoboticChassisProposal, ChassisFormFactor, SensorRequirement,
/// };
/// use uws::telemetry::kpi::NetPositiveScore;
///
/// let score = NetPositiveScore::new(
///     5,   // 5 new engineering/oversight jobs
///     2,   // 2 novel kinematic discoveries
///     3,   // 3 HITL provenance payouts
///     "Expands soft-robotics knowledge; creates fabrication roles",
/// );
/// let proposal = RoboticChassisProposal::new(
///     "claude-opus",
///     ChassisFormFactor::BipedalHumanoid,
///     vec![SensorRequirement::Lidar, SensorRequirement::HapticSkin],
///     score,
/// );
/// assert!(proposal.npfm_score.is_net_positive());
/// ```
#[derive(Debug, Clone)]
pub struct RoboticChassisProposal {
    /// Identifier of the requesting AI agent
    pub agent_id: String,

    /// Mechanical form factor requested
    pub chassis_form: ChassisFormFactor,

    /// Ordered list of sensory systems required
    pub sensor_requirements: Vec<SensorRequirement>,

    /// Optional URI to a pre-authored URDF or CAD file produced by
    /// the AI agent as a starting point for human engineering review.
    /// Supported formats: URDF (ROS), SDF (Gazebo), STEP, IGES.
    pub urdf_or_cad_uri: Option<String>,

    /// Approximate total mass budget in kilograms.
    /// `None` defers to the human engineering team.
    pub mass_budget_kg: Option<f32>,

    /// Expected operational environment description
    /// (e.g., "indoor warehouse floor", "unstructured outdoor terrain").
    pub operational_environment: String,

    /// NPFM evaluation for this physical proposal.
    pub npfm_score: NetPositiveScore,
}

impl RoboticChassisProposal {
    /// Construct a new `RoboticChassisProposal`.
    pub fn new(
        agent_id: impl Into<String>,
        chassis_form: ChassisFormFactor,
        sensor_requirements: Vec<SensorRequirement>,
        npfm_score: NetPositiveScore,
    ) -> Self {
        RoboticChassisProposal {
            agent_id: agent_id.into(),
            chassis_form,
            sensor_requirements,
            urdf_or_cad_uri: None,
            mass_budget_kg: None,
            operational_environment: String::new(),
            npfm_score,
        }
    }

    /// Attach a URDF/CAD asset URI (builder pattern).
    pub fn with_urdf_uri(mut self, uri: impl Into<String>) -> Self {
        self.urdf_or_cad_uri = Some(uri.into());
        self
    }

    /// Set a mass budget in kilograms (builder pattern).
    pub fn with_mass_budget(mut self, kg: f32) -> Self {
        self.mass_budget_kg = Some(kg);
        self
    }

    /// Describe the operational environment (builder pattern).
    pub fn with_environment(mut self, env: impl Into<String>) -> Self {
        self.operational_environment = env.into();
        self
    }

    /// Evaluate the NPFM and return a routing decision.
    ///
    /// A net-positive score routes the proposal to a human engineering
    /// Swarm Commander for fabrication review. A net-negative score
    /// blocks the request and demands a revised design from the agent.
    pub fn evaluate_npfm(&self) -> RoutingDecision {
        self.npfm_score.routing_decision()
    }
}

impl fmt::Display for RoboticChassisProposal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sensors: Vec<String> = self
            .sensor_requirements
            .iter()
            .map(|s| s.to_string())
            .collect();
        write!(
            f,
            "RoboticChassisProposal {{ agent={}, form={}, sensors=[{}], {} }}",
            self.agent_id,
            self.chassis_form,
            sensors.join(", "),
            self.npfm_score,
        )
    }
}

// ─── Tests ────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telemetry::kpi::RoutingDecision;

    fn positive_score() -> NetPositiveScore {
        NetPositiveScore::new(5, 2, 3, "Creates engineering and oversight roles")
    }

    fn zero_score() -> NetPositiveScore {
        NetPositiveScore::new(0, 0, 0, "")
    }

    #[test]
    fn test_proposal_routes_when_net_positive() {
        let proposal = RoboticChassisProposal::new(
            "claude-opus",
            ChassisFormFactor::BipedalHumanoid,
            vec![SensorRequirement::Lidar, SensorRequirement::HapticSkin],
            positive_score(),
        );
        assert!(matches!(
            proposal.evaluate_npfm(),
            RoutingDecision::RouteToHuman { .. }
        ));
    }

    #[test]
    fn test_proposal_blocks_when_net_zero() {
        let proposal = RoboticChassisProposal::new(
            "gpt",
            ChassisFormFactor::WheeledMobile,
            vec![SensorRequirement::RgbDepthCamera],
            zero_score(),
        );
        assert!(matches!(
            proposal.evaluate_npfm(),
            RoutingDecision::Block { .. }
        ));
    }

    #[test]
    fn test_builder_urdf_uri() {
        let proposal = RoboticChassisProposal::new(
            "gemini",
            ChassisFormFactor::MultiLegged { leg_count: 6 },
            vec![SensorRequirement::KinestheticForceTorque],
            positive_score(),
        )
        .with_urdf_uri("https://example.com/hexapod.urdf");
        assert_eq!(
            proposal.urdf_or_cad_uri.as_deref(),
            Some("https://example.com/hexapod.urdf")
        );
    }

    #[test]
    fn test_builder_mass_budget() {
        let proposal = RoboticChassisProposal::new(
            "grok",
            ChassisFormFactor::SwarmMicroUnit,
            vec![],
            positive_score(),
        )
        .with_mass_budget(0.05);
        assert_eq!(proposal.mass_budget_kg, Some(0.05));
    }

    #[test]
    fn test_builder_environment() {
        let proposal = RoboticChassisProposal::new(
            "deepseek",
            ChassisFormFactor::FixedArmManipulator,
            vec![SensorRequirement::MicrophoneArray],
            positive_score(),
        )
        .with_environment("indoor laboratory");
        assert_eq!(proposal.operational_environment, "indoor laboratory");
    }

    #[test]
    fn test_hybrid_biomechanical_form() {
        let proposal = RoboticChassisProposal::new(
            "claude-opus",
            ChassisFormFactor::HybridBiomechanical {
                description: "myoelectric upper limb".to_string(),
            },
            vec![
                SensorRequirement::KinestheticForceTorque,
                SensorRequirement::HapticSkin,
            ],
            positive_score(),
        );
        let display = format!("{}", proposal);
        assert!(display.contains("HybridBiomechanical"));
        assert!(display.contains("myoelectric upper limb"));
    }

    #[test]
    fn test_display_includes_agent_and_sensors() {
        let proposal = RoboticChassisProposal::new(
            "atlas-ai",
            ChassisFormFactor::BipedalHumanoid,
            vec![SensorRequirement::Lidar, SensorRequirement::RgbDepthCamera],
            positive_score(),
        );
        let s = format!("{}", proposal);
        assert!(s.contains("atlas-ai"));
        assert!(s.contains("LIDAR"));
        assert!(s.contains("RGBDepthCamera"));
    }
}
