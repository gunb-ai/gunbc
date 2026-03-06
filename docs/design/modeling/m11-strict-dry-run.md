# M11: Strict Dry-Run Mode

**Status**: Design
**Lane**: B (Workflow Execution Safety)
**Depends on**: M10
**Blocks**: M12

## Problem

The current dry-run mode uses lenient defaults: missing resource/env inputs get
default mocks (empty strings, `ShellResponse::ok("")`). This masks modeling
gaps — a transport node that should declare a resource port silently receives
a default mock and appears to work.

## Design

### 1. DryRunStrictness enum

```rust
pub enum DryRunStrictness {
    /// Current behavior: missing inputs get default mocks.
    Lenient,
    /// Missing resource/env inputs produce poison values that fail on consumption.
    Strict,
}
```

Add to `ExecutionMode::DryRun`:
```rust
ExecutionMode::DryRun(BoundaryMocks, DryRunStrictness)
```

### 2. Poison value model

Add `Value::Poison { reason: String }` variant. When a transport or resource
boundary node has no mock in strict mode, emit `Value::Poison` instead of a
default value.

### 3. Fail-fast on poison consumption

In the executor, before passing a `Value` to a node's input handler, check
for `Value::Poison`. If found, emit a data-flow trace error:
```
strict dry-run failure: node "execute_read_clippy_toml" consumed poison value
  from port "file_content"
  reason: no mock provided for transport boundary node "execute_read_clippy_toml"
  trace: boundary → compare_content → execute_read_clippy_toml
```

### 4. CI/testgen wiring

- `--dry-run=strict` CLI flag
- Testgen-generated tests use strict mode
- CI workflows use strict mode
- Developer `make test` uses lenient (default)

## Migration

Phase 1: Add `DryRunStrictness` enum, default to `Lenient`. No behavior change.
Phase 2: Wire `Strict` into testgen and CI paths.
Phase 3: Add `Value::Poison` and fail-fast.

## Files

- `core/exec/src/execute.rs` — DryRunStrictness, poison injection
- `core/ir/src/value.rs` — Value::Poison variant
- `core/codegen/src/testgen/` — strict mode in generated tests
- `gunbc-app/src/bin/` — CLI flag

## References

- `core/exec/src/execute.rs:1190` — should_intercept_for_mode()
- `core/ir/src/value.rs` — Value enum
- `docs/design/modeling/m10-resource-declarations.md` — prerequisite
