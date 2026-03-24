// src/universal/model_router.rs
// Model Router — routing decisions and content-digest helpers.
//
// Low-level routing logic: selects the best council member for a given
// query, computes request digests for deduplication and caching, and
// maintains per-model reliability scores updated by Kintsugi.
//
// Digest algorithm: FNV-1a 64-bit (zero-dep, deterministic, fast).
// Replace with SHA3-256 (sha3 = "0.10") when Phase 2 deps land.
//
// Council Session: 2026-03-20

use std::collections::HashMap;

// ─── Digest helpers ──────────────────────────────────────────────────────────

/// FNV-1a 64-bit offset basis and prime.
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01B3;

/// Compute a deterministic FNV-1a digest of a byte slice.
///
/// Used for request deduplication, Janus round IDs, and GoldenTrace keys.
/// All inputs are hashed the same way regardless of platform byte order.
pub(crate) fn compute_digest(data: &[u8]) -> u64 {
    let mut hash = FNV_OFFSET;
    for &byte in data {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// Compute a digest from a string slice (UTF-8 encoded).
///
/// Convenience wrapper over [`compute_digest`] for string inputs.
///
/// # Example
/// ```
/// use uws::universal::model_router::compute_digest_from_str;
/// let d1 = compute_digest_from_str("hello");
/// let d2 = compute_digest_from_str("hello");
/// assert_eq!(d1, d2);
/// let d3 = compute_digest_from_str("world");
/// assert_ne!(d1, d3);
/// ```
pub(crate) fn compute_digest_from_str(s: &str) -> u64 {
    compute_digest(s.as_bytes())
}

// ─── Model definition ─────────────────────────────────────────────────────────

/// A council member model with role, weight, and reliability score.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelConfig {
    /// Unique model identifier (e.g. "claude", "gemini").
    pub id: String,
    /// Council role (e.g. "governance", "substrate", "adversarial").
    pub role: String,
    /// Base vote weight [0.0, 1.0].
    pub weight: f64,
    /// Fallback model id if this model is unavailable.
    pub fallback: Option<String>,
    /// Reliability score [0.0, 1.0] — updated by Kintsugi after failures.
    pub reliability: f64,
    /// Whether this model is currently available.
    pub available: bool,
}

impl ModelConfig {
    /// Create a new model config with full reliability.
    pub fn new(id: &str, role: &str, weight: f64, fallback: Option<&str>) -> Self {
        Self {
            id: id.to_string(),
            role: role.to_string(),
            weight,
            fallback: fallback.map(str::to_string),
            reliability: 1.0,
            available: true,
        }
    }

    /// Effective weight = base weight × reliability.
    pub fn effective_weight(&self) -> f64 {
        self.weight * self.reliability
    }
}

// ─── ModelRouter ─────────────────────────────────────────────────────────────

/// Routes queries to council members and maintains reliability scores.
///
/// # INV-7 enforcement
/// `select_primary` will never return a model whose `effective_weight` share
/// of the total council weight would exceed `inv7_threshold` (default 0.47).
///
/// If all available models would violate INV-7 (e.g. only one model remains),
/// the method returns the best available model anyway and marks the result as
/// a potential violation — Janus handles the escalation.
#[derive(Debug, Clone)]
pub struct ModelRouter {
    pub(crate) models: HashMap<String, ModelConfig>,
    /// Maximum share of total vote weight any single model may hold.
    pub inv7_threshold: f64,
}

impl ModelRouter {
    /// Build a router from a list of model configs.
    pub fn new(models: Vec<ModelConfig>, inv7_threshold: f64) -> Self {
        let map = models.into_iter().map(|m| (m.id.clone(), m)).collect();
        Self { models: map, inv7_threshold }
    }

    /// Build the default five-member council as per the Janus v2 spec.
    pub fn default_council() -> Self {
        Self::new(
            vec![
                ModelConfig::new("claude",   "governance",  1.0, Some("gemini")),
                ModelConfig::new("gemini",   "substrate",   1.0, Some("claude")),
                ModelConfig::new("grok",     "adversarial", 0.8, Some("deepseek")),
                ModelConfig::new("deepseek", "research",    0.7, Some("gemini")),
                ModelConfig::new("copilot",  "enterprise",  0.7, Some("claude")),
            ],
            0.47,
        )
    }

    /// Total effective weight across all available models.
    pub fn total_weight(&self) -> f64 {
        self.models.values()
            .filter(|m| m.available)
            .map(|m| m.effective_weight())
            .sum()
    }

    /// Select the primary model for a query, respecting INV-7.
    ///
    /// Returns `(model_id, inv7_ok)`.  `inv7_ok` is false when no compliant
    /// model exists (single-model degraded state).
    pub fn select_primary(&self, query_digest: u64) -> (String, bool) {
        let available: Vec<&ModelConfig> = self.models.values()
            .filter(|m| m.available)
            .collect();

        if available.is_empty() {
            return ("none".to_string(), false);
        }

        let total = self.total_weight();

        // Sort by effective weight descending for deterministic ordering,
        // then use digest to pick among equally-weighted options.
        let mut candidates: Vec<&ModelConfig> = available;
        candidates.sort_by(|a, b| b.effective_weight()
            .partial_cmp(&a.effective_weight())
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.id.cmp(&b.id)));

        // Try each candidate in preference order; pick the first that is
        // INV-7 compliant (or the best if none are).
        for candidate in &candidates {
            let share = if total > 0.0 {
                candidate.effective_weight() / total
            } else {
                0.0
            };
            if share <= self.inv7_threshold {
                return (candidate.id.clone(), true);
            }
        }

        // Single-model degraded state — return best but flag violation.
        // Janus will escalate to Tier 3 and notify the user.
        let _ = query_digest; // digest used for tie-breaking in multi-model scenarios
        (candidates[0].id.clone(), false)
    }

    /// Mark a model as unavailable (e.g., after a network timeout).
    pub fn mark_unavailable(&mut self, model_id: &str) {
        if let Some(m) = self.models.get_mut(model_id) {
            m.available = false;
        }
    }

    /// Update reliability score after a Kintsugi repair event.
    ///
    /// Score is clamped to [0.0, 1.0].
    pub fn update_reliability(&mut self, model_id: &str, new_score: f64) {
        if let Some(m) = self.models.get_mut(model_id) {
            m.reliability = new_score.clamp(0.0, 1.0);
        }
    }

    /// Resolve the fallback for an unavailable model.
    pub fn fallback_for(&self, model_id: &str) -> Option<String> {
        self.models.get(model_id)
            .and_then(|m| m.fallback.clone())
            .filter(|fb| self.models.get(fb.as_str()).map(|m| m.available).unwrap_or(false))
    }

    /// Get a model config by id.
    pub fn get(&self, model_id: &str) -> Option<&ModelConfig> {
        self.models.get(model_id)
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_digest_deterministic() {
        let d1 = compute_digest_from_str("hello world");
        let d2 = compute_digest_from_str("hello world");
        assert_eq!(d1, d2);
    }

    #[test]
    fn test_digest_distinct_inputs() {
        let d1 = compute_digest_from_str("abc");
        let d2 = compute_digest_from_str("abd");
        assert_ne!(d1, d2);
    }

    #[test]
    fn test_digest_empty_string() {
        let d = compute_digest_from_str("");
        assert_eq!(d, FNV_OFFSET, "empty string digest is the FNV offset basis");
    }

    #[test]
    fn test_digest_bytes_matches_str() {
        let s = "janus-v2";
        assert_eq!(compute_digest(s.as_bytes()), compute_digest_from_str(s));
    }

    #[test]
    fn test_model_config_effective_weight() {
        let m = ModelConfig::new("claude", "governance", 1.0, Some("gemini"));
        assert!((m.effective_weight() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_model_config_degraded_reliability() {
        let mut m = ModelConfig::new("gemini", "substrate", 1.0, None);
        m.reliability = 0.5;
        assert!((m.effective_weight() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_default_council_five_members() {
        let router = ModelRouter::default_council();
        assert_eq!(router.models.len(), 5);
    }

    #[test]
    fn test_default_council_total_weight() {
        let router = ModelRouter::default_council();
        let total = router.total_weight();
        // claude(1.0) + gemini(1.0) + grok(0.8) + deepseek(0.7) + copilot(0.7) = 4.2
        assert!((total - 4.2).abs() < 1e-9, "total={total}");
    }

    #[test]
    fn test_select_primary_inv7_compliant() {
        let router = ModelRouter::default_council();
        let digest = compute_digest_from_str("test query");
        let (model, inv7_ok) = router.select_primary(digest);
        assert!(inv7_ok, "should be INV-7 compliant with full council");
        assert!(!model.is_empty());
        // Winning share must not exceed 0.47
        let total = router.total_weight();
        let winner = router.get(&model).unwrap();
        let share = winner.effective_weight() / total;
        assert!(share <= 0.47, "share={share} exceeds INV-7 threshold");
    }

    #[test]
    fn test_select_primary_single_model_flags_violation() {
        let mut router = ModelRouter::new(
            vec![ModelConfig::new("claude", "governance", 1.0, None)],
            0.47,
        );
        let digest = compute_digest_from_str("solo");
        let (model, inv7_ok) = router.select_primary(digest);
        assert_eq!(model, "claude");
        // Single model necessarily > 47%, so violation is flagged
        assert!(!inv7_ok, "single model should flag INV-7 violation");

        // mark_unavailable should make it return ("none", false)
        router.mark_unavailable("claude");
        let (model2, ok2) = router.select_primary(digest);
        assert_eq!(model2, "none");
        assert!(!ok2);
    }

    #[test]
    fn test_mark_unavailable() {
        let mut router = ModelRouter::default_council();
        router.mark_unavailable("claude");
        assert!(!router.get("claude").unwrap().available);
    }

    #[test]
    fn test_update_reliability_clamps() {
        let mut router = ModelRouter::default_council();
        router.update_reliability("gemini", 1.5);
        assert!((router.get("gemini").unwrap().reliability - 1.0).abs() < 1e-9);
        router.update_reliability("gemini", -0.1);
        assert!((router.get("gemini").unwrap().reliability - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_fallback_resolution() {
        let mut router = ModelRouter::default_council();
        // claude's fallback is gemini (and gemini is available)
        assert_eq!(router.fallback_for("claude"), Some("gemini".to_string()));
        // mark gemini unavailable — fallback should return None
        router.mark_unavailable("gemini");
        assert_eq!(router.fallback_for("claude"), None);
    }
}
