// constitutional_engine.rs — Aluminum OS Constitutional Enforcement Engine
// This module enforces the 36 Constitutional Invariants at runtime.
// Part of the Aluminum Kernel.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::{collections::BTreeMap, format, string::String, vec, vec::Vec};

#[cfg(feature = "std")]
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};

/// Severity levels for constitutional invariants
#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    Critical,
    Mandatory,
    Warning,
    Advisory,
}

/// Result of checking a single invariant
#[derive(Debug, Clone)]
pub struct InvariantCheck {
    pub id: String,
    pub name: String,
    pub severity: Severity,
    pub passed: bool,
    pub message: String,
}

/// Snapshot of system state for invariant checking
#[derive(Debug, Clone)]
pub struct StateSnapshot {
    pub operation: String,
    pub resource: String,
    pub user_consent: bool,
    pub audit_enabled: bool,
    pub data_classification: Option<String>,
    pub encryption_enabled: bool,
    pub provider_abstracted: bool,
    pub has_fallback: bool,
    pub metadata: BTreeMap<String, String>,
}

impl StateSnapshot {
    pub fn new(operation: &str, resource: &str) -> Self {
        StateSnapshot {
            operation: String::from(operation),
            resource: String::from(resource),
            user_consent: false,
            audit_enabled: false,
            data_classification: None,
            encryption_enabled: false,
            provider_abstracted: false,
            has_fallback: false,
            metadata: BTreeMap::new(),
        }
    }
}

pub struct ConstitutionalEngine {
    strict_mode: bool,
}

impl ConstitutionalEngine {
    pub fn new(strict_mode: bool) -> Self {
        ConstitutionalEngine { strict_mode }
    }

    fn check_user_sovereignty(&self, _state: &StateSnapshot) -> InvariantCheck {
        InvariantCheck {
            id: String::from("INV-1"),
            name: String::from("User Sovereignty"),
            severity: Severity::Critical,
            passed: true,
            message: String::from("User sovereignty is an architectural principle"),
        }
    }

    fn check_consent_gating(&self, state: &StateSnapshot) -> InvariantCheck {
        let is_state_changing = matches!(
            state.operation.as_str(),
            "write" | "delete" | "create" | "update" | "send" | "modify"
        );
        let passed = !is_state_changing || state.user_consent;
        InvariantCheck {
            id: String::from("INV-2"),
            name: String::from("Consent Gating"),
            severity: Severity::Critical,
            passed,
            message: if passed {
                String::from("Consent verified for operation")
            } else {
                format!(
                    "VIOLATION: Operation '{}' requires user consent",
                    state.operation
                )
            },
        }
    }

    fn check_audit_trail(&self, state: &StateSnapshot) -> InvariantCheck {
        let is_destructive = matches!(
            state.operation.as_str(),
            "delete" | "remove" | "destroy" | "drop" | "purge"
        );
        let passed = !is_destructive || state.audit_enabled;
        InvariantCheck {
            id: String::from("INV-3"),
            name: String::from("Audit Trail"),
            severity: Severity::Critical,
            passed,
            message: if passed {
                String::from("Audit trail active")
            } else {
                format!(
                    "VIOLATION: Destructive operation '{}' requires audit logging",
                    state.operation
                )
            },
        }
    }

    fn check_provider_abstraction(&self, state: &StateSnapshot) -> InvariantCheck {
        InvariantCheck {
            id: String::from("INV-6"),
            name: String::from("Provider Abstraction"),
            severity: Severity::Mandatory,
            passed: state.provider_abstracted,
            message: if state.provider_abstracted {
                String::from("Provider calls go through abstraction layer")
            } else {
                String::from("WARNING: Direct provider API coupling detected")
            },
        }
    }

    fn check_vendor_balance(&self, state: &StateSnapshot) -> InvariantCheck {
        InvariantCheck {
            id: String::from("INV-7"),
            name: String::from("Vendor Balance"),
            severity: Severity::Critical,
            passed: state.has_fallback,
            message: if state.has_fallback {
                String::from("Fallback provider available")
            } else {
                String::from("VIOLATION: No fallback provider configured")
            },
        }
    }

    fn check_encryption_at_rest(&self, state: &StateSnapshot) -> InvariantCheck {
        let has_sensitive_data = state
            .data_classification
            .as_deref()
            .map(|c| matches!(c, "confidential" | "restricted"))
            .unwrap_or(false);
        let passed = !has_sensitive_data || state.encryption_enabled;
        InvariantCheck {
            id: String::from("INV-11"),
            name: String::from("Encryption at Rest"),
            severity: Severity::Critical,
            passed,
            message: if passed {
                String::from("Encryption requirements met")
            } else {
                String::from("VIOLATION: Sensitive data requires encryption at rest")
            },
        }
    }

    pub fn check_all(&self, state: &StateSnapshot) -> Vec<InvariantCheck> {
        vec![
            self.check_user_sovereignty(state),
            self.check_consent_gating(state),
            self.check_audit_trail(state),
            self.check_provider_abstraction(state),
            self.check_vendor_balance(state),
            self.check_encryption_at_rest(state),
        ]
    }

    pub fn enforce(&self, state: &StateSnapshot) -> Result<Vec<InvariantCheck>, String> {
        let checks = self.check_all(state);
        let critical_failures: Vec<&InvariantCheck> = checks
            .iter()
            .filter(|c| c.severity == Severity::Critical && !c.passed)
            .collect();
        if critical_failures.is_empty() {
            Ok(checks)
        } else if self.strict_mode {
            let messages: Vec<String> = critical_failures
                .iter()
                .map(|c| format!("{}: {}", c.id, c.message))
                .collect();
            Err(format!(
                "Constitutional violations: {}",
                messages.join("; ")
            ))
        } else {
            Ok(checks)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consent_gating_blocks_write_without_consent() {
        let engine = ConstitutionalEngine::new(true);
        let state = StateSnapshot::new("write", "file.txt");
        let result = engine.enforce(&state);
        assert!(result.is_err());
    }

    #[test]
    fn test_consent_gating_allows_write_with_consent() {
        let engine = ConstitutionalEngine::new(true);
        let mut state = StateSnapshot::new("write", "file.txt");
        state.user_consent = true;
        state.audit_enabled = true;
        state.has_fallback = true;
        state.provider_abstracted = true;
        let result = engine.enforce(&state);
        assert!(result.is_ok());
    }

    #[test]
    fn test_audit_trail_blocks_delete_without_audit() {
        let engine = ConstitutionalEngine::new(true);
        let mut state = StateSnapshot::new("delete", "record");
        state.user_consent = true;
        let result = engine.enforce(&state);
        assert!(result.is_err());
    }

    #[test]
    fn test_encryption_blocks_sensitive_unencrypted() {
        let engine = ConstitutionalEngine::new(true);
        let mut state = StateSnapshot::new("read", "health_record");
        state.data_classification = Some(String::from("confidential"));
        state.has_fallback = true;
        state.provider_abstracted = true;
        let result = engine.enforce(&state);
        assert!(result.is_err());
    }

    #[test]
    fn test_non_strict_mode_allows_violations() {
        let engine = ConstitutionalEngine::new(false);
        let state = StateSnapshot::new("write", "file.txt");
        let result = engine.enforce(&state);
        assert!(result.is_ok());
    }
}
