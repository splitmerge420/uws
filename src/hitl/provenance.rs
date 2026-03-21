// src/hitl/provenance.rs
// Open-Access Provenance Validation — Democratized HITL Job Class
//
// Unlike the medical module, provenance review requires NO professional
// license.  Any human worker can be hired to review and sign off on AI
// outputs, creating a new class of AI-era employment that augments rather
// than replaces human labor.
//
// Economic principle:
//   "AI disruption → immediate retraining into provenance validation jobs."
//
// A provenance reviewer:
//   1. Receives a batch of AI-generated outputs.
//   2. Reads each item and decides: APPROVE, FLAG, or REJECT.
//   3. Appends a cryptographic sign-off that is recorded in the audit chain.
//   4. Is compensated for each valid sign-off (payout rails TBD in ledger/).
//
// Command surface (planned):
//   uws hitl provenance list   --batch=<BATCH_ID>
//   uws hitl provenance review --item=<ITEM_ID>
//   uws hitl provenance sign   --item=<ITEM_ID> --decision=approve
//
// Council Session: 2026-03-21
// Authority: Dave Sheldon (INV-5)
// Invariants Enforced: INV-1 (Sovereignty), INV-2 (Consent), INV-3 (Audit)

#![allow(dead_code)]

use std::collections::BTreeMap;
use std::fmt;

// ─── Reviewer Identity ────────────────────────────────────────

/// Identity record for a provenance reviewer.
///
/// No professional license is required.  A reviewer needs only:
///   - A stable unique handle (username or wallet address)
///   - A public key for cryptographic sign-offs
#[derive(Debug, Clone)]
pub struct ProvenanceReviewer {
    /// Unique reviewer handle (GitHub username, email, or DID)
    pub reviewer_id: String,
    /// Human-readable display name
    pub display_name: String,
    /// Hex-encoded public key used to verify sign-offs
    pub public_key_hex: String,
    /// Total items reviewed (lifetime counter)
    pub items_reviewed: u64,
    /// Total sign-offs accepted by the network (reputation score)
    pub accepted_signoffs: u64,
}

impl ProvenanceReviewer {
    /// Create a new reviewer record with zero review history.
    pub fn new(
        reviewer_id: impl Into<String>,
        display_name: impl Into<String>,
        public_key_hex: impl Into<String>,
    ) -> Self {
        ProvenanceReviewer {
            reviewer_id: reviewer_id.into(),
            display_name: display_name.into(),
            public_key_hex: public_key_hex.into(),
            items_reviewed: 0,
            accepted_signoffs: 0,
        }
    }

    /// Simple acceptance-rate heuristic (0.0–1.0).
    ///
    /// Returns 0.0 for reviewers with no history to avoid division by zero.
    pub fn acceptance_rate(&self) -> f64 {
        if self.items_reviewed == 0 {
            return 0.0;
        }
        self.accepted_signoffs as f64 / self.items_reviewed as f64
    }
}

// ─── Provenance Item ──────────────────────────────────────────

/// A single AI-generated artifact awaiting human provenance validation.
#[derive(Debug, Clone)]
pub struct ProvenanceItem {
    /// Unique item identifier (UUID or content hash)
    pub item_id: String,
    /// Human-readable description of the AI output
    pub description: String,
    /// The raw AI-generated content (may be a summary or hash of large payloads)
    pub content_preview: String,
    /// Source AI model or agent that produced this output
    pub ai_source: String,
    /// Current validation state
    pub status: ProvenanceStatus,
    /// Sign-offs collected so far (reviewer_id → ProvenanceSignoff)
    pub signoffs: BTreeMap<String, ProvenanceSignoff>,
    /// Minimum number of independent sign-offs required for acceptance
    pub required_signoffs: usize,
}

/// Decision recorded by a provenance reviewer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProvenanceDecision {
    /// Reviewer confirms the output is accurate and useful
    Approve,
    /// Reviewer notes an issue but does not fully reject
    Flag,
    /// Reviewer rejects the output as inaccurate or harmful
    Reject,
}

impl fmt::Display for ProvenanceDecision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProvenanceDecision::Approve => write!(f, "APPROVE"),
            ProvenanceDecision::Flag    => write!(f, "FLAG"),
            ProvenanceDecision::Reject  => write!(f, "REJECT"),
        }
    }
}

/// Lifecycle state of a provenance item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProvenanceStatus {
    /// Awaiting review
    Open,
    /// Enough APPROVE sign-offs received; item is validated
    Validated,
    /// Item was flagged or rejected by reviewers
    Disputed,
    /// Permanently rejected and removed from active pool
    Closed,
}

impl fmt::Display for ProvenanceStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProvenanceStatus::Open      => write!(f, "OPEN"),
            ProvenanceStatus::Validated => write!(f, "VALIDATED"),
            ProvenanceStatus::Disputed  => write!(f, "DISPUTED"),
            ProvenanceStatus::Closed    => write!(f, "CLOSED"),
        }
    }
}

/// A single reviewer's sign-off on a provenance item.
#[derive(Debug, Clone)]
pub struct ProvenanceSignoff {
    pub reviewer_id: String,
    pub decision: ProvenanceDecision,
    /// Optional comment explaining the decision
    pub comment: Option<String>,
    /// Unix timestamp of the sign-off
    pub timestamp_unix: u64,
    /// Hex-encoded signature over (item_id + decision + timestamp)
    pub signature: String,
}

impl ProvenanceItem {
    /// Create a new open provenance item.
    pub fn new(
        item_id: impl Into<String>,
        description: impl Into<String>,
        content_preview: impl Into<String>,
        ai_source: impl Into<String>,
        required_signoffs: usize,
    ) -> Self {
        ProvenanceItem {
            item_id: item_id.into(),
            description: description.into(),
            content_preview: content_preview.into(),
            ai_source: ai_source.into(),
            status: ProvenanceStatus::Open,
            signoffs: BTreeMap::new(),
            required_signoffs,
        }
    }

    /// Record a reviewer's sign-off and update the item status.
    ///
    /// Returns `Err` if the item is already closed or the reviewer has
    /// already submitted a sign-off for this item.
    pub fn record_signoff(
        &mut self,
        signoff: ProvenanceSignoff,
    ) -> Result<(), ProvenanceError> {
        if self.status == ProvenanceStatus::Closed {
            return Err(ProvenanceError::ItemClosed {
                item_id: self.item_id.clone(),
            });
        }
        if self.signoffs.contains_key(&signoff.reviewer_id) {
            return Err(ProvenanceError::DuplicateSignoff {
                reviewer_id: signoff.reviewer_id.clone(),
                item_id: self.item_id.clone(),
            });
        }
        let decision = signoff.decision.clone();
        self.signoffs.insert(signoff.reviewer_id.clone(), signoff);
        self.update_status(decision);
        Ok(())
    }

    /// Recompute item status after a new sign-off is added.
    fn update_status(&mut self, latest_decision: ProvenanceDecision) {
        let approve_count = self
            .signoffs
            .values()
            .filter(|s| s.decision == ProvenanceDecision::Approve)
            .count();
        let reject_count = self
            .signoffs
            .values()
            .filter(|s| s.decision == ProvenanceDecision::Reject)
            .count();

        if reject_count > 0 || latest_decision == ProvenanceDecision::Flag {
            self.status = ProvenanceStatus::Disputed;
        }
        if approve_count >= self.required_signoffs {
            self.status = ProvenanceStatus::Validated;
        }
    }

    /// Number of APPROVE sign-offs collected so far.
    pub fn approve_count(&self) -> usize {
        self.signoffs
            .values()
            .filter(|s| s.decision == ProvenanceDecision::Approve)
            .count()
    }
}

// ─── Batch ────────────────────────────────────────────────────

/// A named collection of provenance items assigned to a reviewer.
///
/// Batches are the unit of work for provenance jobs.  A single reviewer
/// can process an entire batch and receive a payout upon completion.
#[derive(Debug, Clone)]
pub struct ProvenanceBatch {
    pub batch_id: String,
    pub items: Vec<ProvenanceItem>,
    /// Reviewer assigned to this batch (None = unassigned)
    pub assigned_reviewer: Option<String>,
}

impl ProvenanceBatch {
    pub fn new(batch_id: impl Into<String>) -> Self {
        ProvenanceBatch {
            batch_id: batch_id.into(),
            items: Vec::new(),
            assigned_reviewer: None,
        }
    }

    pub fn add_item(&mut self, item: ProvenanceItem) {
        self.items.push(item);
    }

    /// Assign the batch to a reviewer.
    pub fn assign(&mut self, reviewer_id: impl Into<String>) {
        self.assigned_reviewer = Some(reviewer_id.into());
    }

    /// Returns true when every item in the batch has been resolved
    /// (Validated, Disputed, or Closed).
    pub fn is_complete(&self) -> bool {
        self.items
            .iter()
            .all(|i| i.status != ProvenanceStatus::Open)
    }
}

// ─── Error Types ─────────────────────────────────────────────

#[derive(Debug)]
pub enum ProvenanceError {
    /// Attempting to sign off on a closed item
    ItemClosed { item_id: String },
    /// Reviewer already submitted a sign-off for this item
    DuplicateSignoff { reviewer_id: String, item_id: String },
    /// Reviewer not found
    ReviewerNotFound { reviewer_id: String },
}

impl fmt::Display for ProvenanceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProvenanceError::ItemClosed { item_id } =>
                write!(f, "ITEM CLOSED: cannot add sign-off to closed item '{}'", item_id),
            ProvenanceError::DuplicateSignoff { reviewer_id, item_id } =>
                write!(f, "DUPLICATE SIGNOFF: reviewer '{}' already signed item '{}'",
                       reviewer_id, item_id),
            ProvenanceError::ReviewerNotFound { reviewer_id } =>
                write!(f, "REVIEWER NOT FOUND: '{}'", reviewer_id),
        }
    }
}

// ─── Tests ───────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_signoff(reviewer_id: &str, decision: ProvenanceDecision) -> ProvenanceSignoff {
        ProvenanceSignoff {
            reviewer_id: reviewer_id.to_string(),
            decision,
            comment: None,
            timestamp_unix: 1711000000,
            signature: "aabbccdd".to_string(),
        }
    }

    #[test]
    fn test_item_validated_after_enough_approvals() {
        let mut item = ProvenanceItem::new(
            "item-001", "Test output", "preview", "gpt-4o", 2,
        );
        item.record_signoff(make_signoff("alice", ProvenanceDecision::Approve)).unwrap();
        assert_eq!(item.status, ProvenanceStatus::Open);
        item.record_signoff(make_signoff("bob", ProvenanceDecision::Approve)).unwrap();
        assert_eq!(item.status, ProvenanceStatus::Validated);
    }

    #[test]
    fn test_item_disputed_on_flag() {
        let mut item = ProvenanceItem::new(
            "item-002", "Test output", "preview", "claude-3", 3,
        );
        item.record_signoff(make_signoff("carol", ProvenanceDecision::Flag)).unwrap();
        assert_eq!(item.status, ProvenanceStatus::Disputed);
    }

    #[test]
    fn test_duplicate_signoff_rejected() {
        let mut item = ProvenanceItem::new(
            "item-003", "Test output", "preview", "gemini", 2,
        );
        item.record_signoff(make_signoff("dave", ProvenanceDecision::Approve)).unwrap();
        let err = item.record_signoff(make_signoff("dave", ProvenanceDecision::Approve));
        assert!(err.is_err());
    }

    #[test]
    fn test_batch_complete_when_all_resolved() {
        let mut batch = ProvenanceBatch::new("batch-001");
        let mut item = ProvenanceItem::new(
            "item-004", "Test", "preview", "deepseek", 1,
        );
        item.record_signoff(make_signoff("eve", ProvenanceDecision::Approve)).unwrap();
        batch.add_item(item);
        assert!(batch.is_complete());
    }

    #[test]
    fn test_reviewer_acceptance_rate_zero_when_no_history() {
        let r = ProvenanceReviewer::new("user1", "User One", "pubkey_hex");
        assert_eq!(r.acceptance_rate(), 0.0);
    }

    #[test]
    fn test_reviewer_acceptance_rate() {
        let mut r = ProvenanceReviewer::new("user2", "User Two", "pubkey_hex");
        r.items_reviewed = 10;
        r.accepted_signoffs = 8;
        assert!((r.acceptance_rate() - 0.8).abs() < f64::EPSILON);
    }
}
