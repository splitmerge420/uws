#!/usr/bin/env python3
"""
provenance.py — Aluminum OS GoldenTrace Provenance Library

Provides:
  - GoldenTrace: data class representing a provenance record
  - ProvenanceTrailer: formats/parses Golden-Trace git commit trailers
  - validate_commits(): validate a list of commit messages for HITL provenance
  - CLI interface: check-commits, format-trailer, verify

Golden-Trace trailer format (RFC 5322 / git trailer convention):
  Golden-Trace: sha3-256:<64-hex>; HITL=<0.00–1.00>; provider=<name>; npfm=<0.00–1.00>; ts=<ISO-8601>

Examples:
  python provenance.py check-commits HEAD~5..HEAD
  python provenance.py format-trailer --hitl 0.90 --provider Claude --npfm 0.85
  python provenance.py verify <digest>

Author: Aluminum OS / uws project
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import sys
from dataclasses import dataclass, asdict
from datetime import datetime, timezone
from typing import List, Optional, Tuple

# ─── Constants ────────────────────────────────────────────────

TRAILER_KEY = "Golden-Trace"
DIGEST_PREFIX = "sha3-256:"
HITL_MIN = 0.0
HITL_MAX = 1.0
NPFM_THRESHOLD = 0.7

# Regex matching the full Golden-Trace trailer value
_TRAILER_RE = re.compile(
    r"sha3-256:(?P<digest>[0-9a-f]{64})"
    r";\s*HITL=(?P<hitl>\d+\.\d+)"
    r"(?:;\s*provider=(?P<provider>[^;]+))?"
    r"(?:;\s*npfm=(?P<npfm>\d+\.\d+))?"
    r"(?:;\s*ts=(?P<ts>[^;]+))?",
    re.IGNORECASE,
)


# ─── Data models ──────────────────────────────────────────────

@dataclass
class GoldenTrace:
    """A parsed GoldenTrace provenance record."""

    digest: str          # 64-hex SHA3-256 string (without prefix)
    hitl_weight: float   # 0.0–1.0 Human-In-The-Loop weight
    provider: str        # AI provider name
    npfm_score: float    # 0.0–1.0 Net-Positive Flourishing Metric
    timestamp: str       # ISO-8601 UTC timestamp

    def to_trailer_string(self) -> str:
        """Format as a git commit trailer line."""
        return (
            f"{TRAILER_KEY}: {DIGEST_PREFIX}{self.digest}; "
            f"HITL={self.hitl_weight:.2f}; "
            f"provider={self.provider}; "
            f"npfm={self.npfm_score:.2f}; "
            f"ts={self.timestamp}"
        )

    def to_dict(self) -> dict:
        return asdict(self)

    def to_json(self) -> str:
        return json.dumps(self.to_dict(), indent=2)


@dataclass
class CommitValidationResult:
    """Result of validating a single commit's provenance."""

    sha: str
    subject: str
    valid: bool
    trace: Optional[GoldenTrace]
    error: Optional[str]

    def to_dict(self) -> dict:
        d = {
            "sha": self.sha,
            "subject": self.subject,
            "valid": self.valid,
            "trace": self.trace.to_dict() if self.trace else None,
            "error": self.error,
        }
        return d


# ─── Core library ─────────────────────────────────────────────

class ProvenanceTrailer:
    """Parse and format Golden-Trace git commit trailers."""

    @staticmethod
    def parse(message: str) -> Optional[GoldenTrace]:
        """Extract and parse a Golden-Trace trailer from a commit message.

        Returns ``None`` if no valid trailer is found.
        """
        for line in message.splitlines():
            line = line.strip()
            if not line.lower().startswith(TRAILER_KEY.lower()):
                continue
            # Strip the key prefix
            _, _, value = line.partition(":")
            value = value.strip()
            m = _TRAILER_RE.match(value)
            if not m:
                continue
            try:
                return GoldenTrace(
                    digest=m.group("digest"),
                    hitl_weight=float(m.group("hitl")),
                    provider=(m.group("provider") or "unknown").strip(),
                    npfm_score=float(m.group("npfm") or "0.0"),
                    timestamp=(m.group("ts") or "").strip(),
                )
            except (ValueError, AttributeError):
                continue
        return None

    @staticmethod
    def format(
        hitl_weight: float,
        provider: str,
        npfm_score: float,
        prompt_context: str = "",
        timestamp: Optional[str] = None,
    ) -> GoldenTrace:
        """Create a new GoldenTrace, computing the SHA3-256 digest.

        The digest is computed over ``provider|prompt_context|timestamp``.
        """
        if timestamp is None:
            timestamp = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
        raw = f"{provider}|{prompt_context}|{timestamp}"
        digest = hashlib.sha3_256(raw.encode()).hexdigest()
        return GoldenTrace(
            digest=digest,
            hitl_weight=max(HITL_MIN, min(HITL_MAX, hitl_weight)),
            provider=provider,
            npfm_score=max(0.0, min(1.0, npfm_score)),
            timestamp=timestamp,
        )


def validate_commits(
    commit_messages: List[Tuple[str, str, str]],
) -> List[CommitValidationResult]:
    """Validate a list of commits for Golden-Trace provenance.

    Args:
        commit_messages: list of (sha, subject, full_message) tuples.

    Returns:
        List of CommitValidationResult, one per commit.
    """
    results: List[CommitValidationResult] = []
    for sha, subject, message in commit_messages:
        trace = ProvenanceTrailer.parse(message)
        if trace is None:
            results.append(CommitValidationResult(
                sha=sha,
                subject=subject,
                valid=False,
                trace=None,
                error=(
                    f"Missing Golden-Trace trailer. "
                    f"All commits must include: "
                    f"{TRAILER_KEY}: {DIGEST_PREFIX}<64-hex>; HITL=<weight>"
                ),
            ))
            continue

        # Validate HITL weight range
        if not (HITL_MIN <= trace.hitl_weight <= HITL_MAX):
            results.append(CommitValidationResult(
                sha=sha,
                subject=subject,
                valid=False,
                trace=trace,
                error=f"HITL weight {trace.hitl_weight} is out of range [0.0, 1.0]",
            ))
            continue

        # Validate NPFM score
        if trace.npfm_score < NPFM_THRESHOLD:
            results.append(CommitValidationResult(
                sha=sha,
                subject=subject,
                valid=False,
                trace=trace,
                error=(
                    f"NPFM score {trace.npfm_score:.2f} is below the required "
                    f"threshold of {NPFM_THRESHOLD}"
                ),
            ))
            continue

        # Validate digest length and hex chars
        if not re.fullmatch(r"[0-9a-f]{64}", trace.digest):
            results.append(CommitValidationResult(
                sha=sha,
                subject=subject,
                valid=False,
                trace=trace,
                error=f"Digest '{trace.digest}' is not a valid 64-char hex string",
            ))
            continue

        results.append(CommitValidationResult(
            sha=sha,
            subject=subject,
            valid=True,
            trace=trace,
            error=None,
        ))

    return results


def get_commits_in_range(commit_range: str) -> List[Tuple[str, str, str]]:
    """Return (sha, subject, full_message) for each commit in the given range.

    Requires git to be available on PATH.
    """
    try:
        result = subprocess.run(
            ["git", "log", "--format=%H%x00%s%x00%B%x1e", commit_range],
            capture_output=True,
            text=True,
            check=True,
        )
    except subprocess.CalledProcessError as exc:
        raise RuntimeError(f"git log failed: {exc.stderr.strip()}") from exc

    commits = []
    for record in result.stdout.split("\x1e"):
        record = record.strip()
        if not record:
            continue
        parts = record.split("\x00", 2)
        if len(parts) < 3:
            continue
        sha, subject, message = parts
        commits.append((sha.strip(), subject.strip(), message.strip()))
    return commits


# ─── CLI interface ────────────────────────────────────────────

def cmd_check_commits(args: argparse.Namespace) -> int:
    """Validate commits in a range or from stdin."""
    if args.range:
        try:
            commits = get_commits_in_range(args.range)
        except RuntimeError as exc:
            print(f"ERROR: {exc}", file=sys.stderr)
            return 1
    else:
        # Read raw commit messages from stdin (one per blank-line separator)
        raw = sys.stdin.read()
        commits = [("stdin", "(stdin)", raw)]

    if not commits:
        print("No commits to validate.", file=sys.stderr)
        return 0

    results = validate_commits(commits)
    failed = [r for r in results if not r.valid]

    if args.json:
        print(json.dumps([r.to_dict() for r in results], indent=2))
    else:
        for r in results:
            icon = "✅" if r.valid else "❌"
            sha_short = r.sha[:12] if len(r.sha) >= 12 else r.sha
            print(f"{icon} {sha_short}  {r.subject}")
            if r.error:
                print(f"   ERROR: {r.error}")
            elif r.trace:
                print(
                    f"   HITL={r.trace.hitl_weight:.2f}  "
                    f"NPFM={r.trace.npfm_score:.2f}  "
                    f"provider={r.trace.provider}"
                )

    if failed:
        print(
            f"\n❌  {len(failed)} of {len(results)} commits failed provenance check.",
            file=sys.stderr,
        )
        return 1

    print(f"\n✅  All {len(results)} commits have valid Golden-Trace provenance.")
    return 0


def cmd_format_trailer(args: argparse.Namespace) -> int:
    """Generate and print a Golden-Trace trailer."""
    trace = ProvenanceTrailer.format(
        hitl_weight=args.hitl,
        provider=args.provider,
        npfm_score=args.npfm,
        prompt_context=args.context or "",
    )
    if args.json:
        print(trace.to_json())
    else:
        print(trace.to_trailer_string())
    return 0


def cmd_verify(args: argparse.Namespace) -> int:
    """Verify a single Golden-Trace digest is well-formed."""
    digest = args.digest.removeprefix(DIGEST_PREFIX)
    if re.fullmatch(r"[0-9a-f]{64}", digest):
        print(f"✅  Digest '{digest}' is a valid SHA3-256 hex string.")
        return 0
    else:
        print(f"❌  '{digest}' is NOT a valid 64-char lowercase hex SHA3-256 digest.")
        return 1


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Aluminum OS GoldenTrace Provenance Tool",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=__doc__,
    )
    sub = parser.add_subparsers(dest="command", required=True)

    # check-commits
    p_check = sub.add_parser(
        "check-commits",
        help="Validate Golden-Trace trailers on commits",
    )
    p_check.add_argument(
        "range",
        nargs="?",
        help="git commit range (e.g. HEAD~5..HEAD, origin/main..HEAD)",
    )
    p_check.add_argument("--json", action="store_true", help="Output JSON")

    # format-trailer
    p_fmt = sub.add_parser(
        "format-trailer",
        help="Generate a Golden-Trace git commit trailer",
    )
    p_fmt.add_argument(
        "--hitl",
        type=float,
        default=0.90,
        help="HITL weight [0.0–1.0] (default: 0.90)",
    )
    p_fmt.add_argument(
        "--provider",
        default="unknown",
        help="AI provider name (default: unknown)",
    )
    p_fmt.add_argument(
        "--npfm",
        type=float,
        default=0.80,
        help="NPFM score [0.0–1.0] (default: 0.80)",
    )
    p_fmt.add_argument(
        "--context",
        default="",
        help="Prompt context string used for digest computation",
    )
    p_fmt.add_argument("--json", action="store_true", help="Output JSON")

    # verify
    p_verify = sub.add_parser(
        "verify",
        help="Check that a digest is a valid SHA3-256 hex string",
    )
    p_verify.add_argument("digest", help="Digest string (with or without 'sha3-256:' prefix)")

    args = parser.parse_args()

    if args.command == "check-commits":
        return cmd_check_commits(args)
    elif args.command == "format-trailer":
        return cmd_format_trailer(args)
    elif args.command == "verify":
        return cmd_verify(args)
    else:
        parser.print_help()
        return 1


if __name__ == "__main__":
    sys.exit(main())
