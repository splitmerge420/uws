# INV-021 — Error Boundaries
**Category:** Engineering | **Severity:** Mandatory | **Check Type:** Guard Check

> "Every module must have error boundaries that prevent cascade failures."

---

## Statement

No error may silently propagate across module boundaries. Every public function that can
fail must return a typed error result. `bare except` clauses, exception swallowing, and
unhandled panics are constitutional violations.

## Dangerous Patterns (Linter Detects)

```python
except:                  # bare except — swallows ALL exceptions including SystemExit
except Exception: pass   # exception swallowing without logging
```

## Guard Patterns (Must Be Present)

`try`, `except`, `Result`, `Option`, `error_boundary`, `catch`

## Implementation

| Layer | Implementation |
|-------|----------------|
| `src/error.rs` | `GwsError` typed error enum — all errors return structured JSON |
| `src/executor.rs` | Every API call returns `Result<_, GwsError>`; errors mapped to exit code 1 |
| `council_github_client.rs` | `CouncilError` enum — every error variant documented |
| `zero_trust_registry.rs` | `GateError` enum — specific variant per gate |
| `acp_governance.py` | Try/except with explicit logging — never silent |
| `error_boundaries.rego` | `default allow = false`; blocks bare-except modules |
| Linter | `invariant_linter.py` flags bare `except:` clauses |

## Error Boundary Contract

Every module boundary must:

1. Return a typed error, not raise a generic exception
2. Log the error with context before propagating
3. Not swallow errors silently
4. Map errors to the appropriate severity level

## Rust Error Pattern

```rust
// src/error.rs — typed error with structured JSON output
pub enum GwsError {
    AuthError { .. },
    NetworkError { .. },
    ValidationError { .. },
    // ...
}

// Every function:
pub fn execute(op: Op) -> Result<Response, GwsError> {
    // Never: unwrap() in production paths
    // Always: ?-operator or explicit match
}
```

## Python Error Pattern

```python
# All toolchain functions:
def process(data: dict) -> dict:
    try:
        result = _inner_process(data)
        return result
    except SpecificError as e:
        logger.error("process failed: %s", e)  # explicit log
        raise  # re-raise with context, never swallow
```

## Test Vectors

| Scenario | Expected Outcome |
|----------|-----------------|
| Module with `except:` | Linter flags INV-21 |
| `GwsError::AuthError` propagated | Caller receives typed error, not panic |
| `CouncilError` from blocked op | `execute()` returns `Err(CouncilError::InvariantViolation)` |
| `GateError::LogicGateFailed` | Gate returns typed error, audit chain records DENY |

## Constitutional Relations

- **Required by:** INV-24 (Graceful Degradation) — boundaries enable graceful degradation
- **Required by:** INV-3 (Audit Trail) — errors must be auditable, not swallowed
- **Supported by:** `error_boundaries.rego`

## Status

`IMPLEMENTED` — `GwsError`, `CouncilError`, `GateError` typed errors; linter detects
bare exceptions; `error_boundaries.rego` default-deny.
