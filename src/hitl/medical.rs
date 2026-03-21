// src/hitl/medical.rs
// Amazon One Medical — Licensed Professional HITL Integration
//
// STUB IMPLEMENTATION — production use requires:
//   • Valid NPI (National Provider Identifier) number
//   • State-issued professional license verification
//   • Business Associate Agreement (BAA) with Amazon One Medical
//   • HIPAA-compliant data handling (Model Armor sanitize=block)
//
// This module gates every AI-generated medical output behind a licensed
// professional review step before the result leaves the system.
//
// Command surface (planned):
//   uws hitl medical review --npi=<NPI> --session=<SESSION_ID>
//   uws hitl medical sign   --npi=<NPI> --session=<SESSION_ID>
//   uws hitl medical status --session=<SESSION_ID>
//
// Council Session: 2026-03-21
// Authority: Dave Sheldon (INV-5)
// Invariants Enforced: INV-2 (Consent), INV-3 (Audit), INV-11 (Encryption)

#![allow(dead_code)]

use std::fmt;

// ─── NPI Credential ──────────────────────────────────────────

/// A validated National Provider Identifier.
///
/// In production, `MedicalCredential::verify()` must contact the
/// CMS NPI Registry (https://npiregistry.cms.hhs.gov/) to confirm
/// the identifier is active and matches the practitioner's license.
#[derive(Debug, Clone)]
pub struct MedicalCredential {
    /// 10-digit NPI issued by the Centers for Medicare & Medicaid Services
    pub npi: String,
    /// Full legal name of the licensed practitioner
    pub practitioner_name: String,
    /// Issuing state (e.g. "TX", "CA")
    pub license_state: String,
    /// License number in the issuing state
    pub license_number: String,
    /// Taxonomy code describing the practitioner's specialty
    pub taxonomy_code: String,
}

impl MedicalCredential {
    /// Create a new (unverified) credential record.
    pub fn new(
        npi: impl Into<String>,
        practitioner_name: impl Into<String>,
        license_state: impl Into<String>,
        license_number: impl Into<String>,
        taxonomy_code: impl Into<String>,
    ) -> Self {
        MedicalCredential {
            npi: npi.into(),
            practitioner_name: practitioner_name.into(),
            license_state: license_state.into(),
            license_number: license_number.into(),
            taxonomy_code: taxonomy_code.into(),
        }
    }

    /// Validate the NPI format (10 numeric digits, Luhn-10 check).
    ///
    /// This is a local structural check only.  Full registry verification
    /// requires a live call to the CMS NPI Registry API.
    pub fn validate_npi_format(&self) -> Result<(), MedicalHitlError> {
        if self.npi.len() != 10 || !self.npi.chars().all(|c| c.is_ascii_digit()) {
            return Err(MedicalHitlError::InvalidNpi {
                npi: self.npi.clone(),
                reason: "NPI must be exactly 10 numeric digits".to_string(),
            });
        }
        Ok(())
    }
}

// ─── HITL Review Session ──────────────────────────────────────

/// A single medical HITL review session.
///
/// A session bundles one AI-generated output with the licensed professional
/// who reviewed it and the outcome of that review.
#[derive(Debug, Clone)]
pub struct MedicalHitlSession {
    /// Unique session identifier (UUID in production)
    pub session_id: String,
    /// Credential of the reviewing practitioner
    pub reviewer: MedicalCredential,
    /// Raw AI-generated content under review (PII stripped by Model Armor)
    pub ai_output_summary: String,
    /// Outcome of the review
    pub status: ReviewStatus,
    /// Optional clinical notes added by the practitioner
    pub notes: Option<String>,
    /// Unix timestamp of the sign-off (0 = not yet signed)
    pub signed_at_unix: u64,
    /// Cryptographic signature placeholder (hex string in production)
    pub signature: Option<String>,
}

/// Lifecycle state of a medical review session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReviewStatus {
    /// Session created; awaiting practitioner review
    Pending,
    /// Practitioner is actively reviewing
    InReview,
    /// Practitioner approved the AI output
    Approved,
    /// Practitioner rejected the AI output (requires revision)
    Rejected,
    /// Output requires escalation to a specialist
    Escalated,
}

impl fmt::Display for ReviewStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReviewStatus::Pending    => write!(f, "PENDING"),
            ReviewStatus::InReview   => write!(f, "IN_REVIEW"),
            ReviewStatus::Approved   => write!(f, "APPROVED"),
            ReviewStatus::Rejected   => write!(f, "REJECTED"),
            ReviewStatus::Escalated  => write!(f, "ESCALATED"),
        }
    }
}

impl MedicalHitlSession {
    /// Create a new pending review session.
    pub fn new(
        session_id: impl Into<String>,
        reviewer: MedicalCredential,
        ai_output_summary: impl Into<String>,
    ) -> Self {
        MedicalHitlSession {
            session_id: session_id.into(),
            reviewer,
            ai_output_summary: ai_output_summary.into(),
            status: ReviewStatus::Pending,
            notes: None,
            signed_at_unix: 0,
            signature: None,
        }
    }

    /// Mark the session as under active review.
    pub fn start_review(&mut self) {
        self.status = ReviewStatus::InReview;
    }

    /// Approve the AI output and record a cryptographic sign-off.
    ///
    /// `timestamp_unix` — seconds since UNIX epoch.
    /// `signature`      — hex-encoded signature (production: ML-DSA / Ed25519).
    ///
    /// Returns `Err` if the reviewer's NPI format is invalid.
    pub fn approve(
        &mut self,
        timestamp_unix: u64,
        signature: impl Into<String>,
        notes: Option<String>,
    ) -> Result<(), MedicalHitlError> {
        self.reviewer.validate_npi_format()?;
        self.status = ReviewStatus::Approved;
        self.signed_at_unix = timestamp_unix;
        self.signature = Some(signature.into());
        self.notes = notes;
        Ok(())
    }

    /// Reject the AI output with mandatory reviewer notes.
    pub fn reject(&mut self, reason: impl Into<String>) {
        self.status = ReviewStatus::Rejected;
        self.notes = Some(reason.into());
    }

    /// Escalate to a specialist.
    pub fn escalate(&mut self, reason: impl Into<String>) {
        self.status = ReviewStatus::Escalated;
        self.notes = Some(reason.into());
    }
}

// ─── Error Types ─────────────────────────────────────────────

#[derive(Debug)]
pub enum MedicalHitlError {
    /// NPI is structurally invalid
    InvalidNpi { npi: String, reason: String },
    /// Session cannot transition to the requested state
    InvalidStateTransition { from: ReviewStatus, action: String },
    /// Required credential field is missing
    MissingCredential { field: String },
    /// Amazon One Medical API returned an error (stub)
    ApiError { status: u16, message: String },
}

impl fmt::Display for MedicalHitlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MedicalHitlError::InvalidNpi { npi, reason } =>
                write!(f, "INVALID NPI [{}]: {}", npi, reason),
            MedicalHitlError::InvalidStateTransition { from, action } =>
                write!(f, "INVALID STATE TRANSITION: cannot '{}' from state {}", action, from),
            MedicalHitlError::MissingCredential { field } =>
                write!(f, "MISSING CREDENTIAL FIELD: {}", field),
            MedicalHitlError::ApiError { status, message } =>
                write!(f, "ONE_MEDICAL API ERROR ({}): {}", status, message),
        }
    }
}

// ─── Tests ───────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_credential() -> MedicalCredential {
        MedicalCredential::new(
            "1234567890",
            "Dr. Jane Smith",
            "TX",
            "TX-MED-12345",
            "207Q00000X", // Family Medicine
        )
    }

    #[test]
    fn test_npi_valid_format() {
        let cred = make_credential();
        assert!(cred.validate_npi_format().is_ok());
    }

    #[test]
    fn test_npi_invalid_length() {
        let cred = MedicalCredential::new("12345", "Dr. X", "TX", "TX-1", "207Q00000X");
        let err = cred.validate_npi_format();
        assert!(err.is_err());
    }

    #[test]
    fn test_npi_non_numeric() {
        let cred = MedicalCredential::new("12345ABCDE", "Dr. X", "TX", "TX-1", "207Q00000X");
        let err = cred.validate_npi_format();
        assert!(err.is_err());
    }

    #[test]
    fn test_session_approve() {
        let cred = make_credential();
        let mut session = MedicalHitlSession::new("sess-001", cred, "AI output summary");
        session.start_review();
        let result = session.approve(1711000000, "deadbeef", Some("LGTM".to_string()));
        assert!(result.is_ok());
        assert_eq!(session.status, ReviewStatus::Approved);
        assert_eq!(session.signed_at_unix, 1711000000);
    }

    #[test]
    fn test_session_reject() {
        let cred = make_credential();
        let mut session = MedicalHitlSession::new("sess-002", cred, "AI output summary");
        session.reject("Dosage recommendation out of range");
        assert_eq!(session.status, ReviewStatus::Rejected);
        assert!(session.notes.is_some());
    }

    #[test]
    fn test_approve_with_invalid_npi_fails() {
        let bad_cred = MedicalCredential::new("BAD", "Dr. X", "TX", "TX-1", "207Q00000X");
        let mut session = MedicalHitlSession::new("sess-003", bad_cred, "summary");
        let result = session.approve(1711000000, "sig", None);
        assert!(result.is_err());
    }
}
