# INV-002 — Informed Consent

| Field | Value |
|-------|-------|
| **ID** | INV-002 |
| **Slug** | consent |
| **Title** | Informed Consent |
| **Severity** | Critical |
| **Status** | ✅ Specified |
| **Source** | `src/constitutional_engine.rs` |
| **Test fixture** | `src/constitutional_engine.rs` → `test_consent_gating_allows_write_with_consent`, `test_consent_gating_blocks_write_without_consent` |

---

## Rationale

An agent that can act without informed consent is an agent that can be
weaponised. The Aluminum OS constitutional model requires that write
operations affecting external systems (email send, calendar create, file
delete) be explicitly approved by the user before execution.

`--dry-run` is the mechanism by which the system makes the proposed
action visible before executing it. When an agent skips `--dry-run` for
a destructive action, it violates INV-002.

## Invariant Statement

> **Every write operation that affects external state (email, calendar,
> files, contacts, tasks) MUST be previewed via `--dry-run` before
> execution, or carry an explicit user confirmation.**

## Enforcement Mechanism

- `ConstitutionalEngine::check_consent_gating()` — evaluates `StateSnapshot.user_consent`.
  - If `state.user_consent == false` and the operation is a write, the engine returns `passed: false`.
- `executor::execute_method` — respects the `--dry-run` flag before making HTTP requests.
- The Zero Trust Gate does not enforce this invariant directly but its audit log records consent state for post-hoc review.

## Violation Handling

**Severity: Critical** — write operations without consent are blocked.
In strict mode (`ConstitutionalEngine::new(true)`), violations halt execution.
In advisory mode (`new(false)`), a warning is logged.

## Test Fixtures

```rust
// src/constitutional_engine.rs
#[test]
fn test_consent_gating_blocks_write_without_consent() {
    let engine = ConstitutionalEngine::new(true);
    let mut state = StateSnapshot::new("write", "email/send");
    state.user_consent = false;
    let results = engine.check_all(&state);
    let consent_check = results.iter().find(|c| c.id == "INV-2").unwrap();
    assert!(!consent_check.passed);
}
```

## Followups

- [ ] Require `--confirm` flag for all destructive operations in the full CLI binary.
- [ ] Add `INV-002` tag to executor dry-run bypass audit log entries.
