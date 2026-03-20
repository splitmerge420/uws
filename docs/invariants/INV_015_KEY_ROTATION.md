# INV-015 — Key Rotation
**Category:** Security | **Severity:** Mandatory | **Check Type:** Advisory

> "All cryptographic keys must have rotation schedules and automated rotation support."

---

## Statement

No cryptographic key may be used indefinitely. Every key in Aluminum OS must have:

1. A defined maximum lifetime
2. An automated or semi-automated rotation mechanism
3. An audit record of each rotation event (INV-3)
4. A grace period during which both old and new keys are valid

## Key Rotation Schedule

| Key Type | Max Lifetime | Rotation Trigger |
|----------|-------------|-----------------|
| OAuth access tokens | 1 hour | Automatic (refresh token) |
| OAuth refresh tokens | 30 days | User re-authentication |
| Service account keys | 90 days | Automated via `uws auth rotate` |
| AES-256-GCM data keys | 1 year | Scheduled job |
| Council approval tokens | Session-scoped | Each new session |
| PQC signing keys (Phase 3) | 1 year | Automated |

## Implementation

| Layer | Implementation |
|-------|----------------|
| `src/auth.rs` | Access token refresh on expiry |
| `src/token_storage.rs` | Token storage with TTL tracking |
| `src/credential_store.rs` | AES-256-GCM keys with version tracking |
| `src/auth_commands.rs` | `uws auth rotate` command (Phase 2) |
| `uws ms-auth` | Microsoft token refresh pipeline |

## Rotation Audit Record

Each rotation event must produce an audit entry with:

```json
{
  "action": "key_rotation",
  "resource": "<key_id>",
  "decision": "ALLOW",
  "invariants_checked": ["INV-3", "INV-15"],
  "evidence": "key_type=oauth_refresh old_expiry=... new_expiry=..."
}
```

## Constitutional Relations

- **Depends on:** INV-11 (Encryption at Rest) — rotated keys must be stored encrypted
- **Depends on:** INV-3 (Audit Trail) — rotation events must be logged
- **Strengthens:** INV-13 (PQC Readiness) — rotation policy applies to PQC keys too

## Status

`PARTIAL` — OAuth token refresh implemented in `auth.rs`. Full key rotation CLI command
and AES-256-GCM key rotation are Phase 2 items.
