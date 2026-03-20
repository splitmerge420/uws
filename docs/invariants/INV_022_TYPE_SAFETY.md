# INV-022 — Type Safety
**Category:** Engineering | **Severity:** Warning | **Check Type:** Advisory

> "All public interfaces must have type annotations."

---

## Statement

Every public function, method, and class attribute in Aluminum OS must carry explicit
type annotations. Untyped public interfaces are a warning-level finding. Type annotations
serve as machine-verifiable documentation and enable static analysis tools to catch
invariant violations before runtime.

## Language-Specific Requirements

| Language | Requirement | Tool |
|----------|------------|------|
| Rust | All `pub fn` must have typed signatures (enforced by compiler) | `rustc` |
| Python | All `def` in public modules must have `-> ReturnType` and parameter types | `mypy`, `pyright` |
| TypeScript | Strict mode; no `any` in public interfaces | `tsc --strict` |

## Rust — Already Enforced by Compiler

```rust
// Good: fully typed
pub fn run_logic_gate(
    &mut self,
    component_id: &str,
    has_fallback: bool,
    provider_abstracted: bool,
) -> Result<(), GateError> { ... }
```

## Python — Enforcement Examples

```python
# Compliant
def run_logic_gate(
    self,
    component_id: str,
    has_fallback: bool,
    provider_abstracted: bool,
) -> None: ...

# Violation — missing return type
def run_logic_gate(self, component_id, has_fallback, provider_abstracted):
    ...
```

## Implementation

| Layer | Implementation |
|-------|----------------|
| `toolchain/*.py` | All public functions carry type annotations |
| `zero_trust_registry.py` | Fully typed with `Optional`, `List`, `Dict`, `Tuple` |
| `pqc_provider.py` | Fully typed |
| `src/*.rs` | Compiler enforces |
| Linter | `invariant_linter.py` (advisory check; does not block) |

## Constitutional Relations

- **Strengthens:** INV-21 (Error Boundaries) — typed errors require typed interfaces
- **Supports:** INV-23 (Test Coverage) — typed interfaces are easier to test
- **Enables:** INV-36 (Technical Invariant Enforcement) — types are machine-checkable specs

## Status

`IMPLEMENTED` — all `toolchain/*.py` files added in this session carry full type
annotations. Rust compiler enforces types on all `pub fn`.
