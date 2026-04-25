# Integration Manifests

Every external integration registered in `uws` — a provider, MCP server, API client,
or any other component that makes outbound calls or handles user data — **must** declare
a manifest in this directory before it can be merged.

The Zero Trust gate (`toolchain/zero_trust_gate.py`) runs in CI on every pull request
and rejects any PR that:

1. Adds a new provider/integration file **without** a corresponding manifest, or
2. Provides a manifest that is missing any of the three required fields.

---

## Manifest Format

```yaml
# integrations/<name>.yaml
name: <integration-name>
description: <one-line description>
version: "1.0"

# (a) Scopes / permissions — list the minimum required OAuth scopes or API permissions.
# No over-broad scopes (e.g., prefer drive.readonly over drive when writes are not needed).
scopes:
  - "https://www.googleapis.com/auth/example.readonly"

# (b) Data-handling class (INV-004):
#   CLASS_A — highest sensitivity (health, financial, legal, biometric)
#   CLASS_B — standard business data (email, calendar, documents)
#   CLASS_C — public / non-sensitive
data_handling_class: CLASS_B

# (c) Constitutional invariants this integration depends on.
# List the INV-NNN IDs of every invariant that governs this integration.
constitutional_invariants:
  - INV-001   # User Sovereignty
  - INV-002   # Consent Gating
  - INV-003   # Audit Trail

# Optional: fallback providers for INV-007 (Vendor Balance).
# Required when data_handling_class is CLASS_A or the integration is Critical-severity.
fallback_providers: []

# Optional: third-party data sharing declaration (INV-020).
# Set to false if no data is shared with third parties.
third_party_sharing: false

# Optional: applicable jurisdictions for INV-019 (Jurisdictional Compliance).
jurisdictions:
  - GDPR
  - CCPA
```

---

## Adding a New Integration

1. Create `integrations/<your-integration>.yaml` with all required fields.
2. Run `python toolchain/zero_trust_gate.py integrations/<your-integration>.yaml` locally.
3. Fix any reported errors before opening your PR.
4. CI will re-run the gate automatically.

---

## Gate Error Codes

| Code | Meaning |
|---|---|
| `MISSING_SCOPES` | `scopes` field is absent or empty |
| `MISSING_DATA_CLASS` | `data_handling_class` field is absent or invalid |
| `MISSING_INVARIANTS` | `constitutional_invariants` field is absent or empty |
| `INVALID_DATA_CLASS` | Value is not one of `CLASS_A`, `CLASS_B`, `CLASS_C` |
| `INVALID_INVARIANT_ID` | An invariant ID is not in the canonical registry |
| `MISSING_MANIFEST` | A new integration source file has no corresponding manifest |
