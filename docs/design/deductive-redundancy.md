# C22: Deductive Redundancy Elimination (DRE)

> Status: Design · Replaces naive `validate_no_operation_overlap`

## Problem

The naive overlap checker compared raw `OperationKey`s (e.g., `cargo.Build.Clippy`)
across the freshness sub-DAG and tool DAG. This caused false positives for
legitimate repeated operations (two `github.Issues.Get` calls with different IDs)
and required special-casing for singleton operations like `cargo.Build`.

## Core Primitive: Idempotency Fingerprints

An operation's **identity** is not just its `OperationKey` — it's the combination of:

1. The `OperationKey` (e.g., `cloud.aws.s3.PutObject`)
2. The evaluated values of all declared `idempotency_keys`

The `idempotency_keys` come from `OperationBehavior` domain models. For example:

- **S3 PutObject**: identity = `(bucket, key)` — the content payload is the *effect*, not the identity
- **github.Issues.Get**: identity = `(id)` — two calls with different IDs are distinct operations
- **cargo.Build**: identity = `()` (singleton) — `idempotency_keys: null` means the fingerprint is just `[cargo.Build]`

A `StaticFingerprint` is a deterministic combination of:

```
StaticFingerprint {
    operation: OperationKey,
    keys: Vec<(String, Provenance)>,  // idempotency key → provenance
}
```

Where `Provenance` tracks where the value comes from:

```
enum Provenance {
    Literal(Value),           // path = Literal("Makefile")
    Edge(NodeId, PortName),   // id = Edge("list_issues", "id")
    Dynamic,                  // computed at runtime, cannot deduplicate statically
}
```

## Phase 1: Compile-Time Static Fingerprinting

During the lowering phase (`daglang-lower`), the compiler generates a
`StaticFingerprint` for every transport node based on the provenance of its
idempotency keys.

### Validation Rules

If the compiler finds two nodes with the exact same `StaticFingerprint`:

| Behavior | Action |
|----------|--------|
| `WritesState` | **Compile Error**: "Definitive conflict — writing to `bucket='artifacts', key='build.tar'` twice without causal ordering." |
| `ReadOnly` | **Compile Error**: "Redundant operation — calling `github.Issues.Get` with the same upstream ID twice. Bind the first result and reuse." |

Singleton operations (`idempotency_keys: null`) have fingerprint `[OperationKey]`,
which naturally handles the CI freshness overlap problem without special-cased rules.

## Phase 2: Test-Time Dynamic Trace Validation

The compiler cannot statically detect redundancies when inputs are computed
dynamically (string interpolation, `for` loops with duplicate IDs, etc.).

The hermetic test layer (`gunbc-test`) catches these at runtime:

1. **Execution Ledger**: During mock execution, the runtime interceptor records
   the actual evaluated `Value`s for every node's idempotency keys.

2. **Dynamic Rule**: The test runner asserts that an
   `(OperationKey, Hash(actual_idempotency_values))` tuple is never executed
   more than once per workflow run.

3. **Failure**: `"Runtime Redundancy Detected: 'github.Issues.Get' was executed
   twice with identical arguments: {\"id\": 123}. Refactor your DAG to reuse
   the output or dedup the input list."`

## Escape Hatch

Operations with `Determinism::NonDeterministic` or `EventuallyConsistent`
(e.g., polling endpoints) automatically bypass redundant-read checks, because
the system knows the output is expected to change over time.

## Implementation Plan

### Phase 1 (Compile-time)

1. Add `idempotency_keys: Option<Vec<String>>` to `OperationBehavior`
2. Populate for existing service models (cargo, github, gcp, filesystem)
3. Generate `StaticFingerprint` during lowering
4. Validate fingerprint uniqueness per DAG with behavioral dispatch

### Phase 2 (Test-time)

1. Add execution ledger to `gunbc-test` mock interceptor
2. Record `(OperationKey, Hash(values))` tuples during mock execution
3. Assert uniqueness at end of test run (respecting `NonDeterministic` escape)

## Relationship to Previous Work

- **Replaces**: `validate_no_operation_overlap` (removed — was checking raw
  `OperationKey` without considering input identity)
- **Builds on**: `OperationBehavior` domain models from Lane B
- **Extends**: Hermetic test infrastructure in `gunbc-test`
