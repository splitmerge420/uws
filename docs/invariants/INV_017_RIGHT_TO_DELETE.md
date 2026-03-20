# INV-017 — Right to Delete
**Category:** Privacy | **Severity:** Critical | **Check Type:** Advisory

> "Users must be able to delete all their data, including backups, within 72 hours."

---

## Statement

When a user requests deletion of their data, Aluminum OS must:

1. Identify **all** locations where that data is stored (primary, backup, cache, audit)
2. Delete or cryptographically shred the data within **72 hours**
3. Produce an audit record confirming deletion (the record contains no deleted content)
4. Confirm completion to the user

The only exception is audit chain entries required for legal hold — in that case the user
must be notified that a legal hold is active and the data cannot be deleted until the hold
is released.

## Deletion Classes

| Data Type | Deletion Method | SLA |
|-----------|----------------|-----|
| OAuth tokens | Token revocation API | Immediate |
| Cached files | Secure delete (overwrite + unlink) | < 1 hour |
| Credentials in keyring | `keyring.delete_password()` | Immediate |
| GitHub data | GitHub API delete endpoints | < 24 hours |
| Audit chain entries | **Cannot delete** — see Audit Immutability (INV-12) | N/A |

## Audit Chain Exception

Audit chain entries are immutable by design (INV-3, INV-12). When a user exercises
Right to Delete:

- Personal data **within** evidence fields is pseudonymised
- The chain structure and hashes are preserved
- A deletion record is appended noting which fields were pseudonymised

## Implementation

| Layer | Implementation |
|-------|----------------|
| `src/auth_commands.rs` | `uws auth logout` — revokes tokens and deletes local credentials |
| `src/credential_store.rs` | `delete_credentials()` — removes all stored keys |
| `acp_governance.py` | Deletion events require INV-3 audit record |
| `CouncilGitHubClient` | `shred_secret()` — Class A data removal from repo history |

## Constitutional Relations

- **Required by:** INV-19 (Jurisdictional Compliance) — GDPR Article 17, CCPA §1798.105
- **Constrained by:** INV-3 (Audit Trail) — audit chain entries are immutable
- **Enabled by:** INV-4 (Data Classification) — deletion scope defined by classification

## Status

`PARTIAL` — `uws auth logout` and `credential_store.delete_credentials()` implemented.
Full cross-service deletion orchestration is Phase 2.
