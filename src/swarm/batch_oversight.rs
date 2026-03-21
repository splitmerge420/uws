// src/swarm/batch_oversight.rs
// Aluminum OS — Swarm Batch Oversight Engine
//
// Before any batch of operations is executed by the Swarm, this module
// performs a dry-run and calculates the projected `NetPositiveScore`.
//
// Governance rule:
//   If the projected score is NEGATIVE the operation is BLOCKED and a
//   Tier-1 human override is required before execution can proceed.
//
// This enforces the principle that AI must augment, not replace, human
// value — and that any displacement must come with a retraining path.
//
// Author: GitHub Copilot (builder)
// Council Session: 2026-03-21
// Invariants Enforced: INV-2 (Consent), INV-5 (Human Flourishing),
//                      INV-35 (Fail-Closed)

#![allow(dead_code)]

use crate::telemetry::kpi::NetPositiveScore;

// ─── Batch Operation ─────────────────────────────────────────────────────────

/// A description of a single operation within a Swarm batch.
#[derive(Debug, Clone)]
pub struct BatchOperation {
    /// Unique identifier for this operation.
    pub id: String,
    /// Human-readable description of what the operation does.
    pub description: String,
    /// Projected NPFM impact if this operation executes.
    pub projected_score: NetPositiveScore,
}

impl BatchOperation {
    pub fn new(
        id: impl Into<String>,
        description: impl Into<String>,
        projected_score: NetPositiveScore,
    ) -> Self {
        Self {
            id: id.into(),
            description: description.into(),
            projected_score,
        }
    }
}

// ─── Override Reason ─────────────────────────────────────────────────────────

/// The reason a Tier-1 human provided when overriding a blocked batch.
#[derive(Debug, Clone, PartialEq)]
pub struct Tier1Override {
    /// Identity of the human authorising the override.
    pub authorised_by: String,
    /// Free-text justification that must accompany every override.
    pub justification: String,
    /// If the blocking was due to job displacement, does a retraining path
    /// exist for the affected workers?
    pub retraining_path_confirmed: bool,
}

impl Tier1Override {
    pub fn new(
        authorised_by: impl Into<String>,
        justification: impl Into<String>,
        retraining_path_confirmed: bool,
    ) -> Self {
        Self {
            authorised_by: authorised_by.into(),
            justification: justification.into(),
            retraining_path_confirmed,
        }
    }
}

// ─── Batch Oversight Result ───────────────────────────────────────────────────

/// Outcome of the pre-execution oversight check on a batch.
#[derive(Debug, Clone, PartialEq)]
pub enum BatchOutcome {
    /// All operations cleared — net score is non-negative.
    Approved,
    /// One or more operations have a negative projected score.
    /// The batch is BLOCKED until a Tier-1 human provides an override.
    Blocked {
        /// IDs of the specific operations that failed the NPFM check.
        failing_operation_ids: Vec<String>,
        /// The combined projected score that triggered the block.
        combined_score: i64,
    },
    /// A Tier-1 human reviewed the blocked batch and authorised execution.
    ApprovedByTier1Override(Tier1Override),
}

// ─── Batch Oversight Engine ───────────────────────────────────────────────────

/// Pre-execution governance gate for Swarm batch operations.
///
/// Usage
/// -----
/// ```
/// let engine = BatchOversightEngine::new();
/// let ops = vec![/* ... */];
/// let outcome = engine.evaluate(&ops);
/// match outcome {
///     BatchOutcome::Approved => { /* proceed */ }
///     BatchOutcome::Blocked { .. } => { /* request Tier-1 override */ }
///     BatchOutcome::ApprovedByTier1Override(_) => { /* proceed */ }
/// }
/// ```
#[derive(Debug, Default)]
pub struct BatchOversightEngine;

impl BatchOversightEngine {
    pub fn new() -> Self {
        Self
    }

    /// Dry-run evaluation: calculates the projected `NetPositiveScore` for the
    /// entire batch and returns the governance outcome.
    ///
    /// The batch is BLOCKED if **any individual operation** has a negative
    /// projected score **or** if the combined score across all operations is
    /// negative.
    pub fn evaluate(&self, operations: &[BatchOperation]) -> BatchOutcome {
        let mut combined: i64 = 0;
        let mut failing: Vec<String> = Vec::new();

        for op in operations {
            let score = op.projected_score.calculate();
            combined += score;
            if score < 0 {
                failing.push(op.id.clone());
            }
        }

        if failing.is_empty() && combined >= 0 {
            BatchOutcome::Approved
        } else {
            BatchOutcome::Blocked {
                failing_operation_ids: failing,
                combined_score: combined,
            }
        }
    }

    /// Apply a Tier-1 human override to a previously blocked batch.
    ///
    /// Returns `ApprovedByTier1Override` when the override is valid.
    /// Returns `Err` when the override is missing a justification or,
    /// when job displacement is involved, no retraining path is confirmed.
    pub fn apply_tier1_override(
        &self,
        blocked_outcome: &BatchOutcome,
        override_request: Tier1Override,
    ) -> Result<BatchOutcome, String> {
        match blocked_outcome {
            BatchOutcome::Blocked { .. } => {
                if override_request.justification.trim().is_empty() {
                    return Err(
                        "Tier-1 override requires a non-empty justification.".to_string()
                    );
                }
                // If the operation may displace jobs, a retraining path must be
                // confirmed before the override is accepted.
                if !override_request.retraining_path_confirmed {
                    return Err(
                        "Tier-1 override requires confirmation of a retraining path \
                         for any workers displaced by this operation."
                            .to_string(),
                    );
                }
                Ok(BatchOutcome::ApprovedByTier1Override(override_request))
            }
            BatchOutcome::Approved | BatchOutcome::ApprovedByTier1Override(_) => {
                Err("Override can only be applied to a Blocked outcome.".to_string())
            }
        }
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telemetry::kpi::NetPositiveScore;

    fn op_positive(id: &str) -> BatchOperation {
        BatchOperation::new(
            id,
            "Creates new oversight jobs",
            NetPositiveScore::new(1, 1, 1, 0),
        )
    }

    fn op_negative(id: &str) -> BatchOperation {
        BatchOperation::new(
            id,
            "High-throughput, no human benefit",
            NetPositiveScore::new(0, 0, 0, 50),
        )
    }

    fn valid_override() -> Tier1Override {
        Tier1Override::new(
            "Dave Sheldon (INV-5)",
            "Displaces legacy data-entry role; retraining into provenance validation confirmed",
            true,
        )
    }

    #[test]
    fn approved_when_all_operations_positive() {
        let engine = BatchOversightEngine::new();
        let ops = vec![op_positive("op-1"), op_positive("op-2")];
        assert_eq!(engine.evaluate(&ops), BatchOutcome::Approved);
    }

    #[test]
    fn blocked_when_any_operation_negative() {
        let engine = BatchOversightEngine::new();
        let ops = vec![op_positive("op-1"), op_negative("op-2")];
        match engine.evaluate(&ops) {
            BatchOutcome::Blocked { failing_operation_ids, .. } => {
                assert!(failing_operation_ids.contains(&"op-2".to_string()));
                assert!(!failing_operation_ids.contains(&"op-1".to_string()));
            }
            other => panic!("Expected Blocked, got {:?}", other),
        }
    }

    #[test]
    fn blocked_when_combined_score_negative() {
        let engine = BatchOversightEngine::new();
        // op-1: +23, op-2: -50 => combined -27
        let ops = vec![op_positive("op-1"), op_negative("op-2")];
        match engine.evaluate(&ops) {
            BatchOutcome::Blocked { combined_score, .. } => {
                assert!(combined_score < 0);
            }
            other => panic!("Expected Blocked, got {:?}", other),
        }
    }

    #[test]
    fn empty_batch_is_approved() {
        let engine = BatchOversightEngine::new();
        assert_eq!(engine.evaluate(&[]), BatchOutcome::Approved);
    }

    #[test]
    fn tier1_override_requires_justification() {
        let engine = BatchOversightEngine::new();
        let blocked = BatchOutcome::Blocked {
            failing_operation_ids: vec!["op-1".to_string()],
            combined_score: -10,
        };
        let bad_override = Tier1Override::new("Dave Sheldon", "", true);
        assert!(engine.apply_tier1_override(&blocked, bad_override).is_err());
    }

    #[test]
    fn tier1_override_requires_retraining_path() {
        let engine = BatchOversightEngine::new();
        let blocked = BatchOutcome::Blocked {
            failing_operation_ids: vec!["op-1".to_string()],
            combined_score: -10,
        };
        let bad_override =
            Tier1Override::new("Dave Sheldon", "Some justification", false);
        let result = engine.apply_tier1_override(&blocked, bad_override);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("retraining path"));
    }

    #[test]
    fn valid_tier1_override_succeeds() {
        let engine = BatchOversightEngine::new();
        let ops = vec![op_negative("op-1")];
        let blocked = engine.evaluate(&ops);
        let result = engine.apply_tier1_override(&blocked, valid_override());
        assert!(result.is_ok());
        match result.unwrap() {
            BatchOutcome::ApprovedByTier1Override(ov) => {
                assert_eq!(ov.authorised_by, "Dave Sheldon (INV-5)");
            }
            other => panic!("Expected ApprovedByTier1Override, got {:?}", other),
        }
    }

    #[test]
    fn override_fails_on_already_approved_batch() {
        let engine = BatchOversightEngine::new();
        let approved = BatchOutcome::Approved;
        let result = engine.apply_tier1_override(&approved, valid_override());
        assert!(result.is_err());
    }
}
