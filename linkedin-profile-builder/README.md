# LinkedIn Profile Builder & Trust Auditor

A conversational plugin for Claude that builds professional LinkedIn profiles, mines forgotten credentials, and verifies profile accuracy — for individuals, organizations, and platform trust.

## What It Does

Most people are terrible at writing about themselves. They undersell their achievements, forget notable things they've done, and either sound like corporate robots or rambling messes. This plugin fixes that by:

1. **Interviewing you conversationally** — asking the right questions to surface your real career story
2. **Mining your credentials** — searching public web sources for press coverage, achievements, and associations you forgot to mention
3. **Drafting a complete profile** — generating every LinkedIn section with proper formatting, character limits, and industry-appropriate voice
4. **Iterating with you** — revising until it sounds like you, not a template

## Components

| Component | Name | Purpose |
|-----------|------|---------|
| Skill | profile-builder | Full conversational workflow for building/updating a LinkedIn profile |
| Skill | credential-miner | Automated web research to surface forgotten achievements and press coverage |
| Skill | profile-auditor | Verify profile claims, detect bots, and audit organizational affiliations |

## The Trust Problem

LinkedIn is an honor system. Anyone can claim any title, degree, or achievement with zero verification. Bot farms create thousands of fake profiles. Ex-employees can prominently list your company and there's no recourse for founders. This plugin addresses all three problems:

1. **For individuals**: Build a profile where every claim is backed by verifiable public evidence
2. **For organizations**: Audit who's claiming affiliation with your company and whether their claims are accurate
3. **For the platform**: Score profiles on verifiability, detect bot signals, and improve overall trust

## What Makes This Different

- **Conversational, not form-based**: It talks to you like a good career coach, not a web form
- **Credential mining**: It finds things about you that you forgot or undersold
- **Voice matching**: The output matches YOUR speaking style, not generic corporate language
- **Anti-bullshit filter**: No "results-driven professional with a proven track record of leveraging synergies"
- **Trust scoring**: Every claim gets a verification rating (Verified / Supported / Unverified / Flagged)
- **Bot detection**: Multi-signal analysis for identifying fake profiles
- **Organizational audit**: Founders can verify who's legitimately claiming affiliation

## Author

Built by Dave Sheldon (@splitmerge420) as an open-source contribution.

## License

MIT