# INV-004 — Data Classification
**Category:** Data | **Severity:** Mandatory | **Check Type:** Advisory

> "All data must be classified by sensitivity before processing or storage."

---

## Statement

Before any data is stored, processed, or transmitted it must carry a sensitivity
classification. Unclassified data is treated as the highest-risk class until explicitly
reclassified. The classification determines encryption requirements, retention limits,
sharing permissions, and incident-response procedures.

## Classification Schema

| Class | Label | Examples | On Violation |
|-------|-------|----------|--------------|
| A | `restricted` / `confidential` | Credentials, health records, PII, seeds, FHIR | SHRED |
| B | `internal` | Source code, configs, YAML, TOML, Rego | HOLD-AND-NOTIFY |
| C | `public` | Docs, comments, READMEs | ENCRYPTED-CACHE |

## Implementation

| Layer | Implementation |
|-------|----------------|
| `DataClass::from_path()` | Rust — classifies by file extension/name patterns |
| `ConstitutionalEngine` | `check_encryption_at_rest()` — enforces Class A encryption |
| `data_classification.rego` | OPA policy — `default allow = false`; blocks unclassified access |
| `CouncilGitHubClient` | Blocks Class A commits without constitutional authority approval |

## Path-Based Classification Rules (Rust)

```rust
// Class A (SHRED on violation)
.env | secret | credential | key.pem | wallet | seed | fhir | health | hipaa

// Class B (HOLD-AND-NOTIFY)
.rs | .py | .toml | .yaml | .yml | .json | .rego

// Class C (ENCRYPTED-CACHE)
everything else
```

## Test Vectors

| Path | Expected Class |
|------|----------------|
| `.env.production` | A |
| `src/main.rs` | B |
| `README.md` | C |
| `patient_record.fhir` | A |

## Constitutional Relations

- **Required by:** INV-11 (Encryption at Rest) — classification drives encryption
- **Required by:** INV-17 (Right to Delete) — Class A data has 72h deletion SLA
- **Strengthened by:** INV-3 (Audit Trail) — classification decisions are recorded

## Status

`IMPLEMENTED` — `DataClass` in `council_github_client.rs`;
`data_classification.rego` with default-deny posture.
