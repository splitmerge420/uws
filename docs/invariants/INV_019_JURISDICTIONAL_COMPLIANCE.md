# INV-019 — Jurisdictional Compliance
**Category:** Legal | **Severity:** Critical | **Check Type:** Advisory

> "Data storage and processing must comply with jurisdictional requirements (GDPR, CCPA, HIPAA, etc.)."

---

## Statement

Aluminum OS processes data for users across multiple jurisdictions. Before storing or
processing any personal data, the system must determine the applicable legal framework
and apply the strictest relevant requirements.

## Applicable Frameworks

| Framework | Jurisdiction | Key Requirements |
|-----------|-------------|-----------------|
| GDPR | EU / EEA | Consent, right to delete, data minimisation, 72h breach notice |
| CCPA | California, USA | Opt-out of sale, right to delete, disclosure |
| HIPAA | USA (health data) | PHI encryption, access controls, audit logs, BAA |
| PIPEDA | Canada | Consent, purpose limitation, safeguards |
| LGPD | Brazil | Consent, data subject rights, DPO requirement |

## Union-Set Rule (INV-33)

When a user or their data falls under multiple jurisdictions, the **most restrictive**
requirement from each jurisdiction applies. Example: a GDPR subject using a HIPAA-covered
health service must comply with both GDPR's consent requirements AND HIPAA's PHI
encryption requirements simultaneously.

## Implementation

| Layer | Implementation |
|-------|----------------|
| `acp_governance.py` | `JurisdictionEngine` — determines applicable frameworks |
| `health_connectors.py` | HIPAA-compliant data isolation for health records |
| `src/credential_store.rs` | All credentials AES-256-GCM encrypted (HIPAA/GDPR compliance) |
| `data_classification.rego` | Blocks unclassified access |
| INV-17 implementation | 72h deletion SLA satisfies GDPR Article 17 + CCPA |

## Breach Notification SLA

| Framework | Notification Window | Who to Notify |
|-----------|-------------------|--------------|
| GDPR | 72 hours | Supervisory authority |
| HIPAA | 60 days | HHS + affected individuals |
| CCPA | Without unreasonable delay | Affected consumers |

## Constitutional Relations

- **Depends on:** INV-17 (Right to Delete) — deletion SLA is a GDPR/CCPA requirement
- **Depends on:** INV-11 (Encryption at Rest) — HIPAA/GDPR require encryption
- **Depends on:** INV-3 (Audit Trail) — GDPR/HIPAA require audit logs
- **See also:** INV-33 (Union-Set Jurisdiction), INV-34 (Multi-Vantage Detection)

## Status

`PARTIAL` — `credential_store.rs` encryption satisfies HIPAA/GDPR at-rest requirements.
Full jurisdiction engine (`acp_governance.py`) and breach notification pipeline are Phase 2.
