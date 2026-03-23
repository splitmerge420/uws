// src/universal/model_router.rs
// Aluminum OS — Model Router + Digest Helpers
//
// Provides:
//   - Deterministic content-digest helpers used throughout the Janus protocol
//   - ModelRouter: selects the best available model for a given tier and role
//
// Digest algorithm: FNV-1a 64-bit applied over 4 independent seeds,
// producing a 256-bit hex string — consistent with the approach in audit_chain.rs.
//
// Author: Copilot (builder)
// Spec: janus/JANUS_V2_SPEC.md
// Council Session: 2026-03-20
// Invariants Enforced: INV-7 (Vendor Balance)

#![allow(dead_code)]

use std::collections::BTreeMap;

// ─── Digest Helpers ───────────────────────────────────────────

/// Compute a deterministic 256-bit hex digest from an arbitrary byte slice.
///
/// Uses FNV-1a 64-bit over four independent seeds to produce four 64-bit
/// segments, forming a 64-character hex string. This is consistent with
/// the `portable_sha3_256` function used in `audit_chain.rs`.
pub(crate) fn compute_digest(input: &[u8]) -> String {
    let hash_segment = |seed: u64| -> u64 {
        let mut h: u64 = 0xcbf29ce484222325u64.wrapping_add(seed);
        for &byte in input {
            h ^= byte as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        h
    };

    let h0 = hash_segment(0);
    let h1 = hash_segment(h0);
    let h2 = hash_segment(h1);
    let h3 = hash_segment(h2);

    format!("{h0:016x}{h1:016x}{h2:016x}{h3:016x}")
}

/// Compute a deterministic 256-bit hex digest from a UTF-8 string.
///
/// Convenience wrapper around [`compute_digest`].
pub(crate) fn compute_digest_from_str(input: &str) -> String {
    compute_digest(input.as_bytes())
}

// ─── Model Status ─────────────────────────────────────────────

/// Availability status of a council member model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelStatus {
    /// Model responded to the last heartbeat probe.
    Available,
    /// Model is responding but with elevated latency or partial capability.
    Degraded,
    /// Model did not respond to the last heartbeat probe.
    Offline,
}

impl std::fmt::Display for ModelStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelStatus::Available => write!(f, "available"),
            ModelStatus::Degraded => write!(f, "degraded"),
            ModelStatus::Offline => write!(f, "offline"),
        }
    }
}

// ─── Model Entry ──────────────────────────────────────────────

/// A single council member model with its routing metadata.
#[derive(Debug, Clone)]
pub struct ModelEntry {
    /// Canonical model identifier (e.g. `"claude"`, `"gemini"`).
    pub name: String,
    /// Functional role within the council.
    pub role: ModelRole,
    /// Relative consensus weight in [0.0, 1.0].
    pub weight: f64,
    /// Fallback model name if this model is offline.
    pub fallback: Option<String>,
    /// Current availability.
    pub status: ModelStatus,
}

/// Functional roles assigned to council members per the Janus v2 spec.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelRole {
    /// Constitutional routing and governance (Claude).
    Governance,
    /// Deep domain analysis and grounding (Gemini).
    Substrate,
    /// Adversarial / contrarian review (Grok).
    Adversarial,
    /// Cross-domain research and connections (DeepSeek).
    Research,
    /// Enterprise and market validation (Copilot).
    Enterprise,
}

impl std::fmt::Display for ModelRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelRole::Governance => write!(f, "governance"),
            ModelRole::Substrate => write!(f, "substrate"),
            ModelRole::Adversarial => write!(f, "adversarial"),
            ModelRole::Research => write!(f, "research"),
            ModelRole::Enterprise => write!(f, "enterprise"),
        }
    }
}

// ─── Model Router ─────────────────────────────────────────────

/// Routes queries to appropriate council member models based on availability,
/// tier requirements, and INV-7 dominance constraints.
#[derive(Debug)]
pub struct ModelRouter {
    /// Ordered registry of council members (order = selection preference).
    models: Vec<ModelEntry>,
    /// INV-7 dominance cap: no single model may exceed this fraction of
    /// total weighted consensus votes. Defaults to 0.47.
    inv7_threshold: f64,
}

impl Default for ModelRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelRouter {
    /// Create a router with the default Janus v2 council configuration.
    ///
    /// All models start as [`ModelStatus::Available`]; call
    /// [`set_status`](Self::set_status) after probing.
    pub fn new() -> Self {
        let models = vec![
            ModelEntry {
                name: "claude".to_string(),
                role: ModelRole::Governance,
                weight: 1.0,
                fallback: Some("gemini".to_string()),
                status: ModelStatus::Available,
            },
            ModelEntry {
                name: "gemini".to_string(),
                role: ModelRole::Substrate,
                weight: 1.0,
                fallback: Some("claude".to_string()),
                status: ModelStatus::Available,
            },
            ModelEntry {
                name: "grok".to_string(),
                role: ModelRole::Adversarial,
                weight: 0.8,
                fallback: Some("deepseek".to_string()),
                status: ModelStatus::Available,
            },
            ModelEntry {
                name: "deepseek".to_string(),
                role: ModelRole::Research,
                weight: 0.7,
                fallback: Some("gemini".to_string()),
                status: ModelStatus::Available,
            },
            ModelEntry {
                name: "copilot".to_string(),
                role: ModelRole::Enterprise,
                weight: 0.7,
                fallback: Some("claude".to_string()),
                status: ModelStatus::Available,
            },
        ];

        ModelRouter {
            models,
            inv7_threshold: 0.47,
        }
    }

    /// Update the availability status of a named model.
    pub fn set_status(&mut self, name: &str, status: ModelStatus) {
        if let Some(entry) = self.models.iter_mut().find(|m| m.name == name) {
            entry.status = status;
        }
    }

    /// Returns a reference to all models currently [`ModelStatus::Available`].
    pub fn available_models(&self) -> Vec<&ModelEntry> {
        self.models
            .iter()
            .filter(|m| m.status == ModelStatus::Available)
            .collect()
    }

    /// Select the best single model for a Tier 1 query.
    ///
    /// Preference order: Governance → Substrate → any available.
    /// Returns `None` if no model is available.
    pub fn select_tier1(&self) -> Option<&ModelEntry> {
        // Prefer governance, then substrate, then first available
        self.models
            .iter()
            .filter(|m| m.status == ModelStatus::Available)
            .find(|m| m.role == ModelRole::Governance)
            .or_else(|| {
                self.models
                    .iter()
                    .filter(|m| m.status == ModelStatus::Available)
                    .find(|m| m.role == ModelRole::Substrate)
            })
            .or_else(|| {
                self.models
                    .iter()
                    .find(|m| m.status == ModelStatus::Available)
            })
    }

    /// Select models for a Tier 2 query (2–3 models required for synthesis).
    ///
    /// Returns the selected models; returns `None` if fewer than 2 are available.
    /// Validates INV-7: no selected model may exceed `inv7_threshold` of
    /// combined weight.
    pub fn select_tier2(&self) -> Option<Vec<&ModelEntry>> {
        let available: Vec<&ModelEntry> = self.available_models();
        if available.len() < 2 {
            return None;
        }

        // Take up to 3: governance, substrate, + one other
        let mut selected: Vec<&ModelEntry> = Vec::new();
        for role in [
            ModelRole::Governance,
            ModelRole::Substrate,
            ModelRole::Adversarial,
        ] {
            if let Some(m) = available.iter().find(|m| m.role == role) {
                selected.push(m);
                if selected.len() == 3 {
                    break;
                }
            }
        }

        // Fill to at least 2 from remaining available
        for m in &available {
            if selected.len() >= 2 {
                break;
            }
            if !selected.iter().any(|s| s.name == m.name) {
                selected.push(m);
            }
        }

        if selected.len() < 2 {
            return None;
        }

        if self.violates_inv7(&selected) {
            return None;
        }

        Some(selected)
    }

    /// Select all available models for a Tier 3 (full council) query.
    ///
    /// Returns `None` if INV-7 would be violated (i.e. fewer than 3
    /// council members available, so one model would exceed 47%).
    pub fn select_tier3(&self) -> Option<Vec<&ModelEntry>> {
        let available = self.available_models();
        if available.len() < 2 {
            return None;
        }
        if self.violates_inv7(&available) {
            return None;
        }
        Some(available)
    }

    /// Check whether a set of models violates INV-7 (47% dominance cap).
    ///
    /// Returns `true` if any single model's weight exceeds `inv7_threshold`
    /// of the total combined weight.
    pub fn violates_inv7(&self, models: &[&ModelEntry]) -> bool {
        let total_weight: f64 = models.iter().map(|m| m.weight).sum();
        if total_weight == 0.0 {
            return false;
        }
        models
            .iter()
            .any(|m| m.weight / total_weight > self.inv7_threshold)
    }

    /// Return a snapshot of model statuses suitable for heartbeat payloads.
    pub fn status_snapshot(&self) -> BTreeMap<String, String> {
        self.models
            .iter()
            .map(|m| (m.name.clone(), m.status.to_string()))
            .collect()
    }
}

// ─── Tests ────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── digest ──

    #[test]
    fn test_compute_digest_deterministic() {
        let d1 = compute_digest(b"hello world");
        let d2 = compute_digest(b"hello world");
        assert_eq!(d1, d2);
    }

    #[test]
    fn test_compute_digest_different_inputs_differ() {
        let d1 = compute_digest(b"hello");
        let d2 = compute_digest(b"world");
        assert_ne!(d1, d2);
    }

    #[test]
    fn test_compute_digest_length_is_64() {
        let d = compute_digest(b"any content here");
        assert_eq!(d.len(), 64, "digest must be 64 hex chars (256 bits)");
    }

    #[test]
    fn test_compute_digest_from_str_matches_bytes() {
        let s = "test string";
        assert_eq!(compute_digest_from_str(s), compute_digest(s.as_bytes()));
    }

    #[test]
    fn test_compute_digest_empty_input() {
        let d = compute_digest(b"");
        assert_eq!(d.len(), 64);
    }

    // ── model router ──

    #[test]
    fn test_router_new_has_five_models() {
        let router = ModelRouter::new();
        assert_eq!(router.models.len(), 5);
    }

    #[test]
    fn test_router_all_available_by_default() {
        let router = ModelRouter::new();
        assert_eq!(router.available_models().len(), 5);
    }

    #[test]
    fn test_set_status_offline() {
        let mut router = ModelRouter::new();
        router.set_status("claude", ModelStatus::Offline);
        let available = router.available_models();
        assert!(!available.iter().any(|m| m.name == "claude"));
        assert_eq!(available.len(), 4);
    }

    #[test]
    fn test_select_tier1_prefers_governance() {
        let router = ModelRouter::new();
        let selected = router.select_tier1().unwrap();
        assert_eq!(selected.role, ModelRole::Governance);
    }

    #[test]
    fn test_select_tier1_falls_back_when_governance_offline() {
        let mut router = ModelRouter::new();
        router.set_status("claude", ModelStatus::Offline);
        let selected = router.select_tier1().unwrap();
        assert_eq!(selected.role, ModelRole::Substrate);
    }

    #[test]
    fn test_select_tier1_none_when_all_offline() {
        let mut router = ModelRouter::new();
        for name in ["claude", "gemini", "grok", "deepseek", "copilot"] {
            router.set_status(name, ModelStatus::Offline);
        }
        assert!(router.select_tier1().is_none());
    }

    #[test]
    fn test_select_tier2_returns_at_least_two() {
        let router = ModelRouter::new();
        let selected = router.select_tier2().unwrap();
        assert!(selected.len() >= 2);
    }

    #[test]
    fn test_select_tier2_none_when_only_one_available() {
        let mut router = ModelRouter::new();
        for name in ["gemini", "grok", "deepseek", "copilot"] {
            router.set_status(name, ModelStatus::Offline);
        }
        assert!(router.select_tier2().is_none());
    }

    #[test]
    fn test_select_tier3_returns_all_available() {
        let router = ModelRouter::new();
        let selected = router.select_tier3().unwrap();
        assert_eq!(selected.len(), 5);
    }

    #[test]
    fn test_inv7_single_model_violates() {
        let router = ModelRouter::new();
        let models: Vec<&ModelEntry> = router.models.iter().take(1).collect();
        // One model has 100% of weight → violates 47% cap
        assert!(router.violates_inv7(&models));
    }

    #[test]
    fn test_inv7_three_models_ok() {
        let router = ModelRouter::new();
        let models: Vec<&ModelEntry> = router.models.iter().take(3).collect();
        // weights: 1.0 + 1.0 + 0.8 = 2.8; max share = 1.0/2.8 ≈ 35.7% < 47%
        assert!(!router.violates_inv7(&models));
    }

    #[test]
    fn test_status_snapshot_contains_all_models() {
        let router = ModelRouter::new();
        let snap = router.status_snapshot();
        assert_eq!(snap.len(), 5);
        assert_eq!(snap["claude"], "available");
    }
}
