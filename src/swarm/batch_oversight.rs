// src/swarm/batch_oversight.rs
// Swarm Commander — Batch Human Oversight for AI Operations
//
// Implements the "Swarm Commander" concept: a single human overseer can
// review, approve, and cryptographically sign off on a batch of AI
// operations or drone actions in a single session.
//
// This is the primary retraining pathway for workers displaced by
// automation:  instead of replacing humans, the system promotes them
// into Swarm Commander roles where they govern AI behaviour at scale.
//
// Command surface (planned):
//   uws swarm review  --batch=<N>
//   uws swarm approve --batch-id=<ID> [--note="..."]
//   uws swarm reject  --batch-id=<ID> --reason="..."
//   uws swarm status  --batch-id=<ID>
//
// Council Session: 2026-03-21
// Authority: Dave Sheldon (INV-5)
// Invariants Enforced: INV-1 (Sovereignty), INV-2 (Consent), INV-3 (Audit)

#![allow(dead_code)]

use std::collections::BTreeMap;
use std::fmt;

// ─── Batch Operation ─────────────────────────────────────────

/// A single AI operation or drone action awaiting human approval.
#[derive(Debug, Clone)]
pub struct BatchOperation {
    /// Unique identifier for this operation
    pub operation_id: String,
    /// Human-readable summary of what the AI / drone intends to do
    pub description: String,
    /// The AI or drone agent that produced this operation
    pub agent_source: String,
    /// Any parameters or payload associated with the operation
    pub parameters: BTreeMap<String, String>,
    /// Current approval state
    pub status: OperationStatus,
    /// Risk level estimated by the system
    pub risk_level: RiskLevel,
}

/// Lifecycle state of a batched operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperationStatus {
    /// Awaiting human review
    Pending,
    /// Human commander approved — safe to execute
    Approved,
    /// Human commander rejected — must NOT execute
    Rejected,
    /// Execution has completed (post-approval)
    Executed,
}

impl fmt::Display for OperationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OperationStatus::Pending   => write!(f, "PENDING"),
            OperationStatus::Approved  => write!(f, "APPROVED"),
            OperationStatus::Rejected  => write!(f, "REJECTED"),
            OperationStatus::Executed  => write!(f, "EXECUTED"),
        }
    }
}

/// Risk classification for a batched operation.
///
/// Higher-risk operations may require additional sign-offs or a slower
/// review cadence.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
    /// Read-only, no side effects
    Low,
    /// Writes data but reversible
    Medium,
    /// Irreversible or high-impact
    High,
    /// Requires constitutional authority (INV-5)
    Critical,
}

impl fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RiskLevel::Low      => write!(f, "LOW"),
            RiskLevel::Medium   => write!(f, "MEDIUM"),
            RiskLevel::High     => write!(f, "HIGH"),
            RiskLevel::Critical => write!(f, "CRITICAL"),
        }
    }
}

impl BatchOperation {
    /// Create a new pending operation.
    pub fn new(
        operation_id: impl Into<String>,
        description: impl Into<String>,
        agent_source: impl Into<String>,
        risk_level: RiskLevel,
    ) -> Self {
        BatchOperation {
            operation_id: operation_id.into(),
            description: description.into(),
            agent_source: agent_source.into(),
            parameters: BTreeMap::new(),
            status: OperationStatus::Pending,
            risk_level,
        }
    }

    /// Add a key-value parameter to the operation.
    pub fn with_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.parameters.insert(key.into(), value.into());
        self
    }
}

// ─── Batch ────────────────────────────────────────────────────

/// A collection of operations presented to a single Swarm Commander
/// for review in one sitting.
///
/// The batch enforces a maximum size so that the commander is never
/// overwhelmed — cognitive load is a first-class concern.
#[derive(Debug, Clone)]
pub struct OperationBatch {
    pub batch_id: String,
    pub operations: Vec<BatchOperation>,
    /// Maximum operations per batch (default: 10)
    pub max_size: usize,
    /// Commander who owns this review session
    pub commander_id: Option<String>,
    /// Final sign-off recorded after all operations are reviewed
    pub batch_signoff: Option<BatchSignoff>,
}

/// Cryptographic sign-off on an entire batch by the Swarm Commander.
#[derive(Debug, Clone)]
pub struct BatchSignoff {
    pub commander_id: String,
    /// Outcome for the whole batch
    pub decision: BatchDecision,
    /// Optional note from the commander
    pub note: Option<String>,
    /// Unix timestamp of the sign-off
    pub timestamp_unix: u64,
    /// Hex-encoded signature over (batch_id + decision + timestamp)
    pub signature: String,
}

/// The commander's overall decision for a batch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BatchDecision {
    /// All approved operations may proceed
    ApproveAll,
    /// Mixed: some approved, some rejected (see individual operations)
    PartialApprove,
    /// Entire batch rejected — nothing may execute
    RejectAll,
}

impl fmt::Display for BatchDecision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BatchDecision::ApproveAll    => write!(f, "APPROVE_ALL"),
            BatchDecision::PartialApprove => write!(f, "PARTIAL_APPROVE"),
            BatchDecision::RejectAll     => write!(f, "REJECT_ALL"),
        }
    }
}

impl OperationBatch {
    /// Create a new empty batch with the specified size limit.
    pub fn new(batch_id: impl Into<String>, max_size: usize) -> Self {
        OperationBatch {
            batch_id: batch_id.into(),
            operations: Vec::new(),
            max_size,
            commander_id: None,
            batch_signoff: None,
        }
    }

    /// Add an operation to the batch.
    ///
    /// Returns `Err` if the batch is already full.
    pub fn add_operation(
        &mut self,
        op: BatchOperation,
    ) -> Result<(), BatchOversightError> {
        if self.operations.len() >= self.max_size {
            return Err(BatchOversightError::BatchFull {
                batch_id: self.batch_id.clone(),
                max_size: self.max_size,
            });
        }
        self.operations.push(op);
        Ok(())
    }

    /// Assign the batch to a Swarm Commander.
    pub fn assign_commander(&mut self, commander_id: impl Into<String>) {
        self.commander_id = Some(commander_id.into());
    }

    /// Approve a single operation within the batch.
    ///
    /// Returns `Err` if the operation is not found or is not in PENDING state.
    pub fn approve_operation(
        &mut self,
        operation_id: &str,
    ) -> Result<(), BatchOversightError> {
        let op = self.find_operation_mut(operation_id)?;
        if op.status != OperationStatus::Pending {
            return Err(BatchOversightError::InvalidTransition {
                operation_id: operation_id.to_string(),
                current: op.status.clone(),
                requested: "approve".to_string(),
            });
        }
        op.status = OperationStatus::Approved;
        Ok(())
    }

    /// Reject a single operation within the batch.
    pub fn reject_operation(
        &mut self,
        operation_id: &str,
    ) -> Result<(), BatchOversightError> {
        let op = self.find_operation_mut(operation_id)?;
        if op.status != OperationStatus::Pending {
            return Err(BatchOversightError::InvalidTransition {
                operation_id: operation_id.to_string(),
                current: op.status.clone(),
                requested: "reject".to_string(),
            });
        }
        op.status = OperationStatus::Rejected;
        Ok(())
    }

    /// Record the commander's cryptographic sign-off on the entire batch.
    ///
    /// The decision is derived automatically:
    ///   - All APPROVED  → ApproveAll
    ///   - All REJECTED  → RejectAll
    ///   - Mixed         → PartialApprove
    ///
    /// Returns `Err` if no commander is assigned or if any operation
    /// is still PENDING.
    pub fn finalize(
        &mut self,
        timestamp_unix: u64,
        signature: impl Into<String>,
        note: Option<String>,
    ) -> Result<BatchDecision, BatchOversightError> {
        let commander_id = self
            .commander_id
            .clone()
            .ok_or(BatchOversightError::NoCommanderAssigned {
                batch_id: self.batch_id.clone(),
            })?;

        // Reject finalization if any operation is still pending
        if let Some(pending_op) = self
            .operations
            .iter()
            .find(|o| o.status == OperationStatus::Pending)
        {
            return Err(BatchOversightError::UnreviewedOperations {
                batch_id: self.batch_id.clone(),
                pending_id: pending_op.operation_id.clone(),
            });
        }

        let approved = self
            .operations
            .iter()
            .filter(|o| o.status == OperationStatus::Approved)
            .count();
        let rejected = self
            .operations
            .iter()
            .filter(|o| o.status == OperationStatus::Rejected)
            .count();

        let decision = match (approved, rejected) {
            (_, 0) => BatchDecision::ApproveAll,
            (0, _) => BatchDecision::RejectAll,
            _      => BatchDecision::PartialApprove,
        };

        self.batch_signoff = Some(BatchSignoff {
            commander_id,
            decision: decision.clone(),
            note,
            timestamp_unix,
            signature: signature.into(),
        });

        Ok(decision)
    }

    // ─── Internal helpers ─────────────────────────────────────

    fn find_operation_mut(
        &mut self,
        operation_id: &str,
    ) -> Result<&mut BatchOperation, BatchOversightError> {
        self.operations
            .iter_mut()
            .find(|o| o.operation_id == operation_id)
            .ok_or_else(|| BatchOversightError::OperationNotFound {
                operation_id: operation_id.to_string(),
                batch_id: self.batch_id.clone(),
            })
    }

    /// Count of operations in each state.
    pub fn summary(&self) -> BatchSummary {
        let mut summary = BatchSummary::default();
        for op in &self.operations {
            match op.status {
                OperationStatus::Pending  => summary.pending  += 1,
                OperationStatus::Approved => summary.approved += 1,
                OperationStatus::Rejected => summary.rejected += 1,
                OperationStatus::Executed => summary.executed += 1,
            }
        }
        summary.total = self.operations.len();
        summary
    }
}

/// Summary statistics for a batch review session.
#[derive(Debug, Default)]
pub struct BatchSummary {
    pub total: usize,
    pub pending: usize,
    pub approved: usize,
    pub rejected: usize,
    pub executed: usize,
}

// ─── Error Types ─────────────────────────────────────────────

#[derive(Debug)]
pub enum BatchOversightError {
    /// Batch has reached its maximum size
    BatchFull { batch_id: String, max_size: usize },
    /// Requested operation was not found in the batch
    OperationNotFound { operation_id: String, batch_id: String },
    /// Operation state transition is not allowed
    InvalidTransition {
        operation_id: String,
        current: OperationStatus,
        requested: String,
    },
    /// Batch cannot be finalized without a commander
    NoCommanderAssigned { batch_id: String },
    /// At least one operation is still PENDING; finalization blocked
    UnreviewedOperations { batch_id: String, pending_id: String },
}

impl fmt::Display for BatchOversightError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BatchOversightError::BatchFull { batch_id, max_size } =>
                write!(f, "BATCH FULL: '{}' already has {} operations", batch_id, max_size),
            BatchOversightError::OperationNotFound { operation_id, batch_id } =>
                write!(f, "OPERATION NOT FOUND: '{}' in batch '{}'", operation_id, batch_id),
            BatchOversightError::InvalidTransition { operation_id, current, requested } =>
                write!(f, "INVALID TRANSITION: cannot '{}' operation '{}' in state {}",
                       requested, operation_id, current),
            BatchOversightError::NoCommanderAssigned { batch_id } =>
                write!(f, "NO COMMANDER: batch '{}' has no assigned commander", batch_id),
            BatchOversightError::UnreviewedOperations { batch_id, pending_id } =>
                write!(f, "UNREVIEWED OPERATIONS: batch '{}' still has pending item '{}'; \
                           review all operations before finalizing", batch_id, pending_id),
        }
    }
}

// ─── Tests ───────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_op(id: &str) -> BatchOperation {
        BatchOperation::new(id, "AI email draft", "claude-3", RiskLevel::Low)
    }

    #[test]
    fn test_batch_approve_all() {
        let mut batch = OperationBatch::new("batch-001", 10);
        batch.assign_commander("alice");
        batch.add_operation(make_op("op-1")).unwrap();
        batch.add_operation(make_op("op-2")).unwrap();
        batch.approve_operation("op-1").unwrap();
        batch.approve_operation("op-2").unwrap();
        let decision = batch.finalize(1711000000, "sig_hex", None).unwrap();
        assert_eq!(decision, BatchDecision::ApproveAll);
    }

    #[test]
    fn test_batch_reject_all() {
        let mut batch = OperationBatch::new("batch-002", 10);
        batch.assign_commander("bob");
        batch.add_operation(make_op("op-3")).unwrap();
        batch.reject_operation("op-3").unwrap();
        let decision = batch.finalize(1711000001, "sig_hex", None).unwrap();
        assert_eq!(decision, BatchDecision::RejectAll);
    }

    #[test]
    fn test_batch_partial_approve() {
        let mut batch = OperationBatch::new("batch-003", 10);
        batch.assign_commander("carol");
        batch.add_operation(make_op("op-4")).unwrap();
        batch.add_operation(make_op("op-5")).unwrap();
        batch.approve_operation("op-4").unwrap();
        batch.reject_operation("op-5").unwrap();
        let decision = batch.finalize(1711000002, "sig_hex", Some("ok".to_string())).unwrap();
        assert_eq!(decision, BatchDecision::PartialApprove);
    }

    #[test]
    fn test_batch_full_error() {
        let mut batch = OperationBatch::new("batch-004", 2);
        batch.add_operation(make_op("op-6")).unwrap();
        batch.add_operation(make_op("op-7")).unwrap();
        let err = batch.add_operation(make_op("op-8"));
        assert!(err.is_err());
    }

    #[test]
    fn test_finalize_blocked_with_pending_ops() {
        let mut batch = OperationBatch::new("batch-005", 10);
        batch.assign_commander("dave");
        batch.add_operation(make_op("op-9")).unwrap();
        // did not review op-9
        let err = batch.finalize(1711000003, "sig", None);
        assert!(err.is_err());
    }

    #[test]
    fn test_finalize_blocked_without_commander() {
        let mut batch = OperationBatch::new("batch-006", 10);
        batch.add_operation(make_op("op-10")).unwrap();
        batch.approve_operation("op-10").unwrap();
        let err = batch.finalize(1711000004, "sig", None);
        assert!(err.is_err());
    }

    #[test]
    fn test_summary_counts() {
        let mut batch = OperationBatch::new("batch-007", 10);
        batch.assign_commander("eve");
        batch.add_operation(make_op("op-11")).unwrap();
        batch.add_operation(make_op("op-12")).unwrap();
        batch.approve_operation("op-11").unwrap();
        let summary = batch.summary();
        assert_eq!(summary.total, 2);
        assert_eq!(summary.approved, 1);
        assert_eq!(summary.pending, 1);
    }

    #[test]
    fn test_duplicate_approve_fails() {
        let mut batch = OperationBatch::new("batch-008", 10);
        batch.assign_commander("frank");
        batch.add_operation(make_op("op-13")).unwrap();
        batch.approve_operation("op-13").unwrap();
        let err = batch.approve_operation("op-13");
        assert!(err.is_err());
    }
}
