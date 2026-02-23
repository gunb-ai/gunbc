# M12: Coercion Proof Nodes

**Status**: Design
**Lane**: B (Workflow Execution Safety)
**Depends on**: M11

## Problem

Shape coercions (scalar→list, list→scalar, optional unwrap) execute silently.
If a coercion produces the wrong shape, downstream nodes may fail with
confusing errors far from the actual problem.

## Design

### 1. Shape assertion nodes

Add `TypeOp::AssertShape(ShapeContract)` variant:

```rust
pub struct ShapeContract {
    pub expected_kind: ValueKind,
    pub expected_cardinality: Option<Cardinality>,
    pub description: String,
}
```

The assertion node checks:
- `value.kind()` matches `expected_kind`
- For list values, length satisfies cardinality constraint
- On failure, produces a localized diagnostic

### 2. Insertion points

Coercion proof nodes are inserted automatically after:
- `TypeOp::Transform(Coercion::ScalarToList)` — assert output is list
- `TypeOp::Unwrap(WrapperKind::Optional)` — assert output is non-optional
- `TypeOp::Unwrap(WrapperKind::List)` — assert output satisfies inner type

### 3. Generated coercion tests

Testgen emits per-coercion tests that:
1. Provide a valid input
2. Run through the coercion
3. Assert the proof node passes
4. Provide an invalid input
5. Assert the proof node fails with expected diagnostic

## Files

- `core/ir/src/type_op.rs` — AssertShape variant
- `core/ir/src/contract.rs` — ShapeContract type
- `core/exec/src/execute.rs` — assertion execution
- `core/codegen/src/testgen/` — coercion test generation

## References

- `core/ir/src/value.rs` — ValueKind enum
- `core/ir/src/contract.rs` — existing contract tower
