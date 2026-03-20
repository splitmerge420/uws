# INV-008 — Cross-Platform Compatibility
**Category:** Engineering | **Severity:** Mandatory | **Check Type:** Advisory

> "Core functionality must work across macOS, Linux, ChromeOS, iOS, and Android."

---

## Statement

Every core feature of Aluminum OS must be usable across the five target platforms without
platform-specific workarounds in the calling code. Platform-specific implementations may
exist in lower layers but must be hidden behind a common interface.

## Target Platforms

| Platform | Runtime | Notes |
|----------|---------|-------|
| macOS | Native binary + Python | Primary development platform |
| Linux | Native binary + Python | CI/CD platform; production servers |
| ChromeOS | Linux VM + Python | Dave's primary mobile platform |
| iOS | Python (Pythonista / Juno) | Read-heavy; write via API |
| Android | Python + ADB bridge | Android Management API integration |

## Implementation

| Layer | Implementation |
|-------|----------------|
| `src/services.rs` | Abstract service layer; no OS-specific code at this level |
| `toolchain/*.py` | All Python toolchain files use stdlib only (no platform dependencies) |
| `android_chrome.rs` | Android Management API abstraction |
| CI matrix | Tests must run on `ubuntu-latest` at minimum |

## Compliance Checklist

- [ ] No `sys.platform`-gated behaviour in core paths
- [ ] No Windows-only or macOS-only filesystem assumptions
- [ ] Path separators use `std::path::Path` (Rust) or `pathlib.Path` (Python)
- [ ] No hardcoded `/usr/local/bin` or `~/.config` without fallback

## Constitutional Relations

- **Enables:** INV-9 (Offline Capability) — offline must work on all platforms
- **Enables:** INV-10 (Interoperability) — standard formats work everywhere

## Status

`ADVISORY` — architectural principle. Enforced by CI configuration and code review.
No runtime gate (check_type = advisory).
