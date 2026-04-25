# Module 18 — FinancialContextKernel (FCK)

Status: Phase 1 skeleton / Phase 1.5 synthesis target
Source context: ADR-145-DPI v3.0 §7, INV-12, INV-7d, INV-17

## Purpose

The FinancialContextKernel provides a privacy-preserving financial awareness layer that enables:

- subscription discovery (INV-7d);
- financial transparency (INV-12);
- user savings detection;
- cross-sphere integration with healthcare, civic incentives, and infrastructure funding;
- future integration with Digital Dividend (INV-17).

## Design Principles

- consent-first (no data access without explicit user consent);
- local-first analysis where possible (INV-8 alignment);
- detect patterns, not diagnose conditions;
- no coercion: features are additive, not required;
- no hidden monetization: all value surfaces are transparent.

## Phase 1 Skeleton Scope

- module documentation;
- Rust module placeholder (`src/financial_context/`);
- interface definitions for future adapters (e.g., Plaid);
- no external API dependencies;
- no financial data ingestion;
- no payout or billing logic.

## Phase 1.5 Targets

- multi-institution aggregation (opt-in);
- savings detection engine;
- healthcare and infrastructure cross-sphere hooks;
- integration with cost transparency and resource transparency;
- optional integration with Digital Dividend ledger.

## Explicit Non-Goals (Phase 1)

- no Plaid or external financial API integration;
- no medical diagnosis or inference;
- no automated financial decisions;
- no live data marketplace or payouts.

## Acceptance Criteria

- [ ] Module exists in docs.
- [ ] Rust skeleton exists.
- [ ] References to INV-12, INV-7d, and INV-17 are explicit.
- [ ] No runtime side effects introduced into execution spine.
