# Changelog

## v0.2.0 - Trust Verification Layer (2026-03-24)

### Added
- **4-Tier Verification System**: Escalating verification from passive web search through conversational probing, optional documents, to official database checks
- **Trust Score (0-100)**: Transparent scoring formula with breakdown by claim type and verification depth
- **Consent & Transparency Model**: Nothing goes live without approval. Users control the entire verification process.
- **Verification Protocols Reference**: Detailed protocols for employment, education, achievements, skills, metrics, board roles, and publication/patent claims
- **Diploma Mill Detection**: Flags unaccredited institutions, suspicious timelines, and look-alike school names
- **Pay-to-Play Award Detection**: Identifies vanity awards vs legitimate recognitions
- **Bot Detection Enhancement**: Multi-signal weighted scoring with AI-generated content detection
- **Organizational Audit Enhancement**: Structured founder interview, discrepancy categories (title inflation, date stretching, role fabrication, credit claiming), actionable next steps
- **Pre-publish verification**: Trust layer integrated into profile-builder flow (Phase 4) - profiles are verified before delivery
- **credential-miner skill**: Pushed to GitHub (was missing from v0.1.0)

### Changed
- profile-auditor upgraded from basic 3-use-case tool to full Trust Verification Layer with 4 use cases
- profile-builder now includes Phase 4 (Trust Verification) and Phase 5 (Review & Delivery) instead of going straight to review
- Plugin manifest updated with trust/verification/audit/bot-detection keywords
- README updated with trust-first differentiators

## v0.1.0 - Initial Release (2026-03-24)

### Added
- profile-builder skill: Conversational interview + credential mining + profile drafting
- credential-miner skill: Parallel web search batches for surfacing forgotten achievements
- profile-auditor skill: Basic claim verification, bot detection, organizational audit
- LinkedIn field specifications reference
- Plugin manifest and connectors
- Packaged as .plugin for Cowork installation