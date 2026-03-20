---
title: "Aluminos Bridge v3.0 — Constitutional UI Shell"
version: "3.0.0"
date: "2026-03-15"
author: "Claude (Constitutional Scribe)"
sphere_tags: ["S069", "S016", "S056"]
aluminum_layer: "L5-Extension"
council_status: "approved"
notion_source: "https://notion.so/3240c1de73d981b4b4d8cf22a6b781d1"
---

# Aluminos Bridge v3.0 — Constitutional UI Shell

## Overview

The Aluminos Bridge is a Chrome MV3 extension + WebSocket relay server + Progressive Web App (PWA) that serves as the browser-based UI layer for Aluminum OS.

## Architecture

```
┌─────────────────────────┐
│   Chrome Extension      │  ← MV3, content scripts, side panel
│   (Manifest V3)         │
├─────────────────────────┤
│   WebSocket Relay       │  ← Node.js, bridges extension ↔ Aluminum OS
│   (localhost:8765)      │
├─────────────────────────┤
│   PWA Companion         │  ← Offline-capable, installable
│   (Service Worker)      │
├─────────────────────────┤
│   Constitutional Layer  │  ← ConsentKernel verification in UI
│   (consent-ui.js)       │
└─────────────────────────┘
         ↕
┌─────────────────────────┐
│   Aluminum OS Backend   │  ← Sheldonbrain RAG + Janus Router
└─────────────────────────┘
```

## Key Features

- **Constitutional consent UI** — every data action requires visible consent
- **Sphere-tagged browsing** — web content automatically classified via 144-sphere ontology
- **X-Algorithm Truth Substrate** — media integrity verification layer
- **Cross-device mirroring** — session teleportation via constitutional state vectors
- **Interop 2026** — View Transitions API, Navigation API, Container Queries, Scroll-Driven Animations

## Cross-Device Mirroring

One Constitutional Shell. Every device. No "mobile version" vs "desktop version" — one constitutional experience, adapted to hardware:

1. Device A: State vector captured (page, DBs, focus spheres)
2. State signed with PQC and pushed to Aluminum OS backend
3. Device B: Reads latest state vector, restores context
4. Constitutional consent re-verified on new device

## Related Files

- `bridge/apple-bridge-v2/` — Source code (Chrome MV3 extension)
- `bridge/relay-server/` — WebSocket relay (Node.js)
- `bridge/pwa/` — Progressive Web App companion

## Build & Development

```bash
# Extension
cd bridge/apple-bridge-v2 && npm install && npm run build

# Relay server
cd bridge/relay-server && npm install && node server.js

# PWA
cd bridge/pwa && npm install && npm run dev
```

## Deployment

Load unpacked extension in Chrome → chrome://extensions → Developer mode → Load unpacked → select `bridge/apple-bridge-v2/dist/`

---

*Atlas Lattice Foundation © 2026*