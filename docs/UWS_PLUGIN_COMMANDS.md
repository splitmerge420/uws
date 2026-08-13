# uws plugin — Unified Plugin Management

> **Layer**: L5-Extension | **Spheres**: H7.S3, H7.S9 | **Invariants**: INV-1, INV-7, INV-33, INV-35

## Forked Sources

| Key | Fork | Stars | Type |
|-----|------|-------|------|
| mcp | splitmerge420/servers | 81K | MCP Registry |
| cc-market | splitmerge420/cc-marketplace | — | Community Marketplace |

## Commands

```bash
uws plugin list [--source <key>]          # List plugins
uws plugin search <query> [--index <key>] # Search indexes
uws plugin install <name> [--source <key>]# Install (constitutional gate)
uws plugin audit <name>                   # Pre-install audit
uws plugin remove <name>                  # Remove (GoldenTrace)
uws plugin update --all                   # Sync forks + update
uws plugin stats [<name>]                 # Adoption metrics
uws plugin config sources                 # Priority order
uws mcp list                              # MCP servers
uws mcp add <server>                      # Add MCP server
```

## Constitutional Gate

Every install passes through: INV-1 (consent), INV-7 (47% source cap), INV-33 (routing sovereignty), INV-35 (hard fail-closed). GoldenTrace emitted for all operations.

## Layer Mapping

- L1: ConstitutionalGate + GoldenTrace
- L3: CCPI skills → Janus v2, Subagents → Council
- L4: MCP servers + MCP bridge
- L5: Marketplaces + curated indexes + CLI surface

## Related

- `aluminum-os/plugins/PLUGIN_REGISTRY.yaml`
- `aluminum-os/plugins/INTEGRATION_BRIDGE.md`
- `janus/JANUS_V2_SPEC.md`
