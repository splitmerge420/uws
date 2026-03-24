---
name: bot-detector
description: >
  Detect fake, bot-generated, or AI-fabricated LinkedIn profiles using multi-signal weighted
  analysis. Use when the user asks "is this profile real", "bot check", "fake profile detection",
  "is this a real person", "check for bots", or when scanning a set of profiles for authenticity.
  Extracted from profile-auditor to operate as a standalone capability.
metadata:
  version: "0.3.0"
  author: "Dave Sheldon"
  origin: "Extracted from profile-auditor v0.2.0 per ML Engineering team recommendation"
---

# Bot Detector

Standalone bot and fake profile detection using multi-signal weighted scoring with Bayesian confidence estimation. Reports findings to the platform level. Bot detection is a platform responsibility, not a user weapon.

Results are framed as "characteristics consistent with automated generation" — never "this person is a bot."

See full skill documentation in the plugin repository.