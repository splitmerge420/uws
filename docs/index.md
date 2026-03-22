# uws — Universal Workspace CLI

**One CLI. Every productivity ecosystem. Built for humans and AI agents.**

```bash
curl -fsSL https://raw.githubusercontent.com/splitmerge420/uws/main/install.sh | bash
```

---

## What Is uws?

`uws` is a universal, schema-driven command-line interface that gives you — and your AI agents —
structured, JSON-first access to every major productivity ecosystem from a single tool.

| Ecosystem | Services |
|---|---|
| **Google Workspace** | Gmail, Drive, Calendar, Docs, Sheets, Slides, Tasks, Chat, Forms, Keep, Meet |
| **Microsoft 365** | Outlook, OneDrive, Teams, To Do, OneNote, SharePoint, Planner |
| **Apple iCloud** | Calendar (CalDAV), Contacts (CardDAV), Drive (CloudKit), Notes, Reminders |
| **Android** | Management API, Messages (RCS) |
| **Chrome** | Management, Policy, Web Store, ChromeOS |

## Why uws?

- **Zero boilerplate** — one grammar for 12,000+ operations across all providers
- **JSON-first** — every response is clean, structured JSON; perfect for AI agents and pipelines
- **AI-native** — integrates with Claude, GitHub Copilot (MCP), Gemini, and Manus
- **Aluminum OS** — the command surface of a provider-agnostic OS for AI-augmented productivity

## Quick Example

```bash
# Read your unread Gmail
uws gmail users messages list --params '{"userId":"me","q":"is:unread","maxResults":5}'

# Read your unread Outlook
uws ms-mail messages list --params '{"$filter":"isRead eq false","$top":5}'

# List OneDrive files
uws ms-onedrive drive root children list --format table

# Create a calendar event
uws calendar events insert --params '{"calendarId":"primary"}' \
  --json '{"summary":"Team sync","start":{"dateTime":"2026-03-25T10:00:00Z"},"end":{"dateTime":"2026-03-25T11:00:00Z"}}' \
  --dry-run
```

## Get Started

- **[Installation](installation.md)** — curl, npm, Homebrew, from source
- **[Quickstart](quickstart.md)** — your first commands in 5 minutes
- **[AI Agents](agents/overview.md)** — integrate with Claude, Copilot, Gemini, Manus
- **[Architecture](aluminum.md)** — the Aluminum OS vision

---

*[GitHub](https://github.com/splitmerge420/uws) · [Issues](https://github.com/splitmerge420/uws/issues) · [Discussions](https://github.com/splitmerge420/uws/discussions) · [Security](https://github.com/splitmerge420/uws/security)*
