# INV-018 — Data Portability
**Category:** Privacy | **Severity:** Mandatory | **Check Type:** Advisory

> "Users must be able to export all their data in standard formats."

---

## Statement

Every piece of user data managed by Aluminum OS must be exportable on demand in a
standard, machine-readable format that can be imported into any compliant alternative
system. Export must be available at any time, not just during account closure.

## Export Formats by Domain

| Domain | Format | Standard |
|--------|--------|---------|
| Email | MBOX / EML | RFC 4155 |
| Calendar | iCalendar (.ics) | RFC 5545 |
| Contacts | vCard (.vcf) | RFC 6350 |
| Files | ZIP archive | PKWARE |
| Tasks | JSON-LD | W3C |
| Health records | FHIR Bundle (JSON) | HL7 FHIR R4 |
| Audit log | Aluminum Audit JSON | (INV-10) |
| Preferences | JSON | — |

## Implementation

| Layer | Implementation |
|-------|----------------|
| `src/executor.rs` | `--output <file>` flag saves binary/JSON responses to disk |
| `src/formatter.rs` | JSON / YAML / CSV / table output formats |
| `AuditChain.export_json()` | Full audit log export |
| `uws drive files export` | Google Docs/Sheets → PDF/XLSX/CSV |
| `uws ms-onedrive` | OneDrive file download |

## Export Completeness Requirements

- [ ] Export includes **all** data, not a summary
- [ ] Export is self-contained (no dangling references to external systems)
- [ ] Export completes in < 1 hour for typical datasets
- [ ] Export file is integrity-verified (SHA3-256 checksum included)

## Constitutional Relations

- **Required by:** INV-19 (Jurisdictional Compliance) — GDPR Article 20 (data portability)
- **Depends on:** INV-10 (Interoperability) — standard formats make portability trivial
- **Enabled by:** INV-4 (Data Classification) — classification determines export sensitivity

## Status

`PARTIAL` — `--output` flag and audit log export implemented. Full cross-service
export orchestration is Phase 2.
