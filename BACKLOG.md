# Backlog: Syllogistic Type System

The structural typing substrate + language-model-based emit
transition is complete. `emit_identity_type` is deleted. The
identity ratchet is at 4 types (Any, Json, Record, Unit — all
intentionally opaque).

This backlog tracks follow-up debt and future capabilities.

---

## Structural Authority Cleanup

Concrete debt that should be addressed to close the gap between
"emit transition landed" and "the compiler fully lives on structural
types end to end."

### Production emit uses merged registry

`lower_to_*()` currently constructs `TypeRegistry::with_core_types()`
and passes it to the `_with_registry` variant. This is better than
`None` but is not project-aware — DSL-defined structural types are
not visible. The target is threading
`CompileOutput::merged_type_registry()` through the codegen pipeline.

**Files:** `daglang-driver/src/pipeline.rs`, `daglang-emit/src/lower_rust.rs`,
`lower_go.rs`, `lower_c.rs`
**Effort:** Small (plumbing change, callers already accept `Option<&TypeRegistry>`)

### Unknown named types error at emit

`resolve_and_emit("FooBar", ...)` currently returns `"FooBar"` with
a warning. Tests still bless this as acceptable
(`assert_eq!(resolve_and_emit("FooBar", None, Rust), "FooBar")`).
This should be a compile error — emitting unresolved type names into
generated code is the nominal fallback the migration is eliminating.

**Files:** `type_mapping.rs` (resolve_and_emit fallback), tests
**Effort:** Small (change warning to error, update tests to expect failure)

### Document lossy backend container rendering as intentional

The C language model renders `Map<K,V>` as `{V}*`, discarding the
key type. This is correct for C (no native generic map syntax). The
Go model renders maps as `map[{K}]{V}` (preserves both). This
asymmetry should be documented as an intentional backend limitation
in the language model, not left as an implicit gap that gets
reopened every time C emission is reviewed.

**Files:** `language_model.rs` (C_CONTAINERS comment)
**Effort:** Trivial (documentation)

---

## Invariant / Consistency

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

Likely answer: all four stay permanently opaque (they represent
concepts where internal structure is meaningless to backends).
Document the decision.

---

## Future Capabilities

### Language model serialization to JSON IR

Move language model data from static Rust arrays to JSON files
(`backend/rust.ir.json`, etc.). Benefit: modifying backend type
mappings doesn't require recompiling the compiler.

**Effort:** Medium

### `behavior` DSL construct + property-based test generation

New syntax for algebraic law declarations:
```dag
behavior PartialOrder extends Preorder {
  law antisymmetric: leq(a, b), leq(b, a) implies a == b
}
```

Auto-generate property-based tests from `law` declarations.

**Effort:** Large

### Real `containers.dag` with generic substitution

The parser handles `type List<T>` syntax but `T` is never
substituted into the body. Making containers fully DSL-defined
would satisfy the "adding a type requires only a `.dag` definition"
invariant.

**Effort:** Large

### Structural coercion paths

Multi-step coercion via derivation chain walking. Downcast
acknowledgment syntax. Current `is_compatible()` handles all
existing use cases.

**Effort:** Large
