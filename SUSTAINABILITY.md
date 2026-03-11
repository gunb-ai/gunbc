# Sustainability Ledger

Tracks representations that duplicate the same fact, enumeration lists
that grow with the language, and fallbacks that mask divergence. Each
entry is a concrete liability — not a wish list, but a measured cost:
how many files need editing when the language grows by one type, one
expression form, or one transport.

**Governing principle:** The sustainable compiler is one where adding a
new concept means editing one place. Every additional place is a
maintenance multiplier that makes the system exponentially harder to
change.

See `src/README.md` for the invariants (single source of truth,
structural rules over case enumeration) and `POSTMORTEM.md` FC-7 for
the full history of how these were discovered.

---

## Active liabilities

### S1: `register_core_types()` duplicates .dag definitions

**Cost of adding a type:** 2 places (Rust registration + .dag file).
**Risk:** Silent divergence. Credential already diverged (branded String
in Rust vs 6-field product in DSL). `merge_dsl_types` overrides at
compile time, but `TypeRegistry::with_core_types()` callers see the
Rust version.

| File | Role |
|------|------|
| `src/00_foundation/ir/src/type_registry.rs:338` | Rust registration |
| `dsl/std/types.dag` | DSL definition (authoritative) |

**Fix direction:** Delete Rust registrations. Either make `with_core_types()`
compile .dag files, or embed a serialized registry built from .dag at
build time. Interim: add a test that Rust and DSL registrations agree
on structural shape.

### S2: `mock_element_expr` / `try_mock_element_value` enumerate types by name

**Cost of adding a type:** 2 places (match arm in each function), on
top of the .dag definition.
**Risk:** New DSL type gets no mock entry → `None` → test obligation
skipped silently.

| File | Lines | Function |
|------|-------|----------|
| `src/01_surfaces/codegen/src/testgen/codegen.rs:~7270` | ~140 | `mock_element_expr` |
| `src/01_surfaces/codegen/src/testgen/codegen.rs:~6971` | ~100 | `try_mock_element_value` |

**Fix direction:** Thread `TypeRegistry` to the codegen mock emission
path. Replace match arms with `typed_witness_value(type_id, registry)`
lookup. The registry is already available at testgen time — it just
isn't passed to the emitter.

### S3: FnBodyEvalCallableOp catch-all passthrough

**Cost of adding an expression form:** 0 (errors are swallowed), which
is precisely the problem — the evaluator can't regress visibly.
**Risk:** Real evaluation bugs masked. Fn body correctness is untested
in DryRun.

| File | Line | Pattern |
|------|------|---------|
| `src/08_materialize/resolve/src/resolve.rs:~245` | catch-all | All eval errors → passthrough |

**Fix direction:** Architectural decision required.
- **Option A:** Make the evaluator complete. High ongoing cost — every
  new DSL feature needs evaluator support.
- **Option B:** Remove fn body evaluation from DryRun. Test fn body
  results only in Tier 2 (Real mode). The evaluator becomes dead code
  for DryRun; callable nodes are always passthrough in DryRun.
- **Option C:** Structured eval contract. The evaluator declares which
  expression forms it supports. If a fn body uses an unsupported form,
  the node is marked as "eval-opaque" at resolve time (not at runtime
  via catch-all). Errors from supported forms are real failures.

Option C is the sustainable path — it replaces a runtime heuristic
with a compile-time structural declaration.

### S4: `port.cardinality` caches `type_id` information

**Cost:** Low (builder derives it automatically), but it's a second
representation of type-level information.
**Risk:** Low in practice — the builder overrides on every `add_node`.
But code that constructs ports directly (lowerer, tests) can set wrong
cardinality.

| Source of truth | Cache |
|-----------------|-------|
| `TypeId` → `TypeRegistry::infer_cardinality()` | `Port.cardinality` |

**Fix direction:** Evaluate whether consumers can call
`registry.infer_cardinality()` directly instead of reading the cached
field. If the registry is available at all consumption points, the
field can become a derived accessor instead of stored state. Low
priority — the builder derivation prevents most divergence.

### S5: Emit pipeline uses core-only registry (not project-aware)

**Cost:** Emitted code uses `TypeRegistry::with_core_types()` which lacks
DSL-defined structural types. Adding a DSL type doesn't automatically
make it available to the emit layer.
**Risk:** Emit falls back to raw type name strings for unknown types
(e.g. `resolve_and_emit("FooBar", ...) → "FooBar"`). Should be a
compile error.

| File | Issue |
|------|-------|
| `daglang-driver/src/pipeline.rs` | Constructs `with_core_types()` |
| `daglang-emit/src/type_mapping.rs` | `resolve_and_emit` fallback |

**Fix direction:** Thread `CompileOutput::merged_type_registry()` through
the codegen pipeline. Callers already accept `Option<&TypeRegistry>`.
Then change the unknown-type fallback from warning to error.

### S6: ResourceHandle shape in DSL vs Rust `Into<Value>` impl

**Cost of adding a field:** 2 places (DSL type definition + Rust
`Into<Value>` impl in `resource/handle.rs`).
**Risk:** Silent mock/runtime mismatch (already happened — `type` field
was missing from DSL).

| File | Role |
|------|------|
| `dsl/std/resources.dag:19` | DSL definition |
| `src/00_foundation/ir/src/resource/handle.rs:~110` | Rust `Into<Value>` |

**Fix direction:** Generate the `Into<Value>` impl from the DSL type
definition, or have the Rust impl read the DSL-registered type DAG to
determine field names. The DSL definition should be the sole source.

### S7: `resolve_field_type_dag` swallows errors (warning, not error)

**Cost of adding a type dependency:** If a field references an
unregistered type, `resolve_field_type_dag` produces an identity
placeholder with a warning instead of a compile error.
**Risk:** Structural type DAGs contain unresolved placeholders that
propagate silently through the pipeline.

| File | Issue |
|------|-------|
| `daglang-typecheck/src/lib.rs` | `resolve_field_type_dag` returns placeholder |

**Fix direction:** Return `Result<Dag<TypeOp>, String>`. Propagate
through `register_type_def` and `collect_dsl_type_registry`. Missing
types become compile errors.

### S8: C backend discards Map key types silently

**Cost:** None (no new edits needed), but lossy rendering is
undocumented. `Map<K,V>` renders as `{V}*` in C, discarding K.
**Risk:** Low — intentional backend limitation, but undocumented
means it gets re-investigated periodically.

**Fix direction:** Document as intentional in the language model
(`language_model.rs` C_CONTAINERS comment). Not a code change.

### S9: Opaque kernel types lack documented rationale

4 identity types remain: `Any`, `Json`, `Record`, `Unit`. Decision to
keep them opaque is correct but undocumented. Each review cycle
re-examines whether they should be structural.

**Fix direction:** Document the decision. All four represent concepts
where internal structure is meaningless to backends. `Unit` could
theoretically be a zero-field product but gains nothing from it.

### S10: Container types (`List<T>`, `Optional<T>`) are compiler built-ins

The parser handles `type List<T>` syntax but `T` is never substituted
into the body. Containers are hardcoded in `type_lib` as Rust functions,
not defined in .dag files.
**Cost of adding a container type:** Rust code changes in `type_lib`,
parser, typecheck, emit — 4+ places.

**Fix direction:** Make containers DSL-definable with generic
substitution. This is the end-state of "adding a type requires only
a .dag definition." Large effort, but it's the terminal form of S1.

---

## Future capabilities (sustainability-motivated)

These aren't features for their own sake — each one would eliminate
a class of sustainability liability.

### Language model serialization to JSON IR

Move language model data from static Rust arrays to JSON files.
Modifying backend type mappings wouldn't require recompiling the
compiler. Eliminates a Rust-code-edit for what is essentially data.

### `behavior` DSL construct + property-based test generation

Algebraic law declarations (`law antisymmetric: leq(a,b), leq(b,a)
implies a == b`) would auto-generate property-based tests. Eliminates
hand-written test enumeration for algebraic properties.

### Structural coercion paths

Multi-step coercion via derivation chain walking. Would eliminate
manual `is_compatible` case enumeration for complex type relationships.

---

## Resolved

| ID | Description | Resolution | Date |
|----|-------------|------------|------|
| R1 | `PortMultiplicity` duplicated `Cardinality.is_list()` | Deleted `PortMultiplicity` entirely | 2026-03-10 |
| R2 | `base_type()` partial view of type DAG (no Wrap/Brand recursion) | Structural walk through all wrapper layers | 2026-03-10 |
| R3 | `resolve_type_checked()` parser gate on registry access | Direct `get_by_name` before parsing | 2026-03-10 |
| R4 | `scalar_witness_for_base` fabrication for unknown types | Returns `None` (fails explicitly) | 2026-03-10 |
| R5 | `is_placeholder_witness` / `is_compatible(Unknown)` escape hatches | Deleted (dead after R2+R4) | 2026-03-10 |
| R6 | Callable port cardinality inferred from type (wrong for fn params) | Callable ports always scalar | 2026-03-10 |
