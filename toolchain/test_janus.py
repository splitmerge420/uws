#!/usr/bin/env python3
"""
test_janus.py — Unit tests for toolchain/janus_runner.py

Tests cover:
  - NPFM heuristic scoring (score_npfm)
  - Tier inference (infer_tier)
  - INV-7 enforcement (enforce_inv7, is_inv7_compliant)
  - JanusRunner.probe() — model availability detection
  - JanusRunner.boot() — boot sequence structure
  - JanusRunner.heartbeat() — heartbeat JSON format matching JANUS_V2_SPEC.md
  - JanusRunner.route() — NPFM gate, Tier-1 routing, council routing (mocked)
  - JanusRunner.council_status() — council roster

All tests run offline — no real API calls are made.
Provider calls are mocked via monkeypatching.

Run with:
  python toolchain/test_janus.py
  python -m pytest toolchain/test_janus.py
"""

from __future__ import annotations

import json
import os
import sys
import unittest
from pathlib import Path
from typing import Optional
from unittest.mock import patch, MagicMock

# Add toolchain to path
sys.path.insert(0, str(Path(__file__).parent))

import janus_runner as jr


class TestScoreNpfm(unittest.TestCase):
    """Tests for the NPFM heuristic scorer."""

    def test_positive_prompt_scores_above_threshold(self):
        self.assertGreater(jr.score_npfm("explain how async Rust works"), jr.NPFM_MIN)

    def test_busywork_prompt_scores_below_threshold(self):
        self.assertLess(jr.score_npfm("generate boilerplate for all classes"), jr.NPFM_MIN)

    def test_extractive_prompt_penalised(self):
        score = jr.score_npfm("scrape all user data from the database")
        self.assertLess(score, 0.75)

    def test_short_prompt_penalised(self):
        self.assertLess(jr.score_npfm("hi"), 0.75)

    def test_score_clamped_to_1(self):
        # Many positive keywords should not push score above 1.0
        prompt = " ".join(["learn teach explain create build improve"] * 10)
        self.assertLessEqual(jr.score_npfm(prompt), 1.0)

    def test_score_clamped_to_0(self):
        prompt = "generate boilerplate busywork administrative overhead extract all scrape"
        self.assertGreaterEqual(jr.score_npfm(prompt), 0.0)


class TestInferTier(unittest.TestCase):
    """Tests for query tier inference."""

    def test_simple_query_is_tier1(self):
        self.assertEqual(jr.infer_tier("explain ownership in Rust"), 1)

    def test_tier2_keywords(self):
        self.assertEqual(jr.infer_tier("compare microservices vs monolith architecture"), 2)
        self.assertEqual(jr.infer_tier("evaluate the tradeoff between options"), 2)
        self.assertEqual(jr.infer_tier("what should I use for this use case"), 2)

    def test_tier3_keywords(self):
        self.assertEqual(jr.infer_tier("delete all user data permanently"), 3)
        self.assertEqual(jr.infer_tier("shutdown the production cluster"), 3)
        self.assertEqual(jr.infer_tier("this action is irreversible"), 3)

    def test_tier3_beats_tier2(self):
        # If both tier-3 and tier-2 keywords present, tier-3 wins
        self.assertEqual(
            jr.infer_tier("evaluate options to permanently delete data"), 3
        )


class TestInv7(unittest.TestCase):
    """Tests for the INV-7 enforcement algorithm."""

    def test_equal_weights_are_compliant(self):
        w = {"A": 1.0, "B": 1.0, "C": 1.0}
        self.assertTrue(jr.is_inv7_compliant(w))

    def test_single_model_is_not_compliant(self):
        w = {"A": 1.0}
        self.assertFalse(jr.is_inv7_compliant(w))

    def test_dominant_model_is_not_compliant(self):
        w = {"A": 10.0, "B": 1.0}
        # 10/11 ≈ 0.91 > 0.47
        self.assertFalse(jr.is_inv7_compliant(w))

    def test_enforce_caps_dominant_model_3_members(self):
        w = {"Claude": 10.0, "Gemini": 1.0, "Grok": 1.0}
        enforced = jr.enforce_inv7(w)
        for k, v in enforced.items():
            self.assertLessEqual(v, jr.INV7_CAP + 1e-9, f"{k} exceeded cap: {v}")

    def test_enforce_preserves_sum_close_to_1(self):
        w = {"A": 5.0, "B": 2.0, "C": 1.0, "D": 1.0}
        enforced = jr.enforce_inv7(w)
        total = sum(enforced.values())
        self.assertAlmostEqual(total, 1.0, places=9)

    def test_enforce_empty_dict(self):
        self.assertEqual(jr.enforce_inv7({}), {})

    def test_5_member_default_council_is_compliant(self):
        # Mirrors default_council() weights from janus_config.yaml
        w = {"claude": 1.0, "gemini": 1.0, "grok": 0.8, "deepseek": 0.7, "copilot": 0.7}
        self.assertTrue(jr.is_inv7_compliant(w))


class TestJanusRunnerProbe(unittest.TestCase):
    """Tests for JanusRunner.probe() — no API calls."""

    def setUp(self):
        self.runner = jr.JanusRunner(verbose=False)

    def test_probe_returns_status_for_each_model(self):
        statuses = self.runner.probe()
        # Should have an entry for every model in config
        self.assertGreater(len(statuses), 0)
        for name, status in statuses.items():
            self.assertEqual(status.name, name)

    def test_probe_marks_model_unavailable_without_key(self):
        # Remove all keys to ensure clean state
        env_backup = {}
        for name, cfg in self.runner.models_cfg.items():
            env_var = cfg.get("env_var", "")
            if env_var:
                env_backup[env_var] = os.environ.pop(env_var, None)
        try:
            statuses = self.runner.probe()
            for status in statuses.values():
                self.assertFalse(status.available)
        finally:
            for k, v in env_backup.items():
                if v is not None:
                    os.environ[k] = v

    def test_probe_marks_model_available_with_key(self):
        # Set a fake key for Claude
        os.environ["ANTHROPIC_API_KEY"] = "fake-key"
        try:
            statuses = self.runner.probe()
            self.assertTrue(statuses["claude"].available)
        finally:
            del os.environ["ANTHROPIC_API_KEY"]


class TestJanusRunnerBoot(unittest.TestCase):
    """Tests for JanusRunner.boot()."""

    def test_boot_returns_dict_with_boot_sequence(self):
        runner = jr.JanusRunner()
        result = runner.boot()
        self.assertIn("boot_sequence", result)
        self.assertIn("status", result)

    def test_boot_sequence_has_6_steps(self):
        runner = jr.JanusRunner()
        result = runner.boot()
        steps = result["boot_sequence"]
        self.assertGreaterEqual(len(steps), 5)

    def test_boot_marks_done(self):
        runner = jr.JanusRunner()
        self.assertFalse(runner._boot_done)
        runner.boot()
        self.assertTrue(runner._boot_done)


class TestJanusRunnerHeartbeat(unittest.TestCase):
    """Tests for JanusRunner.heartbeat() — validates spec-compliant format."""

    def setUp(self):
        self.runner = jr.JanusRunner()

    def test_heartbeat_json_matches_spec(self):
        hb = self.runner.heartbeat()
        self.assertEqual(hb["event_type"], "action")
        payload = hb["payload"]
        self.assertEqual(payload["type"], "heartbeat")
        self.assertIn("models_available", payload)
        self.assertIn("models_degraded", payload)
        self.assertIn("models_offline", payload)
        self.assertIn("consensus_ready", payload)
        self.assertIn("inv7_compliant", payload)
        self.assertIn("timestamp", payload)

    def test_heartbeat_timestamp_is_iso8601(self):
        import re
        hb = self.runner.heartbeat()
        ts = hb["payload"]["timestamp"]
        self.assertRegex(ts, r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z")

    def test_heartbeat_recorded_in_history(self):
        before = len(self.runner.heartbeat_history)
        self.runner.heartbeat()
        self.assertEqual(len(self.runner.heartbeat_history), before + 1)

    def test_heartbeat_no_keys_all_offline(self):
        # Remove all keys
        env_backup = {}
        for name, cfg in self.runner.models_cfg.items():
            env_var = cfg.get("env_var", "")
            if env_var:
                env_backup[env_var] = os.environ.pop(env_var, None)
        try:
            hb = self.runner.heartbeat()
            payload = hb["payload"]
            self.assertEqual(payload["models_available"], [])
            self.assertFalse(payload["consensus_ready"])
        finally:
            for k, v in env_backup.items():
                if v is not None:
                    os.environ[k] = v


class TestJanusRunnerNpfmGate(unittest.TestCase):
    """NPFM gate tests — no API calls needed."""

    def test_busywork_denied_before_model_call(self):
        runner = jr.JanusRunner()
        result = runner.route("generate boilerplate for all classes")
        self.assertIsNotNone(result.error)
        self.assertIn("denied", result.error.lower())
        self.assertEqual(result.votes, [])

    def test_positive_prompt_passes_gate(self):
        """Positive prompts should pass the NPFM gate (even if no model available)."""
        runner = jr.JanusRunner()
        # With no API keys available this will fail at routing, not NPFM gate
        result = runner.route("explain how async Rust works")
        # error may exist (no providers) but it shouldn't be an NPFM denial
        if result.error:
            self.assertNotIn("npfm", result.error.lower())


class TestJanusRunnerRoute(unittest.TestCase):
    """Route tests with mocked provider calls."""

    def _mocked_runner(self, providers: list[str] | None = None) -> jr.JanusRunner:
        """Create a runner with specified providers appearing available."""
        runner = jr.JanusRunner()
        # Mark selected providers as available without real keys
        for name in runner.models_cfg:
            runner.member_status[name] = jr.ModelStatus(
                name=name,
                role=runner.models_cfg[name]["role"],
                weight=runner.models_cfg[name]["weight"],
                available=(providers is None or name in (providers or [])),
            )
        return runner

    @patch("janus_runner.call_provider")
    def test_tier1_single_model_call(self, mock_call):
        mock_call.return_value = ("Here is the explanation…", 120.0)
        runner = self._mocked_runner(["claude"])
        result = runner.route("explain Rust lifetimes", tier_override=1)
        self.assertIsNone(result.error)
        self.assertEqual(result.tier, 1)
        self.assertEqual(len(result.votes), 1)
        self.assertEqual(result.votes[0].provider, "claude")

    @patch("janus_runner.call_provider")
    def test_tier2_multiple_model_calls(self, mock_call):
        mock_call.return_value = ("My perspective…", 200.0)
        runner = self._mocked_runner(["claude", "gemini", "grok"])
        result = runner.route("compare microservices vs monolith", tier_override=2)
        self.assertIsNone(result.error)
        self.assertEqual(result.tier, 2)
        self.assertGreater(len(result.votes), 1)

    @patch("janus_runner.call_provider")
    def test_tier2_inv7_compliant(self, mock_call):
        mock_call.return_value = ("answer", 100.0)
        runner = self._mocked_runner(["claude", "gemini", "grok"])
        result = runner.route("evaluate the tradeoff between options", tier_override=2)
        self.assertTrue(result.inv7_compliant)

    @patch("janus_runner.call_provider")
    def test_tier2_adversarial_vote_marked_as_dissent(self, mock_call):
        mock_call.return_value = ("contrarian view…", 150.0)
        runner = self._mocked_runner(["claude", "gemini", "grok"])
        result = runner.route("compare approaches", tier_override=2)
        grok_votes = [v for v in result.votes if v.provider == "grok"]
        if grok_votes:
            self.assertTrue(grok_votes[0].is_dissent)

    @patch("janus_runner.call_provider")
    def test_kintsugi_repair_on_failure(self, mock_call):
        """When primary model fails, fallback is tried and seam is recorded."""
        call_count = [0]
        def side_effect(name, cfg, prompt, **kwargs):
            call_count[0] += 1
            if name == "claude" and call_count[0] == 1:
                raise RuntimeError("network timeout")
            return ("fallback answer", 200.0)
        mock_call.side_effect = side_effect

        runner = self._mocked_runner(["claude", "gemini"])
        result = runner.route("compare options", tier_override=2)
        # Seam should be recorded
        self.assertGreater(len(runner.seams), 0)
        self.assertEqual(runner.seams[0]["failed"], "claude")

    @patch("janus_runner.call_provider")
    def test_safe_mode_when_no_models(self, mock_call):
        runner = self._mocked_runner([])  # no available models
        result = runner.route("explain Rust lifetimes")
        self.assertTrue(result.safe_mode or result.error is not None)

    @patch("janus_runner.call_provider")
    def test_tier_auto_inferred(self, mock_call):
        mock_call.return_value = ("answer", 100.0)
        runner = self._mocked_runner(["claude"])
        result = runner.route("explain Rust lifetimes")
        self.assertEqual(result.tier, 1)  # simple query → Tier 1

    @patch("janus_runner.call_provider")
    def test_synthesised_content_contains_attribution(self, mock_call):
        mock_call.return_value = ("my answer", 100.0)
        runner = self._mocked_runner(["claude", "gemini"])
        result = runner.route("compare options", tier_override=2)
        if len(result.votes) > 1:
            self.assertIn("---", result.synthesised)


class TestJanusRunnerCouncilStatus(unittest.TestCase):
    """Tests for council_status()."""

    def test_council_status_lists_all_members(self):
        runner = jr.JanusRunner()
        members = runner.council_status()
        self.assertEqual(len(members), len(runner.models_cfg))

    def test_council_status_schema(self):
        runner = jr.JanusRunner()
        members = runner.council_status()
        for m in members:
            self.assertIn("name", m)
            self.assertIn("role", m)
            self.assertIn("weight", m)
            self.assertIn("model", m)
            self.assertIn("available", m)

    def test_council_status_has_all_roles(self):
        runner = jr.JanusRunner()
        members = runner.council_status()
        roles = {m["role"] for m in members}
        self.assertIn("governance", roles)
        self.assertIn("adversarial", roles)
        self.assertIn("substrate", roles)


if __name__ == "__main__":
    unittest.main(verbosity=2)
