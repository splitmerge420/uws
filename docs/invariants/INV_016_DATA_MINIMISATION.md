# INV-016 — Data Minimisation
**Category:** Privacy | **Severity:** Mandatory | **Check Type:** Advisory

> "Collect only the minimum data necessary for the operation."

---

## Statement

Every data collection, API request, and storage operation must request and retain only
the fields required for the immediate operation. Fields that are not required must be
excluded from requests using `$select`, `fields`, or equivalent query parameters. Bulk
responses must be filtered before storage or display.

## Implementation

| Layer | Implementation |
|-------|----------------|
| `src/executor.rs` | `--params` flag maps to `$select`/`fields` for provider-level filtering |
| Gmail | `fields=messages(id,threadId,snippet,labelIds)` — never pull body unless needed |
| Drive | `fields=files(id,name,mimeType,modifiedTime)` — no content by default |
| Microsoft Graph | `$select=subject,from,receivedDateTime` — no body by default |
| OneDrive | `$select=name,size,lastModifiedDateTime` |

## Minimisation Checklist

- [ ] All `list` operations use `fields` or `$select`
- [ ] No full email body retrieved unless user explicitly requests it
- [ ] Health data: only the fields required for the immediate query are fetched
- [ ] Audit log evidence field: contains metadata, not PII

## Anti-Pattern Examples

```bash
# Excessive: fetches entire message bodies for list
uws gmail users messages list --params '{"userId":"me"}'

# Compliant: fetches only metadata
uws gmail users messages list \
  --params '{"userId":"me","fields":"messages(id,threadId,snippet)"}'
```

## Constitutional Relations

- **Strongest form:** INV-14 (Zero-Knowledge) — ZK eliminates collection entirely
- **Required by:** INV-19 (Jurisdictional Compliance) — GDPR Article 5(1)(c) data minimisation
- **Required by:** INV-17 (Right to Delete) — less data = smaller deletion surface

## Status

`PARTIAL` — `executor.rs` supports field selection via `--params`. Full enforcement
(blocking over-collection) is Phase 2.
