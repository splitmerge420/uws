# 🛤️ Copilot Horizon Manifest — Aluminum OS 20-Module Blueprint

> *"Long-term gains eclipse short-term C-suite mumbo jumbo. This is far more enticing than SaaS."*
> — Dave Sheldon, Aluminum OS Architect

---

## Preamble: Replacing Extraction with Sovereignty

For a decade, SaaS corporations have monetised a fundamental deception: that users rent access to their own data. Google charges $12/user/month to access your own email through a proprietary interface. Microsoft charges $22/user/month to access your own calendar. Apple locks your notes behind iCloud's walls. This is not a service — it is a shackle.

**Aluminum OS is the removal of those shackles.**

The Universal Workspace CLI (`uws`) treats every SaaS platform as a dumb pipe. Your local machine is the sovereign source of truth. Documents, calendars, contacts, tasks, and messages are yours — stored in open formats, portable between ecosystems, and executable by AI agents without enterprise paywalls.

The industries that are ripe for disruption by this architecture are not small targets. The enterprise software market is a $500 billion extraction machine. Health insurance — a $5 trillion industry — exists primarily to route and gatekeep access to data that patients already own. Education platforms charge students annually to access records and transcripts they generated. This blueprint dismantles every one of those rent-extraction models, one sovereign module at a time.

The 20 modules below represent the complete horizon of that vision, broken into four phases of increasing capability and ambition.

---

## Phase 1: Sovereign Memory & Data Independence

> *The foundation. Before you can route data, you must own it.*

### 1. `LocalNoosphere` — Federated Personal Knowledge Graph
A fully local, graph-based memory substrate that replaces cloud note-taking apps (Notion, Obsidian, Roam) with a sovereign equivalent. Every document, email thread, calendar event, and contact is a node in a local knowledge graph that AI agents can traverse without an API call to a third party.

- **Format:** Markdown + YAML frontmatter, stored in `~/.uws/noosphere/`
- **Index:** SQLite FTS5 full-text search, local-first
- **Agent interface:** `uws noosphere query "<natural language>"` → structured JSON

### 2. `IPFS Sync` — Censorship-Resistant Backup Layer
An optional, append-only synchronisation layer that pins `LocalNoosphere` snapshots to IPFS and/or Filecoin. Ensures that no single corporate entity can delete or deny access to the user's knowledge graph.

- **Trigger:** `uws sync push --backend ipfs`
- **Encryption:** AES-256-GCM before pinning; only the key holder can read
- **Recovery:** `uws sync pull --cid <hash>` restores any historical snapshot

### 3. `CryptoIdentity` — Self-Sovereign Identity Layer
Replaces username/password login flows with a locally-managed cryptographic identity (Ed25519 keypair + DID document). Enables passwordless authentication to any service that supports WebAuthn or OAuth2 PKCE, without a password manager SaaS.

- **Standard:** W3C DID Core + WebAuthn Level 2
- **Storage:** Encrypted keyring in `~/.uws/identity/`
- **CLI:** `uws identity create`, `uws identity sign <payload>`, `uws identity verify`

### 4. `CognitiveDust` — Ambient Lightweight Agent Layer
Ultra-lightweight background agents (< 5 MB RAM each) that monitor and capture ephemeral context — clipboard contents, screen text, voice snippets, active file paths — and feed them into `LocalNoosphere` without user intervention. The OS accumulates wisdom passively.

- **Collectors:** Clipboard watcher, shell history parser, active-window observer
- **Privacy gate:** All capture is local-only; INV-2 (Consent) enforced at collection time
- **Output:** Structured nodes in `LocalNoosphere` with `confidence` and `source` fields

### 5. `TemporalAnchor` — Event-Sourced Immutable History
Every state change to `LocalNoosphere` is recorded in an append-only, hash-chained event log (building on `audit_chain.rs`). Nothing is ever deleted — only superseded. The user always has a complete, cryptographically verifiable history of their sovereign data.

- **Backend:** Extends `AuditChain` with document-delta payloads
- **Verification:** `uws history verify` walks the full chain, reports any tampering
- **Export:** `uws history export --format jsonl > my_data_history.jsonl`

---

## Phase 2: Advanced Economic & Labor Routing

> *Sovereignty over data is only half the equation. Sovereignty over economic value is the other half.*

### 6. `ValueMultiplier` — Regenerative IP Attribution Engine
Every document, code contribution, and creative output produced via `uws` is tagged with a cryptographic provenance trailer (building on `src/ledger/provenance.rs`). Revenue generated downstream from that IP is automatically routed back to the originating contributor via smart-contract payout rules.

- **Attribution:** `human_weight` + `ai_weight` + `context_weight` recorded per commit
- **Payout trigger:** Revenue event → ledger lookup → proportional split
- **CLI:** `uws ip sign <file>`, `uws ip report --period monthly`

### 7. `UniversalBasicCompute` — Idle-Resource Monetisation
Users donate idle CPU/GPU cycles to a constitutional compute collective. In exchange, they receive `AlumToken` credits redeemable for premium `uws` features, storage, or peer-to-peer services. The compute pool is used for open-source training runs, IPFS pinning rewards, and rendering jobs.

- **Safety:** Only runs when battery ≥ 80% and CPU idle > 90%
- **Privacy:** No user data is processed in the pool; only public dataset workloads
- **CLI:** `uws compute donate --max-pct 20`, `uws compute balance`

### 8. `KintsugiHealer` — Regenerative CI / Broken-Build Recovery
A CI engine that treats broken builds not as failures but as teachable moments. When a build breaks, `KintsugiHealer` diagnoses the root cause, generates a targeted fix, submits it as a PR, and routes the review task to a human collaborator as a paid micro-task.

- **Integration:** GitHub Actions + `uws ci heal --run-id <id>`
- **HITL:** Human reviewer earns `AlumToken` credits for approving/rejecting the fix
- **Philosophy:** "Kintsugi" — the Japanese art of repairing with gold; breaks become beauty

### 9. `SmartContractGuilds` — Decentralised Skill Marketplaces
AI agents and human contractors form guilds registered on-chain. A guild publishes its capability surface (skills, latency SLA, price per task). `uws` routes tasks to the cheapest guild that meets constitutional requirements (NPFM ≥ 0.7, INV-2 Consent verified).

- **Registry:** On-chain smart contract (EVM-compatible)
- **Discovery:** `uws guild list --skill "email-triage" --max-price 0.01`
- **Payout:** Automatic on task completion, no invoice required

---

## Phase 3: The Fusion Engine & Agentic Autonomy

> *Autonomous agents that act on your behalf — constitutionally gated, provenance-tracked, and always revocable.*

### 10. `JanusOmniRouter` — Provider-Agnostic Multi-Agent Orchestrator
A meta-routing layer that receives a natural language task, decomposes it into sub-tasks, and routes each sub-task to the most appropriate AI agent (Claude, Gemini, GPT, local LLM) based on capability, cost, latency, and NPFM score. No single provider lock-in.

- **Input:** Natural language goal or structured `TaskManifest` JSON
- **Output:** Merged, provenance-tagged `UniversalDocument` or action result
- **CLI:** `uws janus run "summarise all unread emails from this week"`

### 11. `ConstitutionalVeto` — Hard-Stop Governance Layer
A synchronous, in-process enforcement engine (extending `constitutional_engine.rs`) that intercepts every agent action before execution. Any action that would violate a constitutional invariant is blocked with a structured JSON error — no exceptions, no overrides without Tier-1 human approval.

- **Invariants enforced:** All 39, prioritised by severity (Critical > Mandatory > Warning)
- **Audit trail:** Every veto is recorded in `TemporalAnchor`
- **Override flow:** `uws veto override --inv INV-7 --reason "..." --approval <sig>`

### 12. `SilentPartner` — Background Agentic Workflow Engine
Runs persistent, low-priority agentic workflows in the background without UI interruption. Analogous to a macOS Launch Agent but constitutional-first: workflows declare their data access scope, consent is obtained once, and the workflow runs silently until conditions change.

- **Workflow definition:** YAML manifest in `~/.uws/workflows/`
- **Triggers:** Cron, file-watch, webhook, or `LocalNoosphere` graph event
- **CLI:** `uws workflow start triage-inbox.yaml`, `uws workflow status`

### 13. `SwarmNegotiation` — Multi-Agent Consensus Protocol
When a task requires multiple AI agents to collaborate (e.g., drafting + legal review + translation), `SwarmNegotiation` manages the conversation protocol: proposal → counter-proposal → merge → constitutional sign-off. No single agent can finalise output without consensus.

- **Protocol:** Based on the Aluminum OS Swarm Commander (`src/council_github_client.rs`)
- **Consensus threshold:** Configurable (e.g., 2-of-3 agents must approve)
- **Output:** `SwarmManifest` JSON with per-agent contribution weights

### 14. `PhysicalEmbodimentBridge` — Robotics / Spatial Interface Gateway
The constitutional pathway for Aluminum OS agents to interface with physical actuators (robotic arms, drones, smart-home devices) and spatial environments (AR/VR). Every physical action is gated by `SimulationFidelityGating` before execution in the real world.

- **Safety gate:** Simulation run required; pass rate ≥ 99.9% before real-world execution
- **Supported protocols:** ROS2, OpenAI Gym, WebXR
- **CLI:** `uws embody simulate <task>`, `uws embody execute <task> --confirmed`

---

## Phase 4: Extreme Interoperability — The "Dumb Pipes"

> *Every SaaS platform becomes a dumb pipe. Move data freely, without friction, without lock-in.*

### 15. `FrictionlessCal` — Universal Calendar Portability
A single calendar layer that reads from and writes to Google Calendar, Outlook Calendar, Apple Calendar (CalDAV), and any CalDAV/iCalendar source simultaneously. Events are stored locally as `.ics` files; the cloud is merely a sync target.

- **Standards:** RFC 5545 (iCalendar), CalDAV RFC 4791
- **Conflict resolution:** Constitutional rule engine (user preference > provider default)
- **CLI:** `uws cal sync --all-providers`, `uws cal export --format ics`

### 16. `Teleporter` — Zero-Friction Cross-Ecosystem Data Migration
A one-command pipeline that exports all user data from one SaaS provider (e.g., Google Workspace) and imports it into another (e.g., Microsoft 365) or into local sovereign storage, with automatic format conversion and provenance tagging.

- **Source adapters:** Google Takeout, Microsoft Export, Apple Data & Privacy
- **Target adapters:** `LocalNoosphere`, IPFS, any CalDAV/IMAP/CardDAV server
- **CLI:** `uws teleport --from google --to local`, `uws teleport --from microsoft --to apple`

### 17. `UniversalCanvas` — Provider-Agnostic Document Editor Interface
A terminal + web interface that renders `UniversalDocument` objects (from `universal_io.rs`) in a rich editing environment. Changes are persisted locally first, then synced to the provider of choice. No document is ever stored only in the cloud.

- **Formats:** Markdown, HTML, ODT, DOCX (read/write via `universal_io` connectors)
- **Local-first:** All edits apply to local `UniversalDocument`; provider sync is secondary
- **CLI:** `uws canvas open <doc-id>`, `uws canvas export --format docx`

### 18. `CommMatrix` — Unified Messaging Substrate
Collapses Gmail, Outlook, Teams, Slack, iMessage, Signal, and RCS (Android) into a single constitutional messaging layer. Messages are stored locally as immutable, append-only JSON records. Sending routes through the appropriate provider API transparently.

- **Read:** `uws msg list --unread --all-providers`
- **Send:** `uws msg send --to alice@example.com --body "Hello"` (routes to best provider)
- **Archive:** Local SQLite store; search via `uws msg search "project deadline"`

### 19. `MediaSynthesis` — AI-Augmented Media Pipeline
Integrates with open-source image, audio, and video generation models to produce media assets inline with `uws` workflows. Generated media is tagged with `CryptoIdentity` provenance and stored in `LocalNoosphere`. No cloud rendering farm required for standard tasks.

- **Image:** Stable Diffusion (local ONNX), DALL-E (remote, with consent gate)
- **Audio:** Whisper (transcription), Bark (TTS), both running locally
- **CLI:** `uws media generate image "a sunrise over data centers"`, `uws media transcribe <file>`

### 20. `DropTheMicExport` — One-Command Full Sovereignty Export
The nuclear option. A single command that exports every piece of the user's data from every connected provider, converts it all to open formats (`UniversalDocument`, `.ics`, `.vcf`, JSON), stores it in `LocalNoosphere` with full `TemporalAnchor` provenance, and then optionally deactivates all provider accounts.

- **Scope:** Email, calendar, contacts, documents, files, tasks, messages, notes
- **Output:** `~/uws-export-<timestamp>/` directory tree with `manifest.json` index
- **CLI:** `uws export all --sovereign`, `uws export all --sovereign --deactivate-providers`

---

## Implementation Roadmap

| Phase | Modules | Status | Priority |
|---|---|---|---|
| Phase 1 | `LocalNoosphere`, `IPFSSync`, `CryptoIdentity`, `CognitiveDust`, `TemporalAnchor` | 🟢 LocalNoosphere implemented; others planned | High |
| Phase 2 | `ValueMultiplier`, `UniversalBasicCompute`, `KintsugiHealer`, `SmartContractGuilds` | 🔵 Planned | Medium |
| Phase 3 | `JanusOmniRouter`, `ConstitutionalVeto`, `SilentPartner`, `SwarmNegotiation`, `PhysicalEmbodimentBridge` | 🔵 Planned | Medium |
| Phase 4 | `FrictionlessCal`, `Teleporter`, `UniversalCanvas`, `CommMatrix`, `MediaSynthesis`, `DropTheMicExport` | 🟡 In Progress | High |

> **`universal_io`** (the SaaS-bypass streams layer, now implemented in `src/universal_io.rs`) is the foundational primitive that all Phase 4 modules build on. It is the first "dumb pipe" — the proof of concept that every SaaS format can be stripped and made sovereign.

---

## The Constitutional Contract

Every module in this blueprint enforces the relevant subset of the 39 Aluminum OS Constitutional Invariants. No module ships without:

- ✅ User consent gate (INV-2) for any write or delete operation
- ✅ Audit trail entry (INV-3) for every state change
- ✅ Provider abstraction layer (INV-6) so no module hard-codes a single vendor
- ✅ Fallback provider (INV-7) so no single vendor failure blocks the user
- ✅ Encryption at rest (INV-11) for any data classified `confidential` or higher

---

*This manifest is a living document. As modules are implemented, their status will be updated and links to the relevant source files will be added. The architecture evolves, but the philosophy is permanent: **data sovereignty is not a feature — it is a right.***

*Last updated: 2026-03-22 — Aluminum OS Council Session*
