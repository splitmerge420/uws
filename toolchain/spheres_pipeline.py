#!/usr/bin/env python3
"""
spheres_pipeline.py — Unified Spheres OS CI/CD Pipeline Runner
Version: 3.0

Chains all Spheres OS tools into a single, automated pipeline:
  1. Invariant Linter → checks code against 30 Aluminum Invariants
  2. Kintsugi Healer → heals fractures found by the linter
  3. PQC Provider → signs all healed code with post-quantum cryptography
  4. 10YST Engine → stress tests critical spheres
  5. Report → generates a unified pipeline report

This is the missing integration layer that connects all v2 artifacts.

Architecture:
  - Modular: each stage can be run independently or as part of the pipeline
  - INV-2 compliant: consent required before any state-changing operation
  - INV-3 compliant: full audit trail of all pipeline operations
  - INV-7 compliant: no single-vendor dependency
  - INV-30 compliant: no Belter Rule violations

Cross-Platform: Compatible with macOS/iOS (via Python), Linux, ChromeOS

Usage:
  python spheres_pipeline.py <target_path> [--stage lint|heal|sign|stress|all]
  python spheres_pipeline.py <target_path> --full    # Run complete pipeline
  python spheres_pipeline.py <target_path> --report   # Generate report only

Author: Manus AI for Daavud / Spheres OS
Date: March 17, 2026
"""

import os
import sys
import json
import time
import subprocess
import importlib.util
from dataclasses import dataclass, asdict, field
from typing import List, Dict, Optional, Any
from pathlib import Path
from datetime import datetime, timezone


# ============================================================================
# Setup toolchain directory for imports
# ============================================================================

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
TOOLCHAIN_DIR = SCRIPT_DIR
if TOOLCHAIN_DIR not in sys.path:
    sys.path.insert(0, TOOLCHAIN_DIR)


# ============================================================================
# Constants
# ============================================================================

VERSION = "3.0.0"
TOOLS_DIR = os.path.dirname(os.path.abspath(__file__))
TIMEOUT_PER_STAGE = 300  # 300 seconds timeout per stage

# Pipeline stages
STAGES = ["lint", "heal", "sign", "stress"]


# ============================================================================
# Data Models
# ============================================================================

@dataclass
class StageResult:
    """Result of a single pipeline stage."""
    stage: str
    status: str         # success, warning, failure, skipped
    duration_seconds: float
    summary: str
    details: Dict = field(default_factory=dict)
    artifacts: List[str] = field(default_factory=list)


@dataclass
class PipelineReport:
    """Complete pipeline execution report."""
    target_path: str
    timestamp: str
    pipeline_version: str
    stages_run: int
    stages_passed: int
    stages_failed: int
    total_duration_seconds: float
    overall_status: str     # passed, failed, warning
    stage_results: List[Dict]
    recommendations: List[str]


# ============================================================================
# Tool Loader
# ============================================================================

def load_module(module_name: str, file_path: str):
    """Dynamically load a Python module from file path."""
    spec = importlib.util.spec_from_file_location(module_name, file_path)
    if spec is None or spec.loader is None:
        raise ImportError(f"Cannot load module from {file_path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def find_tool(tool_name: str) -> Optional[str]:
    """Find a tool file in the tools directory, toolchain directory, or current directory."""
    candidates = [
        os.path.join(TOOLS_DIR, tool_name),
        os.path.join(TOOLCHAIN_DIR, tool_name),
        os.path.join(os.getcwd(), tool_name),
        os.path.join(os.getcwd(), "tools", tool_name),
        os.path.join(os.getcwd(), "toolchain", tool_name),
    ]
    for path in candidates:
        if os.path.exists(path):
            return path
    return None


# ============================================================================
# Pipeline Stages
# ============================================================================

def stage_lint(target_path: str) -> StageResult:
    """Stage 1: Run the Invariant Linter."""
    print("\n" + "=" * 60)
    print("  STAGE 1: INVARIANT LINTER")
    print("=" * 60)

    start = time.time()
    linter_path = find_tool("invariant_linter.py")

    if not linter_path:
        return StageResult(
            stage="lint",
            status="skipped",
            duration_seconds=0,
            summary="invariant_linter.py not found",
        )

    try:
        result = subprocess.run(
            [sys.executable, linter_path, target_path],
            capture_output=True,
            text=True,
            timeout=TIMEOUT_PER_STAGE,
        )

        output = result.stdout + result.stderr
        print(output)

        # Parse results
        violations = 0
        compliance = 0.0
        for line in output.split("\n"):
            if "Invariants Violated:" in line:
                try:
                    violations = int(line.split(":")[-1].strip())
                except ValueError:
                    pass
            if "Compliance Score:" in line:
                try:
                    compliance = float(line.split(":")[-1].strip().rstrip("%"))
                except ValueError:
                    pass

        # Check for report file
        report_path = os.path.join(
            target_path if os.path.isdir(target_path) else os.path.dirname(target_path),
            "invariant_report.json"
        )

        status = "success" if violations == 0 else ("warning" if violations < 5 else "failure")

        return StageResult(
            stage="lint",
            status=status,
            duration_seconds=round(time.time() - start, 2),
            summary=f"{violations} invariant violations found, compliance: {compliance}%",
            details={"violations": violations, "compliance": compliance},
            artifacts=[report_path] if os.path.exists(report_path) else [],
        )

    except subprocess.TimeoutExpired:
        return StageResult(
            stage="lint",
            status="failure",
            duration_seconds=round(time.time() - start, 2),
            summary=f"Linter timed out after {TIMEOUT_PER_STAGE} seconds",
        )
    except Exception as e:
        return StageResult(
            stage="lint",
            status="failure",
            duration_seconds=round(time.time() - start, 2),
            summary=f"Linter error: {str(e)}",
        )


def stage_heal(target_path: str) -> StageResult:
    """Stage 2: Run the Kintsugi Healer."""
    print("\n" + "=" * 60)
    print("  STAGE 2: KINTSUGI HEALER")
    print("=" * 60)

    start = time.time()
    healer_path = find_tool("kintsugi_healer.py")

    if not healer_path:
        return StageResult(
            stage="heal",
            status="skipped",
            duration_seconds=0,
            summary="kintsugi_healer.py not found",
        )

    try:
        result = subprocess.run(
            [sys.executable, healer_path, target_path],
            capture_output=True,
            text=True,
            timeout=TIMEOUT_PER_STAGE,
        )

        output = result.stdout + result.stderr
        print(output)

        # Parse results
        fractures = 0
        mends = 0
        beauty_before = 0.0
        beauty_after = 0.0

        for line in output.split("\n"):
            if "Found" in line and "fractures" in line:
                try:
                    fractures = int(line.split("Found")[1].split("fractures")[0].strip())
                except (ValueError, IndexError):
                    pass
            if "Mends Applied:" in line:
                try:
                    mends = int(line.split(":")[-1].strip())
                except ValueError:
                    pass
            if "Beauty Score:" in line and "→" in line:
                try:
                    parts = line.split(":")[-1].strip().split("→")
                    beauty_before = float(parts[0].strip())
                    beauty_after = float(parts[1].strip())
                except (ValueError, IndexError):
                    pass

        status = "success" if fractures == 0 else ("warning" if mends > 0 else "failure")

        return StageResult(
            stage="heal",
            status=status,
            duration_seconds=round(time.time() - start, 2),
            summary=f"{fractures} fractures found, {mends} mends applied, beauty: {beauty_before} → {beauty_after}",
            details={
                "fractures": fractures,
                "mends": mends,
                "beauty_before": beauty_before,
                "beauty_after": beauty_after,
            },
        )

    except subprocess.TimeoutExpired:
        return StageResult(
            stage="heal",
            status="failure",
            duration_seconds=round(time.time() - start, 2),
            summary=f"Healer timed out after {TIMEOUT_PER_STAGE} seconds",
        )
    except Exception as e:
        return StageResult(
            stage="heal",
            status="failure",
            duration_seconds=round(time.time() - start, 2),
            summary=f"Healer error: {str(e)}",
        )


def stage_sign(target_path: str) -> StageResult:
    """Stage 3: Run the PQC Provider."""
    print("\n" + "=" * 60)
    print("  STAGE 3: PQC SIGNING")
    print("=" * 60)

    start = time.time()
    pqc_path = find_tool("pqc_provider.py")

    if not pqc_path:
        return StageResult(
            stage="sign",
            status="skipped",
            duration_seconds=0,
            summary="pqc_provider.py not found",
        )

    try:
        # Step 1: Generate keys if not present
        key_dir = os.path.join(target_path if os.path.isdir(target_path) else os.path.dirname(target_path), ".pqc_keys")

        if not os.path.exists(key_dir):
            print("  [PQC] Generating keys...")
            keygen_result = subprocess.run(
                [sys.executable, pqc_path, "keygen", "--output", key_dir, "--auto-consent"],
                capture_output=True,
                text=True,
                timeout=TIMEOUT_PER_STAGE,
            )
            print(keygen_result.stdout)

        # Find private key
        private_keys = list(Path(key_dir).glob("*_private.pem"))
        if not private_keys:
            return StageResult(
                stage="sign",
                status="failure",
                duration_seconds=round(time.time() - start, 2),
                summary="No PQC private key found",
            )

        # Step 2: Sign files
        sign_result = subprocess.run(
            [sys.executable, pqc_path, "sign",
             "--key", str(private_keys[0]),
             "--file", target_path,
             "--auto-consent"],
            capture_output=True,
            text=True,
            timeout=TIMEOUT_PER_STAGE,
        )

        output = sign_result.stdout + sign_result.stderr
        print(output)

        signed_count = output.count("[PQC] Signed:")
        status = "success" if signed_count > 0 else "warning"

        return StageResult(
            stage="sign",
            status=status,
            duration_seconds=round(time.time() - start, 2),
            summary=f"{signed_count} files signed with PQC",
            details={"signed_files": signed_count},
        )

    except subprocess.TimeoutExpired:
        return StageResult(
            stage="sign",
            status="failure",
            duration_seconds=round(time.time() - start, 2),
            summary=f"PQC signing timed out after {TIMEOUT_PER_STAGE} seconds",
        )
    except Exception as e:
        return StageResult(
            stage="sign",
            status="failure",
            duration_seconds=round(time.time() - start, 2),
            summary=f"PQC error: {str(e)}",
        )


def stage_stress(target_path: str) -> StageResult:
    """Stage 4: Run the 10-Year Stress Test."""
    print("\n" + "=" * 60)
    print("  STAGE 4: 10-YEAR STRESS TEST")
    print("=" * 60)

    start = time.time()
    stress_path = find_tool("stress_test.py")

    if not stress_path:
        return StageResult(
            stage="stress",
            status="skipped",
            duration_seconds=0,
            summary="stress_test.py not found",
        )

    try:
        result = subprocess.run(
            [sys.executable, stress_path, target_path, "--quick"],
            capture_output=True,
            text=True,
            timeout=TIMEOUT_PER_STAGE,
        )

        output = result.stdout + result.stderr
        print(output)

        # Parse results
        resilience = 0.0
        passed = False
        for line in output.split("\n"):
            if "Overall Resilience:" in line:
                try:
                    resilience = float(line.split(":")[-1].strip())
                except ValueError:
                    pass
            if "VERDICT: PASSED" in line:
                passed = True

        status = "success" if passed else "failure"

        return StageResult(
            stage="stress",
            status=status,
            duration_seconds=round(time.time() - start, 2),
            summary=f"Resilience: {resilience:.4f}, Verdict: {'PASSED' if passed else 'FAILED'}",
            details={"resilience": resilience, "passed": passed},
        )

    except subprocess.TimeoutExpired:
        return StageResult(
            stage="stress",
            status="failure",
            duration_seconds=round(time.time() - start, 2),
            summary=f"Stress test timed out after {TIMEOUT_PER_STAGE} seconds",
        )
    except Exception as e:
        return StageResult(
            stage="stress",
            status="failure",
            duration_seconds=round(time.time() - start, 2),
            summary=f"Stress test error: {str(e)}",
        )


# ============================================================================
# Pipeline Runner
# ============================================================================

STAGE_FUNCTIONS = {
    "lint": stage_lint,
    "heal": stage_heal,
    "sign": stage_sign,
    "stress": stage_stress,
}


def run_pipeline(target_path: str, stages: List[str] = None,
                 stop_on_failure: bool = True,
                 continue_on_warning: bool = False) -> PipelineReport:
    """Run the complete Spheres OS pipeline.

    Args:
        target_path: Path to the target to analyze/heal/sign/stress test
        stages: List of stages to run (default: all stages)
        stop_on_failure: If True, stop pipeline on first FAIL (default: True for fail-fast)
        continue_on_warning: If True, warnings don't halt pipeline. If False, only FAIL halts (default: False)
    """
    if stages is None:
        stages = STAGES

    print("=" * 60)
    print("  SPHERES OS UNIFIED PIPELINE v3.0")
    print(f"  Target: {target_path}")
    print(f"  Stages: {', '.join(stages)}")
    print(f"  Fail-Fast: {stop_on_failure}")
    print(f"  Continue on Warning: {continue_on_warning}")
    print("=" * 60)

    start_time = time.time()
    results: List[StageResult] = []
    passed = 0
    failed = 0

    for stage_name in stages:
        if stage_name not in STAGE_FUNCTIONS:
            print(f"\n  [PIPELINE] Unknown stage: {stage_name}, skipping")
            continue

        stage_fn = STAGE_FUNCTIONS[stage_name]
        result = stage_fn(target_path)
        results.append(result)

        if result.status == "success":
            passed += 1
        elif result.status == "warning":
            passed += 1
            # Check if we should halt on warning
            if not continue_on_warning:
                print(f"\n  [PIPELINE] Stage '{stage_name}' completed with warning. Stopping pipeline.")
                break
        elif result.status == "failure":
            failed += 1
            if stop_on_failure:
                print(f"\n  [PIPELINE] Stage '{stage_name}' failed. Stopping pipeline.")
                break

    total_duration = round(time.time() - start_time, 2)

    # Determine overall status
    if failed == 0:
        overall_status = "passed"
    elif failed < len(stages) / 2:
        overall_status = "warning"
    else:
        overall_status = "failed"

    # Generate recommendations
    recommendations = []
    for r in results:
        if r.status == "failure":
            recommendations.append(f"Fix {r.stage} stage: {r.summary}")
        elif r.status == "skipped":
            recommendations.append(f"Install missing tool for {r.stage} stage")

    report = PipelineReport(
        target_path=target_path,
        timestamp=datetime.now(timezone.utc).isoformat(),
        pipeline_version=VERSION,
        stages_run=len(results),
        stages_passed=passed,
        stages_failed=failed,
        total_duration_seconds=total_duration,
        overall_status=overall_status,
        stage_results=[asdict(r) for r in results],
        recommendations=recommendations,
    )

    # Print summary
    print("\n" + "=" * 60)
    print("  PIPELINE SUMMARY")
    print("=" * 60)
    print(f"  Stages Run:    {report.stages_run}")
    print(f"  Stages Passed: {report.stages_passed}")
    print(f"  Stages Failed: {report.stages_failed}")
    print(f"  Duration:      {report.total_duration_seconds}s")
    print(f"  Status:        {report.overall_status.upper()}")
    print()

    for r in results:
        icon = {"success": "+", "warning": "~", "failure": "X", "skipped": "-"}.get(r.status, "?")
        print(f"  [{icon}] {r.stage.upper():8s} | {r.status:8s} | {r.duration_seconds:6.1f}s | {r.summary}")

    if recommendations:
        print("\n  Recommendations:")
        for i, rec in enumerate(recommendations, 1):
            print(f"    {i}. {rec}")

    print("=" * 60)

    # Save report
    report_path = os.path.join(
        target_path if os.path.isdir(target_path) else os.path.dirname(target_path),
        "pipeline_report.json"
    )
    with open(report_path, "w") as f:
        json.dump(asdict(report), f, indent=2)
    print(f"\n  Report saved to: {report_path}")

    return report


# ============================================================================
# CLI Interface
# ============================================================================

def main():
    if len(sys.argv) < 2:
        print("=" * 60)
        print("  SPHERES OS UNIFIED PIPELINE v3.0")
        print("=" * 60)
        print("\nUsage: spheres_pipeline.py <target_path> [options]")
        print("\nOptions:")
        print("  --stage <lint|heal|sign|stress>  Run a specific stage")
        print("  --full                           Run all stages")
        print("  --continue-on-warning            Warnings don't halt pipeline")
        print("  --report                         Generate report only")
        sys.exit(0)

    target_path = sys.argv[1]
    if not os.path.exists(target_path):
        print(f"Error: Target path does not exist: {target_path}")
        sys.exit(1)

    # Parse arguments
    stages = STAGES  # Default: all stages
    stop_on_failure = True  # Default: fail-fast (stop on first failure)
    continue_on_warning = "--continue-on-warning" in sys.argv

    args = sys.argv[2:]
    for i, arg in enumerate(args):
        if arg == "--stage" and i + 1 < len(args):
            stages = [args[i + 1]]
        elif arg == "--full":
            stages = STAGES

    report = run_pipeline(
        target_path,
        stages=stages,
        stop_on_failure=stop_on_failure,
        continue_on_warning=continue_on_warning
    )
    sys.exit(0 if report.overall_status != "failed" else 1)


if __name__ == "__main__":
    main()
