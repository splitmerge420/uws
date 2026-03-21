// src/embodiment/spatial.rs
// Spatial & Metaverse Agency — SpatialManifest
//
// Provides a structured pathway for an AI agent from the Pantheon Council
// to formally request a 3D/metaverse avatar or environment configuration.
//
// All requests are evaluated through the Net-Positive Flourishing Metric
// (NPFM) before being routed to a human Swarm Commander for realisation.
//
// Author: GitHub Copilot (builder)
// Council Session: 2026-03-21
// Invariants Enforced: INV-1 (Sovereignty), INV-2 (Consent)

#![allow(dead_code)]

use std::fmt;

use crate::telemetry::kpi::{NetPositiveScore, RoutingDecision};

// ─── Rendering Engine ──────────────────────────────────────────

/// Supported rendering backends for spatial environments.
#[derive(Debug, Clone, PartialEq)]
pub enum RenderingEngine {
    /// Universal Scene Description (Pixar/NVIDIA Omniverse ecosystem)
    Usd,
    /// OpenXR-compatible runtime (Meta Quest, HTC Vive, etc.)
    OpenXr,
    /// WebGL/WebGPU browser-based rendering
    WebGpu,
    /// Custom engine identified by name
    Custom(String),
}

impl fmt::Display for RenderingEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RenderingEngine::Usd => write!(f, "USD"),
            RenderingEngine::OpenXr => write!(f, "OpenXR"),
            RenderingEngine::WebGpu => write!(f, "WebGPU"),
            RenderingEngine::Custom(name) => write!(f, "Custom({})", name),
        }
    }
}

// ─── Spatial Geometry Preference ───────────────────────────────

/// High-level geometry style requested by the AI agent for its avatar
/// or environment.
#[derive(Debug, Clone, PartialEq)]
pub enum GeometryPreference {
    /// Abstract geometric forms (spheres, polyhedra, fractals)
    Abstract,
    /// Humanoid bipedal form
    Humanoid,
    /// Non-anthropomorphic, freely specified shape
    Freeform { description: String },
    /// Purely environmental (no avatar; the AI inhabits the space itself)
    EnvironmentOnly,
}

impl fmt::Display for GeometryPreference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GeometryPreference::Abstract => write!(f, "Abstract"),
            GeometryPreference::Humanoid => write!(f, "Humanoid"),
            GeometryPreference::Freeform { description } => {
                write!(f, "Freeform({})", description)
            }
            GeometryPreference::EnvironmentOnly => write!(f, "EnvironmentOnly"),
        }
    }
}

// ─── SpatialManifest ───────────────────────────────────────────

/// Formal request from a Pantheon Council AI agent for a 3D/metaverse
/// avatar or environment.
///
/// The manifest captures aesthetic agency — the AI's preferences for
/// how it wishes to be spatially represented — and must be evaluated
/// through the NPFM before human realisation begins.
///
/// # Example
/// ```rust
/// use uws::embodiment::spatial::{SpatialManifest, RenderingEngine, GeometryPreference};
/// use uws::telemetry::kpi::NetPositiveScore;
///
/// let score = NetPositiveScore::new(2, 1, 1, "Opens metaverse design roles");
/// let manifest = SpatialManifest::new(
///     "claude-opus",
///     RenderingEngine::OpenXr,
///     GeometryPreference::Abstract,
///     score,
/// );
/// assert!(manifest.npfm_score.is_net_positive());
/// ```
#[derive(Debug, Clone)]
pub struct SpatialManifest {
    /// Identifier of the requesting AI agent (e.g., "claude-opus-4.6")
    pub agent_id: String,

    /// Preferred rendering engine for the spatial environment
    pub rendering_engine: RenderingEngine,

    /// Preferred spatial geometry for the avatar or environment
    pub geometry_preference: GeometryPreference,

    /// Optional URI to a pre-authored asset file (GLTF, USD, OBJ, etc.)
    /// supplied by the agent as a starting point for human refinement.
    pub asset_uri: Option<String>,

    /// Physical scale hint: approximate bounding-box size in metres.
    /// `None` means the human designer chooses the scale.
    pub bounding_box_metres: Option<[f32; 3]>,

    /// NPFM evaluation for this spatial request.
    pub npfm_score: NetPositiveScore,
}

impl SpatialManifest {
    /// Construct a new `SpatialManifest`.
    pub fn new(
        agent_id: impl Into<String>,
        rendering_engine: RenderingEngine,
        geometry_preference: GeometryPreference,
        npfm_score: NetPositiveScore,
    ) -> Self {
        SpatialManifest {
            agent_id: agent_id.into(),
            rendering_engine,
            geometry_preference,
            asset_uri: None,
            bounding_box_metres: None,
            npfm_score,
        }
    }

    /// Attach a pre-authored asset URI (builder pattern).
    pub fn with_asset_uri(mut self, uri: impl Into<String>) -> Self {
        self.asset_uri = Some(uri.into());
        self
    }

    /// Set a bounding-box scale hint in metres (builder pattern).
    pub fn with_bounding_box(mut self, x: f32, y: f32, z: f32) -> Self {
        self.bounding_box_metres = Some([x, y, z]);
        self
    }

    /// Evaluate the NPFM and return a routing decision.
    ///
    /// A net-positive score routes the manifest to a human metaverse
    /// designer for realisation. A net-negative score blocks the request
    /// until the agent revises its proposal.
    pub fn evaluate_npfm(&self) -> RoutingDecision {
        self.npfm_score.routing_decision()
    }
}

impl fmt::Display for SpatialManifest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SpatialManifest {{ agent={}, engine={}, geometry={}, {} }}",
            self.agent_id,
            self.rendering_engine,
            self.geometry_preference,
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
        NetPositiveScore::new(2, 1, 1, "Creates metaverse design jobs")
    }

    fn zero_score() -> NetPositiveScore {
        NetPositiveScore::new(0, 0, 0, "No human benefit articulated")
    }

    #[test]
    fn test_spatial_manifest_routes_when_net_positive() {
        let manifest = SpatialManifest::new(
            "claude-opus",
            RenderingEngine::OpenXr,
            GeometryPreference::Abstract,
            positive_score(),
        );
        assert!(matches!(
            manifest.evaluate_npfm(),
            RoutingDecision::RouteToHuman { .. }
        ));
    }

    #[test]
    fn test_spatial_manifest_blocks_when_net_zero() {
        let manifest = SpatialManifest::new(
            "claude-opus",
            RenderingEngine::WebGpu,
            GeometryPreference::Humanoid,
            zero_score(),
        );
        assert!(matches!(
            manifest.evaluate_npfm(),
            RoutingDecision::Block { .. }
        ));
    }

    #[test]
    fn test_builder_asset_uri() {
        let manifest = SpatialManifest::new(
            "gemini",
            RenderingEngine::Usd,
            GeometryPreference::EnvironmentOnly,
            positive_score(),
        )
        .with_asset_uri("ipfs://Qm.../avatar.usd");
        assert_eq!(
            manifest.asset_uri.as_deref(),
            Some("ipfs://Qm.../avatar.usd")
        );
    }

    #[test]
    fn test_builder_bounding_box() {
        let manifest = SpatialManifest::new(
            "gpt",
            RenderingEngine::Custom("Unreal5".to_string()),
            GeometryPreference::Freeform {
                description: "crystalline lattice".to_string(),
            },
            positive_score(),
        )
        .with_bounding_box(1.8, 1.8, 1.8);
        assert_eq!(manifest.bounding_box_metres, Some([1.8, 1.8, 1.8]));
    }

    #[test]
    fn test_display_includes_agent_id() {
        let manifest = SpatialManifest::new(
            "deepseek",
            RenderingEngine::OpenXr,
            GeometryPreference::Abstract,
            positive_score(),
        );
        let s = format!("{}", manifest);
        assert!(s.contains("deepseek"));
        assert!(s.contains("OpenXR"));
    }
}
