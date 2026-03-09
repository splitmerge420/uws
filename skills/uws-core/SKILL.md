# uws — Universal Workspace CLI Skill

## Overview

`uws` is the Universal Workspace CLI. It is a single command-line tool that provides
structured, JSON-first access to **all major productivity ecosystems**:

| Ecosystem | Services |
|---|---|
| **Google Workspace** | Drive, Gmail, Calendar, Docs, Sheets, Slides, Tasks, People, Chat, Forms, Keep, Meet |
| **Microsoft 365** | Outlook Mail, Outlook Calendar, OneDrive, Teams, To Do, OneNote, Contacts, SharePoint |
| **Apple** | iCloud Calendar (CalDAV), iCloud Contacts (CardDAV), iCloud Drive, Notes, Reminders |
| **Android** | Android Management API, Google Messages (RCS) |
| **Chrome** | Chrome Management, Chrome Policy, Chrome Web Store, ChromeOS Devices |

AI agents (Claude, Manus, Gemini) use `uws` as a universal tool layer to read and write
data across all of a user's workspaces without needing separate integrations.

---

## Installation

```bash
# From source (requires Rust)
git clone https://github.com/splitmerge420/uws
cd uws
cargo build --release
sudo cp target/release/uws /usr/local/bin/uws

# Verify
uws --version
```

---

## Authentication

### Google Workspace
```bash
uws auth setup    # Interactive setup wizard
uws auth login    # OAuth2 browser flow
uws auth status   # Check current auth state
```

### Microsoft 365
```bash
uws ms-auth setup    # Azure app registration guide
uws ms-auth login    # OAuth2 browser flow
uws ms-auth status   # Check current auth state
```
Required env vars:
- `UWS_MS_CLIENT_ID` — Azure app client ID
- `UWS_MS_CLIENT_SECRET` — Azure app client secret
- `UWS_MS_TENANT_ID` — Azure AD tenant (default: `common`)

### Apple iCloud
```bash
uws apple-auth setup    # Step-by-step guide
uws apple-auth status   # Check current auth state
```
Required env vars:
- `UWS_APPLE_ID` — Apple ID email
- `UWS_APPLE_APP_PASSWORD` — App-specific password from appleid.apple.com

---

## Command Syntax

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
| `--page-all` | Auto-paginate all results (NDJSON output) |
| `--dry-run` | Print the request without executing it |

---

## Usage Examples

### Google Workspace

```bash
# List recent emails
uws gmail users messages list --params '{"userId": "me", "maxResults": 10}'

# Create a calendar event
uws calendar events insert \
  --params '{"calendarId": "primary"}' \
  --json '{"summary": "Team standup", "start": {"dateTime": "2026-03-09T09:00:00Z"}, "end": {"dateTime": "2026-03-09T09:30:00Z"}}'

# List Drive files
uws drive files list --params '{"pageSize": 20}' --format table

# Read a Google Doc
uws docs documents get --params '{"documentId": "YOUR_DOC_ID"}'
```

### Microsoft 365

```bash
# List Outlook inbox
uws ms-mail messages list --params '{"$top": 10, "$select": "subject,from,receivedDateTime"}'

# Create a To Do task
uws ms-todo tasks create \
  --json '{"title": "Review PR", "importance": "high"}'

# List OneDrive files
uws ms-onedrive drive root children list

# Get Teams channels
uws ms-teams channels list --params '{"teamId": "YOUR_TEAM_ID"}'
```

### Apple iCloud

```bash
# List iCloud calendars (CalDAV PROPFIND)
uws apple-calendar calendars list

# List iCloud contacts (CardDAV)
uws apple-contacts list

# List Reminders
uws apple-reminders lists list
```

### Android & Chrome

```bash
# List managed Android devices
uws android-devices list --params '{"customerId": "my_customer"}'

# List ChromeOS devices
uws chrome-devices list --params '{"customerId": "my_customer"}'

# Get Chrome policy for an org unit
uws chrome-policy resolve --json '{"policyTargetKey": {"targetResource": "orgunits/ORGUNIT_ID"}}'
```

---

## AI Agent Integration

### For Claude (Anthropic)

Add `uws` to your Claude tool configuration. The CLI outputs clean JSON on stdout,
making it trivially parseable by any LLM.

```json
{
  "name": "uws",
  "description": "Universal Workspace CLI. Reads and writes data across Google Workspace, Microsoft 365, Apple iCloud, Android, and Chrome. Always outputs JSON.",
  "input_schema": {
    "type": "object",
    "properties": {
      "command": {
        "type": "string",
        "description": "Full uws command string, e.g. 'gmail users messages list --params {\"userId\":\"me\"}'"
      }
    },
    "required": ["command"]
  }
}
```

**Claude system prompt addition:**
```
You have access to the `uws` tool which gives you read/write access to the user's
Google Workspace, Microsoft 365, Apple iCloud, Android, and Chrome accounts.
Always use --dry-run first to confirm the operation before executing writes.
Always use --format json for machine-readable output.
```

### For Manus

Place this file at `/home/ubuntu/skills/uws-core/SKILL.md`. Manus will automatically
discover and use `uws` for any task involving email, calendar, files, contacts, or
productivity data across any ecosystem.

**Manus skill invocation pattern:**
```bash
# Read
uws <service> <resource> <method> --params '<json>' --format json

# Write (always dry-run first)
uws <service> <resource> <method> --params '<json>' --json '<body>' --dry-run
uws <service> <resource> <method> --params '<json>' --json '<body>'
```

### For Gemini (via Google AI Studio)

Use `uws` as a function tool in Gemini's function calling API:

```python
import subprocess, json

def uws(command: str) -> dict:
    """Execute a uws command and return parsed JSON output."""
    result = subprocess.run(
        ["uws"] + command.split(),
        capture_output=True, text=True
    )
    try:
        return json.loads(result.stdout)
    except json.JSONDecodeError:
        return {"error": result.stderr, "raw": result.stdout}
```

---

## Cross-Platform Workflow Examples

### Morning Briefing (all ecosystems)
```bash
#!/bin/bash
echo "=== Gmail ==="
uws gmail users messages list --params '{"userId":"me","q":"is:unread","maxResults":5}' --format table

echo "=== Outlook ==="
uws ms-mail messages list --params '{"$top":5,"$filter":"isRead eq false"}' --format table

echo "=== Google Calendar Today ==="
uws calendar events list --params '{"calendarId":"primary","timeMin":"'$(date -u +%Y-%m-%dT00:00:00Z)'","singleEvents":true}' --format table

echo "=== Outlook Calendar Today ==="
uws ms-calendar events list --params '{"$filter":"start/dateTime ge '"'"'$(date -u +%Y-%m-%dT00:00:00Z)'"'"'"}' --format table
```

### Unified File Search
```bash
# Search Google Drive
uws drive files list --params '{"q": "name contains '\''budget'\''", "pageSize": 5}'

# Search OneDrive
uws ms-onedrive search query --params '{"q": "budget"}'
```

---

## Environment Variables Reference

| Variable | Description |
|---|---|
| `GOOGLE_WORKSPACE_CLI_TOKEN` | Pre-obtained Google OAuth2 token |
| `GOOGLE_WORKSPACE_CLI_CREDENTIALS_FILE` | Path to Google OAuth credentials JSON |
| `GOOGLE_WORKSPACE_CLI_CONFIG_DIR` | Config directory (default: `~/.config/uws`) |
| `UWS_MS_CLIENT_ID` | Microsoft Azure app client ID |
| `UWS_MS_CLIENT_SECRET` | Microsoft Azure app client secret |
| `UWS_MS_TENANT_ID` | Azure AD tenant ID |
| `UWS_MS_TOKEN` | Pre-obtained Microsoft Graph token |
| `UWS_APPLE_ID` | Apple ID email address |
| `UWS_APPLE_APP_PASSWORD` | Apple app-specific password |
| `UWS_APPLE_CLIENT_ID` | Apple Sign In client ID |
| `UWS_APPLE_TEAM_ID` | Apple Developer team ID |
| `UWS_APPLE_KEY_ID` | Apple private key ID |
| `UWS_APPLE_PRIVATE_KEY_FILE` | Path to Apple .p8 private key |
| `ANTHROPIC_API_KEY` | Claude API key for AI agent skills |
| `GEMINI_API_KEY` | Gemini API key for AI agent skills |

---

## Output Formats

All commands support `--format` with these options:

| Format | Use Case |
|---|---|
| `json` | Default. Machine-readable. Ideal for AI agents and piping. |
| `table` | Human-readable tabular output for terminals. |
| `yaml` | YAML output for config files and Kubernetes-style tooling. |
| `csv` | Spreadsheet-compatible output. |

---

## Repository

- **GitHub**: https://github.com/splitmerge420/uws
- **Original gws project**: https://github.com/googleworkspace/cli
- **License**: Apache 2.0
- **Maintainer**: Daavud Sheldon

> uws is an independent open-source project, not affiliated with Google, Microsoft, or Apple.
