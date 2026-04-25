// src/embodiment/spatial.rs
// Aluminum OS — Spatial Embodiment Gate
//
// `SpatialManifest` describes a spatial/XR deployment (USD scene, OpenXR
// session, WebGPU canvas, or a custom engine).  Approval requires a
// strictly positive Net-Positive Flourishing Metric score.

#![allow(dead_code)]

use crate::telemetry::kpi::NetPositiveScore;

/// Rendering engine / runtime for the spatial deployment.
#[derive(Debug, Clone, PartialEq)]
pub enum RenderingEngine {
    /// Universal Scene Description (Pixar / NVIDIA Omniverse)
    Usd,
    /// OpenXR — cross-platform VR/AR standard
    OpenXr,
    /// WebGPU — GPU-accelerated browser rendering
    WebGpu,
    /// Project-specific or proprietary engine
    Custom(String),
}

/// Bounding box in world-space metres (axis-aligned).
#[derive(Debug, Clone)]
pub struct BoundingBox {
    pub min_x: f64,
    pub min_y: f64,
    pub min_z: f64,
    pub max_x: f64,
    pub max_y: f64,
    pub max_z: f64,
}

/// Describes a spatial / XR scene for constitutional review.
///
/// `approve()` evaluates the associated NPFM and returns `Err` if the
/// score is ≤ 0 (execution must be blocked).
#[derive(Debug, Clone)]
pub struct SpatialManifest {
    /// Human-readable name for this spatial deployment.
    pub name: String,
    /// The rendering runtime.
    pub rendering_engine: RenderingEngine,
    /// Geometry description (e.g. USDZ path, GLTF path, scene graph reference).
    pub geometry: String,
    /// URI pointing to the primary asset (local path or https URL).
    pub asset_uri: String,
    /// World-space bounding box of the scene.
    pub bounding_box: BoundingBox,
}

/// Outcome of a spatial approval gate.
#[derive(Debug, Clone, PartialEq)]
pub enum SpatialApprovalStatus {
    Approved,
    /// Blocked because the NPFM score was ≤ 0.
    BlockedNpfmNonPositive { score: ordered_float::NotNan<f64> },
}

// NotNan is heavyweight; use a plain wrapper instead to keep zero deps.
mod ordered_float {
    /// Newtype wrapper for f64 that implements PartialEq / Eq for use in
    /// enum variants without pulling in the `ordered-float` crate.
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct NotNan<T>(pub T);
}

impl SpatialManifest {
    /// Evaluate the NPFM gate.
    ///
    /// Returns `Ok(SpatialApprovalStatus::Approved)` when the NPFM is strictly
    /// positive, or an `Err` describing the blocking reason.
    pub fn approve(&self, npfm: &NetPositiveScore) -> Result<SpatialApprovalStatus, String> {
        if !npfm.is_positive() {
            let score = npfm.composite();
            return Err(format!(
                "SpatialManifest '{}' blocked: NPFM score {:.4} is not strictly positive. \
                 Eliminate busywork and route displaced humans into high-agency roles before \
                 deploying spatial experiences.",
                self.name, score
            ));
        }
        Ok(SpatialApprovalStatus::Approved)
    }
}

// ─── Unit tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telemetry::kpi::{AntiBusyworkFactor, JobEntry, JobTier, NetPositiveScore};

    fn manifest() -> SpatialManifest {
        SpatialManifest {
            name: "test-scene".into(),
            rendering_engine: RenderingEngine::OpenXr,
            geometry: "scene.usdz".into(),
            asset_uri: "https://assets.example.com/scene.usdz".into(),
            bounding_box: BoundingBox {
                min_x: -1.0,
                min_y: 0.0,
                min_z: -1.0,
                max_x: 1.0,
                max_y: 2.0,
                max_z: 1.0,
            },
        }
    }

    fn positive_npfm() -> NetPositiveScore {
        let mut abf = AntiBusyworkFactor::new();
        abf.add(JobEntry {
            label: "creative-work".into(),
            tier: JobTier::CreativeGenesis,
            fte_count: 1.0,
            displaced_to_high_agency: false,
        });
        NetPositiveScore::new(abf, 0.8, 0.9)
    }

    fn non_positive_npfm() -> NetPositiveScore {
        // Empty ABF → score = 0, not positive
        let abf = AntiBusyworkFactor::new();
        NetPositiveScore::new(abf, 0.0, 0.0)
    }

    #[test]
    fn spatial_manifest_approved_when_npfm_positive() {
        let result = manifest().approve(&positive_npfm());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), SpatialApprovalStatus::Approved);
    }

    #[test]
    fn spatial_manifest_blocked_when_npfm_non_positive() {
        let result = manifest().approve(&non_positive_npfm());
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(msg.contains("blocked"));
        assert!(msg.contains("NPFM score"));
    }

    #[test]
    fn usd_rendering_engine_variant_works() {
        let mut m = manifest();
        m.rendering_engine = RenderingEngine::Usd;
        let result = m.approve(&positive_npfm());
        assert!(result.is_ok());
    }

    #[test]
    fn webgpu_rendering_engine_variant_works() {
        let mut m = manifest();
        m.rendering_engine = RenderingEngine::WebGpu;
        let result = m.approve(&positive_npfm());
        assert!(result.is_ok());
    }

    #[test]
    fn custom_rendering_engine_variant_works() {
        let mut m = manifest();
        m.rendering_engine = RenderingEngine::Custom("UnrealEngine5".into());
        let result = m.approve(&positive_npfm());
        assert!(result.is_ok());
    }
}
