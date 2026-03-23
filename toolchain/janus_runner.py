#!/usr/bin/env python3
# toolchain/janus_runner.py — Janus v2 Python Orchestration Runner
#
# Provides a lightweight Python orchestration layer that reads janus_config.yaml,
# dispatches queries to council members via their respective APIs, collects votes,
# enforces INV-7, and emits GoldenTrace events.
#
# This runner is used for:
#   - Local development / debugging (bypasses the Rust CLI)
#   - CI integration tests (validates INV-7 compliance end-to-end)
#   - Python-based AI agent frameworks (LangChain, CrewAI, AutoGen, etc.)
#
# Usage:
#   python3 toolchain/janus_runner.py --query "What is 2+2?" --tier 1
#   python3 toolchain/janus_runner.py --query "..." --tier 2 --config janus/janus_config.yaml
#   python3 toolchain/janus_runner.py --heartbeat
#
# Requires: pyyaml (pip install pyyaml)
# Optional for live API calls: openai, anthropic, google-generativeai

from __future__ import annotations

import argparse
import fnmatch
import hashlib
import json
import os
import sys
import time
from dataclasses import asdict, dataclass, field
from pathlib import Path
from typing import Any

# ── Config loading ─────────────────────────────────────────────────────────────

DEFAULT_CONFIG = Path(__file__).parent.parent / "janus" / "janus_config.yaml"


def load_config(path: Path | None = None) -> dict[str, Any]:
    """Load and validate janus_config.yaml. Falls back to embedded defaults."""
    target = path or DEFAULT_CONFIG
    try:
        import yaml  # type: ignore
        with open(target) as f:
            cfg = yaml.safe_load(f)
        return cfg["janus"]
    except ImportError:
        _warn("pyyaml not installed — using hardcoded defaults")
    except FileNotFoundError:
        _warn(f"Config not found at {target} — using hardcoded defaults")
    # Hardcoded fallback
    return {
        "version": "2.0.0",
        "default_tier": 1,
        "inv7_threshold": 0.47,
        "ghost_seat_enabled": True,
        "models": {
            "claude":   {"role": "governance",  "weight": 1.0, "fallback": "gemini"},
            "gemini":   {"role": "substrate",   "weight": 1.0, "fallback": "claude"},
            "grok":     {"role": "adversarial", "weight": 0.8, "fallback": "deepseek"},
            "deepseek": {"role": "research",    "weight": 0.7, "fallback": "gemini"},
            "copilot":  {"role": "enterprise",  "weight": 0.7, "fallback": "claude"},
        },
        "kintsugi": {
            "decay_factor": 0.8,
            "min_reliability": 0.1,
        },
    }


# ── Digest ────────────────────────────────────────────────────────────────────

def compute_digest(s: str) -> str:
    """SHA-256 digest of a string — used for round/trace IDs."""
    return hashlib.sha256(s.encode()).hexdigest()[:16]


# ── Data classes ─────────────────────────────────────────────────────────────

@dataclass
class ModelState:
    id: str
    role: str
    weight: float
    fallback: str | None
    reliability: float = 1.0
    available: bool = True

    @property
    def effective_weight(self) -> float:
        return self.weight * self.reliability


@dataclass
class ModelVote:
    model: str
    answer: str
    confidence: float
    weighted: float


@dataclass
class GoldenTrace:
    trace_id: str
    round_id: str
    kind: str          # action | council_vote | council_consensus | human_override | kintsugi_repair | heartbeat
    model: str
    summary: str
    inv7_ok: bool
    timestamp: str = field(default_factory=lambda: time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()))


@dataclass
class JanusResult:
    round_id: str
    tier: int
    primary_model: str
    answer: str
    votes: list[ModelVote]
    inv7_ok: bool
    human_signoff: bool
    traces: list[GoldenTrace]


# ── Janus Runner ──────────────────────────────────────────────────────────────

class JanusRunner:
    """Python implementation of the Janus v2 routing protocol."""

    def __init__(self, config: dict[str, Any] | None = None) -> None:
        self.cfg = config or load_config()
        self.inv7_threshold: float = self.cfg.get("inv7_threshold", 0.47)
        self.ghost_seat_enabled: bool = self.cfg.get("ghost_seat_enabled", True)
        self._round_counter = 0
        self._traces: list[GoldenTrace] = []
        self._models: dict[str, ModelState] = {}

        for mid, mcfg in self.cfg.get("models", {}).items():
            self._models[mid] = ModelState(
                id=mid,
                role=mcfg.get("role", "unknown"),
                weight=float(mcfg.get("weight", 1.0)),
                fallback=mcfg.get("fallback"),
            )

    # ── Routing ───────────────────────────────────────────────────────────────

    def route(self, query: str, tier: int = 1) -> JanusResult:
        """Route a query to the council at the specified tier."""
        self._round_counter += 1
        round_id = compute_digest(f"round-{self._round_counter}-{query}")
        query_digest = compute_digest(query)

        if tier == 1:
            return self._route_tier1(round_id, query, query_digest)
        elif tier == 2:
            return self._route_tier2(round_id, query, query_digest)
        else:
            return self._route_tier3(round_id, query, query_digest)

    def _route_tier1(self, round_id: str, query: str, query_digest: str) -> JanusResult:
        primary, inv7_ok = self._select_primary(query_digest)
        answer = f"[{primary}] {query}" if primary != "none" else "[Janus: degraded]"

        trace = self._make_trace(round_id, "action", primary,
                                  f"Tier1 route: {query[:60]} → {primary}", inv7_ok)
        self._traces.append(trace)

        return JanusResult(
            round_id=round_id, tier=1, primary_model=primary,
            answer=answer, votes=[], inv7_ok=inv7_ok,
            human_signoff=False, traces=[trace],
        )

    def _route_tier2(self, round_id: str, query: str, query_digest: str) -> JanusResult:
        votes = self._collect_votes(round_id, query, query_digest)
        inv7_ok = self._check_inv7(votes)

        traces: list[GoldenTrace] = []
        for v in votes:
            t = self._make_trace(round_id, "council_vote", v.model,
                                  f"vote confidence={v.confidence:.2f}", inv7_ok)
            traces.append(t)

        primary, answer = self._synthesise(votes, query)
        consensus_t = self._make_trace(round_id, "council_consensus", primary,
                                        f"Tier2 consensus from {len(votes)} votes", inv7_ok)
        traces.append(consensus_t)
        self._traces.extend(traces)

        return JanusResult(
            round_id=round_id, tier=2, primary_model=primary,
            answer=answer, votes=votes, inv7_ok=inv7_ok,
            human_signoff=False, traces=traces,
        )

    def _route_tier3(self, round_id: str, query: str, query_digest: str) -> JanusResult:
        result = self._route_tier2(round_id, query, query_digest)
        override_t = self._make_trace(round_id, "human_override", "human",
                                       "Tier3 human sign-off obtained (stub)", result.inv7_ok)
        result.traces.append(override_t)
        self._traces.append(override_t)
        result.tier = 3
        result.human_signoff = True
        return result

    # ── Vote collection ───────────────────────────────────────────────────────

    def _collect_votes(self, round_id: str, query: str, query_digest: str) -> list[ModelVote]:
        """Collect deterministic stub votes (replace with live API calls)."""
        votes = []
        for m in self._available_models():
            raw = f"{m.id}-{query_digest}"
            h = int(hashlib.sha256(raw.encode()).hexdigest(), 16)
            confidence = 0.5 + (h % 1000) / 2000.0
            votes.append(ModelVote(
                model=m.id,
                answer=f"[{m.id}] {query}",
                confidence=confidence,
                weighted=confidence * m.effective_weight,
            ))
        return votes

    def _call_model_api(self, model_id: str, query: str) -> str:
        """
        Call a live model API. Currently a stub — extend for production use.

        To enable live calls, set the API key env var for each model
        (ANTHROPIC_API_KEY, GEMINI_API_KEY, etc.) and uncomment the
        relevant SDK calls below.
        """
        cfg = self.cfg["models"].get(model_id, {})
        api_env = cfg.get("api_env", "")
        api_key = os.environ.get(api_env, "")
        if not api_key:
            return f"[{model_id}] stub response — set {api_env} for live calls"

        # ── Anthropic (Claude) ────────────────────────────────────────────────
        # import anthropic
        # client = anthropic.Anthropic(api_key=api_key)
        # msg = client.messages.create(model=cfg["model_id"], max_tokens=1024,
        #                              messages=[{"role":"user","content":query}])
        # return msg.content[0].text

        # ── OpenAI-compatible (Copilot, DeepSeek, Grok) ───────────────────────
        # import openai
        # client = openai.OpenAI(api_key=api_key, base_url=cfg["endpoint"])
        # resp = client.chat.completions.create(
        #     model=cfg["model_id"], messages=[{"role":"user","content":query}])
        # return resp.choices[0].message.content

        return f"[{model_id}] stub response (api_key present but SDK calls disabled)"

    # ── Helpers ───────────────────────────────────────────────────────────────

    def _available_models(self) -> list[ModelState]:
        return [m for m in self._models.values() if m.available]

    def _total_weight(self) -> float:
        return sum(m.effective_weight for m in self._available_models())

    def _select_primary(self, query_digest: str) -> tuple[str, bool]:
        available = sorted(self._available_models(),
                           key=lambda m: m.effective_weight, reverse=True)
        if not available:
            return "none", False
        total = self._total_weight()
        for m in available:
            share = m.effective_weight / total if total > 0 else 0.0
            if share <= self.inv7_threshold:
                return m.id, True
        return available[0].id, False  # degraded — violation flagged

    def _synthesise(self, votes: list[ModelVote], query: str) -> tuple[str, str]:
        if not votes:
            return "none", "[Janus: no council members available]"
        winner = max(votes, key=lambda v: v.weighted)
        return winner.model, winner.answer

    def _check_inv7(self, votes: list[ModelVote]) -> bool:
        total = sum(v.weighted for v in votes)
        if total == 0.0:
            return False
        return all(v.weighted / total <= self.inv7_threshold for v in votes)

    def _make_trace(self, round_id: str, kind: str, model: str,
                    summary: str, inv7_ok: bool) -> GoldenTrace:
        raw = f"{round_id}-{kind}-{model}"
        trace_id = compute_digest(raw)
        return GoldenTrace(
            trace_id=trace_id,
            round_id=round_id,
            kind=kind,
            model=model,
            summary=summary,
            inv7_ok=inv7_ok,
        )

    # ── Kintsugi ─────────────────────────────────────────────────────────────

    def kintsugi_repair(self, failed_model: str, reason: str) -> dict[str, Any]:
        """Mark a model as failed and perform Kintsugi repair."""
        m = self._models.get(failed_model)
        if m is None:
            return {"error": f"Unknown model: {failed_model}"}

        kintsugi_cfg = self.cfg.get("kintsugi", {})
        decay = float(kintsugi_cfg.get("decay_factor", 0.8))
        floor = float(kintsugi_cfg.get("min_reliability", 0.1))
        new_reliability = max(m.reliability * decay, floor)

        m.reliability = new_reliability
        m.available = False

        fallback_id = m.fallback or "none"
        fallback_available = (
            fallback_id != "none"
            and self._models.get(fallback_id, ModelState("", "", 0, None)).available
        )
        beauty = (
            kintsugi_cfg.get("beauty_with_fallback", 0.9) if fallback_available
            else kintsugi_cfg.get("beauty_without_fallback", 0.2)
        )

        trace = self._make_trace(
            f"kintsugi-{failed_model}",
            "kintsugi_repair",
            failed_model,
            f"failure: {reason} → repaired by {fallback_id}, "
            f"reliability → {new_reliability:.2f}",
            True,
        )
        self._traces.append(trace)

        return {
            "failed_model": failed_model,
            "repair_model": fallback_id,
            "failure_reason": reason,
            "beauty_score": beauty,
            "new_reliability": new_reliability,
        }

    # ── Heartbeat ─────────────────────────────────────────────────────────────

    def heartbeat(self) -> dict[str, Any]:
        """Emit a Janus heartbeat trace."""
        available   = [m.id for m in self._models.values() if m.available and m.reliability >= 0.5]
        degraded    = [m.id for m in self._models.values() if m.available and m.reliability < 0.5]
        offline     = [m.id for m in self._models.values() if not m.available]
        total       = self._total_weight()
        inv7_ok     = total == 0 or all(
            m.effective_weight / total <= self.inv7_threshold
            for m in self._available_models()
        )

        hb = {
            "event_type": "action",
            "payload": {
                "type": "heartbeat",
                "models_available": available,
                "models_degraded": degraded,
                "models_offline": offline,
                "consensus_ready": len(available) >= 2,
                "inv7_compliant": inv7_ok,
            },
        }
        trace = self._make_trace("heartbeat", "heartbeat", "", json.dumps(hb["payload"]), inv7_ok)
        self._traces.append(trace)
        return hb

    # ── Trace log ─────────────────────────────────────────────────────────────

    def traces(self) -> list[dict[str, Any]]:
        return [asdict(t) for t in self._traces]


# ── CLI entry point ───────────────────────────────────────────────────────────

def _warn(msg: str) -> None:
    print(f"[janus_runner] WARNING: {msg}", file=sys.stderr)


def main() -> None:
    parser = argparse.ArgumentParser(
        prog="janus_runner",
        description="Janus v2 — Constitutional Multi-Agent Runner",
    )
    parser.add_argument("--query", "-q", help="Query to route through the council")
    parser.add_argument("--tier", "-t", type=int, choices=[1, 2, 3], default=1,
                        help="Routing tier (1=simple, 2=complex, 3=critical)")
    parser.add_argument("--heartbeat", action="store_true", help="Emit a heartbeat trace")
    parser.add_argument("--config", "-c", type=Path, default=None,
                        help="Path to janus_config.yaml")
    parser.add_argument("--output", "-o", choices=["json", "pretty"], default="pretty",
                        help="Output format")
    args = parser.parse_args()

    cfg = load_config(args.config)
    runner = JanusRunner(cfg)

    if args.heartbeat:
        result = runner.heartbeat()
        _print(result, args.output)
        return

    if not args.query:
        parser.print_help()
        sys.exit(1)

    result = runner.route(args.query, tier=args.tier)
    output = {
        "round_id": result.round_id,
        "tier": result.tier,
        "primary_model": result.primary_model,
        "answer": result.answer,
        "inv7_ok": result.inv7_ok,
        "human_signoff": result.human_signoff,
        "votes": [asdict(v) for v in result.votes],
        "traces": runner.traces(),
    }
    _print(output, args.output)


def _print(data: Any, fmt: str) -> None:
    if fmt == "json":
        print(json.dumps(data))
    else:
        print(json.dumps(data, indent=2))


if __name__ == "__main__":
    main()
