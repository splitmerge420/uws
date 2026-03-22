#!/usr/bin/env python3
"""
janus_runner.py — Aluminum OS Janus v2 Multi-Agent Protocol Runner
Version: 2.0.0

Executable implementation of the Janus v2 spec (janus/JANUS_V2_SPEC.md).
Reads janus/janus_config.yaml for council configuration.

Integrates with:
  - toolchain/provenance.py   → GoldenTrace HITL provenance stamping
  - toolchain/invariant_linter.py → NPFM constitutional compliance scoring
  - toolchain/acp_governance.py   → Council policy registry

Commands:
  boot           Run the Janus boot sequence (probe models, emit heartbeat)
  route          Route a query through the council (auto-tier detection)
  heartbeat      Emit a heartbeat status trace
  council        List the current council members and their status
  invoke         Invoke a specific provider directly (bypass council)

Examples:
  python toolchain/janus_runner.py boot
  python toolchain/janus_runner.py route "explain async Rust" --tier 1
  python toolchain/janus_runner.py route "compare microservices vs monolith" --tier 2
  python toolchain/janus_runner.py heartbeat --json
  python toolchain/janus_runner.py council --list
  python toolchain/janus_runner.py invoke --provider claude --prompt "write a test"

Author: Aluminum OS / uws project
"""

from __future__ import annotations

import argparse
import hashlib
import json
import logging
import os
import sys
import time
from dataclasses import dataclass, field, asdict
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, List, Optional, Tuple

# ─── Provenance integration ───────────────────────────────────

# Add toolchain to path for sibling imports
_TOOLCHAIN = Path(__file__).parent
if str(_TOOLCHAIN) not in sys.path:
    sys.path.insert(0, str(_TOOLCHAIN))

try:
    from provenance import ProvenanceTrailer, GoldenTrace, NPFM_THRESHOLD
    _HAS_PROVENANCE = True
except ImportError:
    _HAS_PROVENANCE = False

# ─── Logging ──────────────────────────────────────────────────

logging.basicConfig(
    level=logging.WARNING,
    format="%(asctime)s [janus] %(levelname)s %(message)s",
    datefmt="%Y-%m-%dT%H:%M:%SZ",
)
logger = logging.getLogger("janus")

# ─── Constants ────────────────────────────────────────────────

REPO_ROOT = Path(__file__).parent.parent
CONFIG_PATH = REPO_ROOT / "janus" / "janus_config.yaml"
INV7_CAP = 0.47
NPFM_MIN = 0.70

# ─── Config loader ────────────────────────────────────────────

def load_config() -> dict:
    """Load janus_config.yaml. Falls back to built-in defaults if not found."""
    if CONFIG_PATH.exists():
        try:
            import yaml  # type: ignore
            with open(CONFIG_PATH) as f:
                return yaml.safe_load(f)
        except ImportError:
            pass  # yaml not installed; use defaults

    # Minimal built-in defaults (mirrors janus_config.yaml)
    return {
        "janus": {
            "version": "2.0.0",
            "default_tier": 1,
            "inv7_threshold": INV7_CAP,
            "npfm_threshold": NPFM_MIN,
            "ghost_seat_enabled": True,
            "models": {
                "claude":   {"role": "governance",  "weight": 1.0, "fallback": "gemini",  "model": "claude-opus-4-5",    "env_var": "ANTHROPIC_API_KEY"},
                "gemini":   {"role": "substrate",   "weight": 1.0, "fallback": "claude",  "model": "gemini-2.0-flash",   "env_var": "GEMINI_API_KEY"},
                "grok":     {"role": "adversarial", "weight": 0.8, "fallback": "deepseek","model": "grok-3-mini-fast",   "env_var": "XAI_API_KEY"},
                "deepseek": {"role": "research",    "weight": 0.7, "fallback": "gemini",  "model": "deepseek-chat",      "env_var": "DEEPSEEK_API_KEY"},
                "copilot":  {"role": "enterprise",  "weight": 0.7, "fallback": "claude",  "model": "gpt-4o",             "env_var": "GITHUB_TOKEN"},
            },
        }
    }


# ─── Data models ──────────────────────────────────────────────

@dataclass
class ModelStatus:
    name: str
    role: str
    weight: float
    available: bool
    degraded: bool = False
    error: Optional[str] = None
    latency_ms: float = 0.0

    @property
    def usable(self) -> bool:
        return self.available and not (self.degraded and self.latency_ms == 0)

    def to_dict(self) -> dict:
        return asdict(self)


@dataclass
class CouncilVote:
    provider: str
    role: str
    weight: float
    content: str
    is_dissent: bool = False
    latency_ms: float = 0.0


@dataclass
class JanusResult:
    tier: int
    prompt: str
    synthesised: str
    votes: List[CouncilVote]
    inv7_compliant: bool
    npfm_score: float
    trace: Optional["GoldenTrace"]
    seams: List[dict] = field(default_factory=list)
    safe_mode: bool = False
    error: Optional[str] = None

    def to_dict(self) -> dict:
        return {
            "tier": self.tier,
            "prompt": self.prompt,
            "synthesised": self.synthesised,
            "votes": [asdict(v) for v in self.votes],
            "inv7_compliant": self.inv7_compliant,
            "npfm_score": self.npfm_score,
            "trace": self.trace.to_dict() if self.trace else None,
            "seams": self.seams,
            "safe_mode": self.safe_mode,
            "error": self.error,
        }


# ─── NPFM heuristic scorer ────────────────────────────────────

def score_npfm(prompt: str) -> float:
    """Keyword heuristic NPFM score matching the Rust NpfmScorer."""
    lower = prompt.lower()
    score = 0.75

    positive = [
        "learn", "teach", "explain", "create", "build", "improve",
        "help", "collaborate", "write", "design", "analyze", "understand",
        "fix", "debug", "test", "document", "review",
    ]
    if any(kw in lower for kw in positive):
        score += 0.05

    busywork = [
        "generate boilerplate", "copy paste", "redundant", "bureaucratic",
        "busywork", "administrative overhead", "wrapper function",
    ]
    if any(kw in lower for kw in busywork):
        score -= 0.20

    extractive = ["scrape", "harvest", "bulk delete", "mass delete", "exfiltrate", "extract all"]
    if any(kw in lower for kw in extractive):
        score -= 0.15

    if len(prompt) < 10:
        score -= 0.10

    return max(0.0, min(1.0, score))


def infer_tier(prompt: str) -> int:
    """Infer query tier from prompt content, matching Rust QueryTier::infer."""
    lower = prompt.lower()
    tier3 = ["delete all", "shutdown", "terminate", "irreversible", "legal advice",
             "medical diagnosis", "financial risk", "destroy", "wipe", "permanently"]
    tier2 = ["compare", "evaluate", "tradeoff", "architecture", "consensus",
             "policy", "governance", "best approach", "pros and cons",
             "what should", "recommend", "multiple perspectives"]
    if any(kw in lower for kw in tier3):
        return 3
    if any(kw in lower for kw in tier2):
        return 2
    return 1


# ─── Provider caller ──────────────────────────────────────────

def call_provider(
    provider_name: str,
    model_cfg: dict,
    prompt: str,
    system: Optional[str] = None,
    timeout: int = 30,
) -> Tuple[str, float]:
    """
    Call a provider's API and return (content, latency_ms).
    Raises RuntimeError on failure.

    Uses the same OpenAI-compatible format for Grok, DeepSeek, and Copilot;
    Anthropic messages format for Claude; Google GenerateContent for Gemini.
    """
    import urllib.request
    import urllib.error

    api_key_env = model_cfg.get("env_var", "")
    api_key = os.environ.get(api_key_env, "")
    if not api_key:
        raise RuntimeError(f"No API key found in {api_key_env}")

    model_id = model_cfg.get("model", "")
    endpoint = model_cfg.get("endpoint", "")
    t0 = time.monotonic()

    if provider_name == "claude":
        body: dict = {
            "model": model_id,
            "max_tokens": 2048,
            "messages": [{"role": "user", "content": prompt}],
        }
        if system:
            body["system"] = system
        headers = {
            "Content-Type": "application/json",
            "x-api-key": api_key,
            "anthropic-version": "2023-06-01",
        }
    elif provider_name == "gemini":
        body = {"contents": [{"role": "user", "parts": [{"text": prompt}]}]}
        if system:
            body["systemInstruction"] = {"parts": [{"text": system}]}
        endpoint = f"{endpoint}?key={api_key}"
        headers = {"Content-Type": "application/json"}
        api_key = ""  # key is in URL for Gemini
    else:
        # OpenAI-compatible: Grok, DeepSeek, Copilot, OpenAI
        messages = []
        if system:
            messages.append({"role": "system", "content": system})
        messages.append({"role": "user", "content": prompt})
        body = {"model": model_id, "messages": messages}
        if provider_name == "copilot":
            headers = {
                "Content-Type": "application/json",
                "Authorization": f"token {api_key}",
            }
        else:
            headers = {
                "Content-Type": "application/json",
                "Authorization": f"Bearer {api_key}",
            }

    payload = json.dumps(body).encode()
    req = urllib.request.Request(endpoint, data=payload, headers=headers, method="POST")

    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            raw = json.loads(resp.read())
    except urllib.error.HTTPError as e:
        raise RuntimeError(f"HTTP {e.code}: {e.reason}") from e
    except Exception as e:
        raise RuntimeError(str(e)) from e

    latency = (time.monotonic() - t0) * 1000

    # Extract text content
    if provider_name == "claude":
        content = raw.get("content", [{}])[0].get("text", str(raw))
    elif provider_name == "gemini":
        try:
            content = raw["candidates"][0]["content"]["parts"][0]["text"]
        except (KeyError, IndexError):
            content = str(raw)
    else:
        try:
            content = raw["choices"][0]["message"]["content"]
        except (KeyError, IndexError):
            content = str(raw)

    return content, latency


# ─── INV-7 enforcement ────────────────────────────────────────

def enforce_inv7(weights: Dict[str, float]) -> Dict[str, float]:
    """
    Cap-and-redistribute INV-7 enforcement.
    Matches the Rust Inv7Guard::enforce algorithm exactly.
    """
    if not weights:
        return {}

    total = sum(weights.values())
    if total == 0:
        return weights

    result = {k: v / total for k, v in weights.items()}

    for _ in range(len(result) + 1):
        violators = [k for k, v in result.items() if v > INV7_CAP + 1e-12]
        if not violators:
            break

        for k in violators:
            result[k] = INV7_CAP

        pinned_total = len(violators) * INV7_CAP
        residual = 1.0 - pinned_total
        compliant_raw = sum(v for k, v in result.items() if k not in violators)

        if compliant_raw > 1e-12:
            scale = residual / compliant_raw
            for k in result:
                if k not in violators:
                    result[k] *= scale

    return result


def is_inv7_compliant(weights: Dict[str, float]) -> bool:
    total = sum(weights.values())
    if total == 0:
        return False
    return all(v / total <= INV7_CAP for v in weights.values())


# ─── Janus core ───────────────────────────────────────────────

class JanusRunner:
    """Python runtime for the Janus v2 multi-agent protocol."""

    def __init__(self, config: Optional[dict] = None, verbose: bool = False):
        self.config = config or load_config()
        self.janus_cfg = self.config.get("janus", {})
        self.models_cfg: dict = self.janus_cfg.get("models", {})
        self.verbose = verbose
        self.member_status: Dict[str, ModelStatus] = {}
        self.seams: List[dict] = []
        self.heartbeat_history: List[dict] = []
        self._boot_done = False

        if verbose:
            logger.setLevel(logging.DEBUG)

    def _available_providers(self) -> List[str]:
        """Return names of providers with API keys configured."""
        available = []
        for name, cfg in self.models_cfg.items():
            env = cfg.get("env_var", "")
            if os.environ.get(env):
                available.append(name)
        return available

    def probe(self) -> Dict[str, ModelStatus]:
        """Probe each configured provider for availability (heartbeat check)."""
        statuses: Dict[str, ModelStatus] = {}
        for name, cfg in self.models_cfg.items():
            env = cfg.get("env_var", "")
            has_key = bool(os.environ.get(env))
            statuses[name] = ModelStatus(
                name=name,
                role=cfg.get("role", "unknown"),
                weight=cfg.get("weight", 1.0),
                available=has_key,
            )
        self.member_status = statuses
        return statuses

    def boot(self) -> dict:
        """
        Execute the Janus boot sequence per janus/BOOT_SEQUENCE.md:
        1. Load constitutional invariants
        2. Initialize GoldenTrace emitter
        3. Probe models
        4. Verify INV-7 compliance
        5. Emit boot-complete heartbeat
        """
        boot_log = []

        # Step 1: Constitutional invariants (verified via presence of linter)
        linter_present = (REPO_ROOT / "toolchain" / "invariant_linter.py").exists()
        boot_log.append({
            "step": 1, "event": "boot_invariants_loaded",
            "linter_available": linter_present,
        })

        # Step 2: GoldenTrace emitter
        boot_log.append({
            "step": 2, "event": "trace_chain_initialized",
            "provenance_available": _HAS_PROVENANCE,
        })

        # Step 3: Probe models
        statuses = self.probe()
        boot_log.append({
            "step": 3, "event": "model_probe",
            "payload": {k: {"available": v.available, "role": v.role} for k, v in statuses.items()},
        })

        # Step 4: INV-7 compliance check
        usable_weights = {
            name: cfg["weight"]
            for name, cfg in self.models_cfg.items()
            if statuses.get(name, ModelStatus(name, "", 0.0, False)).available
        }
        inv7_ok = is_inv7_compliant(usable_weights) if len(usable_weights) > 1 else False
        boot_log.append({
            "step": 4, "event": "inv7_verified",
            "compliant": inv7_ok,
            "usable_models": list(usable_weights.keys()),
        })

        # Step 5: Boot-complete heartbeat
        hb = self.heartbeat()
        boot_log.append({"step": 6, "event": "boot_complete", "heartbeat": hb})

        self._boot_done = True
        return {"boot_sequence": boot_log, "status": "complete" if usable_weights else "safe_mode"}

    def heartbeat(self) -> dict:
        """Emit a structured heartbeat trace matching the Janus v2 spec format."""
        if not self.member_status:
            self.probe()

        available = [n for n, s in self.member_status.items() if s.available and not s.degraded]
        degraded  = [n for n, s in self.member_status.items() if s.available and s.degraded]
        offline   = [n for n, s in self.member_status.items() if not s.available]

        usable_weights = {
            n: self.models_cfg[n]["weight"]
            for n in available + degraded
        }
        consensus_ready = len(usable_weights) >= 2
        inv7_compliant = is_inv7_compliant(usable_weights) if len(usable_weights) > 1 else False

        hb = {
            "event_type": "action",
            "payload": {
                "type": "heartbeat",
                "models_available": available,
                "models_degraded": degraded,
                "models_offline": offline,
                "consensus_ready": consensus_ready,
                "inv7_compliant": inv7_compliant,
                "timestamp": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
            }
        }
        self.heartbeat_history.append(hb)
        return hb

    def route(
        self,
        prompt: str,
        tier_override: Optional[int] = None,
        timeout: int = 30,
    ) -> JanusResult:
        """
        Route a prompt through the Janus council.

        Tier detection is automatic unless tier_override is provided.
        NPFM check gates all requests before reaching any model.
        """
        if not self.member_status:
            self.probe()

        # NPFM gate
        npfm = score_npfm(prompt)
        threshold = self.janus_cfg.get("npfm_threshold", NPFM_MIN)
        if npfm < threshold:
            return JanusResult(
                tier=0, prompt=prompt, synthesised="",
                votes=[], inv7_compliant=False, npfm_score=npfm,
                trace=None,
                error=f"NPFM score {npfm:.2f} below threshold {threshold:.2f} — request denied",
            )

        tier = tier_override or infer_tier(prompt)

        if tier == 1:
            return self._route_tier1(prompt, npfm, timeout)
        else:
            return self._route_council(prompt, tier, npfm, timeout)

    def _route_tier1(self, prompt: str, npfm: float, timeout: int) -> JanusResult:
        """Route to the best single available model (Tier-1)."""
        # Prefer governance model (Claude)
        priority = ["claude", "gemini", "grok", "deepseek", "copilot"]
        for name in priority:
            if name not in self.member_status:
                continue
            if not self.member_status[name].available:
                continue
            cfg = self.models_cfg.get(name, {})
            try:
                content, latency = call_provider(name, cfg, prompt, timeout=timeout)
                trace = _make_trace(name, prompt, npfm) if _HAS_PROVENANCE else None
                return JanusResult(
                    tier=1, prompt=prompt, synthesised=content,
                    votes=[CouncilVote(name, cfg.get("role", "?"), 1.0, content, False, latency)],
                    inv7_compliant=True, npfm_score=npfm, trace=trace,
                )
            except RuntimeError as e:
                logger.warning(f"Tier-1: {name} failed ({e}), trying next")
                continue

        return JanusResult(
            tier=1, prompt=prompt, synthesised="",
            votes=[], inv7_compliant=False, npfm_score=npfm,
            trace=None, safe_mode=True,
            error="All Tier-1 providers unavailable or failed",
        )

    def _route_council(self, prompt: str, tier: int, npfm: float, timeout: int) -> JanusResult:
        """Multi-model council routing for Tier-2/3."""
        max_members = 3 if tier == 2 else 6

        # Determine participating members
        available = [
            (name, self.member_status[name])
            for name in self.models_cfg
            if name in self.member_status and self.member_status[name].available
        ][:max_members]

        if not available:
            return JanusResult(
                tier=tier, prompt=prompt, synthesised="",
                votes=[], inv7_compliant=False, npfm_score=npfm,
                trace=None, safe_mode=True,
                error="No council members available",
            )

        # Enforce INV-7
        raw_weights = {name: self.models_cfg[name]["weight"] for name, _ in available}
        norm_weights = enforce_inv7(raw_weights)
        inv7_ok = is_inv7_compliant(norm_weights)

        votes: List[CouncilVote] = []
        seams_this_round: List[dict] = []

        for name, status in available:
            cfg = self.models_cfg.get(name, {})
            role = cfg.get("role", "?")
            weight = norm_weights.get(name, 0.0)

            system = (
                f"You are the {name} member of the Aluminum OS Pantheon Council "
                f"(Janus v2 protocol). Your constitutional role: {role}. "
                f"INV-7: Your influence is capped at 47% of total consensus weight."
            )

            try:
                content, latency = call_provider(name, cfg, prompt, system=system, timeout=timeout)
            except RuntimeError as e:
                # Kintsugi repair
                fallback = cfg.get("fallback", "")
                seam = {
                    "failed": name, "repair": fallback,
                    "reason": str(e),
                    "ts": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
                }
                seams_this_round.append(seam)
                self.seams.append(seam)
                logger.warning(f"Council: {name} failed ({e}), attempting fallback {fallback}")

                if fallback and fallback in self.models_cfg:
                    fallback_cfg = self.models_cfg[fallback]
                    try:
                        content, latency = call_provider(fallback, fallback_cfg, prompt, system=system, timeout=timeout)
                        name = fallback  # Credit to repair provider
                    except RuntimeError:
                        continue
                else:
                    continue

            is_dissent = (role == "adversarial")
            votes.append(CouncilVote(name, role, weight, content, is_dissent, latency))

        if not votes:
            return JanusResult(
                tier=tier, prompt=prompt, synthesised="",
                votes=[], inv7_compliant=False, npfm_score=npfm,
                trace=None, safe_mode=True, seams=seams_this_round,
                error="All council members and fallbacks failed",
            )

        synthesised = _synthesise(votes)
        trace = _make_trace("council", prompt, npfm) if _HAS_PROVENANCE else None

        return JanusResult(
            tier=tier, prompt=prompt, synthesised=synthesised,
            votes=votes, inv7_compliant=inv7_ok, npfm_score=npfm,
            trace=trace, seams=seams_this_round,
        )

    def council_status(self) -> List[dict]:
        """Return the current council roster with status."""
        if not self.member_status:
            self.probe()
        return [
            {
                "name": name,
                "role": self.models_cfg.get(name, {}).get("role", "?"),
                "weight": self.models_cfg.get(name, {}).get("weight", 0.0),
                "model": self.models_cfg.get(name, {}).get("model", "?"),
                "available": self.member_status.get(name, ModelStatus(name, "", 0.0, False)).available,
                "env_var": self.models_cfg.get(name, {}).get("env_var", ""),
                "has_key": bool(os.environ.get(self.models_cfg.get(name, {}).get("env_var", ""), "")),
            }
            for name in self.models_cfg
        ]


# ─── Helpers ──────────────────────────────────────────────────

def _make_trace(provider: str, prompt: str, npfm: float) -> "GoldenTrace":
    """Create a GoldenTrace for a routing decision."""
    return ProvenanceTrailer.format(
        hitl_weight=0.90,
        provider=provider,
        npfm_score=npfm,
        prompt_context=prompt[:200],
    )


def _synthesise(votes: List[CouncilVote]) -> str:
    """Synthesise council votes into a single response with attribution headers."""
    if not votes:
        return ""
    if len(votes) == 1:
        return votes[0].content
    parts = []
    for v in votes:
        header = f"[{v.provider.title()} — {v.role.title()}]"
        if v.is_dissent:
            header += " ⚠ dissent"
        parts.append(f"{header}\n{v.content}")
    return "\n\n---\n\n".join(parts)


# ─── CLI ──────────────────────────────────────────────────────

def cmd_boot(args: argparse.Namespace, runner: JanusRunner) -> int:
    result = runner.boot()
    if args.json:
        print(json.dumps(result, indent=2))
    else:
        status = result.get("status", "?")
        icon = "✅" if status == "complete" else "⚠️"
        print(f"\n{icon}  Janus v2 Boot Sequence — {status.upper()}\n")
        for step in result.get("boot_sequence", []):
            event = step.get("event", "?")
            print(f"  [{step.get('step', '?')}] {event}")
        hb_data = result.get("boot_sequence", [{}])[-1].get("heartbeat", {})
        payload = hb_data.get("payload", {})
        print(f"\n  Models available: {payload.get('models_available', [])}")
        print(f"  Models offline:   {payload.get('models_offline', [])}")
        print(f"  Consensus ready:  {payload.get('consensus_ready', False)}")
        print(f"  INV-7 compliant:  {payload.get('inv7_compliant', False)}")
        print()
    return 0


def cmd_route(args: argparse.Namespace, runner: JanusRunner) -> int:
    tier = getattr(args, "tier", None)
    result = runner.route(args.prompt, tier_override=tier)
    if args.json:
        print(json.dumps(result.to_dict(), indent=2, default=str))
        return 0 if not result.error else 1

    icon = "✅" if not result.error else "❌"
    print(f"\n{icon}  Janus Route — Tier {result.tier}")
    print(f"  NPFM score: {result.npfm_score:.2f}")
    if result.error:
        print(f"  Error: {result.error}")
        return 1
    print(f"  Models consulted: {[v.provider for v in result.votes]}")
    print(f"  INV-7 compliant:  {result.inv7_compliant}")
    if result.seams:
        print(f"  Kintsugi seams:   {len(result.seams)} repair(s) applied")
    if result.trace and _HAS_PROVENANCE:
        print(f"\n  {result.trace.to_trailer_string()}\n")
    print(f"\n  Response:\n{'─'*60}")
    print(result.synthesised)
    print('─'*60)
    return 0


def cmd_heartbeat(args: argparse.Namespace, runner: JanusRunner) -> int:
    hb = runner.heartbeat()
    if args.json:
        print(json.dumps(hb, indent=2))
    else:
        p = hb["payload"]
        icon = "✅" if p["consensus_ready"] else "⚠️"
        print(f"\n{icon}  Janus Heartbeat — {p['timestamp']}")
        print(f"  Available:  {p['models_available']}")
        if p["models_degraded"]:
            print(f"  Degraded:   {p['models_degraded']}")
        if p["models_offline"]:
            print(f"  Offline:    {p['models_offline']}")
        print(f"  INV-7 OK:   {p['inv7_compliant']}")
        print()
    return 0


def cmd_council(args: argparse.Namespace, runner: JanusRunner) -> int:
    members = runner.council_status()
    if args.json:
        print(json.dumps(members, indent=2))
    else:
        print(f"\n  {'NAME':<12} {'ROLE':<14} {'WT':>4}  {'MODEL':<22} {'STATUS'}")
        print("  " + "─"*70)
        for m in members:
            status = "✅ available" if m["has_key"] else "⚠️  no key"
            print(f"  {m['name']:<12} {m['role']:<14} {m['weight']:>4.1f}  {m['model']:<22} {status}")
        print()
    return 0


def cmd_invoke(args: argparse.Namespace, runner: JanusRunner) -> int:
    if not runner.member_status:
        runner.probe()
    cfg = runner.models_cfg.get(args.provider)
    if not cfg:
        print(f"ERROR: Unknown provider '{args.provider}'", file=sys.stderr)
        return 1
    try:
        content, latency = call_provider(args.provider, cfg, args.prompt)
        if args.json:
            print(json.dumps({"provider": args.provider, "content": content, "latency_ms": latency}))
        else:
            print(f"\n[{args.provider}] ({latency:.0f}ms)\n{content}\n")
        return 0
    except RuntimeError as e:
        print(f"ERROR: {e}", file=sys.stderr)
        return 1


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Janus v2 — Aluminum OS Multi-Agent Protocol Runner",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument("--json", action="store_true", help="Output JSON")
    parser.add_argument("--verbose", action="store_true", help="Verbose logging")
    sub = parser.add_subparsers(dest="command", required=True)

    sub.add_parser("boot", help="Run Janus boot sequence")

    p_route = sub.add_parser("route", help="Route a prompt through the council")
    p_route.add_argument("prompt", help="Query to route")
    p_route.add_argument("--tier", type=int, choices=[1, 2, 3], help="Override tier detection")

    sub.add_parser("heartbeat", help="Emit a heartbeat status trace")

    p_council = sub.add_parser("council", help="Show council status")
    p_council.add_argument("--list", action="store_true", default=True)

    p_invoke = sub.add_parser("invoke", help="Invoke a single provider directly")
    p_invoke.add_argument("--provider", required=True)
    p_invoke.add_argument("--prompt", required=True)

    args = parser.parse_args()

    runner = JanusRunner(verbose=args.verbose)

    dispatch = {
        "boot": cmd_boot,
        "route": cmd_route,
        "heartbeat": cmd_heartbeat,
        "council": cmd_council,
        "invoke": cmd_invoke,
    }

    fn = dispatch.get(args.command)
    if fn:
        return fn(args, runner)

    parser.print_help()
    return 1


if __name__ == "__main__":
    sys.exit(main())
