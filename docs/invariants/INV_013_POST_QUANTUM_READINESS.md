# INV-013 — Post-Quantum Readiness
**Category:** Security | **Severity:** Mandatory | **Check Type:** Advisory

> "Cryptographic systems must support or plan for NIST PQC (ML-KEM, ML-DSA)."

---

## Statement

All cryptographic subsystems must either already use NIST-standardised post-quantum
algorithms or have a documented, tested upgrade path to them. Systems using classical
crypto (RSA, ECDSA, ECDH) must be wrapped in a layer that can be replaced without
changing the calling interface.

## NIST PQC Standards (FIPS 203/204/205)

| Algorithm | Use Case | FIPS |
|-----------|----------|------|
| ML-KEM-768 (Kyber) | Key encapsulation / encryption | FIPS 203 |
| ML-DSA-65 (Dilithium) | Digital signatures | FIPS 204 |
| SLH-DSA (SPHINCS+) | Hash-based signatures | FIPS 205 |

## Phase Roadmap

| Phase | Cryptography | Status |
|-------|-------------|--------|
| 1 (current) | HMAC-SHA3-256 (structural placeholder) | `pqc_provider.py` |
| 2 | SHA3-256 (real) for audit chain | **Done** — `audit_chain.rs` uses `sha3` crate |
| 3 | ML-DSA-65 for component signing | Upgrade path in `pqc_provider.py` |

## Implementation

| Layer | Implementation |
|-------|----------------|
| `audit_chain.rs` | Real SHA3-256 via `sha3` crate (Phase 2 ✓) |
| `pqc_provider.py` | HMAC-SHA3-256 Phase 1; ML-DSA-65 Phase 3 documented |
| `credential_store.rs` | AES-256-GCM (quantum-resistant symmetric) |
| `zero_trust_registry` | Council approval tokens will be ML-DSA signed in Phase 3 |

## Upgrade Path (Phase 3)

```python
# pqc_provider.py — when dilithium binding available:
from dilithium import Dilithium65
CURRENT_ALGORITHM = "ML-DSA-65"

def sign(self, message: bytes) -> bytes:
    sk = Dilithium65.keygen()[1]
    return Dilithium65.sign(sk, message)
```

The `PQCProvider` interface is intentionally stable so this swap requires no calling-code
changes.

## Constitutional Relations

- **Strengthens:** INV-11 (Encryption at Rest) — quantum-resistant at-rest encryption
- **Strengthens:** INV-12 (Encryption in Transit) — TLS 1.3 supports hybrid PQC ciphers
- **Supports:** INV-5 (Constitutional Authority) — council approval tokens in Phase 3

## Status

`PARTIAL` — SHA3-256 implemented (Phase 2 ✓). ML-DSA-65 upgrade path documented
in `pqc_provider.py`; full implementation is Phase 3.
