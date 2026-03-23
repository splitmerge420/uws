// src/lib.rs
// Aluminum OS — Universal Workspace Library
//
// Module declarations for the UWS constitutional governance layer.
// The binary entry point is src/main.rs (the CLI).
// This file exposes library modules for testing and external consumption.
//
// Council Session: 2026-03-20
// Authority: Dave Sheldon (INV-5)

// ─── Constitutional Governance Layer ─────────────────────────────
// These modules enforce the 39 Constitutional Invariants at runtime.

/// Constitutional enforcement engine — runtime invariant checking.
/// Checks: INV-1 (Sovereignty), INV-2 (Consent), INV-3 (Audit),
///         INV-6 (Provider Abstraction), INV-7 (Vendor Balance),
///         INV-11 (Encryption at Rest)
pub mod constitutional_engine;

/// Council GitHub operations client — constitutional wrapper around GitHub API.
/// Blocks destructive operations. Enforces data classification (Class A/B/C).
/// Appends provenance trailers to all commits.
/// Enforces: INV-1, INV-2, INV-3, INV-5, INV-6, INV-7, INV-11, INV-35
pub mod council_github_client;

/// Append-only SHA3-256 hash-chained audit log.
/// NO modify/delete API exists on this struct -- by design.
/// verify_chain() walks every link to detect tampering.
/// Enforces: INV-3 (Audit Trail), INV-35 (Fail-Closed)
pub mod audit_chain;

// ─── Universal I/O Layer ──────────────────────────────────────────

/// Provider-agnostic document abstraction -- "unshackling" layer.
///
/// Converts proprietary SaaS documents (Google Docs, Microsoft Word, etc.)
/// into locally-owned `UniversalDocument` objects (Markdown body + JSON
/// frontmatter).  Google Workspace and Microsoft 365 become "dumb pipes";
/// the user's local machine is the sovereign source of truth.
///
/// Exposes: `UniversalDocument`, `SaaSConnector` trait,
///          `GoogleWorkspaceConnector`, `MicrosoftWordConnector`,
///          `AppleNoteConnector`, `PlainTextConnector` stubs.
/// Enforces: INV-1 (Sovereignty), INV-6 (Provider Abstraction)
pub mod universal_io;

// ─── Phase 1: Sovereign Memory ────────────────────────────────────

/// LocalNoosphere -- Sovereign personal knowledge graph (Phase 1, Module 1).
///
/// A fully local, in-memory graph database that stores every piece of
/// information the user cares about as typed `GraphNode` objects linked
/// by semantic edges.  Nodes flow in from `universal_io` connectors and
/// back out via the same connectors.  Every state change is recorded as
/// a `TemporalDelta` for TemporalAnchor integration.
///
/// Exposes: `LocalNoosphere`, `GraphNode`, `NodeKind`, `TemporalDelta`.
/// Enforces: INV-1 (Sovereignty), INV-3 (Audit Trail), INV-6 (Provider Abstraction)
pub mod local_noosphere;

// ─── Aluminum Fusion Engine ───────────────────────────────────────

/// The integration layer that unifies three productivity stacks into one OS.
///
/// Implements the 10 Google Engineer Wishes:
/// `AluminumKernel`, `MemorySubstrate` (Blackboard pattern, cross-provider
/// context graph), `IdentitySubstrate` (one user, all clouds),
/// `GovernanceLayer` (pre/post-flight checks), `ProviderRegistry`,
/// `SyncEngine`, `AgentRuntime`, `NaturalLanguageShell`.
///
/// Enforces: INV-1 (Sovereignty), INV-6 (Provider Abstraction),
///           INV-7 (Vendor Balance)
pub mod fusion_engine;

// ─── Agentic Sovereignty Layer ────────────────────────────────────

/// Implements all 10 Google Agentic Sovereignty wishes.
///
/// `CryptographicSigning` (Ed25519 artifact provenance),
/// `AgenticPause` (safe pause/resume for running agents),
/// `HotSwapReasoning` (swap reasoning models mid-session),
/// `UniversalUndo` (semantic undo across providers),
/// `SkillsMarketplace`, `EdgeFirstRAG`, `ConflictResolution`,
/// `ProviderMigration`, `SemanticFileLocking`, `ZeroKnowledgeIdentity`.
///
/// Enforces: INV-1 (Sovereignty), INV-2 (Consent), INV-11 (Encryption)
pub mod agentic_sovereignty;

// ─── Universal Context Layer ──────────────────────────────────────

/// Cross-provider context layer: "the missing pieces of one OS".
///
/// `UniversalSearch` (Gmail + OneDrive + iCloud simultaneously),
/// `UniversalInbox` (all messages in one stream),
/// `UniversalNotifications`, `UniversalClipboard`,
/// `UniversalFileGraph` (one namespace, all clouds),
/// `SchedulingIntelligence`, `GraphUnificationLayer`,
/// `PluginSubstrate`, `InfrastructureCopilot`.
///
/// Enforces: INV-6 (Provider Abstraction), INV-1 (Sovereignty)
pub mod universal_context;

// ─── Council AI Modules ───────────────────────────────────────────

/// Claude Miracles Layer -- 15 features that make the system undeniable.
///
/// `UwsClaude` (claude-code integration), `UwsCouncil` (multi-agent
/// orchestration), `UwsRAG` (Sheldonbrain service), `UwsSync`,
/// `UwsVault`, `UwsJanus` (session state preservation),
/// `UwsPluginEconomy`, `UwsHealth` (circadian protocol),
/// `UwsDiplomatic`, `UwsAudit`.
///
/// Enforces: INV-3 (Audit Trail), INV-8 (Human Override)
pub mod claude_miracles;

/// GPT Pantheon Layer -- 7 genuinely new capabilities from GPT's wish list.
///
/// `ResearchEngine` (deep multi-source research from CLI),
/// `PersonalAdvocate` (AI advocate that knows your rights and interests),
/// `SituationalAwareness` (real-time context monitoring),
/// `WorkflowLearner` (self-improving workflow engine),
/// `EconomicEngine` (personal economic system),
/// `GlobalSignalMonitor`, `PantheonConvene` ("Dave Sheldon Feature").
pub mod gpt_pantheon;

/// Grok Bazinga Layer -- 8 genuinely new capabilities from Grok/Ara.
///
/// `VoiceEngine` (voice as primary interface, wake-word, multilingual),
/// `MultiModalEngine` (image/audio/video I/O),
/// `TruthEngine` (hallucination detection + source verification),
/// `SpatialComputeEngine` (AR/VR native support),
/// `TokenOptimizer` (cost + token optimisation),
/// `CommunityGovernance`, `OfflineEngine` (offline-first with sync),
/// `CosmicAmbitionMode` (galaxy-brain problem decomposition).
pub mod grok_bazinga;

// ─── Janus v2 -- Universal Multi-Agent Router ─────────────────────

/// Janus v2 -- Constitutional multi-agent routing protocol.
///
/// Sub-modules:
/// - `janus`: `JanusRouter`, `RoutingTier`, `ModelVote`, `GoldenTrace`,
///            `HeartbeatTrace`, `KintsugiRepair`, `JanusResult`.
/// - `model_router`: `ModelRouter`, `ModelConfig`,
///                   `compute_digest`, `compute_digest_from_str`.
///
/// Enforces: INV-7 (Vendor Balance <= 47%), INV-8 (Human Override),
///           INV-3 (Audit Trail via GoldenTrace)
pub mod universal;
