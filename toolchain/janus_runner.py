#!/usr/bin/env python3
"""
toolchain/janus_runner.py
Aluminum OS — Janus v2 Python CLI Runner

Provides a lightweight Python interface to the Janus v2 multi-agent
protocol for use in toolchain scripts, CI pipelines, and local testing.

Usage:
    python toolchain/janus_runner.py boot
    python toolchain/janus_runner.py route "list my emails" --tier 1
    python toolchain/janus_runner.py route "analyze quarterly report" --tier 2
    python toolchain/janus_runner.py heartbeat
    python toolchain/janus_runner.py status

Spec: janus/JANUS_V2_SPEC.md
Council Session: 2026-03-20
"""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
import time
from dataclasses import dataclass, field
from enum import Enum
from pathlib import Path
from typing import Optional


# ── Constants ────────────────────────────────────────────────────────────────

GENESIS_DIGEST = "0" * 64
CONFIG_PATH = Path(__file__).parent.parent / "janus" / "janus_config.yaml"
INV7_THRESHOLD = 0.47


# ── Enums ────────────────────────────────────────────────────────────────────

class ModelStatus(str, Enum):
    AVAILABLE = "available"
    DEGRADED = "degraded"
    OFFLINE = "offline"


class ModelRole(str, Enum):
    GOVERNANCE = "governance"
    SUBSTRATE = "substrate"
    ADVERSARIAL = "adversarial"
    RESEARCH = "research"
    ENTERPRISE = "enterprise"


class QueryTier(int, Enum):
    TIER1 = 1
    TIER2 = 2
    TIER3 = 3


class TraceEventType(str, Enum):
    ACTION = "action"
    INVARIANT_CHECK = "invariant_check"
    COUNCIL_VOTE = "council_vote"
    HUMAN_OVERRIDE = "human_override"
    GOLDEN_SEAM = "golden_seam"


class TraceSeverity(str, Enum):
    INFO = "info"
    WARNING = "warning"
    ERROR = "error"
    GOLDEN = "golden"


# ── Digest Helper ─────────────────────────────────────────────────────────────

def compute_digest(data: str | bytes) -> str:
    """Return the SHA-256 hex digest of *data*.

    Uses Python's built-in hashlib so the digests produced here differ from
    the FNV-1a digests produced in the Rust implementation; both are
    deterministic 64-character hex strings suitable for chaining.
    """
    if isinstance(data, str):
        data = data.encode("utf-8")
    return hashlib.sha256(data).hexdigest()


# ── Model Entry ───────────────────────────────────────────────────────────────

@dataclass
class ModelEntry:
    name: str
    role: ModelRole
    weight: float
    fallback: Optional[str]
    status: ModelStatus = ModelStatus.AVAILABLE


# ── GoldenTrace ───────────────────────────────────────────────────────────────

@dataclass
class GoldenTrace:
    index: int
    timestamp_secs: int
    event_type: TraceEventType
    severity: TraceSeverity
    actor: Optional[str]
    invariants: list[str]
    payload: dict[str, str]
    digest: str
    prev_digest: str

    def to_dict(self) -> dict:
        return {
            "index": self.index,
            "ts": self.timestamp_secs,
            "type": self.event_type.value,
            "severity": self.severity.value,
            "actor": self.actor,
            "invariants": self.invariants,
            "payload": self.payload,
            "digest": self.digest[:8],
            "prev": self.prev_digest[:8],
        }

    def to_json_line(self) -> str:
        return json.dumps(self.to_dict(), separators=(",", ":"))


# ── Janus Router ─────────────────────────────────────────────────────────────

class JanusRouter:
    """Pure-Python implementation of the Janus v2 routing protocol.

    Mirrors the Rust ``JanusRouter`` in ``src/universal/janus.rs``.
    """

    def __init__(self) -> None:
        ts = int(time.time())
        self.session_id = compute_digest(f"janus-session-{ts}")
        self._traces: list[GoldenTrace] = []
        self.safe_mode = False
        self.inv7_threshold = INV7_THRESHOLD
        self._models: list[ModelEntry] = [
            ModelEntry("claude",   ModelRole.GOVERNANCE,  1.0, "gemini"),
            ModelEntry("gemini",   ModelRole.SUBSTRATE,   1.0, "claude"),
            ModelEntry("grok",     ModelRole.ADVERSARIAL, 0.8, "deepseek"),
            ModelEntry("deepseek", ModelRole.RESEARCH,    0.7, "gemini"),
            ModelEntry("copilot",  ModelRole.ENTERPRISE,  0.7, "claude"),
        ]

    # ── Public API ─────────────────────────────────────────────

    def boot(self) -> list[GoldenTrace]:
        """Run the Janus boot sequence and return all emitted traces."""
        # Step 1
        self._emit(TraceEventType.ACTION, TraceSeverity.INFO, None, [],
                   {"type": "boot_invariants_loaded", "count": "39"})
        # Step 2
        self._emit(TraceEventType.ACTION, TraceSeverity.INFO, None, [],
                   {"type": "trace_chain_initialized",
                    "session_id": self.session_id})
        # Step 3 — probe models
        for m in self._models:
            self._emit(TraceEventType.ACTION, TraceSeverity.INFO,
                       m.name, [],
                       {"type": "model_probe", "model": m.name,
                        "status": m.status.value})
        # Step 4 — INV-7 check
        available = self._available()
        inv7_ok = len(available) >= 2
        if not inv7_ok:
            self.safe_mode = True
        self._emit(
            TraceEventType.INVARIANT_CHECK,
            TraceSeverity.INFO if inv7_ok else TraceSeverity.WARNING,
            None, ["INV-7"],
            {"available_count": str(len(available)),
             "inv7_compliant": str(inv7_ok).lower(),
             "safe_mode": str(self.safe_mode).lower()},
        )
        # Step 5 — boot complete
        self._emit_heartbeat("boot_complete")
        return list(self._traces)

    def route(self, query: str, tier: QueryTier) -> dict:
        """Route *query* at the given *tier*.

        Returns a dict with keys: tier, models, trace, safe_mode.
        Raises RuntimeError if no models are available.
        """
        query_digest = compute_digest(query)
        models, effective_tier, safe_mode = self._select(tier)

        payload: dict[str, str] = {
            "tier": f"tier{effective_tier.value}",
            "models": ",".join(m.name for m in models),
            "query_digest": query_digest[:16],
            "safe_mode": str(safe_mode).lower(),
        }
        if effective_tier == QueryTier.TIER3:
            payload["human_override_required"] = "true"

        actor = models[0].name if models else None
        trace = self._emit(
            TraceEventType.ACTION, TraceSeverity.INFO,
            actor, ["INV-7"], payload,
        )
        return {
            "tier": f"tier{effective_tier.value}",
            "models": [m.name for m in models],
            "trace": trace.to_dict(),
            "safe_mode": safe_mode,
        }

    def record_failure(self, model_name: str, reason: str) -> GoldenTrace:
        """Mark *model_name* offline and emit a Kintsugi golden-seam trace."""
        for m in self._models:
            if m.name == model_name:
                m.status = ModelStatus.OFFLINE
                break

        fallback = next(
            (m.name for m in self._models
             if m.name != model_name and m.status == ModelStatus.AVAILABLE),
            "none",
        )
        return self._emit(
            TraceEventType.GOLDEN_SEAM, TraceSeverity.GOLDEN,
            model_name, ["INV-35"],
            {"failed_model": model_name, "reason": reason,
             "repair": "fallback_model_selected", "fallback": fallback},
        )

    def heartbeat(self) -> GoldenTrace:
        """Emit a periodic heartbeat trace."""
        return self._emit_heartbeat("heartbeat")

    def status(self) -> dict:
        """Return the current council status snapshot."""
        return {m.name: m.status.value for m in self._models}

    def traces(self) -> list[GoldenTrace]:
        return list(self._traces)

    # ── Internal helpers ────────────────────────────────────────

    def _available(self) -> list[ModelEntry]:
        return [m for m in self._models if m.status == ModelStatus.AVAILABLE]

    def _violates_inv7(self, models: list[ModelEntry]) -> bool:
        total = sum(m.weight for m in models)
        if total == 0.0:
            return False
        return any(m.weight / total > self.inv7_threshold for m in models)

    def _select(
        self, tier: QueryTier
    ) -> tuple[list[ModelEntry], QueryTier, bool]:
        available = self._available()

        def tier3_models() -> list[ModelEntry]:
            if len(available) >= 2 and not self._violates_inv7(available):
                return available
            return []

        def tier2_models() -> list[ModelEntry]:
            roles = [ModelRole.GOVERNANCE, ModelRole.SUBSTRATE,
                     ModelRole.ADVERSARIAL]
            selected: list[ModelEntry] = []
            for role in roles:
                m = next((x for x in available if x.role == role), None)
                if m and m not in selected:
                    selected.append(m)
                    if len(selected) == 3:
                        break
            # Fill to at least 2
            for m in available:
                if len(selected) >= 2:
                    break
                if m not in selected:
                    selected.append(m)
            if len(selected) < 2 or self._violates_inv7(selected):
                return []
            return selected

        def tier1_model() -> list[ModelEntry]:
            gov = next(
                (m for m in available if m.role == ModelRole.GOVERNANCE), None
            )
            sub = next(
                (m for m in available if m.role == ModelRole.SUBSTRATE), None
            )
            chosen = gov or sub or (available[0] if available else None)
            return [chosen] if chosen else []

        if tier == QueryTier.TIER3:
            m3 = tier3_models()
            if m3:
                return m3, QueryTier.TIER3, self.safe_mode
            m2 = tier2_models()
            if m2:
                return m2, QueryTier.TIER2, True
            m1 = tier1_model()
            if m1:
                return m1, QueryTier.TIER1, True
            raise RuntimeError("No models available — cannot route query")

        if tier == QueryTier.TIER2:
            m2 = tier2_models()
            if m2:
                return m2, QueryTier.TIER2, self.safe_mode
            m1 = tier1_model()
            if m1:
                return m1, QueryTier.TIER1, True
            raise RuntimeError("No models available — cannot route query")

        # TIER1
        m1 = tier1_model()
        if m1:
            return m1, QueryTier.TIER1, self.safe_mode
        raise RuntimeError("No models available — cannot route query")

    def _emit(
        self,
        event_type: TraceEventType,
        severity: TraceSeverity,
        actor: Optional[str],
        invariants: list[str],
        payload: dict[str, str],
    ) -> GoldenTrace:
        index = len(self._traces)
        prev_digest = (
            self._traces[-1].digest if self._traces else GENESIS_DIGEST
        )
        content = (
            f"{index}{prev_digest}{event_type.value}"
            + ";".join(f"{k}={v}" for k, v in sorted(payload.items()))
        )
        digest = compute_digest(content)
        trace = GoldenTrace(
            index=index,
            timestamp_secs=int(time.time()),
            event_type=event_type,
            severity=severity,
            actor=actor,
            invariants=invariants,
            payload=payload,
            digest=digest,
            prev_digest=prev_digest,
        )
        self._traces.append(trace)
        return trace

    def _emit_heartbeat(self, heartbeat_type: str) -> GoldenTrace:
        snap = self.status()
        available = [k for k, v in snap.items() if v == "available"]
        degraded = [k for k, v in snap.items() if v == "degraded"]
        offline = [k for k, v in snap.items() if v == "offline"]
        return self._emit(
            TraceEventType.ACTION, TraceSeverity.INFO, None, ["INV-7"],
            {
                "type": heartbeat_type,
                "models_available": ",".join(available),
                "models_degraded": ",".join(degraded),
                "models_offline": ",".join(offline),
                "consensus_ready": str(len(available) >= 2).lower(),
                "inv7_compliant": str(not self.safe_mode).lower(),
                "safe_mode": str(self.safe_mode).lower(),
                "session_id": self.session_id[:16],
            },
        )


# ── CLI ───────────────────────────────────────────────────────────────────────

def cmd_boot(_args: argparse.Namespace) -> int:
    router = JanusRouter()
    traces = router.boot()
    for t in traces:
        print(t.to_json_line())
    return 0


def cmd_route(args: argparse.Namespace) -> int:
    router = JanusRouter()
    router.boot()
    tier = QueryTier(args.tier)
    try:
        result = router.route(args.query, tier)
    except RuntimeError as exc:
        print(json.dumps({"error": str(exc)}), file=sys.stderr)
        return 1
    print(json.dumps(result, indent=2))
    return 0


def cmd_heartbeat(_args: argparse.Namespace) -> int:
    router = JanusRouter()
    router.boot()
    trace = router.heartbeat()
    print(trace.to_json_line())
    return 0


def cmd_status(_args: argparse.Namespace) -> int:
    router = JanusRouter()
    print(json.dumps(router.status(), indent=2))
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="janus_runner",
        description="Janus v2 — Constitutional Multi-Agent Protocol CLI",
    )
    sub = parser.add_subparsers(dest="command", required=True)

    sub.add_parser("boot", help="Run the Janus boot sequence")

    route_p = sub.add_parser("route", help="Route a query through the council")
    route_p.add_argument("query", help="Query text to route")
    route_p.add_argument(
        "--tier", type=int, choices=[1, 2, 3], default=1,
        help="Routing tier (1=simple, 2=complex, 3=critical)",
    )

    sub.add_parser("heartbeat", help="Emit a heartbeat trace")
    sub.add_parser("status", help="Show current council model status")

    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)

    dispatch = {
        "boot": cmd_boot,
        "route": cmd_route,
        "heartbeat": cmd_heartbeat,
        "status": cmd_status,
    }
    handler = dispatch.get(args.command)
    if handler is None:
        parser.print_help()
        return 1
    return handler(args)


if __name__ == "__main__":
    sys.exit(main())
