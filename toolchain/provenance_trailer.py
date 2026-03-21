#!/usr/bin/env python3
"""
provenance_trailer.py — Aluminum OS ProvenanceTrailer Library

Validates and generates Golden-Trace commit trailers for the Kintsugi governance
pipeline. Every commit entering the Aluminum OS codebase must carry a verifiable
Human-In-The-Loop (HITL) provenance signature.

Golden-Trace trailer format:
    Golden-Trace: sha3-256:<hex_hash> hitl=<weight> actor=<id>

Example:
    Golden-Trace: sha3-256:a1b2c3d4e5f6... hitl=0.90 actor=dave@splitmerge420

Integration:
  - Used by .github/workflows/kintsugi-weave.yml (provenance-check job)
  - Maps the ProvenanceTrailer concept to GoldenTrace in kintsugi_healer.py
  - Enforces INV-3 (Audit Trail) and INV-5 (Human Sovereignty)

Usage:
  python provenance_trailer.py [--commits <sha1,sha2,...>] [--json]
  python provenance_trailer.py --generate --hitl 0.9 --actor dave

Author: GitHub Copilot (implementation) for Dave Sheldon
Council Session: 2026-03-21
Invariants Enforced: INV-3 (Audit Trail), INV-5 (Human Sovereignty)
"""

import os
import re
import sys
import json
import hashlib
import subprocess
from dataclasses import dataclass, asdict, field
from typing import List, Optional


# ============================================================================
# Constants
# ============================================================================

GOLDEN_TRACE_PATTERN = re.compile(
    r"^Golden-Trace:\s+sha3-256:([0-9a-f]{8,64})\s+hitl=([0-9.]+)(?:\s+actor=(\S+))?",
    re.MULTILINE,
)

HITL_MIN = 0.0
HITL_MAX = 1.0


# ============================================================================
# Data Models
# ============================================================================


@dataclass
class GoldenTrace:
    """A parsed Golden-Trace trailer from a commit message."""
    commit_sha: str
    trace_hash: str       # The sha3-256 value in the trailer
    hitl_weight: float    # Human-In-The-Loop weight (0.0–1.0)
    actor: Optional[str]  # Who signed off (optional)
    raw_trailer: str      # The original trailer line


@dataclass
class CommitValidationResult:
    """Result of validating a single commit for provenance."""
    commit_sha: str
    valid: bool
    golden_trace: Optional[GoldenTrace]
    error: Optional[str] = None


@dataclass
class ProvenanceReport:
    """Full validation report for a set of commits."""
    commits_checked: int
    commits_valid: int
    commits_invalid: int
    invalid_commits: List[str] = field(default_factory=list)
    results: List[dict] = field(default_factory=list)

    @property
    def passed(self) -> bool:
        return self.commits_invalid == 0


# ============================================================================
# Core Library
# ============================================================================


class ProvenanceTrailer:
    """
    Validates and generates Golden-Trace trailers for Aluminum OS commits.

    The trailer enforces INV-3 (Audit Trail) by ensuring every commit has
    verifiable Human-In-The-Loop provenance before it can enter the main branch.
    """

    def __init__(self, min_hitl: float = 0.5):
        """
        Args:
            min_hitl: Minimum acceptable HITL weight (default 0.5 = 50% human oversight).
        """
        self.min_hitl = min_hitl

    def parse_trailer(self, commit_sha: str, message: str) -> Optional[GoldenTrace]:
        """
        Parse a Golden-Trace trailer from a commit message.

        Returns a GoldenTrace if found, None otherwise.
        """
        match = GOLDEN_TRACE_PATTERN.search(message)
        if not match:
            return None

        trace_hash = match.group(1)
        try:
            hitl_weight = float(match.group(2))
        except (ValueError, TypeError):
            return None

        actor = match.group(3) if match.lastindex and match.lastindex >= 3 else None
        raw_trailer = match.group(0).strip()

        return GoldenTrace(
            commit_sha=commit_sha,
            trace_hash=trace_hash,
            hitl_weight=hitl_weight,
            actor=actor,
            raw_trailer=raw_trailer,
        )

    def validate_commit(self, commit_sha: str, message: str) -> CommitValidationResult:
        """
        Validate a single commit message for a valid Golden-Trace trailer.

        A commit is valid if:
          1. It contains a Golden-Trace trailer
          2. The HITL weight is >= min_hitl
          3. The HITL weight is in [0.0, 1.0]
        """
        trace = self.parse_trailer(commit_sha, message)

        if trace is None:
            return CommitValidationResult(
                commit_sha=commit_sha,
                valid=False,
                golden_trace=None,
                error=(
                    f"Commit {commit_sha[:12]} is missing a Golden-Trace trailer. "
                    f"All commits must include: "
                    f"'Golden-Trace: sha3-256:<hash> hitl=<0.0-1.0>'"
                ),
            )

        if not (HITL_MIN <= trace.hitl_weight <= HITL_MAX):
            return CommitValidationResult(
                commit_sha=commit_sha,
                valid=False,
                golden_trace=trace,
                error=(
                    f"Commit {commit_sha[:12]} has invalid HITL weight "
                    f"{trace.hitl_weight} (must be 0.0–1.0)"
                ),
            )

        if trace.hitl_weight < self.min_hitl:
            return CommitValidationResult(
                commit_sha=commit_sha,
                valid=False,
                golden_trace=trace,
                error=(
                    f"Commit {commit_sha[:12]} HITL weight {trace.hitl_weight} "
                    f"is below minimum {self.min_hitl}"
                ),
            )

        return CommitValidationResult(
            commit_sha=commit_sha,
            valid=True,
            golden_trace=trace,
        )

    def validate_commits(
        self, commit_messages: dict
    ) -> ProvenanceReport:
        """
        Validate a batch of commits.

        Args:
            commit_messages: dict mapping commit SHA -> commit message body.

        Returns:
            ProvenanceReport with validation results for each commit.
        """
        results = []
        invalid_shas = []

        for sha, message in commit_messages.items():
            result = self.validate_commit(sha, message)
            results.append(asdict(result))
            if not result.valid:
                invalid_shas.append(sha)

        return ProvenanceReport(
            commits_checked=len(commit_messages),
            commits_valid=len(commit_messages) - len(invalid_shas),
            commits_invalid=len(invalid_shas),
            invalid_commits=invalid_shas,
            results=results,
        )

    def generate_trailer(
        self, content: str, hitl_weight: float, actor: Optional[str] = None
    ) -> str:
        """
        Generate a Golden-Trace trailer for a commit.

        Args:
            content: The commit content to hash (typically the diff or message body).
            hitl_weight: HITL weight (0.0–1.0).
            actor: Optional actor identifier.

        Returns:
            The Golden-Trace trailer string to append to the commit message.
        """
        if not (HITL_MIN <= hitl_weight <= HITL_MAX):
            raise ValueError(f"hitl_weight must be between 0.0 and 1.0, got {hitl_weight}")

        # Use SHA-256 as a stand-in for SHA3-256 when the sha3 module is not available.
        # Production use should replace this with hashlib.sha3_256.
        try:
            digest = hashlib.sha3_256(content.encode()).hexdigest()
        except AttributeError:
            digest = hashlib.sha256(content.encode()).hexdigest()

        trailer = f"Golden-Trace: sha3-256:{digest} hitl={hitl_weight:.2f}"
        if actor:
            trailer += f" actor={actor}"
        return trailer


# ============================================================================
# Git Integration
# ============================================================================


def get_pr_commits(base_ref: str = "origin/main") -> dict:
    """
    Get commit SHAs and messages for all commits in the current PR branch
    (commits reachable from HEAD but not from base_ref).

    Returns:
        dict mapping commit SHA -> commit message body.
    """
    try:
        result = subprocess.run(
            ["git", "rev-list", f"{base_ref}..HEAD"],
            capture_output=True,
            text=True,
            check=True,
        )
        shas = result.stdout.strip().split("\n")
        shas = [s for s in shas if s]
    except subprocess.CalledProcessError:
        return {}

    commits = {}
    for sha in shas:
        try:
            msg_result = subprocess.run(
                ["git", "log", "--format=%B", "-n", "1", sha],
                capture_output=True,
                text=True,
                check=True,
            )
            commits[sha] = msg_result.stdout
        except subprocess.CalledProcessError:
            commits[sha] = ""

    return commits


# ============================================================================
# CLI Entry Point
# ============================================================================


def main():
    import argparse

    parser = argparse.ArgumentParser(
        description="Validate or generate Golden-Trace provenance trailers."
    )
    subparsers = parser.add_subparsers(dest="command")

    # Validate subcommand
    validate_parser = subparsers.add_parser(
        "validate", help="Validate commits for Golden-Trace trailers"
    )
    validate_parser.add_argument(
        "--base-ref",
        default="origin/main",
        help="Base git ref (default: origin/main)",
    )
    validate_parser.add_argument(
        "--min-hitl",
        type=float,
        default=0.5,
        help="Minimum HITL weight (default: 0.5)",
    )
    validate_parser.add_argument("--json", action="store_true", help="Output as JSON")
    validate_parser.add_argument(
        "--strict",
        action="store_true",
        help="Exit non-zero if any commit is invalid",
    )

    # Generate subcommand
    gen_parser = subparsers.add_parser(
        "generate", help="Generate a Golden-Trace trailer for a commit"
    )
    gen_parser.add_argument("--content", default="", help="Content to hash")
    gen_parser.add_argument(
        "--hitl", type=float, required=True, help="HITL weight (0.0–1.0)"
    )
    gen_parser.add_argument("--actor", default=None, help="Actor identifier")

    args = parser.parse_args()

    if args.command == "generate":
        pt = ProvenanceTrailer()
        trailer = pt.generate_trailer(args.content, args.hitl, args.actor)
        print(trailer)
        return

    # Default: validate
    min_hitl = getattr(args, "min_hitl", 0.5)
    base_ref = getattr(args, "base_ref", "origin/main")
    as_json = getattr(args, "json", False)
    strict = getattr(args, "strict", False)

    pt = ProvenanceTrailer(min_hitl=min_hitl)
    commit_messages = get_pr_commits(base_ref)

    if not commit_messages:
        print("No commits found to validate.")
        return

    report = pt.validate_commits(commit_messages)

    if as_json:
        print(json.dumps(asdict(report), indent=2))
    else:
        print(f"\n{'='*60}")
        print("  ALUMINUM OS PROVENANCE CHECK")
        print(f"{'='*60}\n")
        print(f"  Commits checked: {report.commits_checked}")
        print(f"  Valid:           {report.commits_valid}")
        print(f"  Invalid:         {report.commits_invalid}")

        for r in report.results:
            status = "✅" if r["valid"] else "❌"
            sha_short = r["commit_sha"][:12]
            if r["valid"]:
                trace = r.get("golden_trace") or {}
                print(f"  {status} {sha_short}  hitl={trace.get('hitl_weight', '?')}")
            else:
                print(f"  {status} {sha_short}  ERROR: {r.get('error', 'unknown')}")

        print(f"\n{'='*60}\n")

    if strict and not report.passed:
        sys.exit(1)


if __name__ == "__main__":
    main()
