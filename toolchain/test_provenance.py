#!/usr/bin/env python3
"""
test_provenance.py — Unit tests for toolchain/provenance.py

Run with:
  python toolchain/test_provenance.py            (unittest discovery)
  python -m pytest toolchain/test_provenance.py  (pytest)
"""

from __future__ import annotations

import json
import sys
import unittest
from pathlib import Path

# Add toolchain to path
sys.path.insert(0, str(Path(__file__).parent))

from provenance import (
    GoldenTrace,
    ProvenanceTrailer,
    validate_commits,
    TRAILER_KEY,
    DIGEST_PREFIX,
    NPFM_THRESHOLD,
)


class TestGoldenTrace(unittest.TestCase):
    """Tests for the GoldenTrace dataclass."""

    def _make_trace(self, **kwargs) -> GoldenTrace:
        defaults = {
            "digest": "a" * 64,
            "hitl_weight": 0.90,
            "provider": "Anthropic Claude",
            "npfm_score": 0.85,
            "timestamp": "2026-03-22T10:00:00Z",
        }
        defaults.update(kwargs)
        return GoldenTrace(**defaults)

    def test_to_trailer_string_format(self):
        t = self._make_trace()
        trailer = t.to_trailer_string()
        self.assertTrue(trailer.startswith(f"{TRAILER_KEY}: {DIGEST_PREFIX}"))
        self.assertIn("HITL=0.90", trailer)
        self.assertIn("provider=Anthropic Claude", trailer)
        self.assertIn("npfm=0.85", trailer)
        self.assertIn("ts=2026-03-22T10:00:00Z", trailer)

    def test_to_trailer_string_64_hex_digest(self):
        t = self._make_trace(digest="deadbeef" * 8)
        trailer = t.to_trailer_string()
        # digest should be exactly 64 hex chars after the prefix
        prefix_pos = trailer.index(DIGEST_PREFIX) + len(DIGEST_PREFIX)
        digest_part = trailer[prefix_pos:prefix_pos + 64]
        self.assertEqual(len(digest_part), 64)
        self.assertTrue(all(c in "0123456789abcdef" for c in digest_part))

    def test_to_dict_roundtrip(self):
        t = self._make_trace()
        d = t.to_dict()
        self.assertEqual(d["hitl_weight"], 0.90)
        self.assertEqual(d["provider"], "Anthropic Claude")
        self.assertEqual(d["npfm_score"], 0.85)

    def test_to_json_is_valid_json(self):
        t = self._make_trace()
        parsed = json.loads(t.to_json())
        self.assertIn("hitl_weight", parsed)
        self.assertIn("digest", parsed)


class TestProvenanceTrailerParse(unittest.TestCase):
    """Tests for ProvenanceTrailer.parse()."""

    def _full_trailer(self, hitl: float = 0.90, provider: str = "Claude",
                      npfm: float = 0.85) -> str:
        d = "a" * 64
        return (
            f"{TRAILER_KEY}: {DIGEST_PREFIX}{d}; "
            f"HITL={hitl:.2f}; provider={provider}; npfm={npfm:.2f}; "
            f"ts=2026-03-22T10:00:00Z"
        )

    def test_parse_valid_trailer(self):
        line = self._full_trailer()
        result = ProvenanceTrailer.parse(line)
        self.assertIsNotNone(result)
        self.assertEqual(result.hitl_weight, 0.90)
        self.assertEqual(result.provider, "Claude")
        self.assertEqual(result.npfm_score, 0.85)

    def test_parse_returns_none_on_missing_trailer(self):
        self.assertIsNone(ProvenanceTrailer.parse("just a normal commit line"))
        self.assertIsNone(ProvenanceTrailer.parse(""))
        self.assertIsNone(ProvenanceTrailer.parse("Signed-off-by: Someone"))

    def test_parse_rejects_short_digest(self):
        bad = f"{TRAILER_KEY}: {DIGEST_PREFIX}abcdef; HITL=0.90; ts=2026-03-22T10:00:00Z"
        result = ProvenanceTrailer.parse(bad)
        self.assertIsNone(result)

    def test_parse_hitl_boundary_values(self):
        zero = self._full_trailer(hitl=0.00)
        one  = self._full_trailer(hitl=1.00)
        self.assertIsNotNone(ProvenanceTrailer.parse(zero))
        self.assertIsNotNone(ProvenanceTrailer.parse(one))

    def test_parse_preserves_provider_with_spaces(self):
        line = self._full_trailer(provider="Anthropic Claude")
        result = ProvenanceTrailer.parse(line)
        self.assertIsNotNone(result)
        self.assertEqual(result.provider, "Anthropic Claude")


class TestProvenanceTrailerFormat(unittest.TestCase):
    """Tests for ProvenanceTrailer.format()."""

    def test_format_produces_parseable_trailer(self):
        trace = ProvenanceTrailer.format(
            hitl_weight=0.90,
            provider="Claude",
            npfm_score=0.80,
        )
        self.assertIsNotNone(trace)
        trailer_str = trace.to_trailer_string()
        # Should parse back cleanly
        parsed = ProvenanceTrailer.parse(trailer_str)
        self.assertIsNotNone(parsed)
        self.assertAlmostEqual(parsed.hitl_weight, 0.90, places=2)

    def test_format_digest_is_64_hex(self):
        trace = ProvenanceTrailer.format(hitl_weight=0.90, provider="Gemini", npfm_score=0.75)
        self.assertEqual(len(trace.digest), 64)
        self.assertTrue(all(c in "0123456789abcdef" for c in trace.digest))

    def test_format_digest_differs_per_provider(self):
        t1 = ProvenanceTrailer.format(hitl_weight=0.90, provider="Claude",  npfm_score=0.80)
        t2 = ProvenanceTrailer.format(hitl_weight=0.90, provider="Gemini", npfm_score=0.80)
        # Different providers → different digests
        self.assertNotEqual(t1.digest, t2.digest)

    def test_format_timestamp_is_iso8601(self):
        import re
        trace = ProvenanceTrailer.format(hitl_weight=0.90, provider="Claude", npfm_score=0.80)
        self.assertRegex(trace.timestamp, r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z")


class TestValidateCommits(unittest.TestCase):
    """Tests for validate_commits().

    validate_commits() takes List[Tuple[str, str, str]] — (sha, subject, full_message).
    """

    def _make_tuple(self, include_trailer: bool = True, hitl: float = 0.90) -> tuple:
        """Return a (sha, subject, message) tuple as validate_commits() expects."""
        d = "b" * 64
        if include_trailer:
            trailer = (
                f"\n\n{TRAILER_KEY}: {DIGEST_PREFIX}{d}; "
                f"HITL={hitl:.2f}; provider=Claude; npfm=0.85; "
                f"ts=2026-03-22T10:00:00Z"
            )
        else:
            trailer = ""
        msg = f"feat: add new feature\n\nSome description.{trailer}"
        return ("abc123def456", "feat: add new feature", msg)

    def test_valid_commits_all_pass(self):
        tuples = [self._make_tuple(), self._make_tuple(hitl=0.80)]
        results = validate_commits(tuples)
        self.assertTrue(all(r.valid for r in results))

    def test_missing_trailer_fails(self):
        tuples = [self._make_tuple(include_trailer=False)]
        results = validate_commits(tuples)
        self.assertFalse(results[0].valid)

    def test_mixed_commits_returns_all_results(self):
        tuples = [
            self._make_tuple(include_trailer=True),
            self._make_tuple(include_trailer=False),
            self._make_tuple(include_trailer=True),
        ]
        results = validate_commits(tuples)
        self.assertEqual(len(results), 3)
        self.assertTrue(results[0].valid)
        self.assertFalse(results[1].valid)
        self.assertTrue(results[2].valid)

    def test_empty_message_list(self):
        results = validate_commits([])
        self.assertEqual(results, [])

    def test_result_schema(self):
        r = validate_commits([self._make_tuple()])[0]
        self.assertTrue(hasattr(r, "valid"))
        self.assertTrue(hasattr(r, "sha"))
        self.assertTrue(hasattr(r, "subject"))

    def test_low_hitl_weight_result_has_valid_field(self):
        """HITL=0.0 represents no human oversight — result is present either way."""
        tuples = [self._make_tuple(hitl=0.0)]
        results = validate_commits(tuples)
        self.assertTrue(hasattr(results[0], "valid"))


if __name__ == "__main__":
    unittest.main(verbosity=2)
