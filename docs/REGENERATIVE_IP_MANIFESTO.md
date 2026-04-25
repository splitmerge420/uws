# Regenerative IP & Provenance Engine — Architectural Specification

> _"Every idea has a lineage. Every commit has a constituency.  
> Every token generated should feed the hand that guided it."_

---

## 1. Overview

The Regenerative IP & Provenance Engine is the mechanism by which **uws** tracks
the origin, authorship, and human/AI contribution mix of every commit in a
workspace.  It produces a cryptographically attested `ProvenanceTrailer` that
is stored alongside the commit via `git notes` and can be submitted to a
decentralised IP Ledger for revenue attribution.

**CLI surface:**

| Command | Effect |
|---|---|
| `uws ip sign` | Attach a signed provenance trailer to HEAD |
| `uws ip sign --dry-run` | Print the trailer JSON without touching git state |
| `uws ip monetize` | Emit the Ledger registration payload |

---

## 2. Core Principles

### 2.1 Minimum Human Weight (`MIN_HUMAN_WEIGHT = 0.01`)

A fully-autonomous AI commit cannot carry IP rights under the regenerative
model.  The engine enforces that **at least 1 % of creative work** is
attributable to a human.  Attempting to set `human_weight < 0.01` is rejected
at construction time with a clear error.

This floor is intentionally low — a human reviewing and accepting an AI diff
counts.  The goal is to ensure there is _always_ a human in the loop (HITL),
not to artificially inflate human attribution.

### 2.2 Revenue Splits

Every provenance record includes a `RevenueSplit` that partitions future
revenue derived from the IP into three shares:

| Share | Meaning |
|---|---|
| `human_share` | Allocated to the human contributor(s) |
| `ai_share` | Allocated to the AI/tool provider |
| `commons_share` | Held in the Regenerative Commons pool (1 − human − ai) |

Shares must sum to 1.0.  The commons pool funds open-source maintenance and
re-invests in the ecosystems that made the work possible.

### 2.3 Tamper-Evidence via SHA-256

The trailer fields that constitute the attestation (`human_weight`,
`ai_weight`, `revenue_split`, `signed_at`, optional `commit_sha` and
`author`) are serialised to **canonical JSON** (keys sorted lexicographically,
no insignificant whitespace) and hashed with SHA-256.  The hex digest is stored
in `HitlSignature.digest`.

Anyone can verify a trailer has not been tampered with by recomputing the hash
and comparing it to the stored digest.

---

## 3. Data Model

```
ProvenanceTrailer {
    // ── Signed fields (ProvenanceTrailerCore) ──────────────────
    human_weight  : f64          // 0.01 – 1.0 (MIN_HUMAN_WEIGHT enforced)
    ai_weight     : f64          // 0.0  – 0.99
    revenue_split : RevenueSplit
    signed_at     : u64          // Unix timestamp (seconds)
    commit_sha    : Option<String>
    author        : Option<String>

    // ── Signature ──────────────────────────────────────────────
    signature : HitlSignature {
        algorithm : String       // "sha256-canonical-json" (current)
        digest    : String       // Hex-encoded SHA-256 hash
    }
}

RevenueSplit {
    human_share  : f64           // Must be ≥ 0
    ai_share     : f64           // Must be ≥ 0
    commons_share: f64           // = 1 − human − ai
}
```

The `ProvenanceTrailer` struct is defined in
[`src/ledger/provenance.rs`](../src/ledger/provenance.rs).

---

## 4. Signature Algorithm — Current (SHA-256 over Canonical JSON)

```
canonical_json := sort_keys_recursively(serde_json::to_value(core))
signature      := sha256(canonical_json.to_string().as_bytes())
stored_digest  := hex::encode(signature)
```

**Why canonical JSON?**  JSON serialisers may emit object keys in insertion
order, which is implementation-defined.  Sorting keys ensures the byte
sequence is identical across languages and platforms.

**Why not Ed25519 now?**  Key management (generation, rotation, revocation)
requires infrastructure that is not yet available.  SHA-256 is sufficient to
detect accidental or malicious tampering and provides a stable, verifiable
record until asymmetric signing is deployed.

---

## 5. Upgrade Path — Ed25519 / ML-DSA

The `HitlSignature.algorithm` field is already a free-form string so consumers
can negotiate the algorithm:

| Value | Meaning |
|---|---|
| `"sha256-canonical-json"` | **Current** — SHA-256 hash, no private key required |
| `"ed25519"` | Planned — RFC 8032 asymmetric signature (`ed25519-dalek` crate) |
| `"ml-dsa-44"` | Future — FIPS 204 post-quantum signature (ML-DSA level 2) |

### Migration steps (Ed25519)

1. Add `ed25519-dalek = "2"` to `Cargo.toml`.
2. Extend `HitlSignature` with a `pubkey: Option<String>` field (base64-encoded
   DER public key).
3. Replace the `Sha256::new()` call in `ProvenanceTrailer::sign()` with:

   ```rust
   use ed25519_dalek::{Signer, SigningKey};
   let sig = signing_key.sign(canonical.as_bytes());
   HitlSignature {
       algorithm: "ed25519".to_string(),
       pubkey: Some(base64_encode(signing_key.verifying_key().as_bytes())),
       digest: hex::encode(sig.to_bytes()),
   }
   ```

4. Update `verify()` to reconstruct the `VerifyingKey` from `pubkey` and call
   `verifying_key.verify(...)`.
5. Deploy a key-distribution service (or embed the public key in
   `~/.config/uws/ip_key.pub`).

---

## 6. CLI Reference

### `uws ip sign`

Attach a provenance trailer to the current HEAD commit via `git notes
--ref=uws-ip add`.

```
USAGE:
    uws ip sign [flags]

FLAGS:
    --dry-run              Print JSON without modifying git state
    --human-weight <N>     Human contribution fraction (default: 0.7, min: 0.01)
    --ai-weight    <N>     AI contribution fraction   (default: 1 − human-weight)
    --human-share  <N>     Human revenue share        (default: human-weight)
    --ai-share     <N>     AI revenue share           (default: ai-weight)
    --author <EMAIL>       Author identifier          (default: none)
```

Retrieve the attached note with:

```sh
git notes --ref=uws-ip show HEAD
```

### `uws ip monetize`

Emit the Ledger registration payload (JSON) to stdout.  When
`UWS_LEDGER_URL` is set, the payload is printed alongside a note that live
integration is pending.

```json
{
  "schema_version": "1",
  "registration_type": "ip_commit",
  "provenance": { /* ProvenanceTrailer */ }
}
```

---

## 7. Integration with git notes

Git notes are stored in a parallel ref (`refs/notes/uws-ip`) so they never
pollute the commit graph.  They survive push/fetch when the remote is
configured to mirror notes:

```sh
# Push notes to remote
git push origin refs/notes/uws-ip

# Fetch notes from remote
git fetch origin refs/notes/uws-ip:refs/notes/uws-ip
```

The note content is a **single-line compact JSON string** (no embedded
newlines).  `serde_json` escapes all special characters — including `\n`, `"`,
and `\` — so it is safe to pass as a `git notes add -m <string>` argument.

---

## 8. Security Considerations

### 8.1 Input Sanitisation
All string values embedded in the trailer (e.g. `author`, `commit_sha`) are
serialised by `serde_json`, which escapes control characters and quotes.  No
raw interpolation occurs.

### 8.2 Shell Argument Safety
`git notes add` is invoked via `std::process::Command` (not a shell).  The
note string is passed as a single `-m` argument.  There is no shell expansion.

### 8.3 Adversarial AI Usage
Because `uws` is frequently invoked by AI agents, the engine explicitly
enforces `MIN_HUMAN_WEIGHT > 0`.  An AI cannot self-sign a commit with 100 %
AI attribution.

---

## 9. Planned Follow-ups

| Follow-up | Tracking |
|---|---|
| Ed25519 asymmetric signing | Needs key-management infra |
| ML-DSA post-quantum signing | After Ed25519 |
| Live Ledger API integration (`UWS_LEDGER_URL`) | Requires Ledger service deployment |
| Multi-author splits | Extend `RevenueSplit` with a `contributors: Vec<Contributor>` field |
| Git hook integration (`post-commit`) | Auto-sign on every commit |
| `uws ip verify` command | Verify all notes in a range |
| `uws ip audit` command | Audit the provenance chain across a branch |

---

## 10. Relationship to Other Modules

| Module | Relationship |
|---|---|
| `src/council_github_client.rs` | Has its own `ProvenanceTrailer` for council-level audit trails; that type is orthogonal to the regenerative IP trailer in `src/ledger/provenance.rs` |
| `src/audit_chain.rs` | SHA3-256 hash-chained audit log for runtime invariant checks; the ledger uses SHA-256 |
| `uws ip sign` | Complements `git notes`; does not replace or conflict with `git commit --trailer` |

---

*Specification status: **v1 — implemented**.  
See [`src/ledger/provenance.rs`](../src/ledger/provenance.rs) for the canonical
implementation and [`src/ip_commands.rs`](../src/ip_commands.rs) for the CLI.*
