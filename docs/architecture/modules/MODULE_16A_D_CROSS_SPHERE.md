# Module 16A-D — Interoperability Target Layer

Status: Phase 1 skeleton / Phase 1.5 synthesis target
Source context: UWS provider architecture, ADR-145-DPI v3.0 provider doctrine, Phase 1 baseline

## Correction Note

Earlier text incorrectly classified Module 16A-D as the cross-sphere accountability module family. That was a taxonomy error.

The intended Module 16A-D family is the **interoperability/provider target layer**: building UWS / Aluminum OS into a sovereign command surface that can interoperate across OpenAI, Anthropic, Google, Microsoft, AWS, Apple, Android/Chrome, GitHub, Notion, and adjacent targets.

Cross-sphere accountability remains important, but should live under governance / invariants / ADR-145 extraction rather than occupying the Module 16A-D slot.

## Purpose

Module 16A-D provides the interoperability spine. Its job is to make providers and agent ecosystems behave like device drivers behind the Aluminum / UWS kernel.

Core doctrine:

- no provider is irreplaceable;
- no subscription is a hard dependency;
- the kernel routes, providers execute;
- OpenAI, Anthropic, Google, Microsoft, AWS, and other targets are integration surfaces, not sovereign owners of the system;
- Phase 1 exposes skeleton homes for each target;
- Phase 1.5 synthesizes and normalizes shared driver traits, manifests, auth, observability, and routing.

## Active Agent Execution Stack

The current build process is multi-agent and multi-surface. Future agents should understand the real execution environment, not just the abstract provider map.

### Microsoft Copilot / Tasks Beta

Microsoft Copilot, including Tasks Beta in Microsoft 365, is an active execution layer for parallel GitHub-oriented builds. It has been essential for generating PRs, repo cleanup, provider dispatch work, and longform architecture artifacts.

Integration implication: Microsoft 365 / Copilot should be treated not only as a productivity target, but also as an agent execution substrate. UWS should eventually expose clear patterns for:

- task creation and tracking;
- PR-scoped execution instructions;
- Copilot-generated artifact review;
- audit/provenance trails for agent-produced code.

### Manus Execution Layer

Manus remains a valuable external execution layer for large build runs, repo generation, and implementation acceleration.

Operational caution: Manus can balloon costs quickly if execution is too trigger-happy. A prior build burst consumed approximately $400 in credits over roughly two days. Future Manus use should include:

- explicit task budgets;
- bounded PR scopes;
- stop conditions;
- checkpoint reviews before spawning large build trees;
- cost telemetry where possible.

### Notion AI Neutral Workspace / MCP UI

Notion AI is used as a neutral workspace and UI layer for MCP-oriented integrations across models. It has produced multiple longform codebases and integration plans for this project and may contain valuable implementation artifacts.

Integration implication: Notion should be treated as both:

1. a productivity / knowledge workspace target; and
2. a neutral interface layer for multi-model MCP workflows.

Future UWS work should preserve Notion as a first-class interoperability target, especially for:

- artifact staging;
- longform architecture synthesis;
- MCP workspace orchestration;
- human-readable execution dashboards;
- cross-model handoff records.

## Module 16A — AI Model Provider Interoperability

Primary targets:

- OpenAI;
- Anthropic;
- Google Gemini / Vertex;
- Microsoft Copilot / Azure AI surfaces;
- xAI / Grok where applicable;
- DeepSeek and other frontier / regional models where policy allows.

Skeleton responsibilities:

- define model-family routing concepts;
- distinguish model provider from hyperscaler infrastructure provider;
- prepare for INV-7c model family caps;
- expose future hooks for prompt / context / tool-call normalization;
- keep provider-specific SDK code out of the core kernel until adapters are explicit.

Phase 1 output: docs + provider target list + optional adapter stubs.
Phase 1.5 output: shared `ModelProviderDriver` / `AgentProviderDriver` trait and normalized request envelope.

## Module 16B — Cloud / Hyperscaler Interoperability

Primary targets:

- Microsoft Azure / Azure AI Foundry;
- Google Cloud / Vertex AI / GKE;
- AWS / Bedrock / AgentCore;
- optional neutral infrastructure targets.

Skeleton responsibilities:

- expose provider caps and combined hyperscaler cap attachment points;
- prepare for cost / carbon / water / trust telemetry;
- separate cloud routing from model-family routing;
- prepare for confidential compute / attestation adapters.

Phase 1 output: cloud target docs + dispatch attachment points.
Phase 1.5 output: cloud driver trait and routing metadata model.

## Module 16C — Productivity / OS Surface Interoperability

Primary targets:

- Microsoft 365;
- Microsoft Copilot Tasks Beta;
- Google Workspace;
- Apple iCloud;
- Android / Chrome;
- GitHub;
- Notion / Notion AI;
- Slack / Linear / Figma / Stripe as future extension slots.

Skeleton responsibilities:

- keep UWS as a schema-driven JSON-first command surface;
- normalize mail, calendar, files, notes, tasks, repo, and identity operations;
- avoid hard vendor lock-in;
- preserve provider-native strengths while exposing a common command layer;
- support neutral workspace handoffs through Notion where useful;
- support PR/task execution handoffs through Copilot and GitHub where useful.

Phase 1 output: visible target map and existing provider dispatch alignment.
Phase 1.5 output: normalized command schema and provider capability manifests.

## Module 16D — Protocol / Agent Interoperability

Primary targets:

- MCP;
- A2A;
- OpenAPI / JSON-RPC;
- local tool protocols;
- GitHub PR / issue comment instruction loops;
- Microsoft Copilot Tasks Beta execution loops;
- Manus execution handoff loops;
- Notion AI workspace / MCP UI loops;
- future Janus / Pantheon council routing surfaces.

Skeleton responsibilities:

- define how external agents discover capabilities;
- expose future `AgentCard` / manifest equivalents;
- prepare for tool permissioning, consent, provenance, and audit;
- avoid a single-agent monoculture;
- record execution provenance across Copilot, Manus, Claude, GPT, Grok, Gemini, and Notion AI where available;
- bound automated execution with cost, scope, and stop-condition metadata.

Phase 1 output: protocol target list + docs.
Phase 1.5 output: normalized agent protocol bridge and routing policy.

## Integration Points

- `src/main.rs` dispatch layer;
- `src/github_provider.rs` and existing provider modules;
- future `src/providers/` target skeletons;
- future `src/protocols/` target skeletons;
- `src/governance/attachment_points.rs` for consent / preflight / impact hooks;
- `docs/architecture/PHASE_1_BASELINE.md`.

## Relationship to ADR-145

ADR-145-DPI v3.0 supplies the constitutional doctrine for provider caps, model-family caps, hyperscaler abstraction, local-first preference, cost/resource transparency, trust attestation, and cross-sphere accounting.

Module 16A-D is the implementation-facing interoperability skeleton that gives those doctrines a place to attach.

## Acceptance Criteria

- [ ] Module 16A-D is understood as interoperability, not cross-sphere governance.
- [ ] OpenAI is explicitly included as a first-class model-provider target.
- [ ] Anthropic is modeled as frontier model provider, not hyperscaler.
- [ ] Microsoft Copilot / Tasks Beta is recognized as an agent execution substrate as well as a productivity surface.
- [ ] Manus is recognized as an execution layer with explicit cost / scope guardrails.
- [ ] Notion AI is recognized as a neutral workspace and MCP-oriented UI target.
- [ ] Hyperscalers are modeled separately from model families.
- [ ] Productivity / OS surfaces remain part of UWS core scope.
- [ ] Protocol interoperability includes MCP and A2A attachment points.
- [ ] Agent execution loops include provenance, budget, stop-condition, and handoff metadata.
- [ ] No provider SDK or live auth is introduced without a focused adapter PR.
