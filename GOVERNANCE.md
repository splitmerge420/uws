# Governance

## Project Status

`uws` is an open-source project maintained by [@splitmerge420](https://github.com/splitmerge420).
It is in active development (pre-1.0) and is not yet affiliated with Google, Microsoft, or Apple.

## Decision-Making

This project uses a **Benevolent Dictator For Life (BDFL)** model:

- **[@splitmerge420](https://github.com/splitmerge420)** is the project maintainer and has final say on all decisions.
- Technical direction is guided by the [Aluminum OS Architecture](ALUMINUM.md) specification.
- Major architectural changes require an issue discussion before implementation.

## Core Principles

1. **JSON-first** — every command outputs clean, structured JSON.
2. **Provider-agnostic** — no provider-specific SDK lock-in.
3. **AI-native** — design every command to be usable by AI agents.
4. **Dynamic discovery** — Google API surface is built at runtime from Discovery Documents, never from generated crates.
5. **Security by default** — credentials are encrypted at rest (AES-256-GCM), never echoed.

## Contribution Process

1. **Open an issue** or Discussion before starting significant work.
2. **Fork, branch, implement** — follow [CONTRIBUTING.md](CONTRIBUTING.md).
3. **Open a PR** with a `.changeset/` entry.
4. **One review** from maintainer is required to merge.
5. **CI must be green** — `cargo clippy -- -D warnings`, `cargo test`, `cargo fmt --check`.

## Roadmap

| Milestone | Focus |
|---|---|
| `v0.1` | Core engine — Google Workspace via Discovery Documents |
| `v0.2` | Alexandria — Microsoft 365 (Graph API) |
| `v0.3` | Apple Intents — iCloud CalDAV/CardDAV |
| `v0.4` | Android & Chrome Management |
| `v1.0` | Aluminum OS — unified cross-provider abstraction layer |

## Code of Conduct

This project follows the [Contributor Covenant Code of Conduct](docs/CODE_OF_CONDUCT.md).
Violations can be reported via [GitHub Private Advisory](https://github.com/splitmerge420/uws/security/advisories/new)
or email: security@uws.run.

## License

`uws` is licensed under the [MIT License](LICENSE).
The original [googleworkspace/cli](https://github.com/googleworkspace/cli) core is Apache 2.0 (Justin Poehnelt).
