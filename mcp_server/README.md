# Aluminum OS — MCP Server

This directory contains the Model Context Protocol (MCP) server that exposes the entire `uws` command surface to any MCP-compliant AI agent, including Microsoft Copilot, Claude, Manus, and Gemini.

## Quick Start

```bash
# HTTP mode (for Copilot Studio, remote agents)
python3 mcp_server/server.py --transport http --port 8787

# stdio mode (for local agents like Claude Desktop)
python3 mcp_server/server.py --transport stdio
```

## Registered Tools

| Tool | Description |
|---|---|
| `alum_discover` | Discover all available services, resources, and methods across all providers |
| `alum_execute` | Execute any uws/alum command (universal tool for all 12,000+ operations) |
| `alum_search` | Search across all connected ecosystems simultaneously |
| `alum_mail_send` | Send email via Gmail or Outlook |
| `alum_calendar_create` | Create calendar events on any provider |
| `alum_drive_list` | List files from any cloud storage |
| `alum_tasks_list` | List tasks from any task manager |
| `alum_contacts_search` | Search contacts across all providers |

## Copilot Studio Integration

1. Deploy this server to a publicly accessible endpoint
2. In Copilot Studio, create a new "Custom Connector" pointing to `https://your-server/mcp`
3. Create a Declarative Agent that uses this connector
4. Copilot now has access to all 12,000-20,000+ unified operations

## Architecture

```
Agent (Copilot/Claude/Manus/Gemini)
  │
  ▼
MCP Client
  │
  ▼
This Server (server.py)
  │
  ▼
uws CLI binary
  │
  ├──▶ Google APIs (Discovery Documents)
  ├──▶ Microsoft Graph API
  ├──▶ Apple iCloud (CalDAV/CardDAV/CloudKit)
  └──▶ Android/Chrome Management APIs
```
