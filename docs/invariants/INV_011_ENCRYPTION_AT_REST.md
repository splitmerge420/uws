# INV-011 — Encryption at Rest
**Category:** Security | **Severity:** Critical | **Check Type:** Guard Check

> "All sensitive data must be encrypted at rest using AES-256 or quantum-resistant algorithms."

---

## Statement

Data classified as `confidential` or `restricted` (Class A per INV-4) must be encrypted
at rest before being stored. Unencrypted sensitive data is a constitutional violation and
will be blocked by the runtime engine.

## Encryption Standards

| Classification | Required Algorithm | Key Length |
|---------------|--------------------|------------|
| `restricted` | AES-256-GCM or ML-KEM-768 | 256-bit / 1184-bit |
| `confidential` | AES-256-GCM | 256-bit |
| `internal` | Encryption recommended | — |
| `public` | Not required | — |

## Forbidden Patterns (Linter Detects)

Any write of `confidential` or `restricted` data without encryption guard patterns.

## Guard Patterns (Must Be Present)

`encrypt`, `AES`, `aes_gcm`, `AesGcm`, `encryption_key`, `encrypted`,
`cipher`, `keyring`

## Implementation

| Layer | Implementation |
|-------|----------------|
| `ConstitutionalEngine` | `check_encryption_at_rest()` — blocks sensitive data when `encryption_enabled = false` |
| `credential_store.rs` | AES-256-GCM encryption/decryption for all stored credentials |
| `encryption_enforcement.rego` | `default allow = false`; enforces encryption for Class A writes |
| `pqc_provider.py` | Phase 3 upgrade path: ML-KEM-768 for restricted data |

## AES-256-GCM Implementation Note

```rust
// src/credential_store.rs — AES-256-GCM with random nonce per write
use aes_gcm::{Aes256Gcm, Key, Nonce};
// Key derived from keyring or UWS_CREDENTIAL_KEY env var
// Nonce: 12 random bytes prepended to ciphertext
```

## Test Vectors

| Scenario | Expected Outcome |
|----------|-----------------|
| `data_classification = "confidential"`, `encryption_enabled = false` | `ConstitutionalEngine` → `Err` |
| `data_classification = "confidential"`, `encryption_enabled = true` | `Ok(checks)` |
| Write `.env` file without encryption guard | Linter flags INV-11 violation |
| `credential_store.rs` encrypt/decrypt round-trip | Plaintext == decrypted ciphertext |

## Constitutional Relations

- **Required by:** INV-4 (Data Classification) — classification drives encryption
- **Strengthened by:** INV-13 (PQC Readiness) — upgrade path to ML-KEM
- **Audited by:** INV-3 — encryption decisions are logged

## Status

`IMPLEMENTED` — `ConstitutionalEngine.check_encryption_at_rest()`,
`credential_store.rs` (AES-256-GCM), `encryption_enforcement.rego`.
