---
title: "uws — Universal Workspace CLI — Master Repo Reference"
version: "1.0.0"
date: "2026-03-09"
author: "Claude (Constitutional Scribe)"
sphere_tags: ["S069", "S016", "S024"]
aluminum_layer: "L4-Service"
council_status: "approved"
notion_source: "https://notion.so/31e0c1de73d98105a3a2d19f93165b0f"
cross_refs:
  - repo: "aluminum-os"
    path: "docs/integration/SHELDONBRAIN_INTEGRATION.md"
  - repo: "144-sphere-ontology"
    path: "canonical/MASTER_REFERENCE.md"
---

# uws — Universal Workspace CLI — Master Repo Reference

> **Repo:** github.com/splitmerge420/uws
> **Branch:** uws-universal | **Commits:** 154 | **Language:** Rust (95.7%)
> **Forked from:** googleworkspace/cli (Apache 2.0)

## What It Is

`uws` is the **command surface of Aluminum OS** — a universal, schema-driven CLI that gives humans and AI agents structured, JSON-first access to every major productivity ecosystem from a single tool: Google Workspace, Microsoft 365, Apple iCloud, Android, and Chrome.

## Ecosystems Covered

| Ecosystem | Services |
|---|---|
| **Google Workspace** | Drive, Gmail, Calendar, Docs, Sheets, Slides, Tasks, People, Chat, Forms, Keep, Meet, Classroom |
| **Microsoft 365** | Outlook Mail, Calendar, OneDrive, Teams, To Do, OneNote, Contacts, SharePoint, Planner |
| **Apple** | iCloud Calendar, Reminders, Contacts, Drive, Notes |
| **Android** | Management API, Messages, Devices |
| **Chrome** | Management, Policy, Extensions, ChromeOS Devices |

## The Aluminum Layer

Above `uws` sits `alum` — the Aluminum kernel that abstracts providers the way Kubernetes abstracts cloud:

```bash
alum mail send --to "alice@example.com"           # auto-detects provider
alum drive list --provider microsoft               # OneDrive
alum search "Q1 budget" --provider all             # searches ALL ecosystems
alum sync calendar --from google --to microsoft    # cross-provider sync
```

## AI Agent Integration

Every Pantheon Council member has a defined integration path:

- **Claude** — MCP tool definition + CLAUDE.md
- **Manus** — SKILL.md in skills directory
- **Gemini** — Python subprocess wrapper + .gemini/ config
- **Copilot** — Alexandria CLI spec (COPILOT_CLI_SPEC.md) + integration guide
- **Grok** — GROK_REVIEW.md (adversarial analysis)

## Roadmap

| Phase | Milestone | Status |
|---|---|---|
| 1 | Fork gws, abstract provider layer (uws v0.1) | **In Progress** |
| 2 | Microsoft Graph backend (uws v0.2) | Planned |
| 3 | Apple Intents backend (uws v0.3) | Planned |
| 4 | Aluminum kernel APIs (alum v0.1) | Planned |
| 5 | Full Aluminum-native command surface (alum v1.0) | Vision |

## Credits

- **Daavud Sheldon** — uws architecture and multi-ecosystem extension
- **Justin Poehnelt** — original gws implementation (Google)
- **Microsoft Copilot** — Aluminum OS architectural review
- **Google LLC** — original gws codebase (Apache 2.0)

---

*Vaulted 2026-03-09 by Claude (Constitutional Scribe)*
*Atlas Lattice Foundation © 2026*