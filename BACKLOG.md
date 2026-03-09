# Backlog: Syllogistic Type System

Non-blocking items remaining after the structural type migration.
The core migration is complete — `emit_identity_type` is deleted,
all type emission flows through structural `TypeShape` resolution
via the language model, and the identity ratchet is at 4 types
(Any, Json, Record, Unit — all intentionally opaque).

These items improve architectural purity, add new capabilities, or
tighten invariants. None are required for correct operation.

---

## Invariant / consistency

### Port cardinality derivation

`Port.cardinality` is stored directly on the port struct. Many code
paths (especially `lower_to_ir.rs`) hardcode `Cardinality::Scalar`.
The type DAG already knows if something is a `List` or `Optional`,
so cardinality is redundant stored state. Deriving it would
eliminate a dual-source-of-truth bug class.

**Files:** `dag.rs`, `lower_to_ir.rs`, `plan.rs`
**Effort:** Medium (many Port construction sites to audit)

### Fail-loud Stage 2: Result-returning resolve_field_type_dag

Stage 1 (warnings) is done. Stage 2 would make
`resolve_field_type_dag` return `Result<Dag<TypeOp>, String>`,
propagating compile errors for genuinely missing types. Requires
updating all callers through `register_type_def` and
`collect_dsl_type_registry`.

**Files:** `daglang-typecheck/src/lib.rs` + callers
**Effort:** Medium (function signature chain change)

### Opaque type decisions

4 identity types remain: `Any`, `Json`, `Record`, `Unit`.
- `Any`: top type — opaque by definition
- `Json`: dynamic/untyped — opaque by nature
- `Unit`: zero-element type — could be a zero-field product
- `Record`: anonymous product alias — could be `Product(None, [])`

Decision needed: which are permanently opaque vs. structural.
Likely answer: all four stay opaque (they represent concepts
where internal structure is meaningless to backends).

---

## Maintainability

### Language model serialization to JSON IR

Move language model data from static Rust arrays to JSON files
(`backend/rust.ir.json`, etc.). Benefit: modifying backend type
mappings doesn't require recompiling the compiler. Matters more
with many backends; fine for 3 backends that change rarely.

**Files:** `language_model.rs` + new JSON files
**Effort:** Medium (serde infrastructure)

---

## New capabilities

### `behavior` DSL construct + property-based test generation

New syntax for algebraic law declarations:
```dag
behavior PartialOrder extends Preorder {
  law antisymmetric: leq(a, b), leq(b, a) implies a == b
}
```

Types acquire behaviors via `implements` (already parsed for
services/resources). Auto-generate property-based tests from `law`
declarations.

**Files:** parser, typecheck, testgen
**Effort:** Large (new syntax + semantics + test generation)

### Real `containers.dag` with generic substitution

The parser handles `type List<T>` syntax but `T` is never
substituted into the body. Container behavior lives in Rust-side
`type_lib` constructors. Making containers fully DSL-defined would
satisfy the "adding a type requires only a `.dag` definition"
invariant, but containers are stable infrastructure that won't
change.

**Files:** `daglang-typecheck/src/lib.rs`, `dsl/std/containers.dag`
**Effort:** Large (generic substitution semantics)

### Structural coercion paths

Multi-step coercion via derivation chain walking (e.g., `UInt8 →
Int32` via width widening). Downcast acknowledgment syntax
(`coerce X -> Y where lossy(...)`). Current `is_compatible()` with
structural shape comparison + predicate entailment handles all
existing use cases.

**Files:** `type_registry.rs`, `daglang-syntax`
**Effort:** Large (derivation chain walking, brand policy, new syntax)
