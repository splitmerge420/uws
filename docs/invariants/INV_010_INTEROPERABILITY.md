# INV-010 — Interoperability
**Category:** Data | **Severity:** Mandatory | **Check Type:** Advisory

> "All data formats must be standards-compliant (JSON-LD, FHIR, W3C PROV, etc.)."

---

## Statement

Aluminum OS stores, exports, and exchanges data using open standards. Proprietary binary
formats are prohibited for user data. Every data model that crosses a system boundary
must be expressible in at least one of the canonical formats below.

## Canonical Formats

| Domain | Format | Standard Body |
|--------|--------|--------------|
| General data | JSON / JSON-LD | IETF / W3C |
| Health records | FHIR R4+ | HL7 |
| Provenance | W3C PROV-JSON | W3C |
| Audit log | Aluminum Audit JSON | (this spec) |
| API discovery | Google Discovery v1 | Google |
| Policy | OPA Rego | CNCF |
| Secrets | SOPS + age | independent |
| Calendar/Contacts | iCalendar / vCard | IETF |

## Audit Log Export Format

```json
{
  "algorithm": "SHA3-256",
  "chain_length": N,
  "entries": [
    {
      "index": 0,
      "timestamp": "1742499600Z",
      "actor": "copilot",
      "action": "zero_trust_gate:logic:audit_chain",
      "resource": "audit_chain",
      "decision": "ALLOW",
      "invariants_checked": ["INV-1", "INV-2", "INV-3"],
      "evidence": "...",
      "entry_hash": "...",
      "previous_hash": "0000..."
    }
  ]
}
```

## Implementation

| Layer | Implementation |
|-------|----------------|
| `AuditChain.export_json()` | Exports chain as Aluminum Audit JSON |
| `src/executor.rs` | All responses are JSON; no binary blobs returned to callers |
| `src/formatter.rs` | JSON / table / YAML / CSV output modes |
| `src/discovery.rs` | Parses Google Discovery v1 JSON |

## Constitutional Relations

- **Enables:** INV-18 (Data Portability) — standard formats make portability trivial
- **Required by:** INV-3 (Audit Trail) — audit exports must be machine-readable
- **Enables:** INV-8 (Cross-Platform) — standard formats work on all platforms

## Status

`ADVISORY` — architectural principle. `AuditChain.export_json()` implements the
Aluminum Audit JSON format. All API responses are JSON.
