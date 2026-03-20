#!/usr/bin/env python3
"""
invariant_linter.py — Aluminum OS Constitutional Invariant Linter
Version: 2.1 (Updated to use invariants_registry.py)

Checks code against all 36 Aluminum Invariants defined in invariants_registry.py.
This is Stage 1 of the Spheres OS CI/CD pipeline.

Usage:
  python invariant_linter.py <path> [--strict] [--json]

Author: Manus AI (original), Claude (registry integration) for Dave Sheldon
Date: March 19, 2026
"""

import os
import sys
import re
import json
from dataclasses import dataclass, asdict
from typing import List, Dict
from pathlib import Path

# Import the canonical invariant registry
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from invariants_registry import INVARIANTS


# ============================================================================
# Data Models
# ============================================================================

@dataclass
class Violation:
    invariant_id: str
    invariant_name: str
    severity: str
    file_path: str
    line: int
    code: str
    description: str


@dataclass
class LintReport:
    target_path: str
    files_scanned: int
    total_lines: int
    violations: List[Dict]
    violations_by_severity: Dict[str, int]
    invariants_checked: int
    invariants_passed: int
    invariants_violated: int
    compliance_score: float  # 0.0 to 1.0


# ============================================================================
# Linter Engine
# ============================================================================

class InvariantLinter:
    """Checks code against all Aluminum Invariants from the canonical registry."""

    def __init__(self):
        self.invariants = INVARIANTS

    def _file_matches(self, file_path: str, patterns: List[str]) -> bool:
        """Check if a file matches any of the glob patterns."""
        name = Path(file_path).name
        for pattern in patterns:
            ext = pattern.replace("*", "")
            if name.endswith(ext):
                return True
        return False

    def _check_guard(self, lines: List[str], line_idx: int,
                     guard_patterns: List[str], window: int = 25) -> bool:
        """Check if there's a guard pattern within `window` lines."""
        start = max(0, line_idx - window)
        end = min(len(lines), line_idx + window)
        context = '\n'.join(lines[start:end])
        for gp in guard_patterns:
            if re.search(gp, context, re.IGNORECASE):
                return True
        return False

    def lint_file(self, file_path: str) -> List[Violation]:
        """Lint a single file against all applicable invariants."""
        violations = []
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
                lines = content.split('\n')
        except (UnicodeDecodeError, FileNotFoundError, PermissionError):
            return violations

        for inv_id, inv in self.invariants.items():
            check_type = inv.get("check_type", "advisory")
            applies_to = inv.get("applies_to", [])

            # Skip if this invariant doesn't apply to this file type
            if applies_to and not self._file_matches(file_path, applies_to):
                continue

            if check_type == "guard_check":
                dangerous = inv.get("dangerous_patterns", [])
                guards = inv.get("guard_patterns", [])
                for dp in dangerous:
                    for i, line in enumerate(lines):
                        if re.search(dp, line):
                            if not self._check_guard(lines, i, guards):
                                violations.append(Violation(
                                    invariant_id=inv_id,
                                    invariant_name=inv["name"],
                                    severity=inv["severity"],
                                    file_path=file_path,
                                    line=i + 1,
                                    code=line.strip(),
                                    description=f"{inv_id} ({inv['name']}): {inv['description']}",
                                ))

            elif check_type == "pattern_absence_negative":
                forbidden = inv.get("forbidden_patterns", [])
                for fp in forbidden:
                    for i, line in enumerate(lines):
                        if re.search(fp, line, re.IGNORECASE):
                            violations.append(Violation(
                                invariant_id=inv_id,
                                invariant_name=inv["name"],
                                severity=inv["severity"],
                                file_path=file_path,
                                line=i + 1,
                                code=line.strip(),
                                description=f"{inv_id} ({inv['name']}): Forbidden pattern detected — {inv['description']}",
                            ))

        return violations

    def lint_directory(self, path: str) -> LintReport:
        """Lint an entire directory."""
        all_violations = []
        files_scanned = 0
        total_lines = 0
        extensions = {'.py', '.rs', '.ts', '.tsx', '.js', '.jsx', '.json', '.yaml', '.yml'}

        target = Path(path)
        if target.is_file():
            files = [target]
        else:
            files = [f for f in target.rglob('*')
                     if f.suffix in extensions and f.is_file()
                     and '.git' not in str(f)]

        for file_path in files:
            try:
                with open(file_path, 'r', encoding='utf-8') as f:
                    total_lines += len(f.readlines())
                files_scanned += 1
                file_violations = self.lint_file(str(file_path))
                all_violations.extend(file_violations)
            except Exception:
                pass

        # Count by severity
        by_severity = {"critical": 0, "mandatory": 0, "warning": 0, "advisory": 0}
        violated_invariants = set()
        for v in all_violations:
            by_severity[v.severity] = by_severity.get(v.severity, 0) + 1
            violated_invariants.add(v.invariant_id)

        # Compliance score
        total_checkable = sum(1 for inv in self.invariants.values()
                              if inv.get("check_type") != "advisory")
        passed = total_checkable - len(violated_invariants)
        compliance = passed / max(total_checkable, 1)

        return LintReport(
            target_path=path,
            files_scanned=files_scanned,
            total_lines=total_lines,
            violations=[asdict(v) for v in all_violations],
            violations_by_severity=by_severity,
            invariants_checked=len(self.invariants),
            invariants_passed=len(self.invariants) - len(violated_invariants),
            invariants_violated=len(violated_invariants),
            compliance_score=round(compliance, 3),
        )


def main():
    if len(sys.argv) < 2:
        print("Usage: python invariant_linter.py <path> [--strict] [--json]")
        sys.exit(1)

    target = sys.argv[1]
    strict = "--strict" in sys.argv
    as_json = "--json" in sys.argv

    linter = InvariantLinter()
    report = linter.lint_directory(target)

    if as_json:
        print(json.dumps(asdict(report), indent=2))
    else:
        print(f"\n{'='*60}")
        print(f"  ALUMINUM OS INVARIANT LINTER")
        print(f"  Target: {report.target_path}")
        print(f"{'='*60}\n")
        print(f"  Files Scanned:     {report.files_scanned}")
        print(f"  Total Lines:       {report.total_lines}")
        print(f"  Invariants Checked: {report.invariants_checked}")
        print(f"  Invariants Passed:  {report.invariants_passed}")
        print(f"  Invariants Violated: {report.invariants_violated}")
        print(f"  Compliance Score:   {report.compliance_score:.1%}")
        print(f"\n  Violations by Severity:")
        for sev, count in report.violations_by_severity.items():
            print(f"    {sev}: {count}")

        if report.violations:
            print(f"\n  Top Violations:")
            shown = set()
            for v in report.violations[:20]:
                key = f"{v['invariant_id']}:{v['file_path']}:{v['line']}"
                if key not in shown:
                    shown.add(key)
                    print(f"    [{v['severity'].upper()}] {v['invariant_id']} "
                          f"({v['invariant_name']}) @ {v['file_path']}:{v['line']}")
                    print(f"      {v['code'][:80]}")

        print(f"\n{'='*60}\n")

    # Save report
    report_path = "invariant_report.json"
    with open(report_path, 'w') as f:
        json.dump(asdict(report), f, indent=2)
    print(f"Report saved to: {report_path}")

    if strict and report.violations_by_severity.get("critical", 0) > 0:
        print("STRICT MODE: Critical violations found. Failing.")
        sys.exit(1)


if __name__ == "__main__":
    main()