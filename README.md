<h1 align="center">uws — Universal Workspace CLI</h1>

<p align="center">
  <strong>One CLI to rule them all.</strong><br>
  Google Workspace. Microsoft 365. Apple. GitHub. Android. Chrome.<br>
  Built for humans and AI agents. Zero boilerplate.
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-Apache%202.0-blue.svg" alt="License"></a>
  <a href="https://github.com/splitmerge420/uws"><img src="https://img.shields.io/github/stars/splitmerge420/uws" alt="Stars"></a>
  <a href="https://github.com/splitmerge420/uws/actions/workflows/ci.yml"><img src="https://github.com/splitmerge420/uws/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/googleworkspace/cli"><img src="https://img.shields.io/badge/forked%20from-googleworkspace%2Fcli-brightgreen" alt="Forked from gws"></a>
</p>

> **Not an officially supported product of Google, Microsoft, Apple, or GitHub.**
> Forked from [googleworkspace/cli](https://github.com/googleworkspace/cli) by Justin Poehnelt (Apache 2.0).

---

## What Is uws?

`uws` is a **universal, schema-driven command-line interface** that gives you — and your AI agents — structured, JSON-first access to every major productivity ecosystem from a single tool.

```bash
# GitHub (no OAuth — just a PAT or GITHUB_TOKEN)
uws github repos list
uws github issues list --params '{"owner":"octocat","repo":"Hello-World","state":"open"}'
uws github actions runs --params '{"owner":"octocat","repo":"Hello-World"}'

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
| JSON-first output | **GitHub** (Repos, Issues, PRs, Actions, Search — PAT auth, no OAuth needed) |
| AI agent SKILL.md files | **Android** (Management API, Messages) |
| Schema-driven discovery | **Chrome** (Management, Policy, Web Store, ChromeOS) |
| Model Armor safety layer | **Multi-provider abstraction layer** (Aluminum) + **Claude + Manus + Gemini + Copilot** |

---

## Why GitHub Specifically Benefits

uws turns GitHub into a first-class productivity provider alongside Google and Microsoft:

| GitHub Surface | How uws Unlocks It |
|---|---|
| **100M+ developers already have `GITHUB_TOKEN`** | Zero new auth setup — PAT works immediately |
| **GitHub Actions** | `uses: splitmerge420/uws@main` — call any API from any workflow |
| **GitHub Copilot** | `.github/copilot-instructions.md` — Copilot auto-suggests uws commands |
| **Copilot Extensions** | `copilot-extension.json` — uws skills surfaced in Copilot Chat |
| **gh CLI** | `gh extension install splitmerge420/uws` — one command, full access |
| **GitHub Codespaces** | `.devcontainer/devcontainer.json` — instant dev environment |
| **GitHub Issues / PRs** | Triaging and commenting from CI workflows or AI agents |
| **GitHub Search** | Code, repos, users, commits — all queryable as JSON |

---

## Installation

### 1. One-line installer (Linux / macOS)

```bash
curl -fsSL https://raw.githubusercontent.com/splitmerge420/uws/main/install.sh | sh
```

### 2. gh CLI Extension

```bash
gh extension install splitmerge420/uws
gh uws github repos list        # GITHUB_TOKEN injected automatically
gh uws gmail users messages list --params '{"userId":"me","q":"is:unread"}'
```

### 3. GitHub Actions

```yaml
- uses: splitmerge420/uws@main
  with:
    command: github issues list --params '{"owner":"${{ github.repository_owner }}","repo":"${{ github.event.repository.name }}","state":"open"}'
  id: open-issues
- run: echo '${{ steps.open-issues.outputs.result }}' | jq '.[].title'
```

### 4. GitHub Codespaces

Click **Code → Codespaces → Create codespace**. The `.devcontainer` configuration installs Rust, rust-analyzer, GitHub CLI, and GitHub Copilot automatically.

### 5. From Source (Rust required)

```bash
git clone https://github.com/splitmerge420/uws
cd uws
cargo build --release
sudo cp target/release/uws /usr/local/bin/uws
uws --version
```

### 6. Homebrew *(coming soon)*

```bash
brew install splitmerge420/tap/uws
```

---

## Authentication

### GitHub (no OAuth needed)

```bash
export GITHUB_TOKEN=ghp_xxxxxxxxxxxxxxxxxxxx   # or use gh auth login
uws github repos list

# In GitHub Actions — GITHUB_TOKEN is already available:
uws github issues create --params '{"owner":"...","repo":"..."}' --json '{"title":"..."}'
```

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

### GitHub Services

| Service | Description |
|---|---|
| `github` | Full GitHub REST API — see [skills/github/SKILL.md](skills/github/SKILL.md) |

Auth: set `GITHUB_TOKEN` or `UWS_GITHUB_TOKEN`. No OAuth flow needed.

```bash
uws github repos list
uws github issues list --params '{"owner":"octocat","repo":"Hello-World","state":"open"}'
uws github pulls get --params '{"owner":"octocat","repo":"Hello-World","pull_number":1}'
uws github releases latest --params '{"owner":"octocat","repo":"Hello-World"}'
uws github actions runs --params '{"owner":"octocat","repo":"Hello-World"}'
uws github search repos --params '{"q":"language:rust stars:>1000"}'
uws github user me
uws github notifications list
```

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
echo "=== GitHub notifications ==="
uws github notifications list --params '{"all":false}' \
  | jq '[.[] | {title: .subject.title, type: .subject.type, repo: .repository.full_name}]'

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
uws drive files list --params '{"q": "name contains ''budget''"}' --format table
uws ms-onedrive drive root search --params '{"q": "budget"}' --format table
```

### GitHub CI Monitor (in a workflow)

```yaml
- uses: splitmerge420/uws@main
  with:
    command: >
      github actions runs
      --params '{"owner":"${{ github.repository_owner }}","repo":"${{ github.event.repository.name }}","status":"failure","per_page":3}'
  id: failing-runs

- name: Comment on failing runs
  run: |
    echo '${{ steps.failing-runs.outputs.result }}' \
      | jq '.workflow_runs[] | "\u274c \(.name) — \(.html_url)"'
```

---

## AI Agent Integration

### GitHub Copilot

Copy or symlink `.github/copilot-instructions.md` — it's already in this repo. Copilot will automatically read it and suggest correct `uws github` commands in Copilot Chat.

```bash
# In Copilot Chat (after installing the gh extension):
# "List open issues in this repo" → Copilot suggests:
uws github issues list --params '{"owner":"splitmerge420","repo":"uws","state":"open"}'
```

### Claude (Anthropic)

```json
{
  "name": "uws",
  "description": "Universal Workspace CLI. Reads and writes data across Google Workspace, Microsoft 365, Apple iCloud, GitHub, Android, and Chrome. Always outputs JSON.",
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
| 1b | GitHub provider (`uws github`) — PAT auth, no OAuth | **Complete** |
| 2 | Microsoft Graph backend (`uws` v0.2 / Alexandria) | **In Progress** |
| 3 | Apple Intents backend (`uws` v0.3) | Planned |
| 4 | Aluminum kernel APIs (`alum` v0.1) | Planned |
| 5 | Full Aluminum-native command surface (`alum` v1.0) | Vision |

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). To add a new provider driver, implement the `ProviderDriver` trait and register it in `src/services.rs`. See [ALUMINUM.md](ALUMINUM.md) for the interface spec.

New providers follow the `src/github_provider.rs` pattern:
1. Create `src/<provider>_provider.rs` with `is_<provider>_service()` + `build_request()` + unit tests
2. Register the early dispatch in `src/main.rs`
3. Add a `skills/<provider>/SKILL.md`
4. Add env vars to `.env.example`

---

## Credits

- **Daavud Sheldon** — uws architecture and multi-ecosystem extension
- **Justin Poehnelt** — original `gws` implementation
- **GitHub Copilot** — GitHub provider implementation and adoption engineering
- **Microsoft Copilot** — Aluminum OS architectural review
- **Google LLC** — original `gws` codebase (Apache 2.0)

---

## License

Apache License 2.0. See [LICENSE](LICENSE).

**Star this repo if you believe the future of productivity is one unified command surface.**
[github.com/splitmerge420/uws](https://github.com/splitmerge420/uws)
