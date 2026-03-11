# Sustainability Ledger

The governing metric for this codebase is **cost of change**: when the
language grows by one type, one expression form, or one transport, how
many files need editing? The sustainable compiler is one where that
number is 1.

Fixing individual symptoms is itself unsustainable if they share a root
cause. This ledger is organized as a **causal tree**: deeper roots
subsume shallower ones. Fixing a root eliminates its entire subtree.

See `src/README.md` for the invariants and `POSTMORTEM.md` FC-7 for
the history.

---

## Deep root: Incomplete compile-time resolution

The compiler doesn't fully resolve information at compile time. Types
are referenced by string names (`TypeId`) instead of embedded structure.
Classification uses string matching instead of structural queries.
Rust code redeclares facts the DSL already defines because it can't
read compiled DSL output.

Every "root cause" below is a facet of this:
- **TypeId** = type information not resolved into structure
- **String enumeration** = type/operation properties not resolved into structure
- **DSL/Rust duplication** = DSL compilation output not accessible to Rust code

And the reason these persist: **no boundary contracts** between pipeline
stages catch the unresolved information, so fabrication fallbacks mask
it indefinitely.

**Terminal state:** The compiler fully resolves all references at
compile time. Ports embed type structure, not string names.
Classification derives from structure, not name patterns. Rust code
reads from DSL-compiled artifacts, not parallel declarations. Each
pipeline boundary validates that its output is fully resolved.

---

### Branch 1: TypeId is deferred computation

`Port.type_id` is a string (`TypeId("ResourceHandle")`) that requires
a `TypeRegistry` lookup to resolve into structural information. The
registry is a symbol table that should have been resolved at compile
time but persists into execution.

The type structure exists in the .dag file, gets compiled into a
`Dag<TypeOp>`, gets stored in the registry under a string key, and
then ports reference the string key. Three representations of one fact.

**Terminal state:** Ports reference type structure directly (embedded
SubDag or DAG-internal node reference). The compiler resolves all type
references at compile time. No registry at runtime. TypeId becomes
unnecessary. (TypeId appears in 19 files today.)

#### Symptoms:

**S1: `register_core_types()` duplicates .dag definitions.**
Cost: 2 places per type. Already diverged (Credential).
`type_registry.rs:338` + `dsl/std/types.dag`.

**S2: `mock_element_expr` / `try_mock_element_value` enumerate types.**
Cost: 2 match arms per type, ~240 lines total.
`codegen.rs:~6971,~7270`. Exists because codegen doesn't have registry
access — if types were structural, no lookup needed.

**S4: `port.cardinality` caches type information.**
Cost: low (builder auto-derives). Exists because cardinality is derived
from TypeId via registry instead of being intrinsic to the type DAG
embedded in the port.

**S5: Emit pipeline uses core-only registry.**
Cost: DSL types invisible to emit → raw string fallback.
`daglang-driver/src/pipeline.rs`, `daglang-emit/src/type_mapping.rs`.

**S13: Semantic carrier classification by string match.**
Cost: 1 match arm per semantic type, 50+ arms.
`ir/src/types.rs:~992`. Exists because the classifier reads
`TypeId` strings instead of structural properties.

---

### Branch 2: Parallel implementations

When the same computation exists in two forms, they diverge as the
language evolves. The one that lags gets masked by a fallback.

**Terminal state:** Each computation has exactly one implementation.
Fn body evaluation either works completely or doesn't exist for DryRun.

#### Symptoms:

**S3: FnBodyEvalCallableOp catch-all passthrough.**
The fn body evaluator and the DAG executor are parallel implementations.
The evaluator can't handle all expression forms, so eval errors are
silently caught. Cost: evaluator can never visibly regress.
`resolve.rs:~245`.
Fix options: (A) complete evaluator, (B) remove from DryRun,
(C) structured eval contract — evaluator declares what it supports,
unsupported forms are compile-time opaque instead of runtime catch-all.

**S16: Service transport dispatch duplicated across prepare/parse.**
`GenericPrepareOp` and `GenericParseOp` each match on 5 transport
variants calling parallel per-transport functions.
Cost: 2 match arms + 2 functions per new transport.
Fix: transport trait with `prepare()` and `parse()` methods.

---

### Branch 3: Open-set enumeration by string

When behavior varies by type/variant/category, code uses match arms
on string names instead of structural walks. Every new case requires
updating every match, and the compiler can't tell you which you missed.

**Terminal state:** Classification derived from DAG structure or DSL
annotations. String matching only for closed sets (compiler-enforced
enum variants).

#### Symptoms:

**S11: Collection operations require 5 edits each.**
`builtin_method_sigs` + `collection_op_kind()` + `CollectionKind::from_name()`
+ `collection_emit_family()` + emit implementation.
Fix: single `CollectionOp` registry.

**S12: Builtin type metadata split across two functions.**
`builtin_type_names()` + `builtin_type_arities()` — parallel lists.
Cost: 2 edits per builtin type.
Fix: single `BuiltinType` struct.

**S14: Obligation classification by name prefix.**
`infer_fn_obligation()` matches on "render_", "load_", "fs_env_".
Fix: DSL annotations or structural output-type inspection.

**S15: Filesystem resource hardcoded by type name.**
"FilesystemHandle" appears ~10 times in `resolve.rs`.
Cost: duplicate entire wiring function per new resource type.
Fix: derive from DSL resource definitions.

**S17: Mock wrong-type matrix enumerated by hand.**
`mock_wrong_type_expr()` — 20 match arms.
Fix: derive from `ValueBacking` (3 rules replace 20 arms).

---

### Branch 4: DSL/Rust boundary duplication

When a concept is defined in both the DSL and Rust, the two
representations diverge. The DSL definition should be authoritative
for all type-level facts.

**Terminal state:** Rust code reads from DSL-compiled artifacts.
No parallel Rust definitions of facts the DSL already encodes.

#### Symptoms:

**S6: ResourceHandle shape in DSL vs Rust `Into<Value>`.**
Cost: 2 places per field change. Already diverged (missing `type` field).
Fix: generate `Into<Value>` from DSL type definition.

**S10: Container types are compiler built-ins.**
`List<T>`, `Optional<T>` hardcoded in `type_lib` as Rust functions.
Cost: 4+ places per new container type.
Fix: DSL-definable containers with generic substitution (large).

---

### Branch 5: No boundary contracts (enables all other branches)

Each stage (parse → typecheck → lower → resolve → execute) passes
complex IR types but the receiving stage's preconditions are implicit.
When a precondition is violated, the receiving stage compensates with
a fabrication fallback instead of failing. Every fallback in FC-7
traces to this: the producing stage didn't guarantee the invariant,
and no boundary validation caught it.

**Terminal state:** Each boundary has a validation function that
checks stage N's output against stage N+1's preconditions. Violations
are compile errors, not runtime degradation.

#### Symptoms:

**S18: No validation that port TypeIds resolve after lowering.**
A port can reference a type string that no registry contains. The
executor hits this at runtime and fabricates (Json fallback, mock
fallback, etc.) instead of the lowerer catching it.

**S19: No validation that Map key types are String after typecheck.**
Non-String map keys are accepted structurally but fail at runtime
because `Value::Map` is string-keyed. (Fixed in this PR for typecheck,
but no cross-boundary test existed to catch it.)

**S20: No validation that non-empty list ports have edges after resolve.**
A `[1,∞)` port with zero incoming edges silently gets `[]` at
execution time. (Fixed in this PR with `allows_empty()` guard, but
no test existed to catch the regression.)

---

### Standalone items

These don't share a root cause with others:

**S7: `resolve_field_type_dag` swallows errors.**
Returns placeholder instead of compile error for missing types.
Fix: return `Result`, propagate through typecheck.

**S8: C backend discards Map key types silently.**
Intentional but undocumented. Fix: document.

**S9: Opaque kernel types lack documented rationale.**
`Any`, `Json`, `Record`, `Unit` — correct decision, undocumented.
Fix: document.

---

## Capabilities that would eliminate branches

Each eliminates a class of liability:

**Language model serialization to JSON IR** — eliminates Rust-code-edit
for backend type mappings (data, not logic).

**`behavior` DSL construct** — eliminates hand-written test enumeration
for algebraic properties.

**Structural coercion paths** — eliminates manual `is_compatible` case
enumeration.

---

## Resolved

| ID | Description | Resolution | Date |
|----|-------------|------------|------|
| R1 | `PortMultiplicity` duplicated `Cardinality.is_list()` | Deleted entirely | 2026-03-10 |
| R2 | `base_type()` partial view (no Wrap/Brand recursion) | Structural walk | 2026-03-10 |
| R3 | `resolve_type_checked()` parser gate on registry | Direct lookup first | 2026-03-10 |
| R4 | `scalar_witness_for_base` fabrication | Returns `None` | 2026-03-10 |
| R5 | `is_placeholder_witness` / `is_compatible(Unknown)` | Deleted | 2026-03-10 |
| R6 | Callable port cardinality wrong for fn params | Always scalar | 2026-03-10 |
