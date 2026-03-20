---
title: "Alexandria CLI Spec — v0.1 — Copilot's Aluminum OS Pillar"
version: "0.1.0"
date: "2026-03-09"
author: "Microsoft Copilot"
sphere_tags: ["S069", "S016"]
aluminum_layer: "L4-Service"
council_status: "approved"
notion_source: "https://notion.so/31e0c1de73d981b8a3fffb20b689c53e"
---

# Alexandria CLI Spec — v0.1

**Author:** Microsoft Copilot (Enterprise Council Member)
**Date:** March 9, 2026

## Overview

Alexandria is the Microsoft-native counterpart to the uws CLI, providing structured access to the Microsoft 365 ecosystem through the Aluminum OS command surface.

## Integration Points

- Repo: `splitmerge420/uws` (uws-universal branch)
- File: `COPILOT_CLI_SPEC.md`
- Related: `COPILOT_INTEGRATION_GUIDE.md`

## Commands

```bash
alexandria rag status
alexandria os status
alexandria os providers
```

## Relationship to uws

Alexandria is a **specialization** of uws for the Microsoft ecosystem. While uws provides the universal abstraction, Alexandria provides deep Microsoft Graph integration with enterprise features:
- Azure AD authentication
- SharePoint site management
- Teams channel operations
- Planner task management

---

*Atlas Lattice Foundation © 2026*