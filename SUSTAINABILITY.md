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

**S42: Provider classification duplicated across operation_key and
node_id paths.**
`auto_mock.rs:296-309` (`operation_key_to_mock_provider`) and
`auto_mock.rs:313-326` (`infer_provider_from_node_id`) are parallel
implementations of the same classification: service name → MockProvider.
The operation_key path uses `starts_with("github")|"gcp"`, the node_id
fallback uses `contains("gist")|"anthropic"` with overlapping patterns.
Cost: 2 arms per new provider across 2 functions.
Fix: stamp `MockProvider` on transport nodes at lower time; delete
both inference functions.

**S43: Kitchen-sink REST response fabricates all-provider fields.**
`auto_mock.rs:338-376`. `default_rest_response()` produces a merged
JSON object with fields from ALL providers (OAuth `access_token`,
GitHub `html_url`, GCP `raw`, etc.) when provider can't be inferred.
Cost: response grows with each provider; parse tests can pass on
wrong-provider fields.
Fix: require provider classification (S42); generate provider-specific
mock responses.

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

**S44: Shell output parsing inferred from field shape.**
`daglang-lower/src/lib.rs:6939-6961`. `infer_shell_output_parsing()`
classifies by field count + names: 3 fields with success/stdout/stderr
→ SuccessStdoutStderr; 1 bool → ExitCodeBool; any list field →
SplitLines; everything else → TrimStdout (default).
Cost: new output shapes silently degrade to TrimStdout.
Fix: explicit `@output_parsing` annotation on service operations.

**S45: Response provider inferred from service name substrings.**
`daglang-lower/src/lib.rs:6484-6497`. `infer_response_provider()`
uses `starts_with("github.")|contains("gist")` for GitHub,
`starts_with("gcp.")|"google."` for GCP, `contains("anthropic")` for
Anthropic. Overlapping patterns — "gist" matches inside unrelated
names.
Cost: 1 conditional arm per provider; overlap bugs.
Fix: stamp `ResponseProvider` on service definitions in DSL.

**S46: Transport kind inferred from operation name substrings.**
`daglang-emit/src/computation.rs:600-630`. `infer_transport_kind()`
uses `name.contains("read")` vs `contains("write")` when structured
`ServiceTransportClass` metadata is absent.
Cost: name-based heuristic breaks for non-read/write operations.
Fix: always stamp `ServiceTransportClass`; remove string fallback.

**S47: Container type classification duplicated across emit functions.**
`daglang-emit/src/type_codegen.rs:57-62` and `:227-236` both match
`name.as_str()` on `"List"|"Set"|"Map"` to classify container kinds.
Cost: 2 match sites per new container type, plus language model
backend mappings.
Fix: single `ContainerKind` enum resolved at typecheck time, not
by string match in emit.

**S48: Test class / fermi cost parsed from strings in proc macros.**
`testgen-registry-macros/src/lib.rs:411-443` (repeated at `:458-490`
for live variants). Parses `class.value().to_lowercase()` matching
`"unit"|"hermetic"|"integration"` and `cost.value().to_uppercase()`
matching `"XS"|"S"|"M"|"L"|"XL"`.
Cost: 2 match sites per new class or cost level (one regular, one
live). Dedicated enums exist in gunbc-test but string parsing
duplicates them.
Fix: use enum variants directly in macro input; parse once.

**S49: Test output matcher classified by type name string.**
`daglang-emit/src/test_mock_emit.rs:356-363`. `match type_name.as_str()`
selects IsString/IsSecret/IsBool/IsInt/NonEmpty matchers; unknown
types fall back to `Any` matcher.
Cost: 1 arm per new value type; unknown types get weakest assertion.
Fix: derive matcher from `ValueBacking` (3 rules, not N arms).

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

**S7: `resolve_field_type_dag` swallows errors.**
`daglang-typecheck/src/lib.rs:1252-1334`. Returns `identity(type_name)`
placeholder instead of compile error for missing types. The `identity()`
DAG is a single-node pass-through that preserves the type name string
but contains no structural information (`type_lib.rs:41-52`).
The same pattern repeats in 7+ locations: `TypeRegistry::register_product()`
(line 483), `register_coproduct()` (line 531), `resolve_expr()` (line
612), and all `_checked()` variants. No caller validates the returned
DAG — all blindly embed it in parent type structures.
**Downstream cascade:** (1) `value_backing()` can't determine runtime
backing from Identity → returns `Err`; (2) `value_compatible_with_type_id`
catches with `.unwrap_or(ValueBacking::Json)` → any value accepted;
(3) emit layer emits type name verbatim → target-language compile error;
(4) testgen returns `None` → zero test coverage for unresolved types.
Fix: return `Result<Dag, TypeResolutionError>`. The `_checked()` API
variants already exist — make them the only path.

**S40: Optional type syntax not normalized at compile boundary.**
`auto_mock.rs:49-65` tries 3 syntaxes for Optional (`T?`, `Optional<T>`,
`OptionalT`). Same 3-way handling in `ir/src/types.rs:975`.
Cost: every consumer re-implements syntax normalization.
Fix: lowerer normalizes all Optional shorthands to `Optional<T>` before
emit; downstream sees one canonical form.

**S41: Cardinality-based mock fabrication hides empty-list cases.**
`auto_mock.rs:238-257`. `default_value_for_port` wraps scalar mock
values into `Value::List(vec![value; cardinality.min.max(1)])`.
When `min=0`, the `max(0,1)=1` silently fabricates a non-empty list,
hiding cases where the port legitimately receives `[]`.
Fix: lowerer should validate cardinality compatibility; mock gen
should respect `min=0` by sometimes producing empty lists.

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

These don't share a root cause with other branches:

**S8: C backend discards Map key types silently.**
`daglang-emit/src/lower_c.rs:652`. `ContainerShape::Map(_, value)` —
key type explicitly discarded with `_`. Intentional: C has no native
map type; maps emit as `T*` (pointer to value). Key handling is in
custom hash table implementations that don't track compile-time key
type. Other backends (Go, Rust) preserve key types via
`type_mapping.rs:79-83`. Harmless because `Value::Map` is always
string-keyed (enforced by typecheck at `lib.rs:1272-1279`).
Fix: add a comment in `lower_c.rs` explaining the design choice.

**S9: Opaque kernel types lack documented rationale.**
`type_registry.rs:263-267` registers `Unit`, `Json`, `Any`, `Record`
as `identity()` DAGs. These are bootstrap placeholders needed before
`.dag` files are processed (documented at `type_registry.rs:257-262`).
After typecheck, `merge_dsl_types()` can overwrite them with structural
definitions. Correct design — each has distinct semantics:
- `Unit` ≡ zero-valued (matches Bool unit variants, NoneType)
- `Json` ≡ universal serializable (escape hatch, accepts any value)
- `Any` ≡ type variable (compatible with everything in checks)
- `Record` ≡ untyped product (fallback for products without structure)
The `Any`/`Json` distinction is enforced in `types.rs:1128-1149` but
nowhere documented. 153 refs to `Any`, 76 to `Json`, 60 to `Unit`,
19 to `Record`.
Fix: add doc comment on `register_kernel_types()` explaining each
type's semantics and when to use which.

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
S1, S2, S3, S4, S5, S7, S13, S36, S37, S40 — and all of Branch 1.
S42, S43 (provider and kitchen-sink mock) — v2 has typed transport metadata.
S47 (container classification) — v2 emitter uses TypeExpr, not strings.

**Inherited by v2 (re-implement correctly):**
S34 (callable wiring) — v2 lowerer writes fail-closed from scratch.
S38 (emitted code untested) — v2 emits tests alongside code.
S44 (shell output parsing) — needs explicit annotation in v2 DSL too.
S45, S46 (provider/transport inference) — v2 should stamp metadata at lower time.

**Already fixed in v1:**
S19 (Map keys), S20 (non-empty list default), S38 (partial — smoke tests).

**Remains as v1 maintenance only:**
S39 (weakened tests) — v1 tests, v2 writes its own.
S3 prioritized sequence items — all moot once v2 is primary.
S48, S49 (test class parsing, output matchers) — v1 testgen-only concerns.

## v1 health guards

Minimum-investment ratchets that freeze known debt so it can't silently
worsen while v2 is in progress. These don't fix symptoms — they prevent
regression.

| Guard | Symptom | Test / mechanism | What it catches |
|-------|---------|------------------|-----------------|
| `#[must_use]` on `wire_callable_return_outputs()` | S34 | `-D warnings` in CI | New call sites that ignore the `Result` |
| `ratchet_fail_open_types` | S23, S35 | `ir/src/types.rs` | New DSL types appearing on ports without `ValueBacking` |
| `ratchet_identity_types_in_core_registry` | (pre-existing) | `ir/src/type_registry.rs` | New identity/opaque types sneaking into the registry |
| fidelity classification tests | S3 | `fidelity.rs` tests | `evaluate_fn_body()` regressions for supported expression forms |

**How the fidelity tests guard S3:** `classify_callable` calls `eval_fn()`
which calls `evaluate_fn_body()` and unwraps. If the evaluator regresses
for any expression form used by `test_policy.dag`, the fidelity tests fail.
No explicit S3 ratchet needed.

**Ratchet direction:** all ratchets are one-way. The fail-open type list
can only shrink (types become resolvable). The identity type list can only
shrink (types get structural DAGs). If either test fails because a type
was *removed* from the registry, that's a real regression worth investigating.

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

### Branch 7: Evaluator assumes small, non-recursive fn bodies

`daglang-eval` was built to evaluate simple DSL fn bodies (a few `let`
bindings, one return expression). The v2 self-hosted compiler pushes it
far beyond that: hundreds of mutually-calling functions, deep
self-recursion (tokenizer per-character, parser per-token), and large
accumulating data structures threaded through every call.

The evaluator had no concept of "this function will call itself 10,000
times" because no prior DSL function did.

**Terminal state:** The evaluator handles recursive DSL programs as
first-class workloads — bounded stack, shared immutable context,
amortized data structure operations.

#### Root causes and fixes (2026-03-11):

**S40: `data_values` deep-cloned on every function call.**
`Env.data_values` was `HashMap<String, serde_json::Value>`, cloned
on every `evaluate_fn_body_with_data` call and every `Env::child()`.
For the v2 tokenizer with 42 keyword/punctuation entries, this was
42 `serde_json::Value` deep-copies per recursive call.
Fix: `Rc<HashMap<...>>` — clone is pointer-sized.

**S41: No recursion depth limit.**
Self-recursive DSL functions consumed native Rust stack frames without
bound. Stack overflow produced a segfault, not a diagnostic.
Fix: `call_depth` counter in `Env`, checked at function entry.
Limit: 10,000. Clean error on violation.

**S42: O(N²) list/string accumulation.**
`eval_binop(Add)` for lists did `a.clone(); result.extend(b)` —
cloning the entire left-hand side. The tokenizer's `tokens + [tok]`
pattern cloned the growing list on every token. N tokens = O(N²)
total allocations.
Fix: `BinOp::Add` arm in `eval_expr_tc` handles list/string concat
with owned values directly (`a.extend(b)`, `a.push_str(&b)`), turning
append from O(N) to O(1) amortized.

**S43: No tail-call optimization for self-recursive functions.**
`tokenize_loop`, `scan_string_body`, `process_escapes_loop`,
`parse_stmts`, `parse_items`, `parse_expr_loop`, `kahn_iterate` — all
self-recursive tail-call functions. Each call grew the native Rust stack
by ~500-1000 bytes (debug mode). Processing a 200-character string via
`scan_string_body` consumed ~200 KB of stack just for that one scan.
Fix: trampoline. `eval_expr_tc` threads a `tail_ctx: bool` through
`IfElse` branches, `Match` arms, `Return`, and `Call`. When a
self-recursive call is detected in tail position, `eval_call_tc`
returns a `tail_call` signal instead of recursing. `eval_fn_body_rc`
catches it in a loop and rebinds inputs. Zero stack growth for
self-recursive tail calls.

#### Discovered during debugging (2026-03-11):

**S50: Block-scope `return` didn't exit enclosing function.**
`evaluate_block_body` was a parallel implementation of `eval_fn_body`'s
statement loop. When unified into `eval_stmts`, the `return` stmt
inside `if cond { return x }` blocks produced `Ok(result)` instead
of `Err(early_return)`. The enclosing function continued executing past
the return point.
Root cause: "no parallel implementations" violation. Two copies of the
same loop diverged.
Fix: `is_fn_body: bool` parameter on `eval_stmts`. Fn body context
produces `Ok` (we ARE the function). Block context produces
`Err(early_return)` (propagate up to enclosing function).

**S51: `cmp_op` errors on Unit instead of returning false.**
`char_at` returns `Unit` for out-of-bounds positions. Downstream
DSL code passes the result to `is_digit(ch)` which does `ch >= "0"`.
The evaluator's `cmp_op` treated `(Unit, Str)` as a type error
instead of a false comparison.
Root cause: no boundary contract between `char_at` (returns
`String | Unit`) and its callers (assume `String`). The evaluator
should handle the `Unit` case gracefully since the DSL has no way to
distinguish `String` and `Unit` at the type level.
Fix: `cmp_op` returns `Bool(false)` when either side is `Unit`.

**S52: Parser mutual recursion not covered by TCO.**
The v2 parser has ~80 mutually-recursive functions (`parse_items` →
`parse_item` → `parse_type_def` → `parse_field_list` → `parse_field`
→ `parse_type_expr` → ...). These are NOT self-recursive, so TCO
doesn't apply. Each call adds ~500 bytes to the native stack in debug
mode. Parsing a module with 10 type definitions reaches ~30 call
frames deep × ~500 bytes = ~15 KB, and the default 8MB test thread
stack overflows when combined with the tokenizer and evaluator frames.
Status: not a correctness issue — tests use `with_parser_stack(16MB)`
helper. The proper fix is mutual TCO (trampoline across function
boundaries), which requires tracking tail position across call sites.
Lower priority because the depth is bounded by AST nesting (not input
length), and the 10,000 depth limit catches runaway cases.

**S53: Data value round-trip through JSON loses type info.**
`compile_tokenizer_module()` evaluates data declarations to `Value`,
converts to `serde_json::Value` via `value_to_json`, stores in
`EmbeddedCompileOutput.data_values`, then the evaluator converts back
via `json_to_eval_value`. The round-trip loses `Value::Enum` → becomes
`Value::Str` (e.g., `KwModule` enum variant → `"KwModule"` string).
For the tokenizer's keyword table this works (the values are just tags),
but for data declarations with nested structure, the lossy round-trip
could cause subtle bugs.
Fix: store `Value` directly in `EmbeddedCompileOutput.data_values`
instead of going through JSON. The JSON intermediary exists because
`evaluate_fn_body_with_data` takes `&HashMap<String, serde_json::Value>`
— changing it to take `&HashMap<String, Value>` would eliminate the
round-trip entirely.

#### Remaining performance debt (not crash risk):

**`Env::from_inputs` clones inputs on every non-self call.** For the
parser, this includes the `ParserState` with its token list. Not a
crash (heap, not stack), but quadratic in call count × state size.
Fix: `Rc`/cow for `Value`. (Partially addressed: `Env.bindings` is
now `Rc<HashMap>` with copy-on-write, eliminating most clones. The
`from_inputs` entry point still clones the caller's HashMap into
a new Rc.)

**Map field flattening clones every field.** `let s = Record {...}`
creates `s__source`, `s__tokens` etc. by cloning. Fix: lazy flattening
or reference-counted fields.

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
