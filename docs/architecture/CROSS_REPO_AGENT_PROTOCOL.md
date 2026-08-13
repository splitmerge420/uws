# Cross-Repo Agent Protocol

Status: Phase 1 coordination layer

## Purpose

Define how multiple AI agents collaborate across repositories without losing context, duplicating work, or breaking architecture.

---

## Agent Roles (Observed)

- GPT → synthesis, verification, repo structuring
- Microsoft 365 Copilot → longform planning
- GitHub Copilot → code execution (variable memory)
- Manus → large-scale execution (cost-sensitive)
- Notion AI → workspace + MCP UI

---

## Core Loop

```text
generate → verify → stress-test → merge
```

---

## Required Practices

### 1. Context Rehydration

Every execution agent must receive:

- current module
- target repo
- relevant docs

### 2. PR Discipline

- one concern per PR
- explicit intent
- no hidden side effects

### 3. Parallel Verification


### 4. Cost Control

For Manus / large runs:

- define budget
- define stop conditions

### 5. Provenance

All generated artifacts should record:

- agent source
- timestamp
- transformation chain

---

## Failure Modes to Avoid

- agent drift (inconsistent architecture)
- memory loss (GitHub Copilot resets)
- cost explosion (Manus)
- repo fragmentation

---

## Next Steps

- [ ] integrate with governance preflight
- [ ] add execution policy structs
- [ ] add provenance metadata to codebase
