# INV-014 — Zero-Knowledge Where Possible
**Category:** Privacy | **Severity:** Advisory | **Check Type:** Advisory

> "Prefer zero-knowledge proofs for identity verification and data sharing."

---

## Statement

Where technically feasible, Aluminum OS should verify identity and authorisation
attributes without transmitting the underlying data. A system that can verify "user is
over 18" without receiving the date of birth, or "document is authentic" without
receiving the document content, is preferred over one that requires full disclosure.

## Zero-Knowledge Use Cases

| Use Case | ZK Approach | Alternative (Fallback) |
|----------|-------------|----------------------|
| Age verification | ZK age proof (e.g., zk-SNARKs) | OAuth + age claim |
| Credential ownership | DID + Verifiable Credential | Username + password |
| Health record access | ZK health attestation | FHIR access token |
| Council vote anonymisation | ZK vote proof | Named ballot |
| Audit chain verification | Hash chain verification (already ZK-like) | Full log exposure |

## Relationship to Existing Implementation

The `audit_chain.rs` SHA3-256 hash chain is already zero-knowledge with respect to
entry content: a verifier can confirm the chain is intact without reading the evidence
fields. This is the simplest form of ZK already implemented.

## Phase 3 Integration Points

- `pqc_provider.py` — Phase 3 will optionally wrap ML-DSA with a ZK proof of key possession
- `council_github_client.rs` — Dave approval tokens could be replaced with ZK proofs
  of constitutional authority without revealing the token itself

## Constitutional Relations

- **Strengthens:** INV-11 (Encryption at Rest) — ZK reduces exposure surface
- **Strengthens:** INV-16 (Data Minimisation) — ZK is the strongest form of minimisation
- **Strengthens:** INV-20 (No Silent Sharing) — ZK means nothing is shared at all

## Status

`ADVISORY` — architectural principle. Hash chain verification implements the simplest
ZK property. Full ZK proof integration is Phase 3.
