---
name: profile-auditor
description: >
  Audit a LinkedIn profile or professional claims for accuracy and verifiability. Use when
  the user asks to "verify a profile", "check if this person is legit", "audit credentials",
  "is this profile real", "verify employment claims", or when a company founder wants to
  verify claims made about their organization. Also triggers on "bot check" or "fake profile".
metadata:
  version: "0.1.0"
  author: "Dave Sheldon"
---

# Profile Auditor

Three audit modes:
1. Self-audit — verify your own claims before publishing
2. Third-party audit — check if someone else's profile is legit or a bot
3. Organizational audit — founders verify who's claiming affiliation

## Verification Scoring

- Verified (Green): Multiple independent public sources confirm
- Supported (Blue): At least one public source consistent
- Unverified (Amber): No public evidence, but plausible
- Flagged (Red): Public evidence contradicts or claim is implausible

## Bot Detection Signals

High confidence: Stock photos, zero external footprint, fake companies, rapid connection accumulation
Medium confidence: Name/photo mismatch, suspicious endorsement networks, automated posting patterns
Low confidence: Incomplete profile, no photo, few connections (may be real but lazy)

## Organizational Audit

Founder confirms real employees and titles. Plugin searches all LinkedIn profiles claiming affiliation. Cross-references and generates discrepancy report with recommended actions.