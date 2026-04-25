# Private Repo Harvest Plan

Status: Phase 1 planning layer

## Purpose

Define a safe, repeatable method to extract value from private Atlas Lattice repos into public or semi-public system layers without leaking sensitive or unverified material.

---

## Core Rule

Private repos are **source strata**, not direct dependencies.

---

## Harvest Pipeline

```text
private repo
  -> classify (domain / sensitivity)
  -> extract concept
  -> write doc in uws/aluminum-os
  -> create skeleton module if needed
  -> optionally implement later
```

---

## Sensitivity Levels

- Level 0: safe public
- Level 1: requires review
- Level 2: restricted
- Level 3: never public

---

## Required Metadata for Every Extraction

```text
source_repo:
source_path:
date_extracted:
agent_source:
sensitivity_level:
target_repo:
target_module:
notes:
```

---

## Initial Harvest Targets

High-value clusters to review first:

- healthcare-ai
- banking-revolution
- energy-grid-ai
- water-management-ai
- surveillance-capitalism

---

## Non-Goals (Phase 1)

- no bulk copying
- no automated ingestion
- no exposing private data

---

## Next Steps

- [ ] create extraction templates
- [ ] tag private repos by domain
- [ ] begin selective doc extraction
