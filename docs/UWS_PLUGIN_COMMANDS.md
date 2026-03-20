# uws plugin — Unified Plugin Management

> **Layer**: L5-Extension | **Spheres**: H7.S3, H7.S9 | **Invariants**: INV-1, INV-7, INV-33, INV-35

## Forked Sources

| Key | Fork | Stars | Type |
|-----|------|-------|------|
| official | splitmerge420/claude-plugins-official | 13.5K | Marketplace |
| claude-code | splitmerge420/claude-code | 80K | Reference |
| mcp | splitmerge420/servers | 81K | MCP Registry |
| awesome | splitmerge420/awesome-claude-code | 29K | Curated Index |
| composio | splitmerge420/awesome-claude-plugins | 1.2K | Curated Index |
| ccpi | splitmerge420/claude-code-plugins-plus-skills | 1.7K | 340 plugins + 1367 skills |
| metrics | splitmerge420/awesome-claude-plugins2 | — | Adoption Metrics |
| cc-market | splitmerge420/cc-marketplace | — | Community Marketplace |
| mcp-bridge | splitmerge420/claude-code-mcp | — | MCP Bridge |
| ccplugins | splitmerge420/awesome-claude-code-plugins | — | Curated Index |

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
uws mcp bridge claude-code                # CLI↔MCP bridge
```

## Constitutional Gate

Every install passes through: INV-1 (consent), INV-7 (47% source cap), INV-33 (routing sovereignty), INV-35 (hard fail-closed). GoldenTrace emitted for all operations.

## Layer Mapping

- L1: ConstitutionalGate + GoldenTrace
- L2: PluginLoader (sandbox), claude-code reference
- L3: CCPI skills → Janus v2, Subagents → Council
- L4: MCP servers + MCP bridge
- L5: Marketplaces + curated indexes + CLI surface

## Related

- `aluminum-os/plugins/PLUGIN_REGISTRY.yaml`
- `aluminum-os/plugins/INTEGRATION_BRIDGE.md`
- `janus/JANUS_V2_SPEC.md`
