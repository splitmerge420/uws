---
name: credential-miner
description: >
  Research and surface a person's professional credentials from web sources. Use when the user
  asks to "find my credentials", "what's out there about me", "research my background",
  "dig up my achievements", or "find press about me".
metadata:
  version: "0.1.0"
  author: "Dave Sheldon"
---

# Credential Miner

Systematically search public web sources to surface professional achievements, press coverage, published work, and notable associations.

## Search Strategy — Parallel Batches

Batch 1: Direct name + company searches
Batch 2: Company-specific context and press
Batch 3: Industry achievements, speaking, patents, publications
Batch 4: Social and media presence across platforms

## Verification Rules

- NEVER include unverified information
- Present ALL findings to user for confirmation
- Flag potential name collisions
- Absence of evidence is not evidence of absence