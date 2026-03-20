# uws Ecosystem CLI Map v1.0

> **Policy**: NO-DELETE / APPEND-ONLY  
> **Spheres**: H7.S3, H7.S9  
> **Date**: 2026-03-20  

## Forked Ecosystem → uws Command Surface

This document maps every integrable pattern from the 10 forked Claude plugin repos to uws CLI commands. See `aluminum-os/plugins/ECOSYSTEM_MERGE.md` for the full layer-mapped inventory and `aluminum-os/plugins/CONTRADICTION_RESOLUTION.md` for the 14 identified contradictions.

### Plugin Management (`uws plugin`)

```
uws plugin list [--source <key>]           # List from: official|ccpi|awesome|composio|ccplugins|cc-market
uws plugin search <query> [--index <key>]  # Search across all 6 curated indexes
uws plugin install <name> [--source <key>] # Constitutional gate: INV-1 + INV-7 + INV-33 + INV-35
uws plugin remove <name>                   # Append-only audit: GoldenTrace emitted, not actually purged
uws plugin update [--all]                  # Sync all forks with upstream + update installed
uws plugin audit <name>                    # Pre-install constitutional analysis
uws plugin stats [<name>]                  # Adoption metrics from quemsah tracker (8,336 repos indexed)
uws plugin config sources                  # Edit priority: official > ccpi > awesome > composio > ccplugins > cc-market
```

### MCP Server Management (`uws mcp`)

```
uws mcp list                               # All servers from splitmerge420/servers (81K★)
uws mcp add <server>                       # Install: github|git|postgres|puppeteer|slack|gdrive|filesystem|memory|fetch|sqlite
uws mcp add golden-trace-mcp               # Constitutional: Kintsugi audit via MCP (L1)
uws mcp add consent-kernel-mcp             # Constitutional: ConsentKernel via MCP (L1)
uws mcp add council-mcp                    # Constitutional: Pantheon Council routing (L3)
uws mcp add sheldonbrain-mcp               # Constitutional: 144-sphere RAG (L3)
uws mcp bridge claude-code                 # Activate claude-code-mcp CLI↔MCP bridge
uws mcp status                             # Health check all connected servers
```

### Orchestration (`uws janus`)

Maps 10 community orchestrators to Janus v2 modes:

```
uws janus start --mode autonomous          # ← Auto-Claude pattern (Tier 3-4 autonomy)
uws janus start --topology parallel         # ← Claude Squad / Happy Coder pattern
uws janus start --topology swarm            # ← Claude Swarm pattern
uws janus start --mode code_first           # ← claude-code-flow pattern
uws janus start --mode lightweight          # ← sudocode pattern
uws janus start --mode ralph                # ← Ralph technique (canonical: ClaytonFarr playbook)
uws janus start --mode production           # ← The Agentic Startup pattern
uws janus start --runtime native            # ← TSK Rust-based delegation
uws janus decompose <task>                  # ← Claude Task Master hierarchical decomposition
uws janus heartbeat                         # 60s health check across all active agents
uws janus council status                    # Pantheon Council seat status + INV-7 compliance
```

### Security (`uws sec`)

```
uws sec audit <path>                       # Trail of Bits security skills → INV-30
uws sec injection-scan                     # parry prompt injection scanner (L5) + INV-35 (L1)
uws sec consent check <command>            # ConsentKernel + Dippy risk classification
uws sec pentest <target>                   # CCPI penetration-tester skill
uws sec tdd-guard enable                   # TDD Guard file operation monitoring
```

### Session & Memory (`uws session`)

```
uws session recall <query>                 # recall full-text search (L5 fallback)
uws session brain <query>                  # Sheldonbrain RAG query (L3 primary)
uws session restore [--format checkpoint]  # claude-code-tools / claudekit restore
uws session history                        # cchistory session browser
uws session analyze                        # Vibe-Log session analysis
```

### Configuration (`uws config`)

```
uws config project                         # CLAUDE.md project-level (L5)
uws config global                          # SuperClaude global config (L2)
uws config profile switch <name>           # ClaudeCTX profile switching (L5)
uws config doctor                          # claude-rules-doctor dead rules check
```

### Monitoring (`uws monitor`)

```
uws monitor usage                          # CC Usage / ccflare / better-ccflare aggregation
uws monitor tokens                         # Real-time token tracking
uws monitor dashboard                      # ccflare web UI launch
uws monitor leaderboard                    # viberank community stats
```

### DevOps (`uws devops`)

```
uws devops ansible <playbook>              # CCPI ansible-playbook-creator
uws devops iac generate                    # cc-devops-skills IaC generation
uws devops container launch                # Container Use (Dagger) / run-claude-docker
uws devops ci run                          # /run-ci slash command equivalent
uws devops release                         # /release slash command equivalent
```

### Documentation (`uws docs`)

```
uws docs generate                          # /create-docs slash command
uws docs update                            # /update-docs slash command
uws docs changelog add                     # /add-to-changelog
uws docs mermaid <schema>                  # /mermaid diagram generation
```

### Git Workflow (`uws git`)

```
uws git commit                             # /commit conventional format
uws git commit --fast                      # /commit-fast auto-select
uws git pr create                          # /create-pr / /create-pull-request
uws git pr fix                             # /fix-pr unresolved comments
uws git issue fix <id>                     # /fix-github-issue / /fix-issue
uws git issue analyze <id>                 # /analyze-issue spec creation
uws git worktree create                    # /create-worktrees for open PRs
uws git branch update                      # /update-branch-name with prefixes
```

## Slash Command Mapping

All 50+ community slash commands from awesome-claude-code are preserved and accessible via two paths:

1. **Native**: `/command-name` within Claude Code sessions (unchanged)
2. **uws**: `uws <category> <action>` in any terminal (extends reach beyond Claude Code)

Neither path is deleted. The dual-access is the golden seam per no-delete policy.

## Cross-References

- `aluminum-os/plugins/PLUGIN_REGISTRY.yaml` — Source registry
- `aluminum-os/plugins/INTEGRATION_BRIDGE.md` — Constitutional gate architecture  
- `aluminum-os/plugins/ECOSYSTEM_MERGE.md` — Full layer-mapped inventory
- `aluminum-os/plugins/CONTRADICTION_RESOLUTION.md` — 14 contradictions + kintsugi repairs
- `janus/JANUS_V2_SPEC.md` — Multi-agent protocol spec
- `docs/MASTER_REFERENCE.md` — uws CLI master reference
