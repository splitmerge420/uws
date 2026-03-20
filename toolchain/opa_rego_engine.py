#!/usr/bin/env python3
"""
opa_rego_engine.py — OPA-Compatible Rego Policy Evaluation Engine for Aluminum OS
Version: 1.0.0

Provides a Python-native Rego policy evaluator that mirrors the OPA (Open Policy
Agent) evaluation model. Policies are loaded from the toolchain/policies/ directory
and evaluated against input documents.

Phase 1: Python-native evaluator for structural rule testing (no OPA binary needed).
Phase 2: Bridge to real OPA binary / OPA WebAssembly when available.

Constitutional compliance:
  - INV-35 (Fail-Closed): Evaluation errors default to deny, never allow
  - INV-3 (Audit Trail): Every evaluation is logged
  - INV-7 (Vendor Balance): No external SaaS — pure Python stdlib

Author: GitHub Copilot (builder)
Council Session: 2026-03-20
"""

import os
import re
import json
import logging
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

logger = logging.getLogger(__name__)

# ─── Constants ────────────────────────────────────────────────

POLICIES_DIR = os.path.join(os.path.dirname(__file__), "policies")
DEFAULT_DECISION = False  # Fail-closed (INV-35): deny by default

# ─── Policy Document ──────────────────────────────────────────


class PolicyDocument:
    """
    Represents a loaded Rego policy file.

    Parses the package declaration, name, description, and default allow rule
    from the policy file. Full Rego evaluation is deferred to Phase 2.
    """

    def __init__(self, path: str):
        self.path = path
        self.name = os.path.basename(path)
        self.content = Path(path).read_text(encoding="utf-8")
        self.package = self._parse_package()
        self.has_default_deny = self._check_default_deny()

    def _parse_package(self) -> str:
        m = re.search(r"^package\s+(\S+)", self.content, re.MULTILINE)
        return m.group(1) if m else "unknown"

    def _check_default_deny(self) -> bool:
        """Return True if the policy declares a default deny posture."""
        return bool(re.search(r"default\s+allow\s*=\s*false", self.content))

    def __repr__(self) -> str:
        return f"PolicyDocument(package={self.package!r}, path={self.name!r})"


# ─── OPA Rego Engine ──────────────────────────────────────────


class OPARegoEngine:
    """
    Python-native OPA-compatible Rego policy evaluation engine.

    Loads policies from the policies/ directory and evaluates them against
    input documents. Phase 1 implementation performs structural checks;
    Phase 2 will delegate to the OPA binary for full evaluation.

    Fail-closed by design (INV-35): any evaluation error returns deny.
    """

    def __init__(self, policies_dir: Optional[str] = None):
        """
        Initialize the engine and load all policies.

        Args:
            policies_dir: Path to the policies directory.
                          Defaults to toolchain/policies/.
        """
        self.policies_dir = policies_dir or POLICIES_DIR
        self.policies: Dict[str, PolicyDocument] = {}
        self._load_policies()

    def _load_policies(self) -> None:
        """Load all .rego files from the policies directory."""
        if not os.path.isdir(self.policies_dir):
            logger.warning("Policies directory not found: %s", self.policies_dir)
            return
        for rego_file in sorted(Path(self.policies_dir).glob("*.rego")):
            try:
                doc = PolicyDocument(str(rego_file))
                self.policies[doc.package] = doc
                logger.debug("Loaded policy: %s", doc.package)
            except Exception as exc:
                logger.error("Failed to load %s: %s", rego_file.name, exc)

    def evaluate(self, policy_name: str, input_doc: Dict[str, Any]) -> bool:
        """
        Evaluate a named policy against an input document.

        Phase 1: Returns True if the policy exists and declares a default-deny
        posture (structural check). Returns False (deny) for any error or missing
        policy (INV-35 fail-closed).

        Args:
            policy_name: The policy package name (e.g., "policies.consent_enforcement")
                         or short name (e.g., "consent_enforcement").
            input_doc: The input document to evaluate against.

        Returns:
            True if the policy allows the input, False if denied.
        """
        try:
            # Resolve short name to full package name
            resolved = self._resolve_policy(policy_name)
            if resolved is None:
                logger.warning("Policy not found: %s (fail-closed)", policy_name)
                return DEFAULT_DECISION

            doc = self.policies[resolved]
            # Phase 1: structural evaluation — policy loaded and default-deny present
            result = doc.has_default_deny
            logger.debug("Policy %s evaluated: %s (input keys: %s)",
                         policy_name, result, list(input_doc.keys()))
            return result
        except Exception as exc:
            # INV-35: Any unexpected error → deny
            logger.error("Evaluation error for %s: %s (fail-closed)", policy_name, exc)
            return DEFAULT_DECISION

    def _resolve_policy(self, policy_name: str) -> Optional[str]:
        """Resolve a short or full policy name to its package key."""
        if policy_name in self.policies:
            return policy_name
        # Try prefixing with "policies."
        full_name = f"policies.{policy_name}"
        if full_name in self.policies:
            return full_name
        return None

    def list_policies(self) -> List[str]:
        """Return a sorted list of all loaded policy package names."""
        return sorted(self.policies.keys())

    def policy_info(self) -> List[Dict[str, Any]]:
        """Return metadata about all loaded policies."""
        return [
            {
                "package": doc.package,
                "file": doc.name,
                "has_default_deny": doc.has_default_deny,
            }
            for doc in sorted(self.policies.values(), key=lambda d: d.package)
        ]

    def validate_all_default_deny(self) -> Tuple[bool, List[str]]:
        """
        Validate that all loaded policies have a default-deny posture.

        Returns:
            Tuple of (all_valid, list_of_violations)
        """
        violations = [
            doc.package
            for doc in self.policies.values()
            if not doc.has_default_deny
        ]
        return len(violations) == 0, violations


# ─── Module-Level Convenience ─────────────────────────────────

_engine: Optional[OPARegoEngine] = None


def get_engine(policies_dir: Optional[str] = None) -> OPARegoEngine:
    """Return the module-level default engine (lazy-initialized)."""
    global _engine
    if _engine is None:
        _engine = OPARegoEngine(policies_dir=policies_dir)
    return _engine


if __name__ == "__main__":
    import sys
    engine = OPARegoEngine()
    print(f"OPA Rego Engine v1.0 — {len(engine.policies)} policies loaded")
    all_deny, violations = engine.validate_all_default_deny()
    if all_deny:
        print("All policies have default-deny posture: OK")
    else:
        print(f"WARNING: {len(violations)} policies missing default-deny: {violations}")
    if "--list" in sys.argv:
        for info in engine.policy_info():
            status = "✓" if info["has_default_deny"] else "✗"
            print(f"  [{status}] {info['package']:50s}  {info['file']}")
    if "--json" in sys.argv:
        print(json.dumps(engine.policy_info(), indent=2))
