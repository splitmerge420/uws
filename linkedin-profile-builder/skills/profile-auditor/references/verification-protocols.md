# Verification Protocols: Multi-Layer Claim Verification

Detailed protocols for verifying specific types of professional claims. Each claim type has escalating verification tiers — the higher the claim's impact, the more layers of verification required.

## Verification Philosophy

**Consent-first, conversational, transparent.** Every verification step is explained to the user. Nothing goes live without explicit approval. The AI acts as a friendly but thorough fact-checker, not an interrogator.

## Verification Tiers

### Tier 1: Passive Verification (All Claims)
Automated web search — no user action required beyond initial consent.

- Cross-reference claim against public web sources
- Check company websites, press releases, news articles
- Search LinkedIn for corroborating profiles (colleagues who list same company/dates)
- Check social media for contemporaneous mentions
- Result: Verified / Supported / Unverified / Flagged

### Tier 2: Conversational Verification (Medium-Impact Claims)
AI asks follow-up questions to probe depth and consistency.

- Ask for specific details only someone who held the role would know
- Request names of colleagues, managers, or direct reports
- Ask about specific projects, tools, or processes used
- Check for temporal consistency (did the timeline make sense?)
- Look for the "texture of truth" — real experiences have messy, specific details; fabrications tend to be smooth and generic
- Result: Consistent / Inconsistent / Needs Escalation

### Tier 3: Document Verification (High-Impact Claims)
User provides supporting documentation voluntarily.

- Request photos of diplomas, certificates, or awards
- Ask for links to published work, patents, or official records
- Invite screenshot of internal records (redacted as needed)
- User can decline any request — declining doesn't flag the claim, it just stays "Unverified"
- Result: Document Confirmed / Document Pending / User Declined

### Tier 4: Third-Party Corroboration (Critical Claims)
Cross-reference with external parties or databases.

- Search official databases (patent offices, bar associations, medical boards, SEC filings)
- Check university alumni directories or commencement records
- Look for conference speaker listings, award recipient lists
- Search professional licensing databases
- Result: Externally Confirmed / Not Found in Records / Contradicted

## Claim-Specific Protocols

### Employment Claims

**What to verify:** Title, company, dates, responsibilities, impact metrics

**Tier 1 (Automatic):**
- `"[Name]" "[Company]"` web search
- Company website team/about pages (current + archived via Wayback Machine references)
- Press releases mentioning person + company
- Other LinkedIn profiles listing same company (do they corroborate the department, timeline?)
- Glassdoor/Indeed company reviews mentioning relevant team details
- Conference bios listing the affiliation

**Tier 2 (Conversational):**
- "Who was your direct manager at [Company]?"
- "What tools or tech stack did you use daily?"
- "Walk me through a typical project you led there"
- "What was the team structure — how many people, what were the roles?"
- Check: Do the details they provide match the seniority level they claim? A VP should describe strategic decisions, not just task execution.

**Tier 3 (Document — only for C-suite, founder, or disputed claims):**
- Offer letter or employment contract (redacted)
- Pay stub or tax document showing employer name
- Company email screenshot
- Business registration showing their name (for founders)

**Impact scoring for employment metrics:**
- "Grew revenue 300%" → Tier 4: Search for any public evidence of company growth during their tenure
- "Managed a team of 50" → Tier 2: Ask about team structure, reporting lines, hiring process
- "Built the product from scratch" → Tier 2: Ask for technical details + Tier 1: search for product launch press

### Education Claims

**What to verify:** Degree type, institution, graduation year, honors, field of study

**Tier 1 (Automatic):**
- `"[Name]" "[Institution]" alumni OR graduated`
- University alumni directories (many are public)
- Commencement programs (sometimes posted online)
- Thesis/dissertation databases (ProQuest, institutional repositories)
- Student newspaper archives mentioning the person

**Tier 2 (Conversational):**
- "What was your thesis or capstone project about?"
- "Who was your advisor or a professor you remember well?"
- "What dorm or neighborhood did you live in?"
- "What year did you start, and did you graduate on time?"
- Check: Graduate-level knowledge questions in their claimed field (a PhD in chemistry should be able to discuss their research area with specificity)

**Tier 3 (Document — for advanced degrees, medical/law/engineering):**
- Diploma photo
- Transcript (redacted grades are fine — we just need institution + degree confirmation)
- Student ID or alumni card
- Professional license derived from the degree (bar number, medical license, PE stamp)

**Tier 4 (External — for doctoral and professional degrees):**
- Search the institution's dissertation database
- Check professional licensing boards (state bar, medical board, nursing board)
- Search for published academic work from their claimed institution/department
- Verify the institution itself is accredited (diploma mill detection)

**Diploma Mill Red Flags:**
- Institution has no physical campus
- "Accredited" by an unrecognized accreditation body
- Degree awarded in unusually short time
- Institution name is very similar to a prestigious school but slightly different
- No faculty listings or all faculty are adjuncts with no institutional affiliations

### Achievement & Award Claims

**What to verify:** Award name, granting organization, year, significance

**Tier 1 (Automatic):**
- `"[Award Name]" [year] recipients OR winners`
- Granting organization's website — do they list past recipients?
- Press coverage of the award ceremony
- `"[Name]" "[Award Name]"` direct search

**Tier 2 (Conversational):**
- "Who presented the award?" or "What was the ceremony like?"
- "What were you nominated for specifically?"
- "Who else won that year or in your category?"
- Check: Is the award actually prestigious, or is it a pay-to-play "honor"?

**Pay-to-Play Award Red Flags:**
- Award requires a "nomination fee" or "membership"
- No selection committee or transparent selection criteria
- Everyone who applies seems to win
- The granting organization's primary business model appears to be selling awards

### Skill & Expertise Claims

**What to verify:** Proficiency level, practical application, recognition by peers

**Tier 1 (Automatic):**
- GitHub contributions in claimed technologies
- Published articles or talks on the topic
- Certifications from recognized bodies (AWS, Google, Microsoft, etc.)
- Stack Overflow or forum contributions demonstrating expertise
- Open-source project contributions

**Tier 2 (Conversational):**
- "Tell me about a complex problem you solved using [skill]"
- "How do you stay current in [skill area]?"
- "What's a common misconception about [skill] that you've had to correct?"
- Check: Depth of knowledge. Someone who truly has a skill can discuss edge cases, tradeoffs, and real-world complications — not just textbook definitions.

### Metrics & Numbers Claims

**What to verify:** Revenue numbers, user counts, growth percentages, team sizes

**Tier 1 (Automatic):**
- Company press releases mentioning metrics
- Crunchbase, PitchBook, or similar for funding/revenue data
- App store download counts (for app companies)
- Web traffic estimates (SimilarWeb, etc.)
- SEC filings for public companies

**Tier 2 (Conversational):**
- "What was the baseline before you started, and what did you grow it to?"
- "Over what time period?"
- "Was this your individual contribution or a team effort?"
- "How was this measured — what tool or system tracked it?"
- Check: Are the numbers internally consistent?

**Plausibility Checks:**
- Does the claimed growth rate match industry norms?
- Are the absolute numbers reasonable for the company size?
- If they claim credit for a team achievement, do they acknowledge the team?
- Do the numbers conflict with publicly available company data?

### Board & Advisory Role Claims

**What to verify:** Organization, role type, dates, actual involvement

**Tier 1 (Automatic):**
- Organization's website listing board/advisory members
- Annual reports or 990 filings (for nonprofits — board members are listed)
- Press releases announcing board appointments
- SEC filings (for public company boards)

**Tier 2 (Conversational):**
- "How often does the board meet?"
- "What was a key decision you participated in?"
- "How did you get involved with the organization?"
- Check: Advisory roles are often informal and unverifiable. Focus on whether the person can describe real involvement.

### Publication & Patent Claims

**Tier 1 (Automatic):**
- Google Scholar search for publications
- Google Patents / USPTO / EPO search for patents
- Publisher/journal website search
- ResearchGate, ORCID, or institutional repository listings
- Citation count and impact metrics

**Tier 4 (External):**
- These are the most verifiable claims — if the publication or patent exists, it's in a database
- If NOT found: could be a trade publication, internal whitepaper, or ghostwritten — ask for clarification
- Patent numbers are directly verifiable

## Bot Detection Enhancement

### Multi-Signal Scoring

Each signal gets a weight. Total score determines bot probability.

| Signal | Weight | Detection Method |
|--------|--------|-----------------|
| No verifiable external footprint | 25 | Tier 1 searches return zero results |
| Profile photo is stock/AI-generated | 20 | Reverse image search, AI detection tools |
| Employment at non-existent companies | 20 | Company website doesn't exist, no other employees found |
| Network is mostly other suspected bots | 15 | Analyze connection overlap with known bot clusters |
| Activity is purely automated | 10 | Posting at exact intervals, only shares, no original content |
| Generic buzzword bio | 5 | NLP analysis of About section originality |
| Skills don't match experience | 5 | Cross-reference skills list with employment history |

**Score interpretation:**
- 0-15: Likely real (may just have thin web presence)
- 16-35: Suspicious — recommend manual review
- 36-60: Probable bot — multiple red flags
- 61+: Almost certainly fake — strong evidence across multiple signals

### AI-Generated Profile Detection

Emerging signals for profiles written by AI:
- Unnaturally consistent tone across all sections
- No typos, no personality, no quirks
- Generic accomplishment statements with no verifiable specifics
- "Perfect" formatting that no human would naturally produce
- Buzzword density exceeding industry norms
- Claims that are plausible but never specific enough to verify

## Organizational Audit Enhancement

### Founder Verification Interview

When a company founder requests an org audit, conduct a structured interview:

1. **Company basics:** Legal name, DBA names, incorporation date, state, current status
2. **Personnel history:** For each person who ever worked there, collect: name, title, approximate dates, whether they left on good terms
3. **Title accuracy:** "Did anyone ever hold the title of VP/Director/C-suite?" — founders often know if an ex-intern is now claiming to have been CTO
4. **Milestone timeline:** Key events (product launches, funding rounds, pivots, acquisitions) with approximate dates — used to cross-reference employee claims
5. **Known disputes:** Any former employees with known grievances who might be misrepresenting their role?

### Discrepancy Categories

When claims don't match founder records:

| Category | Example | Recommended Action |
|----------|---------|-------------------|
| Title inflation | Claimed "VP of Engineering", was actually "Senior Developer" | Flag as title discrepancy |
| Date stretching | Claimed 3 years, actually worked 8 months | Flag with correct dates |
| Role fabrication | Claims to have worked there, founder has no record | Flag as unrecognized |
| Credit claiming | Claims to have "built the product", was on marketing team | Flag with context |
| Company misrepresentation | Claims company had 200 employees, actually had 12 | Flag with context |

### Recommended Actions for Founders

After audit, provide actionable next steps:
1. **Claim your company page** on LinkedIn if not already done
2. **Report inaccurate profiles** through LinkedIn's dispute process
3. **Document accurate records** so future audits are faster
4. **Consider publishing** a team page on your company website as a public record
5. **For serious misrepresentation** (e.g., someone claiming a title to get hired elsewhere), consult legal counsel — this may constitute fraud