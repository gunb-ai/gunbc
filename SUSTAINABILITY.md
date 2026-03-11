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

**S21: Workflow claim handle type derived from claim-name prefix.**
`workflow/src/process_registry.rs:159-170` maps `"file:"`, `"tool:"`,
`"ledger:"`, `"network:"` to handle type strings in
`claim_handle_type_id()`.
Cost: 1 prefix arm + tests per new claim namespace / handle type.
Fix: derive handle type from the claim/resource definition, not the
claim ID prefix.

**S22: Service transport resolution classified by module/name prefixes.**
`resolve.rs:1025-1037` routes modules by `"services."` / `"workspace."`
/ `"extdeps."`, and `resolve.rs:1111-1116` parses transport role from
`"service_transport::{prepare,execute,parse}::"` name prefixes.
Cost: new service namespace or transport role requires resolver edits.
Fix: lower transport role/service-ness into explicit metadata or enum
tags and dispatch on that.

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

**S23: `value_compatible_with_type_id()` fails open for unknown types.**
`ir/src/types.rs:1188-1192` converts an unknown `TypeId` into
`ValueBacking::Json`, so compatibility checks accept any value instead
of failing.
Cost: generated tests and mock validation can silently bless
unregistered or misspelled types.
Fix: return `Err` / fail closed when `value_backing_for_type_id()`
misses.

**S24: Testgen boundary analysis swallows lowering failure.**
`codegen/src/testgen/codegen.rs:4829-4831` does
`gunbc_exec::lower(self.dag).ok()` and falls back to the unlowered
analysis when lowering fails.
Cost: boundary-mockability tests can be generated against the wrong
graph shape, hiding lowering regressions instead of surfacing them.
Fix: propagate the lower error and fail generation.

**S25: Virtual backend transport classification defaults unknown nodes
to REST.**
`codegen/src/testgen/registry_gen.rs:42-59` classifies by request /
response type strings, and `registry_gen.rs:57-58,160-165` defaults
missing/unknown cases to `Rest`.
Cost: new transport classes silently get REST stubs and partial test
coverage instead of a compile-time error.
Fix: require stamped `transport_class` metadata or return `Err` for
unknown transport executors.

**S30: Testgen re-derives type info by parsing TypeId strings.**
`auto_mock.rs:39-120` has `parse_unary_generic_type_id()`,
`optional_inner_type_id()`, `parse_container_alias_inner()` — all
parsing `<` and `>` in TypeId strings to extract container/element
types. This information is already structurally encoded in the type DAG.
Fix: query the type DAG structure (wrapper kind, inner SubDag) instead
of parsing the string name.

**S31: Lowerer doesn't stamp OperationKey on all transport nodes.**
`auto_mock.rs:285-326` falls back from `node.operation_key` to
`infer_provider_from_node_id()` which string-matches node IDs for
"github", "gcp", "anthropic", etc. The lowerer should stamp every
transport node with typed `OperationKey` metadata, eliminating the
string heuristic.

**S32: Executor auto-mocks missing resource/env inputs in lenient mode.**
`execute/mod.rs:46-51` documents that missing inputs get default mocks
in lenient mode instead of failing. The lowerer should validate all
required inputs have incoming data edges; execution should not
auto-fabricate values.

**S33: Coercion exists because lowerer doesn't validate cardinality
compatibility.**
The entire coercion module (`coerce.rs`) exists because the executor
must wrap/unwrap values to match cardinality mismatches (WrapScalar,
OptionalToList) at runtime. The lowerer should validate cardinality
compatibility at compile time and insert explicit coercion nodes where
needed, rather than having the executor compensate silently.

---

### Branch 6: Split metadata authorities

Metadata families are still encoded as multiple hand-maintained Rust
entrypoints instead of one authoritative registry. Adding a provider or
metadata variant means editing every projection of the same fact.

**Terminal state:** Each metadata family has one authoritative registry.
Lookup, listing, scope helpers, and codegen derive from that source.

#### Symptoms:

**S26: LLM provider metadata split across constructors, lookup, and
helper lists.**
`ir/src/transport/llm/provider.rs:59-102` defines provider factories,
`provider_by_id()`, and `builtin_provider_ids()` as separate copies of
the built-in provider set, while `ir/src/transport/llm/mod.rs:97-105`
duplicates the same IDs in `LlmScopeContract::{openai, anthropic}`.
Cost: 4 edits per built-in provider, with drift possible between lookup,
listing, and scope-contract helpers.
Fix: single static provider registry; derive lookup, built-in IDs, and
scope helpers from it.

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

### Review-identified items (2026-03-10)

These came from PR review and represent concrete next-fix priorities:

**S34: Lowerer fails open on callable return wiring.**
`let _ = wire_callable_return_outputs(...)` with a TODO about making
it hard-fail later. Real wiring bugs in patterns/transport-backed
callables disappear silently.
Fix: return `LowerError::WiringFailure` (same as argument wiring
already does). This is the sharp next lowerer fix.

**S35: `value_compatible_with_type_id` erases unknown types to Json.**
(Same underlying issue as S23, surfaced from a different entry point.)
`ir/src/types.rs` still converts unknown-type errors into
`ValueBacking::Json` "for backwards compat," meaning compatibility
checks accept any value for unregistered types instead of failing.
Fix: fail closed — unknown types are incompatible with everything.

**S36: gunbc-interp not authoritative in production.**
The interpreter exists but isn't the production execution path yet.
This is a parallel implementation (Branch 2) that will drift.
Fix: either make it authoritative or delete it.

**S37: Builtin/intrinsic metadata duplicated across typecheck and eval.**
`builtin_method_sigs` in typecheck and the intrinsic dispatch table
in `daglang-eval` are parallel lists of the same operations.
Fix: single intrinsic registry consumed by both.

**S38: Emitted code never compiled or run downstream.**
Emit tests only check emitted text as strings (`.contains("fn main")`).
The Go, C, and MIPS emitters could produce broken output and no test
catches it. Even the Rust emitter's output is only verified by string
checks, not by `cargo build` or `cargo test` on the emitted code.
Fix: compiler emits tests alongside code; acceptance = emitted tests
pass in the target language.

**S39: Tests weakened during syntax migration.**
Several test fixtures were flattened to constants (`return "done"`,
`passed = 0`) and assertions reduced from semantic checks to "compiles
successfully" or "function name appears in output." This lowers
confidence that the refactor preserved behavior, not just buildability.
Fix: restore semantic assertions (collection/manifest checks, output
value verification).

---

## Prioritized sequence

Based on review feedback, ordered by signal value (which fix
prevents the most silent regressions):

1. **Constrain fn-body-eval passthrough (S3).** Remove or sharply
   scope the catch-all. Option C (structured eval contract) is
   preferred: evaluator declares capabilities, unsupported forms
   are compile-time opaque. Eval errors from supported forms are
   real failures.

2. **Fail-closed callable return wiring (S34).** Turn
   `let _ = wire_callable_return_outputs(...)` into structured
   failure once the lowerer can trace service result bindings.

3. **Finish fail-closed type cleanup (S23, S35).** Make
   `value_compatible_with_type_id` and `value_backing_for_type_id`
   fail closed for unknown types. No more Json-as-top-type escape.

4. **Restore test semantic strength (S38).** Re-add collection/
   manifest/output-value assertions that were flattened during
   the syntax migration.

5. **Map<K,V> typecheck diagnostic (S19).** Emit an actual
   diagnostic for non-String map keys instead of silently falling
   through. (Partially addressed — typecheck now rejects, but no
   diagnostic message is emitted to the user.)

---

## Investigation notes (2026-03-11)

### S34: callable return wiring — not a 1-line fix

Attempted to replace `let _ = wire_callable_return_outputs(...)` with `?`
propagation. Two blockers:

1. **Service call result bindings** (`let resp = fs.read(...);
   return { body: resp.body }`). The lowerer's `lower_expr` can't
   resolve `resp` because `collect_local_let_bindings` excludes
   `Expr::ServiceCall` bindings (line 12267). The ident falls through
   to "unresolved ident" error. Failing tests:
   `compile_*_uses_bound_resource_capability_call_succeeds`.

2. **Collection ops in return position** (`return { readable:
   map(partitioned.readable, e => e.path) }`). `lower_expr` handles
   `Expr::Call` only for endpoint-registered callables, not intrinsic
   collection operations. Failing tests: `compile_data_from_module_*`
   (via `dsl/std/patterns.dag::classify_files`).

**Prerequisite** for hard-fail: expand `lower_expr` to trace service
call result bindings (through parse transport node outputs) and to
handle collection intrinsic calls in return position. Both are lowerer
improvements, not wiring policy changes. The `func`/`pattern` items
that hit this path don't have `fn_body` — they rely entirely on
`__out:` passthrough wiring, which is the designed fallback.

Note: a warning-based approach was considered but rejected per
`src/README.md` "No fallbacks that fabricate" invariant.

### S23/S35: fail-closed types — deeper than expected

Attempted to change `value_compatible_with_type_id` from
`.unwrap_or(ValueBacking::Json)` to `.unwrap_or(false)` (unknown types
incompatible with everything). This is correct per the invariant, but
the blast radius is large:

1. **36 mock type mismatches** in auto-testgen for bootstrap. All are
   DSL-defined types (`Line`, `Frame`, `Tier`, `SemanticColor`,
   `Milliseconds`, `Char`, `SymbolId`, `GitignoreCategory`, etc.) not
   in the core-only `TypeRegistry` used by `value_backing_for_type_id`.

2. **`ResourceHandle`** not registered in core type registry despite
   being used as a TypeId on 9 port definitions. Quick fix: add to
   the handle-type check at `types.rs:1182`.

3. **Registry-level gap**: `TypeRegistry::value_backing()` returns
   `Err` for DSL record types and sum types even when they ARE
   registered — the registry has their type DAGs but can't determine
   runtime backing from `TypeOp` structure. Records back to `Map`,
   sum types back to `String`/`Enum`, branded types (`Milliseconds`,
   `Char`) back to their wrapped primitive — none of this is derivable
   from the current type DAG representation.

4. **`Bool?` syntax**: `optional_inner_type_id` handles `Optional<T>`
   and `OptionalT` but not `T?` — the DSL shorthand isn't normalized
   before reaching the compatibility check.

**Fix sequence** (each enables the next):
- (a) Teach `value_backing()` to classify registered-but-unclassifiable
  types: record → Map, sum → String. Requires either a `TypeKind`
  discriminant on registry entries or structural inspection of the type
  DAG.
- (b) Add `value_compatible_with_registry(type_id, value, &registry)`
  variant so callers with DSL types (testgen) can use a full registry.
  Callers without (executor mock validation) continue using core-only
  and fail closed.
- (c) Flip `value_compatible_with_type_id` to `.unwrap_or(false)`.
- (d) Update `test_gen.rs` mock generation to panic on unknown types.

Steps (b)-(d) are mechanical once (a) is done.

---

## v2 self-hosted compiler: impact on ledger

v2 (see `docs/design/v2-self-hosted-compiler.md`) eliminates the deep
root by design: types are TypeExpr values, not strings. No TypeRegistry.
No deferred resolution. No parallel interpreter.

**Eliminated by v2 design (no v1 fix needed):**
S1, S2, S3, S4, S5, S13, S36, S37 — and all of Branch 1.

**Inherited by v2 (re-implement correctly):**
S34 (callable wiring) — v2 lowerer writes fail-closed from scratch.
S38 (emitted code untested) — v2 emits tests alongside code.

**Already fixed in v1:**
S19 (Map keys), S20 (non-empty list default), S38 (partial — smoke tests).

**Remains as v1 maintenance only:**
S39 (weakened tests) — v1 tests, v2 writes its own.
S3 prioritized sequence items — all moot once v2 is primary.

---

## Capabilities that would eliminate branches

**v2 self-hosted compiler** — eliminates Branches 1, 2, 4 entirely.
Reduces Branch 3 (string enumeration moves to .dag data declarations).
See `docs/design/v2-project-plan.md`.

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
