# INV-012 — Encryption in Transit
**Category:** Security | **Severity:** Critical | **Check Type:** Pattern Absence

> "All network communication must use TLS 1.3+ or equivalent."

---

## Statement

Every network connection made by Aluminum OS must use TLS 1.3 or higher. Plain HTTP to
non-localhost addresses is prohibited. This applies to all API calls, webhook deliveries,
health-check pings, and inter-service communication.

## Forbidden Patterns (Linter Detects)

```python
# Plain HTTP to non-local addresses
http://api.example.com/...
http://github.com/...
```

The linter pattern `http://(?!localhost|127\.0\.0\.1|0\.0\.0\.0)` detects non-local
plain HTTP and flags it as an INV-12 violation. Localhost plain HTTP is allowed for
local development only.

## Allowed Exceptions

| Exception | Condition |
|-----------|-----------|
| `http://localhost` | Local development only; must not reach production |
| `http://127.0.0.1` | Same as above |
| `http://0.0.0.0` | Bind address for local servers only |

## Implementation

| Layer | Implementation |
|-------|----------------|
| `src/client.rs` | All outbound HTTP uses TLS via `reqwest` (Phase 3) |
| `src/executor.rs` | All API endpoints are HTTPS |
| Linter | `invariant_linter.py` flags `http://` non-local URLs |
| `src/discovery.rs` | Discovery documents fetched from `https://` only |

## TLS Configuration Requirements

- Minimum version: TLS 1.3
- Certificate validation: always enabled (no `danger_accept_invalid_certs`)
- HSTS: enforced for all public-facing endpoints

## Test Vectors

| URL | Expected |
|-----|----------|
| `https://api.github.com/...` | ALLOWED |
| `http://localhost:8080/...` | ALLOWED (local exception) |
| `http://api.openai.com/...` | BLOCKED — linter INV-12 flag |

## Constitutional Relations

- **Paired with:** INV-11 (Encryption at Rest) — data encrypted both ways
- **Required by:** INV-19 (Jurisdictional Compliance) — GDPR/HIPAA require TLS
- **Strengthened by:** INV-13 (PQC Readiness) — TLS 1.3 is PQC-ready with hybrid ciphers

## Status

`IMPLEMENTED` — linter pattern in `invariant_linter.py`; all executor endpoints use HTTPS.
Phase 3 will enforce TLS version via `reqwest` configuration.
