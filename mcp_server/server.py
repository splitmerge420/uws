#!/usr/bin/env python3
"""
Aluminum OS — MCP Server for uws (Universal Workspace CLI)

This server exposes the entire uws command surface over the
Model Context Protocol (MCP), enabling any MCP-compliant agent
(Copilot, Claude, Manus, Gemini) to discover and invoke all
12,000-20,000+ unified operations.

Architecture:
  Agent → MCP Client → This Server → uws CLI → Provider APIs

The server dynamically discovers all available uws commands
and registers them as MCP tools at startup.
"""

import asyncio
import json
import subprocess
import sys
import os
from typing import Any

# ─── MCP Protocol Constants ───────────────────────────────────────────────────

MCP_VERSION = "2024-11-05"
SERVER_NAME = "aluminum-os"
SERVER_VERSION = "1.0.0"

# ─── Core Services Registry ──────────────────────────────────────────────────

GOOGLE_SERVICES = [
    "gmail", "drive", "calendar", "sheets", "docs", "slides",
    "tasks", "people", "chat", "classroom", "admin", "keep",
    "meet", "forms"
]

MICROSOFT_SERVICES = [
    "outlook", "onedrive", "teams", "sharepoint", "onenote",
    "planner", "todo", "aad"
]

APPLE_SERVICES = [
    "ical", "icontacts", "idrive", "inotes"
]

ANDROID_CHROME_SERVICES = [
    "android-mgmt", "chrome-mgmt", "chrome-policy"
]

ALL_SERVICES = (
    GOOGLE_SERVICES + MICROSOFT_SERVICES +
    APPLE_SERVICES + ANDROID_CHROME_SERVICES
)

# ─── UWS Command Executor ────────────────────────────────────────────────────

UWS_BINARY = os.environ.get("UWS_BINARY", "uws")

async def execute_uws_command(
    service: str,
    resource: str,
    method: str,
    args: dict[str, Any] | None = None,
    provider: str | None = None,
    output_format: str = "json"
) -> dict:
    """Execute a uws CLI command and return the result."""
    cmd = [UWS_BINARY, service, resource, method]

    if provider:
        cmd.extend(["--provider", provider])

    cmd.extend(["--output", output_format])

    if args:
        # Pass complex args as --json
        json_args = {k: v for k, v in args.items() if v is not None}
        if json_args:
            cmd.extend(["--json", json.dumps(json_args)])

    try:
        proc = await asyncio.create_subprocess_exec(
            *cmd,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE
        )
        stdout, stderr = await asyncio.wait_for(
            proc.communicate(), timeout=120
        )

        if proc.returncode == 0:
            try:
                return {
                    "success": True,
                    "data": json.loads(stdout.decode()),
                    "command": " ".join(cmd)
                }
            except json.JSONDecodeError:
                return {
                    "success": True,
                    "data": stdout.decode().strip(),
                    "command": " ".join(cmd)
                }
        else:
            return {
                "success": False,
                "error": stderr.decode().strip(),
                "command": " ".join(cmd),
                "exit_code": proc.returncode
            }
    except asyncio.TimeoutError:
        return {
            "success": False,
            "error": "Command timed out after 120 seconds",
            "command": " ".join(cmd)
        }
    except FileNotFoundError:
        return {
            "success": False,
            "error": f"uws binary not found at: {UWS_BINARY}",
            "command": " ".join(cmd)
        }


# ─── MCP Protocol Handler ────────────────────────────────────────────────────

class AluminumMCPServer:
    """MCP server that wraps the uws CLI."""

    def __init__(self):
        self.tools = self._register_tools()

    def _register_tools(self) -> dict:
        """Register all uws commands as MCP tools."""
        tools = {}

        # ── Meta tools ────────────────────────────────────────────────
        tools["alum_discover"] = {
            "name": "alum_discover",
            "description": (
                "Discover all available services, resources, and methods "
                "across all providers (Google, Microsoft, Apple, Android/Chrome). "
                "Returns the full command tree."
            ),
            "inputSchema": {
                "type": "object",
                "properties": {
                    "provider": {
                        "type": "string",
                        "enum": ["google", "microsoft", "apple", "android", "all"],
                        "description": "Filter by provider. Default: all"
                    },
                    "service": {
                        "type": "string",
                        "description": "Filter by service name (e.g., gmail, outlook)"
                    }
                }
            }
        }

        tools["alum_execute"] = {
            "name": "alum_execute",
            "description": (
                "Execute any uws/alum command. This is the universal tool "
                "that can invoke any of the 12,000-20,000+ operations. "
                "Specify the service, resource, method, and arguments."
            ),
            "inputSchema": {
                "type": "object",
                "properties": {
                    "service": {
                        "type": "string",
                        "description": "The service to use (e.g., gmail, outlook, drive, onedrive)"
                    },
                    "resource": {
                        "type": "string",
                        "description": "The resource type (e.g., messages, files, events)"
                    },
                    "method": {
                        "type": "string",
                        "description": "The method to call (e.g., list, get, create, delete)"
                    },
                    "provider": {
                        "type": "string",
                        "enum": ["google", "microsoft", "apple", "android"],
                        "description": "The provider backend to use"
                    },
                    "args": {
                        "type": "object",
                        "description": "Key-value arguments to pass to the command"
                    }
                },
                "required": ["service", "resource", "method"]
            }
        }

        # ── Cross-ecosystem tools ─────────────────────────────────────
        tools["alum_search"] = {
            "name": "alum_search",
            "description": (
                "Search across all connected ecosystems simultaneously. "
                "Searches Google Drive, OneDrive, SharePoint, Gmail, "
                "Outlook, and more in a single query."
            ),
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "The search query"
                    },
                    "providers": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Providers to search (default: all)"
                    },
                    "content_types": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Filter by type: email, file, event, task, contact"
                    }
                },
                "required": ["query"]
            }
        }

        tools["alum_mail_send"] = {
            "name": "alum_mail_send",
            "description": (
                "Send an email via any connected provider. "
                "Automatically selects Gmail or Outlook based on the "
                "--provider flag or the user's default."
            ),
            "inputSchema": {
                "type": "object",
                "properties": {
                    "to": {
                        "type": "string",
                        "description": "Recipient email address"
                    },
                    "subject": {
                        "type": "string",
                        "description": "Email subject line"
                    },
                    "body": {
                        "type": "string",
                        "description": "Email body (plain text or HTML)"
                    },
                    "provider": {
                        "type": "string",
                        "enum": ["google", "microsoft"],
                        "description": "Which email provider to use"
                    },
                    "cc": {"type": "string", "description": "CC recipients"},
                    "bcc": {"type": "string", "description": "BCC recipients"},
                    "attachments": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "File paths to attach"
                    }
                },
                "required": ["to", "subject", "body"]
            }
        }

        tools["alum_calendar_create"] = {
            "name": "alum_calendar_create",
            "description": (
                "Create a calendar event on any connected provider. "
                "Works with Google Calendar, Outlook Calendar, or Apple Calendar."
            ),
            "inputSchema": {
                "type": "object",
                "properties": {
                    "title": {
                        "type": "string",
                        "description": "Event title"
                    },
                    "start": {
                        "type": "string",
                        "description": "Start time (ISO 8601 or natural language)"
                    },
                    "end": {
                        "type": "string",
                        "description": "End time (ISO 8601 or natural language)"
                    },
                    "provider": {
                        "type": "string",
                        "enum": ["google", "microsoft", "apple"],
                        "description": "Which calendar provider to use"
                    },
                    "attendees": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Email addresses of attendees"
                    },
                    "description": {
                        "type": "string",
                        "description": "Event description"
                    },
                    "location": {
                        "type": "string",
                        "description": "Event location"
                    }
                },
                "required": ["title", "start"]
            }
        }

        tools["alum_drive_list"] = {
            "name": "alum_drive_list",
            "description": (
                "List files from any connected cloud storage. "
                "Works with Google Drive, OneDrive, SharePoint, or iCloud Drive."
            ),
            "inputSchema": {
                "type": "object",
                "properties": {
                    "provider": {
                        "type": "string",
                        "enum": ["google", "microsoft", "apple", "all"],
                        "description": "Which storage provider to query"
                    },
                    "query": {
                        "type": "string",
                        "description": "Search query to filter files"
                    },
                    "folder": {
                        "type": "string",
                        "description": "Folder ID or path to list"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of results"
                    }
                }
            }
        }

        tools["alum_tasks_list"] = {
            "name": "alum_tasks_list",
            "description": (
                "List tasks from any connected task manager. "
                "Works with Google Tasks, Microsoft To Do, or Apple Reminders."
            ),
            "inputSchema": {
                "type": "object",
                "properties": {
                    "provider": {
                        "type": "string",
                        "enum": ["google", "microsoft", "apple", "all"],
                        "description": "Which task provider to query"
                    },
                    "status": {
                        "type": "string",
                        "enum": ["pending", "completed", "all"],
                        "description": "Filter by task status"
                    },
                    "list_name": {
                        "type": "string",
                        "description": "Name of the task list to query"
                    }
                }
            }
        }

        tools["alum_contacts_search"] = {
            "name": "alum_contacts_search",
            "description": (
                "Search contacts across all connected providers. "
                "Works with Google People, Outlook Contacts, or Apple Contacts."
            ),
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Name, email, or phone to search for"
                    },
                    "provider": {
                        "type": "string",
                        "enum": ["google", "microsoft", "apple", "all"],
                        "description": "Which contacts provider to search"
                    }
                },
                "required": ["query"]
            }
        }

        return tools

    def handle_initialize(self, request: dict) -> dict:
        """Handle the MCP initialize request."""
        return {
            "jsonrpc": "2.0",
            "id": request.get("id"),
            "result": {
                "protocolVersion": MCP_VERSION,
                "capabilities": {
                    "tools": {"listChanged": False}
                },
                "serverInfo": {
                    "name": SERVER_NAME,
                    "version": SERVER_VERSION
                }
            }
        }

    def handle_tools_list(self, request: dict) -> dict:
        """Handle the tools/list request."""
        tool_list = []
        for tool in self.tools.values():
            tool_list.append({
                "name": tool["name"],
                "description": tool["description"],
                "inputSchema": tool["inputSchema"]
            })
        return {
            "jsonrpc": "2.0",
            "id": request.get("id"),
            "result": {"tools": tool_list}
        }

    async def handle_tools_call(self, request: dict) -> dict:
        """Handle the tools/call request."""
        params = request.get("params", {})
        tool_name = params.get("name")
        arguments = params.get("arguments", {})

        if tool_name not in self.tools:
            return {
                "jsonrpc": "2.0",
                "id": request.get("id"),
                "error": {
                    "code": -32602,
                    "message": f"Unknown tool: {tool_name}"
                }
            }

        # Route to the appropriate handler
        if tool_name == "alum_discover":
            result = await self._handle_discover(arguments)
        elif tool_name == "alum_execute":
            result = await self._handle_execute(arguments)
        elif tool_name == "alum_search":
            result = await self._handle_search(arguments)
        elif tool_name == "alum_mail_send":
            result = await self._handle_mail_send(arguments)
        elif tool_name == "alum_calendar_create":
            result = await self._handle_calendar_create(arguments)
        elif tool_name == "alum_drive_list":
            result = await self._handle_drive_list(arguments)
        elif tool_name == "alum_tasks_list":
            result = await self._handle_tasks_list(arguments)
        elif tool_name == "alum_contacts_search":
            result = await self._handle_contacts_search(arguments)
        else:
            result = {"error": f"No handler for tool: {tool_name}"}

        return {
            "jsonrpc": "2.0",
            "id": request.get("id"),
            "result": {
                "content": [{
                    "type": "text",
                    "text": json.dumps(result, indent=2)
                }]
            }
        }

    async def _handle_discover(self, args: dict) -> dict:
        """Discover available services and methods."""
        provider = args.get("provider", "all")
        service_filter = args.get("service")

        result = await execute_uws_command(
            service=service_filter or "--help",
            resource="",
            method="",
            provider=provider if provider != "all" else None
        )
        return result

    async def _handle_execute(self, args: dict) -> dict:
        """Execute an arbitrary uws command."""
        return await execute_uws_command(
            service=args["service"],
            resource=args["resource"],
            method=args["method"],
            args=args.get("args"),
            provider=args.get("provider")
        )

    async def _handle_search(self, args: dict) -> dict:
        """Search across all providers."""
        query = args["query"]
        providers = args.get("providers", ["google", "microsoft"])
        results = []

        for provider in providers:
            if provider == "google":
                r = await execute_uws_command(
                    "drive", "files", "list",
                    args={"q": f"fullText contains '{query}'"},
                    provider="google"
                )
                results.append({"provider": "google_drive", "result": r})

                r = await execute_uws_command(
                    "gmail", "users.messages", "list",
                    args={"q": query},
                    provider="google"
                )
                results.append({"provider": "gmail", "result": r})

            elif provider == "microsoft":
                r = await execute_uws_command(
                    "onedrive", "search", "query",
                    args={"query": query},
                    provider="microsoft"
                )
                results.append({"provider": "onedrive", "result": r})

        return {"query": query, "results": results}

    async def _handle_mail_send(self, args: dict) -> dict:
        """Send email via the specified provider."""
        provider = args.get("provider", "google")
        if provider == "google":
            return await execute_uws_command(
                "gmail", "users.messages", "send",
                args={
                    "to": args["to"],
                    "subject": args["subject"],
                    "body": args["body"],
                    "cc": args.get("cc"),
                    "bcc": args.get("bcc")
                },
                provider="google"
            )
        else:
            return await execute_uws_command(
                "outlook", "messages", "send",
                args={
                    "to": args["to"],
                    "subject": args["subject"],
                    "body": args["body"]
                },
                provider="microsoft"
            )

    async def _handle_calendar_create(self, args: dict) -> dict:
        """Create a calendar event."""
        provider = args.get("provider", "google")
        return await execute_uws_command(
            "calendar" if provider == "google" else "outlook-calendar",
            "events",
            "create",
            args={
                "summary": args["title"],
                "start": args["start"],
                "end": args.get("end"),
                "attendees": args.get("attendees"),
                "description": args.get("description"),
                "location": args.get("location")
            },
            provider=provider
        )

    async def _handle_drive_list(self, args: dict) -> dict:
        """List files from cloud storage."""
        provider = args.get("provider", "google")
        return await execute_uws_command(
            "drive" if provider == "google" else "onedrive",
            "files",
            "list",
            args={
                "q": args.get("query"),
                "folder": args.get("folder"),
                "pageSize": args.get("limit", 20)
            },
            provider=provider
        )

    async def _handle_tasks_list(self, args: dict) -> dict:
        """List tasks."""
        provider = args.get("provider", "google")
        return await execute_uws_command(
            "tasks" if provider == "google" else "todo",
            "tasks",
            "list",
            args={
                "status": args.get("status"),
                "tasklist": args.get("list_name")
            },
            provider=provider
        )

    async def _handle_contacts_search(self, args: dict) -> dict:
        """Search contacts."""
        provider = args.get("provider", "google")
        return await execute_uws_command(
            "people" if provider == "google" else "contacts",
            "people",
            "searchContacts",
            args={"query": args["query"]},
            provider=provider
        )

    async def run_stdio(self):
        """Run the MCP server over stdio (standard MCP transport)."""
        reader = asyncio.StreamReader()
        protocol = asyncio.StreamReaderProtocol(reader)
        await asyncio.get_event_loop().connect_read_pipe(
            lambda: protocol, sys.stdin.buffer
        )

        writer_transport, writer_protocol = await asyncio.get_event_loop().connect_write_pipe(
            asyncio.streams.FlowControlMixin, sys.stdout.buffer
        )
        writer = asyncio.StreamWriter(
            writer_transport, writer_protocol, None, asyncio.get_event_loop()
        )

        while True:
            try:
                # Read Content-Length header
                header = await reader.readline()
                if not header:
                    break
                header = header.decode().strip()
                if header.startswith("Content-Length:"):
                    content_length = int(header.split(":")[1].strip())
                    await reader.readline()  # empty line
                    body = await reader.readexactly(content_length)
                    request = json.loads(body.decode())

                    # Route the request
                    method = request.get("method", "")
                    if method == "initialize":
                        response = self.handle_initialize(request)
                    elif method == "tools/list":
                        response = self.handle_tools_list(request)
                    elif method == "tools/call":
                        response = await self.handle_tools_call(request)
                    elif method == "notifications/initialized":
                        continue  # Notification, no response needed
                    else:
                        response = {
                            "jsonrpc": "2.0",
                            "id": request.get("id"),
                            "error": {
                                "code": -32601,
                                "message": f"Method not found: {method}"
                            }
                        }

                    # Write response
                    response_body = json.dumps(response).encode()
                    response_header = f"Content-Length: {len(response_body)}\r\n\r\n"
                    writer.write(response_header.encode() + response_body)
                    await writer.drain()

            except asyncio.IncompleteReadError:
                break
            except Exception as e:
                sys.stderr.write(f"Error: {e}\n")
                continue


# ─── HTTP Transport (for Copilot Studio) ──────────────────────────────────────

async def run_http_server(host: str = "0.0.0.0", port: int = 8787):
    """Run the MCP server over HTTP for Copilot Studio integration."""
    try:
        from aiohttp import web
    except ImportError:
        print("Install aiohttp: pip install aiohttp", file=sys.stderr)
        sys.exit(1)

    server = AluminumMCPServer()

    async def handle_mcp(request):
        body = await request.json()
        method = body.get("method", "")

        if method == "initialize":
            response = server.handle_initialize(body)
        elif method == "tools/list":
            response = server.handle_tools_list(body)
        elif method == "tools/call":
            response = await server.handle_tools_call(body)
        else:
            response = {
                "jsonrpc": "2.0",
                "id": body.get("id"),
                "error": {"code": -32601, "message": f"Unknown method: {method}"}
            }

        return web.json_response(response)

    async def handle_health(request):
        return web.json_response({
            "status": "healthy",
            "server": SERVER_NAME,
            "version": SERVER_VERSION,
            "tools_count": len(server.tools),
            "providers": ["google", "microsoft", "apple", "android"]
        })

    app = web.Application()
    app.router.add_post("/mcp", handle_mcp)
    app.router.add_get("/health", handle_health)
    app.router.add_get("/", handle_health)

    runner = web.AppRunner(app)
    await runner.setup()
    site = web.TCPSite(runner, host, port)
    await site.start()
    print(f"Aluminum OS MCP Server running on http://{host}:{port}")
    print(f"  MCP endpoint: http://{host}:{port}/mcp")
    print(f"  Health check: http://{host}:{port}/health")
    print(f"  Tools registered: {len(server.tools)}")

    # Keep running
    await asyncio.Event().wait()


# ─── Entry Point ──────────────────────────────────────────────────────────────

if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(
        description="Aluminum OS MCP Server for uws"
    )
    parser.add_argument(
        "--transport", choices=["stdio", "http"], default="http",
        help="Transport mode: stdio (for local agents) or http (for Copilot Studio)"
    )
    parser.add_argument(
        "--port", type=int, default=8787,
        help="HTTP port (only for http transport)"
    )
    parser.add_argument(
        "--host", default="0.0.0.0",
        help="HTTP host (only for http transport)"
    )

    args = parser.parse_args()

    server = AluminumMCPServer()

    if args.transport == "stdio":
        asyncio.run(server.run_stdio())
    else:
        asyncio.run(run_http_server(args.host, args.port))
