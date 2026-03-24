---
"@splitmerge420/uws": minor
---

**Janus v2 implementation + 6 additional Rust modules wired (Batch 3)**

### Janus v2 — Constitutional Multi-Agent Router (`src/universal/`)

New sub-crate `universal` implementing the Janus v2 protocol from `janus/JANUS_V2_SPEC.md`:

**`src/universal/janus.rs`** — `JanusRouter` implementing the full Janus v2 spec:
- Tier 1 / Tier 2 / Tier 3 routing with correct latency targets (500 ms / 3 s / 30 s)
- INV-7 enforcement: no single model may hold > 47% of weighted votes
- GoldenTrace events emitted at every decision point (action, council_vote, council_consensus, human_override, kintsugi_repair, heartbeat)
- Kintsugi failure recovery: marks failed models unavailable, applies 20% reliability decay, selects fallback model, computes beauty score
- Heartbeat traces with full model availability snapshot
- 30 new unit tests covering all routing tiers, INV-7, Kintsugi, heartbeat, Ghost Seat flag

**`src/universal/model_router.rs`** — `ModelRouter` + digest helpers:
- FNV-1a 64-bit `compute_digest` / `compute_digest_from_str` (zero-dep, deterministic)
- `ModelConfig` with effective weight = base_weight × reliability
- `ModelRouter::default_council()` — five-member council (Claude, Gemini, Grok, DeepSeek, Copilot)
- INV-7 compliant `select_primary()`, `mark_unavailable()`, `update_reliability()`, `fallback_for()`
- 14 new unit tests

**`janus/janus_config.yaml`** — Full configuration file per Janus v2 spec:
- Per-model roles, weights, fallbacks, API env vars, endpoints, model IDs
- Kintsugi decay parameters, Ghost Seat configuration
- Tier latency targets, GoldenTrace log settings

**`toolchain/janus_runner.py`** — Python orchestration runner:
- Reads `janus/janus_config.yaml` (with hardcoded fallback if PyYAML absent)
- Implements Tier 1 / Tier 2 / Tier 3 routing with same INV-7 logic as Rust
- Kintsugi repair, heartbeat, stub API call hooks for live integration
- CLI: `python3 toolchain/janus_runner.py --query "..." --tier 2`
- `--heartbeat` mode for health monitoring

### 6 Modules Wired into `lib.rs`

Previously existing but unregistered modules now fully integrated:

| Module | Capabilities |
|---|---|
| `fusion_engine` | `AluminumKernel`, `MemorySubstrate` (Blackboard, cross-provider context graph), `IdentitySubstrate`, `GovernanceLayer`, `ProviderRegistry`, `SyncEngine`, `AgentRuntime`, `NaturalLanguageShell` |
| `agentic_sovereignty` | `CryptographicSigning` (Ed25519), `AgenticPause`, `HotSwapReasoning`, `UniversalUndo`, `SkillsMarketplace`, `EdgeFirstRAG`, `ConflictResolution`, `ProviderMigration`, `SemanticFileLocking`, `ZeroKnowledgeIdentity` |
| `universal_context` | `UniversalSearch`, `UniversalInbox`, `UniversalNotifications`, `UniversalClipboard`, `UniversalFileGraph`, `SchedulingIntelligence`, `GraphUnificationLayer`, `PluginSubstrate`, `InfrastructureCopilot` |
| `claude_miracles` | `UwsClaude`, `UwsCouncil`, `UwsRAG`, `UwsSync`, `UwsVault`, `UwsJanus`, `UwsPluginEconomy`, `UwsHealth`, `UwsDiplomatic`, `UwsAudit` |
| `gpt_pantheon` | `ResearchEngine`, `PersonalAdvocate`, `SituationalAwareness`, `WorkflowLearner`, `EconomicEngine`, `GlobalSignalMonitor`, `PantheonConvene` |
| `grok_bazinga` | `VoiceEngine`, `MultiModalEngine`, `TruthEngine`, `SpatialComputeEngine`, `TokenOptimizer`, `CommunityGovernance`, `OfflineEngine`, `CosmicAmbitionMode` |

### Bug Fixes (in files being wired in)
- `fusion_engine.rs`: replaced `tokio::sync::RwLock` + `.await` with `std::sync::RwLock` (no tokio dep needed); all 4 `MemorySubstrate` methods converted from `async fn` to sync
- `grok_bazinga.rs`: added `Clone` to `VerifiedClaim`, `TruthVerdict`, `Evidence` structs (required by `verify()` method)
- `fusion_engine.rs`: fixed `is_some()` → `contains_key()` and `map_or(true, …)` → `is_none_or(…)` (clippy::unnecessary_map_or)

### Cargo.toml
- Added `serde_json = { version = "1", optional = true }` gated behind `std` feature (required by `fusion_engine` and `universal_context`)

### Test Count
- Before: **87 tests**
- After: **127 tests** (+40)
- All 127 pass, zero clippy errors (`cargo clippy --lib -- -D warnings`)
