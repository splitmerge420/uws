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

## Module 16A — AI Model Provider Interoperability

Primary targets:

- OpenAI;
- Anthropic;
- Google Gemini / Vertex;
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
- Google Workspace;
- Apple iCloud;
- Android / Chrome;
- GitHub;
- Notion;
- Slack / Linear / Figma / Stripe as future extension slots.

Skeleton responsibilities:

- keep UWS as a schema-driven JSON-first command surface;
- normalize mail, calendar, files, notes, tasks, repo, and identity operations;
- avoid hard vendor lock-in;
- preserve provider-native strengths while exposing a common command layer.

Phase 1 output: visible target map and existing provider dispatch alignment.
Phase 1.5 output: normalized command schema and provider capability manifests.

## Module 16D — Protocol / Agent Interoperability

Primary targets:

- MCP;
- A2A;
- OpenAPI / JSON-RPC;
- local tool protocols;
- future Janus / Pantheon council routing surfaces.

Skeleton responsibilities:

- define how external agents discover capabilities;
- expose future `AgentCard` / manifest equivalents;
- prepare for tool permissioning, consent, provenance, and audit;
- avoid a single-agent monoculture.

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
- [ ] Hyperscalers are modeled separately from model families.
- [ ] Productivity / OS surfaces remain part of UWS core scope.
- [ ] Protocol interoperability includes MCP and A2A attachment points.
- [ ] No provider SDK or live auth is introduced without a focused adapter PR.
