# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | ✅ Active  |

## Reporting a Vulnerability

**Please do not open a public GitHub issue for security vulnerabilities.**

To report a security vulnerability, use one of the following private channels:

1. **GitHub Private Vulnerability Reporting** (preferred):  
   [github.com/splitmerge420/uws/security/advisories/new](https://github.com/splitmerge420/uws/security/advisories/new)

2. **Email**: security@uws.run (PGP key available on request)

### What to Include

- A clear description of the vulnerability
- Steps to reproduce
- Potential impact (data exposure, credential theft, etc.)
- Any suggested mitigation

### Response Timeline

- **Acknowledgement**: Within 48 hours
- **Initial assessment**: Within 5 business days
- **Patch / advisory**: Within 30 days for critical issues

---

## Security Design Principles

`uws` is designed with security as a first-class concern:

### Credential Storage
- OAuth tokens and API secrets are stored using **AES-256-GCM** encryption in `~/.config/uws/`
- The encryption key is derived from the system keyring (via `keyring`) where available
- Credentials are **never** written to stdout, logs, or environment variables

### Input Validation
- All user-supplied file paths are validated against directory traversal (`../` injection)
- URL path segments are percent-encoded before embedding in API URLs
- AI-generated inputs pass through the **Model Armor** sanitization layer when enabled

### Network Safety
- All API calls use TLS (HTTPS) exclusively
- Certificate verification is enforced (no `--insecure` flag exists)
- OAuth tokens are sent as Bearer headers, never in query parameters

### Audit Trail
- All write operations emit an immutable, hash-chained audit entry (`audit_chain.rs`)
- The audit log can be exported and verified: `uws audit export`

### Supply Chain
- Release binaries are signed with **GitHub Artifact Attestations** (SLSA 3)
- The release workflow uses pinned action versions with SHA hashes
- All dependencies are audited with `cargo audit` in CI

---

## AI Agent Safety

Because `uws` is frequently invoked by AI/LLM agents, additional mitigations apply:

| Threat | Mitigation |
|--------|------------|
| Prompt injection via API responses | Model Armor integration (`--sanitize`) |
| Path traversal in `--output-dir` | `validate::validate_safe_output_dir()` |
| Credential exfiltration | No credential echoing; redacted in `--dry-run` |
| Destructive operations | `--dry-run` mode; `CouncilGitHubClient` blocks force-push/delete |
| Vendor lock-in data loss | `LocalNoosphere` local-first; `DropTheMicExport` nuclear export |

---

## Known Limitations

- Apple iCloud integration uses app-specific passwords (no OAuth 2.0)
- The Microsoft Graph integration does not yet support client-credential (service account) flows
- `LocalNoosphere` currently stores data in memory only (no disk encryption at rest in v0.1)

---

*Security policy last updated: 2026-03-22*
