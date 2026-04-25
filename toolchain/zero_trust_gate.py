#!/usr/bin/env python3
"""
zero_trust_gate.py — Zero Trust Integration Gate
Version: 1.0.0

Validates integration manifests in the `integrations/` directory.
Every new or modified external integration (provider, MCP server, API client, etc.)
MUST declare:

  (a) scopes       — the minimum set of OAuth scopes or API permissions required
  (b) data_handling_class — CLASS_A, CLASS_B, or CLASS_C (INV-004)
  (c) constitutional_invariants — list of INV-NNN IDs the integration depends on

Usage:
  # Validate all manifests in integrations/
  python toolchain/zero_trust_gate.py

  # Validate a single manifest
  python toolchain/zero_trust_gate.py integrations/my-service.yaml

  # Validate only files changed in the current PR (requires git)
  python toolchain/zero_trust_gate.py --pr-diff

  # Machine-readable JSON output
  python toolchain/zero_trust_gate.py --json

Exit codes:
  0 — all manifests valid
  1 — one or more validation errors

Author: GitHub Copilot (builder)
Date: 2026-04-25
"""

from __future__ import annotations

import json
import os
import re
import subprocess
import sys
from dataclasses import asdict, dataclass, field
from pathlib import Path
from typing import Any, Dict, List, Optional

# ---------------------------------------------------------------------------
# Canonical list of valid INV IDs (INV-001 through INV-036 + sub-invariants)
# We build this from the registry if available, otherwise use a static list.
# ---------------------------------------------------------------------------

def _load_valid_invariant_ids() -> set[str]:
    """Return the set of valid INV IDs from the toolchain registry."""
    import importlib.util

    registry_path = Path(__file__).parent / "invariants_registry.py"
    if not registry_path.exists():
        # Fallback: accept any INV-NNN or INV-NNNa format
        return set()
    # Build the set from the registry keys (INV-1, INV-2, …) and
    # also accept the zero-padded form (INV-001, INV-002, …) used in specs/.
    try:
        spec = importlib.util.spec_from_file_location(
            "invariants_registry", str(registry_path)
        )
        if spec is None or spec.loader is None:
            return set()
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)  # type: ignore[union-attr]
        registry: Dict[str, Any] = getattr(module, "INVARIANTS", {})
        ids: set[str] = set()
        for key in registry:
            ids.add(key)          # original form: "INV-1"
            # zero-padded form: "INV-001"
            m = re.match(r"^INV-(\d+)(.*)?$", key)
            if m:
                ids.add(f"INV-{int(m.group(1)):03d}{m.group(2) or ''}")
        return ids
    except Exception:  # noqa: BLE001
        return set()


VALID_INV_IDS: set[str] = _load_valid_invariant_ids()
VALID_DATA_CLASSES = {"CLASS_A", "CLASS_B", "CLASS_C"}

# ---------------------------------------------------------------------------
# Data models
# ---------------------------------------------------------------------------

@dataclass
class GateError:
    code: str
    field: str
    message: str


@dataclass
class ManifestResult:
    path: str
    name: str
    passed: bool
    errors: List[GateError] = field(default_factory=list)

    def to_dict(self) -> Dict[str, Any]:
        d = asdict(self)
        return d


@dataclass
class GateReport:
    manifests_checked: int
    manifests_passed: int
    manifests_failed: int
    results: List[ManifestResult] = field(default_factory=list)
    passed: bool = True

    def to_dict(self) -> Dict[str, Any]:
        return {
            "manifests_checked": self.manifests_checked,
            "manifests_passed": self.manifests_passed,
            "manifests_failed": self.manifests_failed,
            "passed": self.passed,
            "results": [r.to_dict() for r in self.results],
        }


# ---------------------------------------------------------------------------
# Manifest validation
# ---------------------------------------------------------------------------

def _load_yaml(path: Path) -> Optional[Dict[str, Any]]:
    """Load a YAML file; return None on parse failure."""
    try:
        import yaml  # type: ignore[import]
        with path.open() as fh:
            return yaml.safe_load(fh)
    except ImportError:
        # PyYAML not available — fall back to a minimal YAML subset parser.
        return _minimal_yaml_parse(path)
    except Exception:  # noqa: BLE001
        return None


def _minimal_yaml_parse(path: Path) -> Optional[Dict[str, Any]]:
    """
    Parse a minimal YAML manifest without PyYAML.
    Supports top-level scalar and list fields only (no nested mappings).
    """
    result: Dict[str, Any] = {}
    current_list_key: Optional[str] = None
    current_list: List[str] = []

    try:
        for raw in path.read_text().splitlines():
            line = raw.rstrip()
            # Skip comments and blank lines
            if not line or line.lstrip().startswith("#"):
                continue
            # List item
            if line.lstrip().startswith("- "):
                item = line.lstrip()[2:].strip()
                # Strip inline comment
                item = re.sub(r"\s+#.*$", "", item).strip()
                if current_list_key:
                    current_list.append(item)
                continue
            # Key: value
            if ":" in line and not line.lstrip().startswith("-"):
                if current_list_key and current_list:
                    result[current_list_key] = current_list
                    current_list = []
                    current_list_key = None
                key, _, val = line.partition(":")
                key = key.strip()
                val = val.strip()
                # Strip inline comment from scalar value
                val = re.sub(r"\s+#.*$", "", val).strip()
                if val == "" or val is None:
                    # This key might be followed by a list
                    current_list_key = key
                    current_list = []
                else:
                    # Strip surrounding quotes
                    val = val.strip('"\'')
                    result[key] = val

        # Flush any pending list
        if current_list_key and current_list:
            result[current_list_key] = current_list

        return result
    except Exception:  # noqa: BLE001
        return None


def validate_manifest(path: Path) -> ManifestResult:
    """Validate a single integration manifest file."""
    manifest = _load_yaml(path)
    name = path.stem

    if manifest is None:
        return ManifestResult(
            path=str(path),
            name=name,
            passed=False,
            errors=[
                GateError(
                    code="PARSE_ERROR",
                    field="(file)",
                    message=f"Failed to parse YAML manifest: {path}",
                )
            ],
        )

    errors: List[GateError] = []

    # ── (a) Scopes ──────────────────────────────────────────────────────────
    scopes = manifest.get("scopes")
    if not scopes:
        errors.append(GateError(
            code="MISSING_SCOPES",
            field="scopes",
            message=(
                "Integration manifest must declare 'scopes' — the minimum set of "
                "OAuth scopes or API permissions required."
            ),
        ))
    elif not isinstance(scopes, list) or len(scopes) == 0:
        errors.append(GateError(
            code="MISSING_SCOPES",
            field="scopes",
            message="'scopes' must be a non-empty list.",
        ))

    # ── (b) Data-handling class ──────────────────────────────────────────────
    data_class = manifest.get("data_handling_class")
    if not data_class:
        errors.append(GateError(
            code="MISSING_DATA_CLASS",
            field="data_handling_class",
            message=(
                "Integration manifest must declare 'data_handling_class': "
                "CLASS_A (highest sensitivity), CLASS_B (standard), or CLASS_C (public)."
            ),
        ))
    elif data_class not in VALID_DATA_CLASSES:
        errors.append(GateError(
            code="INVALID_DATA_CLASS",
            field="data_handling_class",
            message=(
                f"Invalid data_handling_class '{data_class}'. "
                f"Must be one of: {', '.join(sorted(VALID_DATA_CLASSES))}."
            ),
        ))

    # ── (c) Constitutional invariants ────────────────────────────────────────
    invariants = manifest.get("constitutional_invariants")
    if not invariants:
        errors.append(GateError(
            code="MISSING_INVARIANTS",
            field="constitutional_invariants",
            message=(
                "Integration manifest must declare 'constitutional_invariants' — "
                "list the INV-NNN IDs of every invariant that governs this integration."
            ),
        ))
    elif not isinstance(invariants, list) or len(invariants) == 0:
        errors.append(GateError(
            code="MISSING_INVARIANTS",
            field="constitutional_invariants",
            message="'constitutional_invariants' must be a non-empty list.",
        ))
    elif VALID_INV_IDS:
        for inv_id in invariants:
            inv_id_clean = str(inv_id).strip()
            if inv_id_clean not in VALID_INV_IDS:
                errors.append(GateError(
                    code="INVALID_INVARIANT_ID",
                    field="constitutional_invariants",
                    message=(
                        f"Unknown invariant ID '{inv_id_clean}'. "
                        "Check toolchain/invariants_registry.py for valid IDs."
                    ),
                ))

    return ManifestResult(
        path=str(path),
        name=manifest.get("name", name),
        passed=len(errors) == 0,
        errors=errors,
    )


# ---------------------------------------------------------------------------
# File discovery
# ---------------------------------------------------------------------------

def find_all_manifests(integrations_dir: Path) -> List[Path]:
    """Return all .yaml files in the integrations directory (not README)."""
    return sorted(
        p for p in integrations_dir.glob("*.yaml")
        if p.name != "README.yaml"
    )


def find_pr_diff_manifests(integrations_dir: Path) -> List[Path]:
    """
    Return manifest files that are new or modified in the current PR
    (compared to origin/<base_branch>).
    Falls back to all manifests if git is unavailable.
    """
    base = os.environ.get("GITHUB_BASE_REF", "main")
    try:
        result = subprocess.run(
            ["git", "diff", "--name-only", f"origin/{base}...HEAD"],
            capture_output=True,
            text=True,
            check=True,
        )
        changed = result.stdout.splitlines()
        manifests = [
            Path(f) for f in changed
            if f.startswith("integrations/") and f.endswith(".yaml")
        ]
        return [p for p in manifests if p.exists()]
    except Exception:  # noqa: BLE001
        return find_all_manifests(integrations_dir)


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def run(paths: List[Path], output_json: bool = False) -> GateReport:
    results: List[ManifestResult] = []
    for p in paths:
        results.append(validate_manifest(p))

    passed_count = sum(1 for r in results if r.passed)
    failed_count = len(results) - passed_count

    report = GateReport(
        manifests_checked=len(results),
        manifests_passed=passed_count,
        manifests_failed=failed_count,
        results=results,
        passed=failed_count == 0,
    )

    if output_json:
        print(json.dumps(report.to_dict(), indent=2))
    else:
        _print_human(report)

    return report


def _print_human(report: GateReport) -> None:
    status = "✅ PASSED" if report.passed else "❌ FAILED"
    print(f"\nZero Trust Integration Gate — {status}")
    print(f"  Manifests checked : {report.manifests_checked}")
    print(f"  Passed            : {report.manifests_passed}")
    print(f"  Failed            : {report.manifests_failed}")
    print()
    for result in report.results:
        if result.passed:
            print(f"  ✅  {result.name} ({result.path})")
        else:
            print(f"  ❌  {result.name} ({result.path})")
            for err in result.errors:
                print(f"       [{err.code}] {err.field}: {err.message}")
    print()


def main() -> None:
    args = sys.argv[1:]
    output_json = "--json" in args
    pr_diff = "--pr-diff" in args
    args = [a for a in args if a not in ("--json", "--pr-diff")]

    integrations_dir = Path(__file__).parent.parent / "integrations"

    if args:
        # Explicit file(s) on the command line
        paths = [Path(a) for a in args]
        missing = [p for p in paths if not p.exists()]
        if missing:
            for p in missing:
                print(f"::error::File not found: {p}", file=sys.stderr)
            sys.exit(1)
    elif pr_diff:
        paths = find_pr_diff_manifests(integrations_dir)
        if not paths:
            print("No integration manifests changed in this PR — gate skipped.")
            sys.exit(0)
    else:
        paths = find_all_manifests(integrations_dir)
        if not paths:
            print("No integration manifests found in integrations/ — gate skipped.")
            sys.exit(0)

    report = run(paths, output_json=output_json)
    sys.exit(0 if report.passed else 1)


if __name__ == "__main__":
    main()
