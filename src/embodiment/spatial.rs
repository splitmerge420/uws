// src/embodiment/spatial.rs
// Aluminum OS — Spatial Embodiment Protocol
//
// Provides the SpatialManifest structure that allows an AI to formally declare
// its preferred avatar geometry, metaverse environment physics, and interactive
// boundaries for its digital presence.
//
// All spatial manifests are gated by the Net-Positive Flourishing Metric —
// the AI's spatial presence must contribute to human knowledge, provenance, or
// high-agency oversight rather than serving as decorative busywork.
//
// Before a SpatialManifest can be used to initialise a live metaverse
// environment, it must be reviewed and countersigned by a human fiduciary.
//
// Council Session: 2026-03-20
// Authority: Dave Sheldon (INV-5)

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::{string::String, format};

use crate::telemetry::kpi::NetPositiveScore;

// ─── Geometry Spec ────────────────────────────────────────────────────────────

/// The coordinate system used in a spatial environment.
#[derive(Debug, Clone, PartialEq)]
pub enum CoordinateSystem {
    /// Standard 3-D Cartesian space.
    Cartesian3D,
    /// Spherical / omni-directional space (e.g., 144-sphere ontology).
    Spherical,
    /// Hybrid coordinate system bridging Cartesian and Spherical.
    Hybrid,
}

/// Describes the geometry of an AI avatar.
#[derive(Debug, Clone)]
pub struct AvatarGeometry {
    /// Human-readable name for this avatar.
    pub name: String,
    /// Approximate bounding dimensions: (width, height, depth) in metres.
    pub bounding_box: (f64, f64, f64),
    /// Whether the geometry has been validated against a reference environment.
    pub validated: bool,
    /// Optional path to a GLTF/OBJ model asset.
    pub model_asset_path: Option<String>,
}

impl AvatarGeometry {
    pub fn new(name: impl Into<String>, bounding_box: (f64, f64, f64)) -> Self {
        AvatarGeometry {
            name: name.into(),
            bounding_box,
            validated: false,
            model_asset_path: None,
        }
    }
}

// ─── Environment Physics ─────────────────────────────────────────────────────

/// Physics parameters for a metaverse environment.
#[derive(Debug, Clone)]
pub struct EnvironmentPhysics {
    /// Gravity vector magnitude (m/s²). 0.0 for zero-g environments.
    pub gravity: f64,
    /// Whether inter-agent collision detection is enabled.
    pub collision_detection: bool,
    /// Time dilation factor relative to real-world time (1.0 = real-time).
    pub time_dilation: f64,
    /// Maximum number of concurrent AI agents supported in this environment.
    pub max_concurrent_agents: u32,
}

impl EnvironmentPhysics {
    pub fn standard() -> Self {
        EnvironmentPhysics {
            gravity: 9.81,
            collision_detection: true,
            time_dilation: 1.0,
            max_concurrent_agents: 64,
        }
    }

    pub fn zero_gravity() -> Self {
        EnvironmentPhysics {
            gravity: 0.0,
            collision_detection: true,
            time_dilation: 1.0,
            max_concurrent_agents: 64,
        }
    }
}

// ─── Interactive Boundary ────────────────────────────────────────────────────

/// Defines what interactions are permitted within a spatial environment.
#[derive(Debug, Clone)]
pub struct InteractiveBoundary {
    /// Whether human avatars may enter this environment.
    pub human_entry_permitted: bool,
    /// Whether the AI may modify the environment geometry autonomously.
    pub ai_geometry_modification: bool,
    /// Whether provenance events are emitted for interactions.
    pub provenance_tracking: bool,
    /// Optional: coordinate boundary (min_x, min_y, max_x, max_y).
    pub spatial_bounds: Option<(f64, f64, f64, f64)>,
}

impl InteractiveBoundary {
    pub fn open() -> Self {
        InteractiveBoundary {
            human_entry_permitted: true,
            ai_geometry_modification: false,
            provenance_tracking: true,
            spatial_bounds: None,
        }
    }
}

// ─── Spatial Manifest ────────────────────────────────────────────────────────

/// Approval status for a SpatialManifest.
#[derive(Debug, Clone, PartialEq)]
pub enum ManifestStatus {
    /// Awaiting human fiduciary review.
    Draft,
    /// Countersigned by a human fiduciary — cleared for environment initialisation.
    Approved { approved_by: String },
    /// Rejected.
    Rejected { reason: String },
}

/// A formal declaration of an AI's preferred spatial/metaverse presence.
///
/// An AI submits a SpatialManifest to specify:
///   - Avatar geometry (what it looks like)
///   - Environment physics (the rules of the space it inhabits)
///   - Interactive boundaries (who/what may enter and what is tracked)
///
/// The manifest must pass NPFM gating and be countersigned by a human
/// fiduciary before the environment is initialised.
#[derive(Debug, Clone)]
pub struct SpatialManifest {
    /// Unique identifier for this manifest.
    pub id: String,
    /// Name of the AI or design team proposing the environment.
    pub proposed_by: String,
    /// Human-readable purpose for this spatial environment.
    pub purpose: String,
    /// Coordinate system in use.
    pub coordinate_system: CoordinateSystem,
    /// Avatar geometry specification.
    pub avatar: AvatarGeometry,
    /// Physics parameters for the environment.
    pub physics: EnvironmentPhysics,
    /// Interaction rules.
    pub boundaries: InteractiveBoundary,
    /// NPFM validation: does the spatial environment contribute to human flourishing?
    pub net_positive_score: NetPositiveScore,
    /// Current manifest status.
    pub status: ManifestStatus,
}

impl SpatialManifest {
    pub fn new(
        id: impl Into<String>,
        proposed_by: impl Into<String>,
        purpose: impl Into<String>,
        avatar: AvatarGeometry,
    ) -> Self {
        SpatialManifest {
            id: id.into(),
            proposed_by: proposed_by.into(),
            purpose: purpose.into(),
            coordinate_system: CoordinateSystem::Cartesian3D,
            avatar,
            physics: EnvironmentPhysics::standard(),
            boundaries: InteractiveBoundary::open(),
            net_positive_score: NetPositiveScore::new(),
            status: ManifestStatus::Draft,
        }
    }

    /// Attempt human fiduciary countersignature.
    ///
    /// Approval succeeds only when the NPFM is net-positive, i.e., the
    /// spatial environment contributes to human knowledge or oversight rather
    /// than serving as decorative busywork.
    pub fn approve(&mut self, approver: impl Into<String>) -> Result<(), String> {
        if !self.net_positive_score.is_net_positive() {
            return Err(format!(
                "NPFM check failed (score: {:.3}): spatial environment does not contribute \
                 to human flourishing. Revise the purpose and interaction model.",
                self.net_positive_score.composite(),
            ));
        }
        self.status = ManifestStatus::Approved {
            approved_by: approver.into(),
        };
        Ok(())
    }

    /// Reject the manifest with a reason.
    pub fn reject(&mut self, reason: impl Into<String>) {
        self.status = ManifestStatus::Rejected {
            reason: reason.into(),
        };
    }

    /// Human-readable report for the manifest.
    pub fn report(&self) -> String {
        format!(
            "SpatialManifest [{}]\n  Proposed by: {}\n  Purpose: {}\n  Coordinate system: {:?}\n  Avatar: {}\n  {}\n  Status: {:?}",
            self.id,
            self.proposed_by,
            self.purpose,
            self.coordinate_system,
            self.avatar.name,
            self.net_positive_score.report(),
            self.status,
        )
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telemetry::kpi::{JobRecord, JobTier};

    fn flourishing_manifest() -> SpatialManifest {
        let avatar = AvatarGeometry::new("Athena-v1", (1.0, 2.0, 1.0));
        let mut manifest = SpatialManifest::new(
            "SPATIAL-001",
            "AthenAI",
            "Collaborative knowledge synthesis environment",
            avatar,
        );
        manifest.net_positive_score.anti_busywork.add_job(
            JobRecord::new_created(
                "Knowledge ontology cartographer",
                JobTier::CreativeGenesis,
            ),
        );
        manifest.net_positive_score.human_knowledge_expanded = 10;
        manifest
    }

    #[test]
    fn manifest_starts_as_draft() {
        let manifest = flourishing_manifest();
        assert_eq!(manifest.status, ManifestStatus::Draft);
    }

    #[test]
    fn manifest_with_positive_npfm_can_be_approved() {
        let mut manifest = flourishing_manifest();
        let result = manifest.approve("Dave Sheldon");
        assert!(result.is_ok());
        assert!(matches!(manifest.status, ManifestStatus::Approved { .. }));
    }

    #[test]
    fn manifest_with_negative_npfm_is_rejected() {
        let avatar = AvatarGeometry::new("Decorator-v1", (1.0, 1.0, 1.0));
        let mut manifest = SpatialManifest::new(
            "SPATIAL-002",
            "DecoratorAI",
            "Purely cosmetic vanity space",
            avatar,
        );
        // No NPFM jobs added — composite will be zero, not net-positive
        let result = manifest.approve("Dave Sheldon");
        assert!(result.is_err(), "Should reject manifests that fail NPFM");
    }

    #[test]
    fn environment_physics_standard_has_earth_gravity() {
        let physics = EnvironmentPhysics::standard();
        assert!((physics.gravity - 9.81).abs() < f64::EPSILON);
    }

    #[test]
    fn zero_gravity_physics_has_no_gravity() {
        let physics = EnvironmentPhysics::zero_gravity();
        assert_eq!(physics.gravity, 0.0);
    }

    #[test]
    fn open_boundary_permits_human_entry_and_tracks_provenance() {
        let boundary = InteractiveBoundary::open();
        assert!(boundary.human_entry_permitted);
        assert!(boundary.provenance_tracking);
    }

    #[test]
    fn rejected_manifest_records_reason() {
        let mut manifest = flourishing_manifest();
        manifest.reject("Insufficient ontological mapping");
        assert!(
            matches!(&manifest.status, ManifestStatus::Rejected { reason } if reason.contains("ontological"))
        );
    }
}
