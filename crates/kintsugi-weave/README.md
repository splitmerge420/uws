# Kintsugi Weave

**Regenerative CI engine for the `uws` / Aluminum OS project.**

Kintsugi Weave runs on every PR and produces a "weave report" — a constitutional health check that evaluates three gates:

| Gate | Status | Wired In |
|------|--------|----------|
| NPFM Gate | **stub** | PR #7 |
| Provenance (GoldenTrace-ID trailer) | **real** (soft-fail) | PR #12 (enforcing) |
| Swarm Commander | **stub** | TBD |

## Usage

```bash
# Run all gates, get JSON report
kintsugi-weave run --pr-body "$PR_BODY" --pr-number 42 --sha "$SHA"

# Get GitHub Markdown comment (pipe to gh pr comment)
kintsugi-weave format-comment --pr-body "$PR_BODY" --pr-number 42 --sha "$SHA"

# Individual gates
kintsugi-weave npfm-gate
kintsugi-weave provenance-validate --pr-body "$PR_BODY"
kintsugi-weave swarm-summary
```

## Building

```bash
cargo build --release
# Binary at: target/release/kintsugi-weave
```

## Testing

```bash
cargo test
```

## Gate Details

### 1. NPFM Gate (stub)

Detects Non-Productive Feature Multiplication — PRs that add many features with low constitutional alignment. Currently stubbed at score `0.0` (clean).

**TODO(#7):** Wire real NPFM implementation once PR #7 lands.

### 2. Provenance Validation (real, soft-fail)

Checks that the PR body contains a `GoldenTrace-ID:` trailer line. This is a real check — it scans the PR body for the trailer. A missing trailer produces a ⚠️ warning but does not block merge in this iteration.

**TODO(#12):** Switch to enforcing mode once GoldenTrace (#12) is fully wired.

### 3. Swarm Commander (stub)

Summarises the swarm review state. Currently stubbed as PASS.

**TODO:** Wire real Swarm Commander once swarm review integration lands.

## Followups (before gates can be made enforcing)

1. **PR #7 (NPFM)** — Implement the real NPFM scoring algorithm. Once landed, replace `npfm_gate()` stub with a call to its API.
2. **PR #12 (GoldenTrace)** — Full GoldenTrace provenance system. Once landed, flip provenance gate from soft-fail to enforcing.
3. **Swarm Commander** — Design and implement the swarm review summary API. Wire it into `swarm_commander_summary()`.
4. **Rate-limit / retry** — The GitHub comment step in the workflow uses a simple `gh` call; add retry logic for flaky network.
5. **Caching** — Cache the `kintsugi-weave` binary in CI to speed up the workflow.
