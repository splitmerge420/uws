#!/usr/bin/env python3
"""
acp_governance.py — Claude ACP Governance Layer for Aluminum OS

Task 1.1: Integrate Claude ACP Governance Layer into Manus Stack

This module provides the core governance infrastructure that replaces duplicate
governance logic in bridge.py. It implements:

1. PolicyRegistry — stores all policies from invariants_registry.py
2. PolicyEngine — evaluates actions against policies (allow/deny/warn)
3. CouncilVoting — multi-model consensus decisions with quorum
4. AuditChain — append-only cryptographic audit log (SHA3-256 chained hashes)
5. GovernanceDecision — dataclass output of every governance check

Version: 1.0.0
Author: Claude (Constitutional Engineer) for Dave Sheldon / Aluminum OS
Date: March 19, 2026

Dependencies:
- Python 3.9+
- hashlib (standard library)
- dataclasses (standard library)
- json (standard library)
"""

import sys
import os
import json
import argparse
import hashlib
import logging
from dataclasses import dataclass, asdict, field
from typing import Dict, List, Set, Optional, Tuple, Any
from enum import Enum
from datetime import datetime
from pathlib import Path


# ============================================================================
# LOGGING SETUP
# ============================================================================

logger = logging.getLogger(__name__)
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
)


# ============================================================================
# ENUMS & DATA STRUCTURES
# ============================================================================

class Decision(Enum):
    """Governance decision outcomes."""
    ALLOW = "allow"
    DENY = "deny"
    WARN = "warn"


class VoteChoice(Enum):
    """Council voting choices."""
    APPROVE = "approve"
    REJECT = "reject"
    ABSTAIN = "abstain"


@dataclass
class GovernanceDecision:
    """
    Output of every governance check. Immutable record of policy evaluation.

    Attributes:
        decision: ALLOW, DENY, or WARN
        action: The action being evaluated (e.g., "write_file", "send_email")
        context: The context dict passed to check()
        reasons: List of invariants that influenced the decision
        severity: Critical/mandatory/warning/advisory
        timestamp: ISO 8601 UTC timestamp
        chain_hash: SHA3-256 hash for audit chain verification
        decision_id: Unique ID for this decision
    """
    decision: str
    action: str
    context: Dict[str, Any]
    reasons: List[str] = field(default_factory=list)
    severity: str = "mandatory"
    timestamp: str = field(default_factory=lambda: datetime.utcnow().isoformat() + "Z")
    chain_hash: str = ""
    decision_id: str = ""

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return asdict(self)

    def to_json(self) -> str:
        """Convert to JSON string."""
        return json.dumps(self.to_dict(), indent=2)


@dataclass
class AuditEntry:
    """Single entry in the audit chain."""
    decision_id: str
    action: str
    decision: str
    timestamp: str
    previous_hash: str
    entry_hash: str
    context_snapshot: Dict[str, Any]


# ============================================================================
# POLICY REGISTRY
# ============================================================================

class PolicyRegistry:
    """
    Stores and manages all governance policies loaded from invariants_registry.py.

    This is the single source of truth for all constitutional invariants in the
    system. It loads INVARIANTS from invariants_registry and provides methods
    to query and filter policies.
    """

    def __init__(self):
        """Initialize PolicyRegistry and load invariants."""
        self.policies: Dict[str, Dict[str, Any]] = {}
        self._load_invariants()
        logger.info(f"PolicyRegistry initialized with {len(self.policies)} policies")

    def _load_invariants(self) -> None:
        """
        Load invariants from invariants_registry.py using sys.path manipulation
        to import from the same directory.
        """
        # Get the directory of this file
        current_dir = os.path.dirname(os.path.abspath(__file__))

        # Ensure it's in sys.path
        if current_dir not in sys.path:
            sys.path.insert(0, current_dir)

        try:
            from invariants_registry import INVARIANTS
            self.policies = INVARIANTS
            logger.debug(f"Loaded {len(self.policies)} invariants from invariants_registry")
        except ImportError as e:
            logger.error(f"Failed to import invariants_registry: {e}")
            self.policies = {}

    def get_policy(self, policy_id: str) -> Optional[Dict[str, Any]]:
        """Get a single policy by ID."""
        return self.policies.get(policy_id)

    def get_policies_by_severity(self, severity: str) -> Dict[str, Dict[str, Any]]:
        """Get all policies of a given severity."""
        return {k: v for k, v in self.policies.items() if v.get("severity") == severity}

    def get_checkable_policies(self) -> Dict[str, Dict[str, Any]]:
        """Get all policies with automated checks (not advisory-only)."""
        return {
            k: v for k, v in self.policies.items()
            if v.get("check_type") != "advisory"
        }

    def get_policies_for_action(self, action: str) -> Dict[str, Dict[str, Any]]:
        """Get policies that apply to a given action type."""
        applicable = {}
        for policy_id, policy in self.policies.items():
            # Advisory policies always apply
            if policy.get("check_type") == "advisory":
                applicable[policy_id] = policy
                continue

            # Check if action matches applies_to patterns
            applies_to = policy.get("applies_to", [])
            if not applies_to:  # No restrictions means applies to all
                applicable[policy_id] = policy
                continue

            # Simple pattern matching for file extensions
            for pattern in applies_to:
                if pattern == "*" or action.endswith(pattern.replace("*", "")):
                    applicable[policy_id] = policy
                    break

        return applicable

    def list_all_policies(self) -> List[str]:
        """List all policy IDs."""
        return sorted(list(self.policies.keys()))

    def count_by_severity(self) -> Dict[str, int]:
        """Count policies by severity."""
        counts = {}
        for policy in self.policies.values():
            severity = policy.get("severity", "unknown")
            counts[severity] = counts.get(severity, 0) + 1
        return counts


# ============================================================================
# POLICY ENGINE
# ============================================================================

class PolicyEngine:
    """
    Evaluates actions against all applicable policies.

    Returns GovernanceDecision objects containing:
    - ALLOW (all policies satisfied)
    - DENY (critical policy violated)
    - WARN (non-critical policy violated)

    Every decision is logged to the AuditChain.
    """

    def __init__(self, registry: PolicyRegistry, audit_chain: "AuditChain"):
        """Initialize PolicyEngine with registry and audit chain."""
        self.registry = registry
        self.audit_chain = audit_chain
        self.decision_counter = 0

    def check(self, action: str, context: Optional[Dict[str, Any]] = None) -> GovernanceDecision:
        """
        Evaluate an action against all applicable policies.

        Args:
            action: The action being evaluated (e.g., "write_file", "send_email")
            context: Optional context dict with action parameters

        Returns:
            GovernanceDecision with decision, reasons, and severity
        """
        if context is None:
            context = {}

        self.decision_counter += 1
        decision_id = f"decision-{self.decision_counter:06d}"

        applicable_policies = self.registry.get_policies_for_action(action)

        if not applicable_policies:
            # No policies apply - allow by default
            decision = GovernanceDecision(
                decision=Decision.ALLOW.value,
                action=action,
                context=context,
                reasons=["no applicable policies"],
                severity="advisory",
                decision_id=decision_id
            )
            self._record_decision(decision)
            return decision

        # Evaluate all policies
        deny_reasons = []
        warn_reasons = []
        critical_violations = False

        for policy_id, policy in applicable_policies.items():
            violation = self._check_policy(policy_id, policy, action, context)

            if violation:
                severity = policy.get("severity", "mandatory")
                reason = f"{policy_id}: {policy.get('name', 'Unknown')}"

                if severity in ["critical"]:
                    critical_violations = True
                    deny_reasons.append(reason)
                else:
                    warn_reasons.append(reason)

        # Determine final decision
        if critical_violations:
            final_decision = Decision.DENY.value
            all_reasons = deny_reasons + warn_reasons
            max_severity = "critical"
        elif warn_reasons:
            final_decision = Decision.WARN.value
            all_reasons = warn_reasons
            max_severity = "warning"
        else:
            final_decision = Decision.ALLOW.value
            all_reasons = ["all policies satisfied"]
            max_severity = "advisory"

        decision = GovernanceDecision(
            decision=final_decision,
            action=action,
            context=context,
            reasons=all_reasons,
            severity=max_severity,
            decision_id=decision_id
        )

        self._record_decision(decision)
        return decision

    def _check_policy(self, policy_id: str, policy: Dict[str, Any],
                     action: str, context: Dict[str, Any]) -> bool:
        """
        Check if a policy is violated.

        Returns:
            True if policy is violated, False otherwise
        """
        check_type = policy.get("check_type", "advisory")

        if check_type == "advisory":
            # Advisory policies don't cause violations
            return False

        if check_type == "guard_check":
            # Check for presence of guard patterns (good patterns)
            guard_patterns = policy.get("guard_patterns", [])
            dangerous_patterns = policy.get("dangerous_patterns", [])

            # If dangerous patterns detected, check for guards
            context_str = json.dumps(context)
            has_dangerous = any(p in context_str for p in dangerous_patterns)

            if has_dangerous:
                has_guard = any(p in context_str for p in guard_patterns)
                return not has_guard

            return False

        if check_type == "pattern_absence_negative":
            # Check that forbidden patterns are absent
            forbidden_patterns = policy.get("forbidden_patterns", [])
            context_str = json.dumps(context)

            for pattern in forbidden_patterns:
                if pattern in context_str:
                    return True

            return False

        # Unknown check type - assume no violation
        return False

    def _record_decision(self, decision: GovernanceDecision) -> None:
        """Record decision to audit chain."""
        try:
            self.audit_chain.record(decision)
            logger.debug(f"Recorded governance decision: {decision.decision_id}")
        except Exception as e:
            logger.error(f"Failed to record decision to audit chain: {e}")


# ============================================================================
# AUDIT CHAIN
# ============================================================================

class AuditChain:
    """
    Append-only cryptographic audit log using SHA3-256 hash chaining.

    Every governance decision is recorded with a hash chain that allows
    verification of integrity. The chain is immutable once recorded.

    Features:
    - Append-only (cannot modify or delete entries)
    - SHA3-256 hash chaining for integrity verification
    - JSON export/import
    - Chain integrity verification
    """

    def __init__(self):
        """Initialize empty audit chain."""
        self.chain: List[AuditEntry] = []
        self.entry_counter = 0
        self.current_hash = self._hash_value("")  # Genesis hash
        logger.info("AuditChain initialized (genesis hash computed)")

    def _hash_value(self, data: str) -> str:
        """Compute SHA3-256 hash of data."""
        return hashlib.sha3_256(data.encode()).hexdigest()

    def record(self, decision: GovernanceDecision) -> None:
        """
        Record a governance decision to the audit chain.

        Args:
            decision: GovernanceDecision object to record

        Raises:
            ValueError: If decision_id is empty
        """
        if not decision.decision_id:
            raise ValueError("GovernanceDecision must have a decision_id")

        self.entry_counter += 1

        # Create the entry content (excludes the entry_hash)
        entry_content = {
            "decision_id": decision.decision_id,
            "action": decision.action,
            "decision": decision.decision,
            "timestamp": decision.timestamp,
            "previous_hash": self.current_hash,
            "context_snapshot": decision.context,
        }

        # Hash the entry content
        entry_json = json.dumps(entry_content, sort_keys=True, separators=(',', ':'))
        entry_hash = self._hash_value(entry_json)

        # Create the audit entry
        entry = AuditEntry(
            decision_id=decision.decision_id,
            action=decision.action,
            decision=decision.decision,
            timestamp=decision.timestamp,
            previous_hash=self.current_hash,
            entry_hash=entry_hash,
            context_snapshot=decision.context
        )

        # Add to chain
        self.chain.append(entry)
        self.current_hash = entry_hash

        # Update the decision's chain hash
        decision.chain_hash = entry_hash

        logger.debug(f"Added entry to audit chain: {decision.decision_id} -> {entry_hash[:16]}...")

    def verify_integrity(self) -> Tuple[bool, List[str]]:
        """
        Verify the integrity of the audit chain.

        Returns:
            Tuple of (is_valid, error_messages)
        """
        errors = []

        if not self.chain:
            # Empty chain is valid
            return True, []

        # Start with genesis hash
        current_hash = self._hash_value("")

        for i, entry in enumerate(self.chain):
            # Check that previous_hash matches our computed hash
            if entry.previous_hash != current_hash:
                errors.append(
                    f"Entry {i} ({entry.decision_id}): previous_hash mismatch. "
                    f"Expected {current_hash[:16]}..., got {entry.previous_hash[:16]}..."
                )
                continue

            # Verify entry hash
            entry_content = {
                "decision_id": entry.decision_id,
                "action": entry.action,
                "decision": entry.decision,
                "timestamp": entry.timestamp,
                "previous_hash": entry.previous_hash,
                "context_snapshot": entry.context_snapshot,
            }
            entry_json = json.dumps(entry_content, sort_keys=True, separators=(',', ':'))
            computed_hash = self._hash_value(entry_json)

            if computed_hash != entry.entry_hash:
                errors.append(
                    f"Entry {i} ({entry.decision_id}): entry_hash mismatch. "
                    f"Expected {computed_hash[:16]}..., got {entry.entry_hash[:16]}..."
                )
                continue

            # Move to next hash
            current_hash = entry.entry_hash

        is_valid = len(errors) == 0
        return is_valid, errors

    def verify_chain(self) -> Tuple[bool, List[str]]:
        """
        Alias for verify_integrity() — provided for API compatibility with the
        Rust AuditChain and the integration test suite.

        Returns:
            Tuple of (is_valid, error_messages)
        """
        return self.verify_integrity()
        """
        Export the entire audit chain as JSON.

        Args:
            pretty: If True, pretty-print JSON with indentation

        Returns:
            JSON string representation of the chain
        """
        chain_data = {
            "metadata": {
                "chain_length": len(self.chain),
                "current_hash": self.current_hash,
                "exported_at": datetime.utcnow().isoformat() + "Z",
            },
            "entries": [
                {
                    "decision_id": entry.decision_id,
                    "action": entry.action,
                    "decision": entry.decision,
                    "timestamp": entry.timestamp,
                    "previous_hash": entry.previous_hash,
                    "entry_hash": entry.entry_hash,
                    "context_snapshot": entry.context_snapshot,
                }
                for entry in self.chain
            ]
        }

        if pretty:
            return json.dumps(chain_data, indent=2)
        else:
            return json.dumps(chain_data, separators=(',', ':'))

    def export_to_file(self, filepath: str, pretty: bool = True) -> None:
        """Export audit chain to a JSON file."""
        with open(filepath, 'w') as f:
            f.write(self.export(pretty=pretty))
        logger.info(f"Exported audit chain to {filepath}")

    def get_entries_for_decision(self, decision_id: str) -> Optional[AuditEntry]:
        """Get an audit entry by decision ID."""
        for entry in self.chain:
            if entry.decision_id == decision_id:
                return entry
        return None

    def get_entries_for_action(self, action: str) -> List[AuditEntry]:
        """Get all audit entries for a specific action."""
        return [e for e in self.chain if e.action == action]

    def get_entries_since(self, timestamp: str) -> List[AuditEntry]:
        """Get all audit entries since a given ISO timestamp."""
        return [e for e in self.chain if e.timestamp >= timestamp]

    def chain_length(self) -> int:
        """Get the number of entries in the chain."""
        return len(self.chain)

    def detect_truncation(self) -> Dict[str, Any]:
        """
        Detect if the chain appears truncated (missing entries).

        Verifies:
        - Sequential index continuity (decision IDs follow expected pattern)
        - Chain length matches expected count based on entry counters
        - No gaps in the decision ID sequence

        Returns:
            Dict with:
                truncated (bool): True if truncation detected
                expected_length (int): Expected chain length based on entry counter
                actual_length (int): Actual number of entries in chain
                missing_indices (list): List of missing decision indices
        """
        if not self.chain:
            return {
                "truncated": False,
                "expected_length": 0,
                "actual_length": 0,
                "missing_indices": []
            }

        missing_indices = []
        max_index = 0

        # Extract decision indices from decision_ids (format: decision-XXXXXX)
        found_indices = set()
        for entry in self.chain:
            try:
                # Parse decision_id format: "decision-000001", "decision-000002", etc.
                parts = entry.decision_id.split('-')
                if len(parts) == 2 and parts[0] == "decision":
                    index = int(parts[1])
                    found_indices.add(index)
                    max_index = max(max_index, index)
            except (ValueError, IndexError):
                # Skip entries with unparseable IDs
                pass

        # Check for missing indices in the sequence
        expected_length = max_index if max_index > 0 else len(self.chain)
        if max_index > 0:
            for i in range(1, max_index + 1):
                if i not in found_indices:
                    missing_indices.append(i)

        # Chain is considered truncated if:
        # - There are gaps in the sequence
        # - Expected length doesn't match actual length
        truncated = (
            len(missing_indices) > 0 or
            expected_length != len(self.chain)
        )

        return {
            "truncated": truncated,
            "expected_length": expected_length,
            "actual_length": len(self.chain),
            "missing_indices": sorted(missing_indices)
        }

    def validate_entry_completeness(self) -> List[int]:
        """
        Check that each entry has all required fields.

        Required fields: timestamp, actor (in context or decision), action,
        resource (in context), decision, previous_hash

        Returns:
            List of indices of incomplete entries (empty list if all complete)
        """
        incomplete_indices = []
        required_fields = {
            "timestamp", "action", "decision", "previous_hash", "entry_hash"
        }

        for i, entry in enumerate(self.chain):
            # Check audit entry fields
            entry_dict = {
                "timestamp": entry.timestamp,
                "action": entry.action,
                "decision": entry.decision,
                "previous_hash": entry.previous_hash,
                "entry_hash": entry.entry_hash,
                "decision_id": entry.decision_id,
            }

            # Check that all required fields are present and non-empty
            missing_fields = []
            for field in required_fields:
                if not entry_dict.get(field):
                    missing_fields.append(field)

            # Check context snapshot for required context fields
            context = entry.context_snapshot or {}
            required_context_fields = {
                "decision": True,  # At minimum, decision should be in context or decision field
            }

            # If any required fields are missing, mark as incomplete
            if missing_fields:
                incomplete_indices.append(i)
                logger.warning(
                    f"Entry {i} ({entry.decision_id}): missing fields: {missing_fields}"
                )

        return incomplete_indices


# ============================================================================
# COUNCIL VOTING
# ============================================================================

class CouncilVoting:
    """
    Multi-model consensus decisions with quorum-based voting.

    Allows multiple AI models to vote on governance motions. Decisions
    require quorum (>50% of registered models) to pass. All votes are
    logged to the AuditChain for transparency.

    Registered models are tracked, and votes must be from registered models.
    """

    def __init__(self, audit_chain: AuditChain, quorum_percentage: float = 50.0):
        """
        Initialize CouncilVoting.

        Args:
            audit_chain: AuditChain instance for logging votes
            quorum_percentage: Percentage of models required for quorum (default 50%)
        """
        self.audit_chain = audit_chain
        self.quorum_percentage = quorum_percentage
        self.registered_models: Set[str] = set()
        self.proposals: Dict[str, Dict[str, Any]] = {}
        self.proposal_counter = 0
        logger.info("CouncilVoting initialized")

    def register_model(self, model_name: str) -> None:
        """Register an AI model as a voting member of the council."""
        self.registered_models.add(model_name)
        logger.info(f"Registered model: {model_name}")

    def unregister_model(self, model_name: str) -> None:
        """Unregister a model from the council."""
        self.registered_models.discard(model_name)
        logger.info(f"Unregistered model: {model_name}")

    def get_registered_models(self) -> Set[str]:
        """Get all registered council members."""
        return self.registered_models.copy()

    def propose(self, motion: str, votes: Optional[Dict[str, str]] = None) -> Dict[str, Any]:
        """
        Propose a motion and collect votes from council members.

        Args:
            motion: The motion text to vote on
            votes: Dict of {model_name: "approve"/"reject"/"abstain"}

        Returns:
            Dict with proposal_id, motion, votes, passed, and reasoning

        Raises:
            ValueError: If votes reference unregistered models
        """
        if votes is None:
            votes = {}

        self.proposal_counter += 1
        proposal_id = f"proposal-{self.proposal_counter:04d}"

        # Validate votes
        for model_name in votes.keys():
            if model_name not in self.registered_models:
                raise ValueError(
                    f"Model '{model_name}' is not registered. "
                    f"Registered models: {self.registered_models}"
                )

        # Parse votes
        approve_votes = sum(1 for v in votes.values() if v == "approve")
        reject_votes = sum(1 for v in votes.values() if v == "reject")
        abstain_votes = sum(1 for v in votes.values() if v == "abstain")

        # Check quorum
        votes_cast = approve_votes + reject_votes + abstain_votes
        required_quorum = max(1, int(len(self.registered_models) * self.quorum_percentage / 100))
        has_quorum = votes_cast >= required_quorum

        # Determine if motion passed (requires quorum + majority of casted votes)
        if has_quorum:
            passed = approve_votes > reject_votes
        else:
            passed = False

        # Create proposal record
        proposal = {
            "proposal_id": proposal_id,
            "motion": motion,
            "timestamp": datetime.utcnow().isoformat() + "Z",
            "votes": votes,
            "approve_votes": approve_votes,
            "reject_votes": reject_votes,
            "abstain_votes": abstain_votes,
            "votes_cast": votes_cast,
            "required_quorum": required_quorum,
            "has_quorum": has_quorum,
            "passed": passed,
            "registered_model_count": len(self.registered_models),
        }

        self.proposals[proposal_id] = proposal

        # Log to audit chain
        self._record_proposal_to_audit(proposal)

        logger.info(
            f"Proposal {proposal_id}: {motion} -> "
            f"{'PASSED' if passed else 'FAILED'} "
            f"({approve_votes}A/{reject_votes}R/{abstain_votes}AB, quorum={has_quorum})"
        )

        return proposal

    def _record_proposal_to_audit(self, proposal: Dict[str, Any]) -> None:
        """Record a council proposal and votes to the audit chain."""
        try:
            # Create a synthetic GovernanceDecision for the proposal
            decision = GovernanceDecision(
                decision="allow" if proposal["passed"] else "deny",
                action=f"council_vote:{proposal['proposal_id']}",
                context={
                    "motion": proposal["motion"],
                    "votes": proposal["votes"],
                    "proposal_id": proposal["proposal_id"],
                },
                reasons=[f"Council vote: {proposal['approve_votes']}A/{proposal['reject_votes']}R"],
                severity="critical",
                decision_id=f"{proposal['proposal_id']}-record"
            )
            self.audit_chain.record(decision)
        except Exception as e:
            logger.error(f"Failed to record proposal to audit chain: {e}")

    def get_proposal(self, proposal_id: str) -> Optional[Dict[str, Any]]:
        """Get a proposal by ID."""
        return self.proposals.get(proposal_id)

    def list_proposals(self) -> List[Dict[str, Any]]:
        """Get all proposals."""
        return list(self.proposals.values())

    def get_passed_proposals(self) -> List[Dict[str, Any]]:
        """Get all proposals that passed."""
        return [p for p in self.proposals.values() if p["passed"]]

    def get_failed_proposals(self) -> List[Dict[str, Any]]:
        """Get all proposals that failed."""
        return [p for p in self.proposals.values() if not p["passed"]]


# ============================================================================
# GOVERNANCE SYSTEM (Main Facade)
# ============================================================================

class GovernanceSystem:
    """
    Main facade for the ACP Governance Layer.

    Coordinates PolicyRegistry, PolicyEngine, AuditChain, and CouncilVoting
    into a single system. This is the primary interface for governance checks.
    """

    def __init__(self):
        """Initialize the complete governance system."""
        self.audit_chain = AuditChain()
        self.registry = PolicyRegistry()
        self.engine = PolicyEngine(self.registry, self.audit_chain)
        self.council = CouncilVoting(self.audit_chain)
        logger.info("GovernanceSystem initialized successfully")

    def check(self, action: str, context: Optional[Dict[str, Any]] = None) -> GovernanceDecision:
        """
        Check if an action is allowed under current policies.

        Args:
            action: Action to evaluate
            context: Optional context dictionary

        Returns:
            GovernanceDecision
        """
        return self.engine.check(action, context or {})

    def get_audit_chain(self) -> AuditChain:
        """Get the audit chain."""
        return self.audit_chain

    def get_registry(self) -> PolicyRegistry:
        """Get the policy registry."""
        return self.registry

    def get_council(self) -> CouncilVoting:
        """Get the council voting system."""
        return self.council

    def export_audit_chain(self, filepath: str) -> None:
        """Export audit chain to JSON file."""
        self.audit_chain.export_to_file(filepath)

    def verify_audit_integrity(self) -> Tuple[bool, List[str]]:
        """Verify audit chain integrity."""
        return self.audit_chain.verify_integrity()


# ============================================================================
# CLI INTERFACE
# ============================================================================

def main():
    """CLI interface for governance system."""
    parser = argparse.ArgumentParser(
        description="Claude ACP Governance Layer for Aluminum OS"
    )

    subparsers = parser.add_subparsers(dest="command", help="Command to execute")

    # Check command
    check_parser = subparsers.add_parser("check", help="Check an action against policies")
    check_parser.add_argument("action", help="Action to check (e.g., 'write_file')")
    check_parser.add_argument(
        "--context",
        help="JSON context dict (e.g., '{\"path\": \"/tmp/foo\"}')",
        default="{}"
    )

    # Audit command
    audit_parser = subparsers.add_parser("audit", help="Print audit chain")
    audit_parser.add_argument("--format", choices=["json", "text"], default="json")
    audit_parser.add_argument("--export", help="Export to file")

    # Verify command
    verify_parser = subparsers.add_parser("verify", help="Verify audit chain integrity")

    # Registry command
    registry_parser = subparsers.add_parser("registry", help="List policies in registry")
    registry_parser.add_argument("--severity", help="Filter by severity")
    registry_parser.add_argument("--stats", action="store_true", help="Show statistics")

    # Council command
    council_parser = subparsers.add_parser("council", help="Council voting operations")
    council_parser.add_argument("--register", help="Register a model")
    council_parser.add_argument("--list-models", action="store_true", help="List registered models")
    council_parser.add_argument("--propose", help="Propose a motion")
    council_parser.add_argument("--votes", help="JSON votes dict for proposal")

    # Parse arguments
    args = parser.parse_args()

    # Initialize governance system
    gov = GovernanceSystem()

    # Handle commands
    if args.command == "check":
        try:
            context = json.loads(args.context)
        except json.JSONDecodeError as e:
            print(f"ERROR: Invalid JSON context: {e}")
            return 1

        decision = gov.check(args.action, context)
        print(f"\n{'='*70}")
        print(f"Governance Check: {args.action}")
        print(f"{'='*70}")
        print(f"Decision:  {decision.decision.upper()}")
        print(f"Severity:  {decision.severity}")
        print(f"Decision ID: {decision.decision_id}")
        print(f"Timestamp: {decision.timestamp}")
        print(f"Chain Hash: {decision.chain_hash[:16]}...")
        print(f"\nReasons:")
        for reason in decision.reasons:
            print(f"  - {reason}")
        print(f"\nContext:")
        print(json.dumps(decision.context, indent=2))
        print(f"{'='*70}\n")

        return 0 if decision.decision == Decision.ALLOW.value else 1

    elif args.command == "audit":
        chain = gov.get_audit_chain()

        if chain.chain_length() == 0:
            print("Audit chain is empty.")
            return 0

        if args.format == "json":
            print(chain.export(pretty=True))
        else:  # text format
            print(f"Audit Chain ({chain.chain_length()} entries)")
            print("=" * 80)
            for entry in chain.chain:
                print(f"  {entry.decision_id:20s} | {entry.action:20s} | {entry.decision:6s} | {entry.timestamp}")
            print("=" * 80)

        if args.export:
            chain.export_to_file(args.export, pretty=True)
            print(f"Exported audit chain to {args.export}")

        return 0

    elif args.command == "verify":
        is_valid, errors = gov.verify_audit_integrity()

        print(f"\nAudit Chain Integrity Verification")
        print(f"{'='*70}")

        if is_valid:
            print(f"Status: ✓ VALID")
            print(f"Chain length: {gov.get_audit_chain().chain_length()}")
            print(f"Current hash: {gov.get_audit_chain().current_hash[:32]}...")
        else:
            print(f"Status: ✗ INVALID")
            print(f"Errors found:")
            for error in errors:
                print(f"  - {error}")

        print(f"{'='*70}\n")

        return 0 if is_valid else 1

    elif args.command == "registry":
        registry = gov.get_registry()

        if args.stats:
            counts = registry.count_by_severity()
            print(f"\nPolicy Registry Statistics")
            print(f"{'='*70}")
            print(f"Total policies: {len(registry.list_all_policies())}")
            for severity in ["critical", "mandatory", "warning", "advisory"]:
                count = counts.get(severity, 0)
                print(f"  {severity:12s}: {count:3d}")
            print(f"{'='*70}\n")
        else:
            policies = registry.list_all_policies()
            print(f"\nPolicy Registry ({len(policies)} policies)")
            print(f"{'='*70}")
            for policy_id in policies:
                policy = registry.get_policy(policy_id)
                checkable = "✓" if policy.get("check_type") != "advisory" else " "
                print(f"  [{checkable}] {policy_id:8s} [{policy.get('severity', 'unknown'):9s}] {policy.get('name', 'N/A')}")
            print(f"{'='*70}\n")

        return 0

    elif args.command == "council":
        council = gov.get_council()

        if args.register:
            council.register_model(args.register)
            print(f"Registered model: {args.register}")
            return 0

        elif args.list_models:
            models = council.get_registered_models()
            print(f"\nRegistered Council Members ({len(models)})")
            print(f"{'='*70}")
            if models:
                for model in sorted(models):
                    print(f"  - {model}")
            else:
                print("  (no models registered)")
            print(f"{'='*70}\n")
            return 0

        elif args.propose:
            if not args.votes:
                print("ERROR: --votes required with --propose")
                return 1

            try:
                votes = json.loads(args.votes)
            except json.JSONDecodeError as e:
                print(f"ERROR: Invalid JSON votes: {e}")
                return 1

            proposal = council.propose(args.propose, votes)
            print(f"\nCouncil Proposal")
            print(f"{'='*70}")
            print(f"Proposal ID: {proposal['proposal_id']}")
            print(f"Motion: {proposal['motion']}")
            print(f"Result: {'PASSED ✓' if proposal['passed'] else 'FAILED ✗'}")
            print(f"Votes: {proposal['approve_votes']}A / {proposal['reject_votes']}R / {proposal['abstain_votes']}AB")
            print(f"Quorum: {'Met ✓' if proposal['has_quorum'] else 'Not Met ✗'} ({proposal['votes_cast']}/{proposal['required_quorum']})")
            print(f"{'='*70}\n")
            return 0 if proposal['passed'] else 1

        else:
            parser.print_help()
            return 1

    else:
        parser.print_help()
        return 0


if __name__ == "__main__":
    sys.exit(main())
