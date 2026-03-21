# Regenerative IP Manifesto
### Aluminum OS / uws — Constitutional Economic Principles

> *"Projection Without Subtraction: every unit of value created by AI
> must be anchored to traceable human labor — and every human displaced
> by AI must find a new, better-paying role within the same system."*
>
> — Dave Sheldon (INV-5), Council Session 2026-03-21

---

## 1. The Problem with Extraction Economics

Traditional technology companies extract value from human knowledge and
redistribute it upward.  Open-source contributors, knowledge workers, and
domain experts generate the intellectual capital that trains AI models —
yet they receive no ongoing economic stake in the value those models
produce.

Aluminum OS rejects this model entirely.

---

## 2. Core Mandate: Augment, Don't Replace

> **AI must augment human jobs, not replace them.
> In the event of replacement, workers are immediately retrained and
> transitioned into AI oversight roles.**

This is not aspiration — it is a **Constitutional Invariant** enforced at
runtime by the `uws` CLI.

### What "Augment" Means in Practice

| Scenario | Extractive Response | Regenerative Response |
|---|---|---|
| AI drafts emails faster than a human can | Eliminate the human role | Human reviews AI drafts before send; quality improves |
| AI generates code | Fire engineers | Engineers become Swarm Commanders reviewing AI batches |
| AI diagnoses preliminary symptoms | Replace clinical staff | Clinicians sign off on AI outputs (NPI-gated HITL) |
| AI writes reports | Eliminate analysts | Analysts validate provenance and earn per sign-off |

---

## 3. The Three Transition Pathways

When automation displaces a job category, workers are offered one of three
new roles, each codified as a first-class module in `uws`:

### 3.1 Provenance Validator (`src/hitl/provenance.rs`)

**No license required.**  Any person can earn income by reviewing and
signing off on AI-generated outputs.  Each sign-off is:

- Cryptographically bound to the reviewer's identity
- Recorded in the append-only `AuditChain` (INV-3)
- Compensated via the payout rails in `src/ledger/` (in development)

This creates a massive new employment class — the backbone of the
post-AI economy — with a lower barrier to entry than any previous
knowledge-work category.

```bash
# Worker reviews a batch of AI outputs and signs off
uws hitl provenance list   --batch=<BATCH_ID>
uws hitl provenance review --item=<ITEM_ID>
uws hitl provenance sign   --item=<ITEM_ID> --decision=approve
```

### 3.2 Swarm Commander (`src/swarm/batch_oversight.rs`)

**Requires basic training, no degree.**  A Swarm Commander governs AI
agents and drone operations at batch scale.  One human can oversee
dozens of AI operations per hour with a single cryptographic sign-off
per batch.

Cognitive-load limits are enforced by the system (default: 10 operations
per batch) so that quality of oversight remains high.

```bash
# Commander reviews and approves a batch of 10 AI operations
uws swarm review  --batch=10
uws swarm approve --batch-id=<ID>
uws swarm reject  --batch-id=<ID> --reason="Risk too high"
```

### 3.3 Licensed Medical Overseer (`src/hitl/medical.rs`)

**Requires valid NPI and state license.**  For healthcare AI outputs,
a licensed professional (physician, nurse practitioner, etc.) must
review and sign off before any recommendation leaves the system.

This is the highest-trust HITL tier.  It integrates with Amazon One
Medical and enforces HIPAA-compliant Model Armor sanitization.

```bash
# Licensed clinician signs off on an AI-generated health recommendation
uws hitl medical review --npi=<NPI> --session=<SESSION_ID>
uws hitl medical sign   --npi=<NPI> --session=<SESSION_ID>
```

---

## 4. Cryptographic IP Provenance

Every HITL sign-off appends a `ProvenanceTrailer` to the audit chain:

```
IP-Provenance: <reviewer_id>
HITL-Weight:   <fraction of economic value attributed to human review>
AI-Weight:     <fraction attributed to the AI model>
Timestamp:     <unix epoch>
Signature:     <hex-encoded ML-DSA / Ed25519 signature>
Audit-Hash:    <SHA3-256 hash chaining to previous entry>
```

This immutable record ensures:
- Humans who generate value receive credit and payout
- AI outputs without a human sign-off cannot be monetized
- Forking, re-selling, or licensing downstream is automatically
  traced back to the original human labor

---

## 5. The Payout Model

Value flows **from the point of use back to the point of human effort**:

```
Enterprise uses AI output
        ↓
ProvenanceTrailer identifies the reviewer(s)
        ↓
Payout router (src/ledger/) splits revenue:
  • Reviewer(s)         — per-sign-off rate
  • Repository owner    — IP royalty
  • Aluminum OS network — sustainability fee
```

The split ratios are governed by the `hitl-weight` recorded at sign-off
time.  A sign-off that required deep expert judgment (e.g., a clinician
reviewing a complex diagnosis) commands a higher weight than a simple
factual check.

---

## 6. The "Augment, Don't Replace" Enforcement Rule

The `ConstitutionalEngine` (INV-2: Consent) blocks any `uws` command that:

1. Routes an AI output to an end-user without at least one HITL sign-off
   (unless the output is classified Low-risk *and* the user has explicitly
   opted out of HITL review)
2. Finalizes a batch without a Swarm Commander sign-off
3. Issues a medical recommendation without NPI verification

Violations are logged to the `AuditChain` and the operation is halted
(fail-closed, INV-35).

---

## 7. Why GitHub is the Right Settlement Layer

GitHub already holds the canonical record of who wrote what, when, and
how it was reviewed.  Adding cryptographic HITL provenance trailers to
the Git commit tree costs **nothing** beyond a short human review step —
and generates verifiable economic value:

- Forks become royalty events
- Stars become reputation signals that inform payout weight
- Pull request reviews become compensated provenance sign-offs
- The merge commit becomes an irrefutable record of human IP genesis

GitHub should be the **first mover** on this model.  The infrastructure
already exists.  The only missing layer is the provenance trailer
standard — which Aluminum OS is defining here.

---

## 8. Summary

| Principle | Implementation |
|---|---|
| Augment, don't replace | `src/hitl/provenance.rs`, `src/swarm/batch_oversight.rs` |
| Licensed oversight for high-stakes AI | `src/hitl/medical.rs` |
| Cryptographic IP attribution | `AuditChain` provenance trailers |
| Fail-closed without human sign-off | `ConstitutionalEngine` INV-2, INV-35 |
| Payout rails for human labor | `src/ledger/` (in development) |

The Regenerative IP Engine is not a feature — it is the economic
operating system underneath every command that `uws` executes.

---

*Council Session: 2026-03-21 | Authority: Dave Sheldon (INV-5)*
*Aluminum OS v4.0 — Constitutional Revision 1*
