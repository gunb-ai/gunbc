# H4 Design: Loop Extra Inputs Passthrough

## Problem

Current loop lowering assumes body input is only the iterated element. Real workflows need additional context (config, auth mode, branch, thresholds) for each iteration.

## Decision

Support explicit passthrough context for loop bodies while preserving current element semantics.

## Proposed DSL Surface

```text
for item in items with {repo, branch, policy} {
  body(item, repo, branch, policy)
}
```

## Lowering Rules

- Element input keeps existing loop semantics.
- Passthrough inputs are wired to every body invocation.
- Passthrough cardinality must satisfy body input cardinality.
- Conflicts between element and passthrough names are compile-time errors.

## Invariants

- Passthrough ports are read-only within loop body wiring.
- No implicit capture: only names listed in `with { ... }` are forwarded.
- Deterministic port ordering for stable codegen.

## Migration Plan

1. Parse `with { ... }` loop clause.
2. Extend loop pattern IR with passthrough port list.
3. Update lowerers and runtime loop expansion.
4. Add compatibility tests for existing loops (no `with` clause).

## Follow-up Implementation Tasks

- `H4.1` Parser support for loop passthrough clause.
- `H4.2` IR extension for passthrough port declarations.
- `H4.3` Lowering/runtime wiring for passthrough values.
- `H4.4` Name-collision and cardinality validation rules.
- `H4.5` Backward-compatibility and snapshot tests.
