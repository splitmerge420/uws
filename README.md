<h1 align="center">uws — Universal Workspace CLI</h1>

<p align="center">
  <strong>One CLI to rule them all.</strong><br>
  Google Workspace. Microsoft 365. Apple. Android. Chrome.<br>
  Built for humans and AI agents. Zero boilerplate.
</p>

<p align="center">
  <a href="https://github.com/splitmerge420/uws/actions/workflows/ci.yml"><img src="https://github.com/splitmerge420/uws/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License: MIT"></a>
  <a href="https://github.com/splitmerge420/uws"><img src="https://img.shields.io/github/stars/splitmerge420/uws?style=social" alt="Stars"></a>
  <a href="https://github.com/googleworkspace/cli"><img src="https://img.shields.io/badge/forked%20from-googleworkspace%2Fcli-brightgreen" alt="Forked from gws"></a>
</p>

> **Not an officially supported product of Google, Microsoft, or Apple.**
> Forked from [googleworkspace/cli](https://github.com/googleworkspace/cli) by Justin Poehnelt (Apache 2.0).

---

## What Is uws?

`uws` is a **universal, schema-driven command-line interface** that gives you — and your AI agents — structured, JSON-first access to every major productivity ecosystem from a single tool.

```bash
# Google Workspace
uws gmail users messages list --params '{"userId": "me", "q": "is:unread"}'

# Microsoft 365
uws ms-mail messages list --params '{"$filter": "isRead eq false"}'

# Apple iCloud
uws apple-calendar events list

# Android
uws android-devices list --params '{"customerId": "my_customer"}'

# Chrome
uws chrome-policy resolve --json '{"policyTargetKey": {"targetResource": "orgunits/ID"}}'
```

All output is clean JSON. All commands work the same way. All ecosystems. One tool.

---

## The Bigger Picture: Aluminum OS

`uws` is the **command surface** of a larger vision called **Aluminum** — an AI-native OS substrate that abstracts Google, Microsoft, and Apple into interchangeable provider drivers, the same way Kubernetes abstracts cloud providers.

```bash
alum mail send --to "alice@example.com"           # auto-detects provider
alum drive list --provider microsoft               # OneDrive
alum calendar create --ai "team sync tomorrow"     # AI-assisted, any backend
alum search "Q1 budget" --provider all             # searches ALL ecosystems simultaneously
alum sync calendar --from google --to microsoft    # cross-provider sync
```

> *"You're not building three CLIs. You're building one CLI with three backends."*
> — Microsoft Copilot, architectural review, March 2026

Read the full architecture specification: **[ALUMINUM.md](ALUMINUM.md)**

---

## Why This Exists

Google shipped [gws](https://github.com/googleworkspace/cli) — a brilliant CLI for Google Workspace that proved the pattern: AI agents work best when they have a clean, JSON-first command surface over APIs. We forked it and extended the pattern to **every ecosystem**.

| What gws gave us | What uws adds |
|---|---|
| Google Workspace (Drive, Gmail, Calendar, Docs, Sheets, Slides, Tasks, People, Chat, Forms, Keep, Meet) | **Microsoft 365** (Outlook, OneDrive, Teams, To Do, OneNote, SharePoint, Planner) |
| OAuth2 auth model | **Apple iCloud** (CalDAV, CardDAV, CloudKit) |
| JSON-first output | **Android** (Management API, Messages) |
| AI agent SKILL.md files | **Chrome** (Management, Policy, Web Store, ChromeOS) |
| Schema-driven discovery | **Multi-provider abstraction layer** (Aluminum) |
| Model Armor safety layer | **Claude + Manus + Gemini + Copilot** agent runtime |

---

## Installation

### One-Liner (Recommended)

No Rust required. Downloads a pre-built binary for your platform from GitHub Releases:

```bash
curl -fsSL https://raw.githubusercontent.com/splitmerge420/uws/main/install.sh | bash
```

Installs to `/usr/local/bin/uws` (with sudo) or `~/.local/bin/uws` (without sudo).

To install a specific version:

```bash
UWS_VERSION=v0.1.0 curl -fsSL https://raw.githubusercontent.com/splitmerge420/uws/main/install.sh | bash
```

### npm / pnpm

```bash
pnpm add -g @splitmerge420/uws   # or: npm install -g @splitmerge420/uws
```

### From Source (Rust required)

```bash
git clone https://github.com/splitmerge420/uws
cd uws
cargo build --release
sudo cp target/release/uws /usr/local/bin/uws
uws --version
```

### Homebrew *(coming soon)*

```bash
brew install splitmerge420/tap/uws
```

### GitHub Codespaces

Click **Code → Codespaces → Create codespace** on the repo page. Rust, Python, and Node are pre-configured — no local install needed.

---

## Authentication

### Google Workspace

```bash
uws auth setup    # Interactive wizard — creates OAuth credentials
uws auth login    # Browser-based OAuth2 flow
uws auth status   # Check current auth state
```

### Microsoft 365

```bash
uws ms-auth setup    # Step-by-step Azure app registration guide
uws ms-auth login    # OAuth2 browser flow
uws ms-auth status   # Check current auth state
```

```bash
export UWS_MS_CLIENT_ID=<azure-app-client-id>
export UWS_MS_CLIENT_SECRET=<azure-app-client-secret>
export UWS_MS_TENANT_ID=common
```

### Apple iCloud

```bash
uws apple-auth setup    # Guide for app-specific password setup
uws apple-auth status   # Check current auth state
```

```bash
export UWS_APPLE_ID=your@icloud.com
export UWS_APPLE_APP_PASSWORD=xxxx-xxxx-xxxx-xxxx   # from appleid.apple.com
```

---

## Command Reference

### Syntax

```
uws <service> <resource> [sub-resource] <method> [flags]
```

### Universal Flags

| Flag | Description |
|---|---|
| `--params <JSON>` | URL/query parameters as JSON |
| `--json <JSON>` | Request body as JSON (POST/PATCH/PUT) |
| `--upload <PATH>` | Upload a local file as media |
| `--output <PATH>` | Save binary response to file |
| `--format <FMT>` | Output format: `json` (default), `table`, `yaml`, `csv` |
| `--api-version <VER>` | Override API version |
| `--page-all` | Auto-paginate all results (NDJSON) |
| `--page-limit <N>` | Max pages with `--page-all` (default: 10) |
| `--dry-run` | Print the request without executing it |

### Google Workspace Services

| Service | Description |
|---|---|
| `drive` | Files, folders, and shared drives |
| `gmail` | Send, read, and manage email |
| `calendar` | Calendars and events |
| `docs` | Google Docs |
| `sheets` | Google Sheets |
| `slides` | Google Slides |
| `tasks` | Task lists and tasks |
| `people` | Contacts and profiles |
| `chat` | Chat spaces and messages |
| `forms` | Google Forms |
| `keep` | Google Keep notes |
| `meet` | Google Meet conferences |
| `classroom` | Google Classroom |
| `workflow` | Cross-service productivity workflows |

### Microsoft 365 Services

| Service | Description |
|---|---|
| `ms-mail` | Outlook Mail: read, send, manage email |
| `ms-calendar` | Outlook Calendar: events and meetings |
| `ms-onedrive` | OneDrive: files, folders, sharing |
| `ms-teams` | Teams: channels, messages, meetings |
| `ms-todo` | Microsoft To Do: task lists |
| `ms-onenote` | OneNote: notebooks, sections, pages |
| `ms-contacts` | Outlook Contacts |
| `ms-sharepoint` | SharePoint: sites and document libraries |
| `ms-planner` | Microsoft Planner: plans and tasks |

### Apple Ecosystem Services

| Service | Description |
|---|---|
| `apple-calendar` | iCloud Calendar (CalDAV) |
| `apple-reminders` | Reminders (CalDAV VTODO) |
| `apple-contacts` | iCloud Contacts (CardDAV) |
| `apple-drive` | iCloud Drive (CloudKit) |
| `apple-notes` | iCloud Notes (CloudKit) |

### Android & Chrome Services

| Service | Description |
|---|---|
| `android-management` | Android Management API: enterprise devices |
| `android-messages` | Google Messages: RCS Business Messaging |
| `android-devices` | Android devices in Google Workspace |
| `chrome-management` | Chrome device telemetry and app reports |
| `chrome-policy` | Chrome Policy API |
| `chrome-extensions` | Chrome Web Store and extension management |
| `chrome-devices` | ChromeOS device management |

---

## Usage Examples

### Morning Briefing (all ecosystems)

```bash
#!/bin/bash
echo "=== Gmail (unread) ==="
uws gmail users messages list \
  --params '{"userId":"me","q":"is:unread","maxResults":5}' --format table

echo "=== Outlook (unread) ==="
uws ms-mail messages list \
  --params '{"$top":5,"$filter":"isRead eq false","$select":"subject,from,receivedDateTime"}' \
  --format table

echo "=== Today on Google Calendar ==="
uws calendar events list \
  --params '{"calendarId":"primary","timeMin":"'$(date -u +%Y-%m-%dT00:00:00Z)'","singleEvents":true}' \
  --format table
```

### Cross-Provider File Search

```bash
uws drive files list --params '{"q": "name contains '\''budget'\''"}' --format table
uws ms-onedrive drive root search --params '{"q": "budget"}' --format table
```

---

## AI Agent Integration

### Claude (Anthropic)

```json
{
  "name": "uws",
  "description": "Universal Workspace CLI. Reads and writes data across Google Workspace, Microsoft 365, Apple iCloud, Android, and Chrome. Always outputs JSON.",
  "input_schema": {
    "type": "object",
    "properties": {
      "command": { "type": "string", "description": "Full uws command string" }
    },
    "required": ["command"]
  }
}
```

See [CLAUDE.md](CLAUDE.md) for the full Claude integration guide.

### Manus

Place `skills/uws-core/SKILL.md` in your Manus skills directory. Manus will automatically discover and use `uws` for any task involving email, calendar, files, contacts, or productivity data across any ecosystem.

See [AGENTS.md](AGENTS.md) for the full multi-agent integration guide.

### Gemini

```python
import subprocess, json

def uws(command: str) -> dict:
    result = subprocess.run(["uws"] + command.split(), capture_output=True, text=True)
    try:
        return json.loads(result.stdout)
    except json.JSONDecodeError:
        return {"error": result.stderr, "raw": result.stdout}
```

### GitHub Copilot (MCP)

`uws` runs as a local **Model Context Protocol** server, giving GitHub Copilot chat direct access to all 12,000+ unified operations:

```bash
# Start the MCP server (stdio transport — for Claude Desktop, VS Code, Cursor)
python3 mcp_server/server.py --transport stdio

# Start the MCP server (HTTP transport — for Copilot Studio, remote agents)
python3 mcp_server/server.py --transport http --port 8787
```

Add to your **Claude Desktop** `config.json`:

```json
{
  "mcpServers": {
    "uws": {
      "command": "python3",
      "args": ["/path/to/uws/mcp_server/server.py", "--transport", "stdio"]
    }
  }
}
```

Or copy `mcp_server/mcp.json` into your project root to configure any MCP-compatible tool automatically.

---

## Spheres OS Toolchain

The `toolchain/` directory contains the full CI/CD pipeline and governance infrastructure for the 144-Sphere Ontology and Constitutional Invariant system.

| File | Description |
|---|---|
| `toolchain/invariants_registry.py` | Canonical registry of all 39 Constitutional Invariants (INV-1 through INV-36 incl. sub-invariants) |
| `toolchain/invariant_linter.py` | Stage 1: Lint code against all invariants |
| `toolchain/kintsugi_healer.py` | Stage 2: Kintsugi Code Healing — fracture → mend → beauty score |
| `toolchain/pqc_provider.py` | Stage 3: Post-Quantum Cryptographic signing (ML-DSA/Dilithium) with encrypted key storage |
| `toolchain/stress_test.py` | Stage 4: Resilience stress testing with graceful degradation |
| `toolchain/spheres_pipeline.py` | Orchestrator: Lint → Heal → Sign → Stress (fail-fast, 300s timeout) |
| `toolchain/acp_governance.py` | Agent Constitutional Protocol — PolicyRegistry, CouncilVoting, AuditChain |
| `toolchain/opa_rego_engine.py` | Python-native OPA/Rego policy evaluator with W3C PROV provenance |
| `toolchain/graphiti_temporal.py` | Temporal knowledge graph for causal/event tracking alongside ChromaDB |
| `toolchain/predictive_fix_engine.py` | Multi-provider LLM fix engine with timeout + caching |
| `toolchain/manus_sphere_adapter.py` | Maps Manus ontology to canonical 144-Sphere System B (Houses) |
| `toolchain/fix_cataloguer.py` | Fix cataloguing with Notion sync engine |
| `toolchain/three_tier_archive.py` | Hot (memory) → Warm (SQLite) → Cold (content-addressed) archive |
| `toolchain/health_connectors.py` | FHIR R4 / AHCEP health connector stubs + OneMedical crisis adapter |
| `toolchain/policies/*.rego` | Rego policy files for consent, data classification, and audit |
| `ingestion/master_ingestion.json` | 200-repo ingestion map |
| `ingestion/verified_ontology.md` | Verified ontology documenting System A vs System B |

---

## Architecture & Roadmap

`uws` is built on the **Aluminum OS** architecture. Read the full spec: **[ALUMINUM.md](ALUMINUM.md)**

| Phase | Milestone | Status |
|---|---|---|
| 1 | Fork gws → abstract provider layer (`uws` v0.1) | **Complete** |
| 2 | Microsoft Graph backend (`uws` v0.2 / Alexandria) | **In Progress** |
| 3 | Apple Intents backend (`uws` v0.3) | Planned |
| 4 | Aluminum kernel APIs (`alum` v0.1) | Planned |
| 5 | Full Aluminum-native command surface (`alum` v1.0) | Vision |

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). To add a new provider driver, implement the `ProviderDriver` trait and register it in `src/services.rs`. See [ALUMINUM.md](ALUMINUM.md) for the interface spec.

---

## Credits

- **Daavud Sheldon** — uws architecture and multi-ecosystem extension
- **Justin Poehnelt** — original `gws` implementation
- **Microsoft Copilot** — Aluminum OS architectural review
- **Google LLC** — original `gws` codebase (Apache 2.0)

---

## License

Apache License 2.0. See [LICENSE](LICENSE).

**Star this repo if you believe the future of productivity is one unified command surface.**
[github.com/splitmerge420/uws](https://github.com/splitmerge420/uws)