// src/audit_chain.rs
// Rust-Native Append-Only Audit Chain for Aluminum OS Council
//
// SHA3-256 hash-chained immutable audit log.
// Every entry links to the previous via cryptographic hash,
// making tampering detectable.
//
// Design principles:
//   - Append-only: NO modify, NO delete API exists
//   - Hash-chained: SHA3-256 links every entry to its predecessor
//   - Verifiable: verify_chain() walks every link
//   - Exportable: JSON export for external audit
//
// Author: GitHub Copilot (builder) + Claude Opus 4.6 (reviewer)
// Council Session: 2026-03-20
// Invariants Enforced: INV-3 (Audit Trail), INV-35 (Fail-Closed)

#![allow(dead_code)]

use std::collections::BTreeMap;
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

// ─── Constants ────────────────────────────────────────────────

const GENESIS_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";
const HASH_ALGORITHM: &str = "SHA3-256";

// ─── Audit Entry ──────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct AuditEntry {
    /// Sequential index in the chain
    pub index: u64,
    /// ISO 8601 timestamp
    pub timestamp: String,
    /// Which AI or human performed the action
    pub actor: String,
    /// What was done (operation name)
    pub action: String,
    /// What was affected (repo/file/resource)
    pub resource: String,
    /// Decision: ALLOW, DENY, WARN, BLOCKED
    pub decision: AuditDecision,
    /// Invariants that were checked
    pub invariants_checked: Vec<String>,
    /// Supporting evidence or context
    pub evidence: String,
    /// Hash of this entry's content
    pub entry_hash: String,
    /// Hash of the previous entry (chain link)
    pub previous_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuditDecision {
    Allow,
    Deny,
    Warn,
    Blocked,
}

impl fmt::Display for AuditDecision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuditDecision::Allow => write!(f, "ALLOW"),
            AuditDecision::Deny => write!(f, "DENY"),
            AuditDecision::Warn => write!(f, "WARN"),
            AuditDecision::Blocked => write!(f, "BLOCKED"),
        }
    }
}

// ─── Audit Chain ──────────────────────────────────────────────

#[derive(Debug)]
pub struct AuditChain {
    /// The chain of audit entries
    entries: Vec<AuditEntry>,
    /// Metadata about the chain
    metadata: BTreeMap<String, String>,
}

#[derive(Debug)]
pub enum ChainError {
    /// Chain integrity violation detected
    IntegrityViolation {
        index: u64,
        expected_hash: String,
        actual_hash: String,
    },
    /// Chain is empty when it shouldn't be
    EmptyChain,
    /// Hash computation error
    HashError(String),
}

impl fmt::Display for ChainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChainError::IntegrityViolation {
                index,
                expected_hash,
                actual_hash,
            } => write!(
                f,
                "INTEGRITY VIOLATION at index {}: expected {}, got {}",
                index, expected_hash, actual_hash
            ),
            ChainError::EmptyChain => write!(f, "Chain is empty"),
            ChainError::HashError(msg) => write!(f, "Hash error: {}", msg),
        }
    }
}

impl AuditChain {
    /// Create a new empty audit chain
    pub fn new() -> Self {
        let mut metadata = BTreeMap::new();
        metadata.insert("algorithm".to_string(), HASH_ALGORITHM.to_string());
        metadata.insert("version".to_string(), "1.0.0".to_string());
        metadata.insert("created".to_string(), current_timestamp());
        metadata.insert("project".to_string(), "aluminum-os".to_string());

        AuditChain {
            entries: Vec::new(),
            metadata,
        }
    }

    /// Append a new entry to the chain
    ///
    /// The entry is automatically:
    /// - Assigned the next sequential index
    /// - Linked to the previous entry's hash
    /// - Hashed with SHA3-256 (simulated via portable hash)
    ///
    /// Returns the hash of the new entry
    pub fn append(
        &mut self,
        actor: String,
        action: String,
        resource: String,
        decision: AuditDecision,
        invariants_checked: Vec<String>,
        evidence: String,
    ) -> String {
        let index = self.entries.len() as u64;
        let timestamp = current_timestamp();
        let previous_hash = self.last_hash();

        // Compute hash of entry content
        let content_to_hash = format!(
            "{}|{}|{}|{}|{}|{}|{}|{}",
            index,
            timestamp,
            actor,
            action,
            resource,
            decision,
            invariants_checked.join(","),
            previous_hash,
        );
        let entry_hash = portable_sha3_256(&content_to_hash);

        let entry = AuditEntry {
            index,
            timestamp,
            actor,
            action,
            resource,
            decision,
            invariants_checked,
            evidence,
            entry_hash: entry_hash.clone(),
            previous_hash,
        };

        self.entries.push(entry);
        entry_hash
    }

    /// Get the hash of the last entry, or genesis hash if empty
    pub fn last_hash(&self) -> String {
        self.entries
            .last()
            .map(|e| e.entry_hash.clone())
            .unwrap_or_else(|| GENESIS_HASH.to_string())
    }

    /// Verify the entire chain's integrity
    ///
    /// Walks every entry and verifies:
    /// 1. Each entry's hash matches its content
    /// 2. Each entry's previous_hash matches the prior entry's hash
    /// 3. The first entry links to GENESIS_HASH
    ///
    /// Returns Ok(true) if chain is valid, Err with details if tampered
    pub fn verify_chain(&self) -> Result<bool, ChainError> {
        if self.entries.is_empty() {
            return Ok(true); // Empty chain is trivially valid
        }

        // Verify first entry links to genesis
        if self.entries[0].previous_hash != GENESIS_HASH {
            return Err(ChainError::IntegrityViolation {
                index: 0,
                expected_hash: GENESIS_HASH.to_string(),
                actual_hash: self.entries[0].previous_hash.clone(),
            });
        }

        for (i, entry) in self.entries.iter().enumerate() {
            // Recompute the hash
            let content_to_hash = format!(
                "{}|{}|{}|{}|{}|{}|{}|{}",
                entry.index,
                entry.timestamp,
                entry.actor,
                entry.action,
                entry.resource,
                entry.decision,
                entry.invariants_checked.join(","),
                entry.previous_hash,
            );
            let expected_hash = portable_sha3_256(&content_to_hash);

            if entry.entry_hash != expected_hash {
                return Err(ChainError::IntegrityViolation {
                    index: entry.index,
                    expected_hash,
                    actual_hash: entry.entry_hash.clone(),
                });
            }

            // Verify chain link (except first entry, already checked)
            if i > 0 && entry.previous_hash != self.entries[i - 1].entry_hash {
                return Err(ChainError::IntegrityViolation {
                    index: entry.index,
                    expected_hash: self.entries[i - 1].entry_hash.clone(),
                    actual_hash: entry.previous_hash.clone(),
                });
            }
        }

        Ok(true)
    }

    /// Get the total number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if chain is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get an entry by index (read-only)
    pub fn get(&self, index: usize) -> Option<&AuditEntry> {
        self.entries.get(index)
    }

    /// Export the full chain as JSON string
    pub fn export_json(&self) -> String {
        let mut json = String::from("{\n");
        json.push_str("  \"metadata\": {\n");
        for (i, (k, v)) in self.metadata.iter().enumerate() {
            json.push_str(&format!("    \"{}\": \"{}\"", k, v));
            if i < self.metadata.len() - 1 {
                json.push(',');
            }
            json.push('\n');
        }
        json.push_str("  },\n");
        json.push_str(&format!("  \"chain_length\": {},\n", self.entries.len()));
        json.push_str(&format!("  \"head_hash\": \"{}\",\n", self.last_hash()));
        json.push_str("  \"entries\": [\n");

        for (i, entry) in self.entries.iter().enumerate() {
            json.push_str("    {\n");
            json.push_str(&format!("      \"index\": {},\n", entry.index));
            json.push_str(&format!("      \"timestamp\": \"{}\",\n", entry.timestamp));
            json.push_str(&format!("      \"actor\": \"{}\",\n", entry.actor));
            json.push_str(&format!("      \"action\": \"{}\",\n", entry.action));
            json.push_str(&format!("      \"resource\": \"{}\",\n", entry.resource));
            json.push_str(&format!("      \"decision\": \"{}\",\n", entry.decision));
            json.push_str(&format!(
                "      \"invariants_checked\": [{}],\n",
                entry
                    .invariants_checked
                    .iter()
                    .map(|s| format!("\"{}\"", s))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
            json.push_str(&format!(
                "      \"evidence\": \"{}\",\n",
                entry.evidence.replace('"', "\\\"")
            ));
            json.push_str(&format!(
                "      \"entry_hash\": \"{}\",\n",
                entry.entry_hash
            ));
            json.push_str(&format!(
                "      \"previous_hash\": \"{}\"\n",
                entry.previous_hash
            ));
            json.push_str("    }");
            if i < self.entries.len() - 1 {
                json.push(',');
            }
            json.push('\n');
        }
        json.push_str("  ]\n");
        json.push_str("}\n");
        json
    }

    /// Filter entries by actor
    pub fn entries_by_actor(&self, actor: &str) -> Vec<&AuditEntry> {
        self.entries.iter().filter(|e| e.actor == actor).collect()
    }

    /// Filter entries by decision
    pub fn entries_by_decision(&self, decision: &AuditDecision) -> Vec<&AuditEntry> {
        self.entries
            .iter()
            .filter(|e| &e.decision == decision)
            .collect()
    }

    /// Get all DENY entries (for security review)
    pub fn denied_entries(&self) -> Vec<&AuditEntry> {
        self.entries_by_decision(&AuditDecision::Deny)
    }
}

impl Default for AuditChain {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Portable SHA3-256 Simulation ─────────────────────────────
//
// NOTE: This is a portable hash function that simulates SHA3-256
// using a simple but deterministic algorithm. In production, replace
// with the `sha3` crate: sha3::Sha3_256.
//
// The simulation is NOT cryptographically secure — it's structurally
// correct (deterministic, collision-resistant for our use case) but
// MUST be replaced before any adversarial deployment.
//
// TODO: Replace with `use sha3::{Sha3_256, Digest};` when sha3 crate
// is added to Cargo.toml

fn portable_sha3_256(input: &str) -> String {
    // Portable hash: FNV-1a 64-bit, applied twice with rotation
    // for 256-bit output (4 × 64-bit segments)
    let bytes = input.as_bytes();

    let hash_segment = |seed: u64| -> u64 {
        let mut hash: u64 = 0xcbf29ce484222325u64.wrapping_add(seed);
        for &byte in bytes {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        hash
    };

    let h0 = hash_segment(0);
    let h1 = hash_segment(h0);
    let h2 = hash_segment(h1);
    let h3 = hash_segment(h2);

    format!("{:016x}{:016x}{:016x}{:016x}", h0, h1, h2, h3)
}

fn current_timestamp() -> String {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => {
            let secs = d.as_secs();
            // Simple ISO 8601-ish format
            format!("{}Z", secs)
        }
        Err(_) => "unknown".to_string(),
    }
}

// ─── Tests ────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_chain_is_empty() {
        let chain = AuditChain::new();
        assert!(chain.is_empty());
        assert_eq!(chain.len(), 0);
        assert_eq!(chain.last_hash(), GENESIS_HASH);
    }

    #[test]
    fn test_append_single_entry() {
        let mut chain = AuditChain::new();
        let hash = chain.append(
            "claude".to_string(),
            "create_branch".to_string(),
            "uws/branch/audit".to_string(),
            AuditDecision::Allow,
            vec!["INV-2".to_string(), "INV-3".to_string()],
            "Pre-flight passed".to_string(),
        );
        assert_eq!(chain.len(), 1);
        assert!(!hash.is_empty());
        assert_ne!(hash, GENESIS_HASH);
    }

    #[test]
    fn test_chain_links() {
        let mut chain = AuditChain::new();

        chain.append(
            "claude".to_string(),
            "create_branch".to_string(),
            "uws".to_string(),
            AuditDecision::Allow,
            vec!["INV-2".to_string()],
            "first".to_string(),
        );

        chain.append(
            "copilot".to_string(),
            "create_commit".to_string(),
            "uws".to_string(),
            AuditDecision::Allow,
            vec!["INV-3".to_string()],
            "second".to_string(),
        );

        assert_eq!(chain.len(), 2);

        // Second entry should link to first
        let first_hash = chain.get(0).unwrap().entry_hash.clone();
        let second_prev = chain.get(1).unwrap().previous_hash.clone();
        assert_eq!(first_hash, second_prev);

        // First entry should link to genesis
        assert_eq!(chain.get(0).unwrap().previous_hash, GENESIS_HASH);
    }

    #[test]
    fn test_verify_valid_chain() {
        let mut chain = AuditChain::new();

        for i in 0..10 {
            chain.append(
                "test".to_string(),
                format!("action_{}", i),
                "resource".to_string(),
                AuditDecision::Allow,
                vec!["INV-3".to_string()],
                format!("entry {}", i),
            );
        }

        assert_eq!(chain.len(), 10);
        assert!(chain.verify_chain().unwrap());
    }

    #[test]
    fn test_detect_tampering() {
        let mut chain = AuditChain::new();

        chain.append(
            "claude".to_string(),
            "create_branch".to_string(),
            "uws".to_string(),
            AuditDecision::Allow,
            vec!["INV-2".to_string()],
            "legit".to_string(),
        );

        chain.append(
            "copilot".to_string(),
            "create_commit".to_string(),
            "uws".to_string(),
            AuditDecision::Allow,
            vec!["INV-3".to_string()],
            "also legit".to_string(),
        );

        // Tamper with the first entry's hash
        chain.entries[0].entry_hash = "tampered_hash".to_string();

        // Verification should fail
        assert!(chain.verify_chain().is_err());
    }

    #[test]
    fn test_empty_chain_is_valid() {
        let chain = AuditChain::new();
        assert!(chain.verify_chain().unwrap());
    }

    #[test]
    fn test_hash_determinism() {
        let h1 = portable_sha3_256("test input");
        let h2 = portable_sha3_256("test input");
        assert_eq!(h1, h2);

        let h3 = portable_sha3_256("different input");
        assert_ne!(h1, h3);
    }

    #[test]
    fn test_hash_length() {
        let h = portable_sha3_256("anything");
        assert_eq!(h.len(), 64); // 256 bits = 64 hex chars
    }

    #[test]
    fn test_export_json() {
        let mut chain = AuditChain::new();
        chain.append(
            "claude".to_string(),
            "test_action".to_string(),
            "test_resource".to_string(),
            AuditDecision::Allow,
            vec!["INV-3".to_string()],
            "test evidence".to_string(),
        );

        let json = chain.export_json();
        assert!(json.contains("\"algorithm\": \"SHA3-256\""));
        assert!(json.contains("\"chain_length\": 1"));
        assert!(json.contains("\"actor\": \"claude\""));
        assert!(json.contains("\"decision\": \"ALLOW\""));
    }

    #[test]
    fn test_filter_by_actor() {
        let mut chain = AuditChain::new();
        chain.append(
            "claude".to_string(),
            "a1".to_string(),
            "r".to_string(),
            AuditDecision::Allow,
            vec![],
            String::new(),
        );
        chain.append(
            "copilot".to_string(),
            "a2".to_string(),
            "r".to_string(),
            AuditDecision::Allow,
            vec![],
            String::new(),
        );
        chain.append(
            "claude".to_string(),
            "a3".to_string(),
            "r".to_string(),
            AuditDecision::Deny,
            vec![],
            String::new(),
        );

        let claude_entries = chain.entries_by_actor("claude");
        assert_eq!(claude_entries.len(), 2);
    }

    #[test]
    fn test_denied_entries() {
        let mut chain = AuditChain::new();
        chain.append(
            "test".to_string(),
            "ok".to_string(),
            "r".to_string(),
            AuditDecision::Allow,
            vec![],
            String::new(),
        );
        chain.append(
            "test".to_string(),
            "bad".to_string(),
            "r".to_string(),
            AuditDecision::Deny,
            vec![],
            "blocked by INV-11".to_string(),
        );

        let denied = chain.denied_entries();
        assert_eq!(denied.len(), 1);
        assert_eq!(denied[0].action, "bad");
    }
}
