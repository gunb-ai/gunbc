# Sustainability Ledger

The governing metric for this codebase is **cost of change**: when the
language grows by one type, one expression form, or one transport, how
many files need editing? The sustainable compiler is one where that
number is 1.

Fixing individual symptoms is itself unsustainable if they share a root
cause. This ledger is organized as a **causal tree**: deeper roots
subsume shallower ones. Fixing a root eliminates its entire subtree.

See `src/v1/README.md` for the invariants.

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

`Port.type_id` is a string that requires a `TypeRegistry` lookup to
resolve into structural information. Three representations of one fact.

**Terminal state:** Ports reference type structure directly. No registry
at runtime. TypeId becomes unnecessary. (TypeId appears in 19 files.)

#### Symptoms:

**S1:** `register_core_types()` duplicates .dag definitions. 2 places per type.
**S2:** `mock_element_expr` / `try_mock_element_value` enumerate types. ~240 lines.
**S4:** `port.cardinality` caches type info derivable from the type DAG.
**S5:** Emit pipeline uses core-only registry; DSL types invisible → string fallback.
**S13:** Semantic carrier classification by string match. 50+ match arms.

---

### Branch 2: Parallel implementations

When the same computation exists in two forms, they diverge. The one
that lags gets masked by a fallback.

**Terminal state:** Each computation has exactly one implementation.

#### Symptoms:

**S3:** FnBodyEvalCallableOp catch-all passthrough. Eval errors silently
caught → evaluator can never visibly regress. Fix: structured eval
contract (option C) — evaluator declares capabilities, unsupported
forms are compile-time opaque.

~~**S16:** Service transport dispatch duplicated across prepare/parse.~~
**DONE.** `TransportTripletSpec` + `build_transport_triplet()` in `transport.rs`. 4 duplicated sites → 1.

**S42:** Provider classification duplicated across `operation_key` and
`node_id` paths. Fix: stamp `MockProvider` at lower time.

**S43:** Kitchen-sink REST response fabricates all-provider fields.
Fix: require provider classification (S42); provider-specific mocks.

---

### Branch 3: Open-set enumeration by string

Code uses match arms on string names instead of structural walks.
Every new case requires updating every match.

**Terminal state:** Classification derived from DAG structure or DSL
annotations. String matching only for closed sets.

#### Symptoms:

~~**S11:** Collection ops require 5 edits each.~~ **DONE.** `CollectionKind` registry in `ir/patterns/collection.rs`.
~~**S12:** Builtin type metadata split across two functions.~~ **DONE.** `BuiltinType` struct + 58-entry `BUILTIN_TYPES` registry in `types.rs`.
~~**S14:** Obligation classification by name prefix.~~ **DONE.** Structural output-type classification in lowerer.
~~**S15:** Filesystem resource hardcoded by type name.~~ **DONE.** `FILESYSTEM_HANDLE_TYPE` constant + `is_filesystem_resource_port()`.
~~**S17:** Mock wrong-type matrix — 20 match arms.~~ **DONE.** `ValueBacking::mock_value()` replaces match matrix.
**S21:** Workflow claim handle type from claim-name prefix. Fix: derive from definition.
~~**S22:** Service transport resolution by module/name prefixes.~~ **DONE.** `TransportObligation` structural dispatch in resolver.
~~**S44:** Shell output parsing inferred from field shape.~~ **DONE.** `output_parsing` DSL annotation + parser support.
~~**S45:** Response provider inferred from service name substrings.~~ **DONE.** `response_provider` DSL annotation + `Node.response_provider` field.
~~**S46:** Transport kind inferred from operation name substrings.~~ **DONE.** Already structural (no substring inference existed).
**S47:** Container type classification duplicated across emit functions. Fix: `ContainerKind` enum at typecheck.
~~**S48:** Test class / fermi cost parsed from strings in proc macros.~~ **DONE.** Fermi enum variants in testgen.
~~**S49:** Test output matcher classified by type name string.~~ **DONE.** `ValueBacking::output_matcher_path()`.

---

### Branch 4: DSL/Rust boundary duplication

When a concept is defined in both the DSL and Rust, the two diverge.

**Terminal state:** Rust code reads from DSL-compiled artifacts.

#### Symptoms:

**S6:** ResourceHandle shape in DSL vs Rust `Into<Value>`. Already diverged.
**S10:** Container types are compiler built-ins. 4+ places per new container.

---

### Branch 5: Permissive boundary types (enables all other branches)

Each stage passes IR types that can represent invalid states.
Downstream stages compensate with fabrication fallbacks instead of
failing, because the type doesn't prevent the invalid state.

**Terminal state:** Each boundary uses a type that makes the invalid
state unrepresentable. No validation walks needed — the Rust compiler
enforces the contract. When you want to add a boundary check, instead
refactor the upstream output type so the check is unnecessary.

#### Symptoms:

~~**S18:** No validation that port TypeIds resolve after lowering.~~
**DONE.** `validate_port_type_ids()` implemented with builtin/generic fallbacks. Advisory
mode — not wired into pipeline until DSL types propagate to IR TypeRegistry.
**S19:** No validation that Map key types are String after typecheck. (Fixed.)
**S20:** No validation that non-empty list ports have edges after resolve. (Fixed.)
**S23/S35:** `value_compatible_with_type_id` fails open for unknown types →
Json-as-top-type escape. Fix sequence: (a) teach `value_backing()` to
classify record→Map, sum→String; (b) add registry-aware variant for
testgen; (c) flip to `.unwrap_or(false)`; (d) panic on unknown in mock gen.
**S24:** Testgen swallows lowering failure via `.ok()`. Fix: propagate error.
**S25:** Virtual backend defaults unknown transport to REST. Fix: require stamped metadata.
**S30:** Testgen re-derives type info by parsing TypeId strings. Fix: query type DAG.
~~**S31:** Lowerer doesn't stamp OperationKey on all transport nodes.~~
**DONE.** All 12 transport nodes (prepare+execute+parse × 4 sites) now stamped.
~~**S32:** Executor auto-mocks missing inputs in lenient mode.~~
**DONE.** `DryRunStrictness::Strict` (default) / `Lenient` on `ExecuteConfig`.
~~**S33:** Coercion exists because lowerer doesn't validate cardinality compatibility.~~
**DONE.** `CardinalityIncompatibility` validation via `validate_coercions()`, wired into `verify_dag()`.
**S7:** `resolve_field_type_dag` swallows errors → `identity()` placeholder cascades
through `value_backing()` → Json → any value accepted → wrong emit → no test coverage.
Fix: return `Result`; make `_checked()` the only path.
**S34:** Lowerer fails open on callable return wiring (`let _ = wire_callable_return_outputs`).
Prereqs: `lower_expr` must trace service call result bindings and handle collection
intrinsic calls in return position.
~~**S40:** Optional type syntax not normalized.~~ **DONE.** `normalize_optional_type_id()` canonicalizes `T?`/`OptionalT`/`Optional<T>`.
~~**S41:** Cardinality-based mock fabrication hides empty-list cases.~~ **DONE.** `ValueBacking::mock_value()` returns empty list for List backing.
**S62:** `[when]` guards on `func` body service calls not lowered into DAG IR.
Guards are silently dropped, making conditional service calls execute unconditionally.
Discovered via IAM preflight incident (2026-03-09); affected code deleted.
**S64:** `lower_expr().ok()` — 9 sites in lowerer silently discard lowering errors,
producing partial DAGs with unwired edges and no diagnostic. Affected: service call
arguments, conditional synthesis, match dispatch, list construction, string interpolation.
Fix: propagate `Result` through all `lower_expr` call sites.
**S65:** `std::env::var()` in lowerer — `resolve_profile_config_expr` performs
environment I/O during a pure lowering pass (`lib.rs:1155`). Fix: separate profile
config resolution step that produces resolved values as lowerer inputs.
~~**S66:** `gunbc-resolve` depends on `daglang-driver`.~~ **DONE.** `daglang-driver` made
optional via `compile` Cargo feature. Pure resolution API (`resolve_compiled_dsl`) has
no driver dependency. Compilation convenience API gated behind `features = ["compile"]`.
**S67:** ~~`ValueType::Unknown` sentinel~~ **DONE.** `Unknown` variant renamed to
`Inferred` with documented semantics ("valid type exists but inference cannot name it").
`display_name()` returns `String` (not `Option`); `is_inferred()` guard replaces all
`matches!(_, Unknown)` checks. `TypecheckWarning::InferredType` added for soft failures.
Match and if/else arms now propagate concrete types when all branches agree.
**S68:** Computation classification duplicated in typecheck and emit. Both
`daglang-typecheck` (`classify_computation`, `classify_fn_body`) and `daglang-emit`
(`computation.rs`) derive node classification from `LoweredOp`. Fix: stamp
`Computation` on nodes at lowering time; consumers read, don't re-derive.
**S69:** `EmitCollectionFamily` enum lives in `daglang-syntax` AST (`lib.rs:608`),
coupling the parser to codegen concerns. Fix: move to emit or lower crate.
**S70:** ~~Emit input types too permissive~~ — DONE. `classify_for_emit()`
walks the DAG once; `EmitInput` / `EmitCallable` / `EmitTransport` /
`EmitPrimitive` / `EmitCollection` / `EmitPipeline` structs replace raw
variant matching. Legacy API wraps new classified path.
**S71:** Thread-local `TmpCounter` in emit (`fn_codegen.rs:72–96`) makes temp name
generation non-deterministic. Fix: explicit counter passed through emission functions.

---

### Branch 6: Split metadata authorities

**Terminal state:** Each metadata family has one authoritative registry.

~~**S26:** LLM provider metadata split across constructors, lookup, and helper
lists.~~ **DONE.** Consolidated static registry in `transport/llm/provider.rs`.

---

### Standalone items

**S8:** C backend discards Map key types. Intentional (C has no native map).
**S9:** Opaque kernel types (Unit/Json/Any/Record) lack documented rationale.
**S63:** `std/patterns.dag` mixes generic patterns with GCP-specific auth
implementations. Provider-specific code belongs under `extdeps/`, not `std/`.

---

### Branch 7: Untyped runtime for a typed compiler

`daglang-eval` was built for simple fn bodies. The v2 self-hosted compiler
uses it as a general-purpose interpreter for 80+ mutually-recursive functions,
deep self-recursion, and multi-stage pipeline contracts.

The evaluator provides none of the properties a compiler runtime needs:
- **Type safety**: `Value::Unit` flows where `Value::Map` (Module) is
  expected with no error until downstream fails (S57).
- **Stack safety**: recursive DSL functions consume native Rust stack
  frames, with only partial self-recursive TCO (S55).
- **Error contracts**: parse failures return `none`, which leaks through
  untyped stage boundaries into downstream stages (S56).

These are not independent bugs — they cascade. S55 (TCO bug) moved the
frontier forward. S56 (error laundering) made a parse failure look like
a resolve crash. S57 (no runtime type enforcement) is the architectural
reason laundering was even possible. Every pipeline boundary where the
v1 evaluator passes unvalidated values is a potential laundering site.

This is the bootstrap problem. The v2 compiler cannot typecheck itself
until it achieves self-hosting.

**Terminal state:** Self-hosting. Compiled code eliminates stack issues,
static types prevent laundering, typed Result returns replace none-leaking
optional fields.

**Interim mitigations:** Explicit error checks between pipeline stages
(S56 fix). Shape validation at stage boundaries. TCO for self-recursive
tail calls including `return` in non-tail blocks (S55 fix). Module
extraction deferred until after error gates in `compile_sources`.

#### Evaluator fixes (2026-03-11):

**S58:** `data_values` deep-cloned per call. Fix: `Rc<HashMap>`. (was mislabeled S40)
**S59:** No recursion depth limit → segfault. Fix: `call_depth` counter, limit 10,000.
**S60:** O(N²) list/string accumulation via clone+extend. Fix: owned-value concat.
**S61:** No TCO for self-recursive functions. Fix: trampoline with `tail_ctx`.

#### Discovered during debugging (2026-03-11):

**S50:** Block-scope `return` didn't exit enclosing function — parallel
implementation of statement loop diverged. Fix: `is_fn_body` parameter.
**S51:** `cmp_op` errored on Unit instead of returning false.
Fix: `Bool(false)` when either side is Unit.
**S52:** Parser mutual recursion not covered by TCO. Not a correctness
issue — tests use `with_parser_stack(16MB)`. Bounded by AST depth, not
input length.
**S53:** Data value JSON round-trip loses `Value::Enum` → `Value::Str`.
Fix: store `Value` directly, eliminate JSON intermediary.
**S54:** Emitter adds service params to signatures but not call sites.
Fix: module-level callee→item-kind map consulted by `emit_call`.
**S55:** `return self_call()` in non-tail blocks didn't trampoline —
conflated block tail position with function return position. Fix: remove
`allow_tco` guard on `Return` handler.
**S56:** Parse errors laundered as resolve crashes — test helper extracted
`parse_result.module` without checking error. Fix: fail on error before
module extraction; validate shape.
~~**S57:** No runtime type enforcement at DSL function boundaries.~~
**DONE.** `check_call_inputs()` / `check_return_value()` in `eval_stack.rs` validate
parameter and return types via `value_compatible_with_type_id`. Thread-local
`TYPE_WARNINGS` collection (advisory). `LoweredFnBody` carries `param_types` +
`return_type` metadata from AST.

#### Remaining performance debt (not crash risk):

`Env::from_inputs` clones inputs on every non-self call. Partially
addressed (`Env.bindings` is `Rc<HashMap>` with COW). Map field
flattening clones every field. Fix: `Rc`/COW for `Value`.

#### Stack depth tradeoff: explicit eval stack vs self-hosting

The v1 evaluator amplifies stack usage ~8-16x because each DSL function
call pushes ~1.6KB of native Rust frames (eval_fn_body → eval_stmts →
eval_expr → eval_call tower). A compiled parser function costs ~100-200
bytes. This is why the interpreter needs 16-32MB stack for files that a
native binary would handle in <1MB.

**Key property:** stack usage is bounded by AST nesting depth (grammar-
bounded), NOT by file size. A 170KB file with flat definitions uses the
same stack as the current 17KB one.

**Option A: Explicit evaluation stack.** Replace recursive eval dispatch
with a `Vec<Frame>` on the heap. Eliminates stack amplification, makes
mutual recursion free, removes `with_parser_stack` scaffolding. Cost:
medium-large refactor of eval.rs (2-3 sessions). Becomes dead code after
self-hosting.

**Option B: Proceed to self-hosting.** The v2 pipeline is 100% implemented
(7 .dag files, 7,197 lines, 55/56 tests passing; 1 `#[ignore]` on
full pipeline stack overflow at scale — eliminated by self-hosting).
Remaining work:
Phase 1 (emit per-module Rust + driver → first native binary, 3-5 sessions),
Phase 2 (progressive self-compilation M1-M9, 5-10 sessions),
Phase 3 (fixed-point verification, 2-3 sessions). Total: ~12-20 sessions.

**Decision:** if current 16-32MB stack sizes hold until self-hosting,
Option B is preferred — the eval.rs refactor has zero residual value.
If stack sizes need bumping again, Option A becomes justified as an
interim fix.

---

## Prioritized sequence

Ordered by signal value (which fix prevents the most silent regressions):

1. **Constrain fn-body-eval passthrough (S3).** Structured eval contract:
   evaluator declares capabilities, unsupported forms are compile-time opaque.
2. **Fail-closed callable return wiring (S34).** Requires lowerer improvements
   (service call result bindings + collection intrinsics in return position).
3. **Fail-closed types (S23/S35).** Teach `value_backing()` to classify
   DSL types, then flip to `.unwrap_or(false)`.
4. **Restore test semantic strength (S38/S39).** Re-add collection/manifest/
   output-value assertions.
5. **Map<K,V> typecheck diagnostic (S19).** Partially addressed.

---

## v2 self-hosted compiler: impact on ledger

v2 eliminates the deep root by design: types are TypeExpr values, not
strings. No TypeRegistry. No deferred resolution. No parallel interpreter.

**Eliminated by v2 (no v1 fix needed):**
All of Branch 1 (S1, S2, S4, S5, S13). Branch 2 parallels (S3, S36, S37).
Branch 5 type issues (S7, S40). Branch 2 provider issues (S42, S43).
Branch 3 container classification (S47).

**Inherited by v2 (re-implement correctly):**
S34 (callable wiring), S38 (emitted code untested), S44 (shell output
parsing annotation), S45/S46 (provider/transport metadata stamping).

**Already fixed in v1:** S19, S20, S38 (partial).

**v1 maintenance only:** S39, S48, S49, Branch 7 evaluator fixes.

**Capabilities that would further reduce debt:**
- Language model serialization to JSON IR (backend type mappings as data)
- `behavior` DSL construct (algebraic property test enumeration)
- Structural coercion paths (eliminate `is_compatible` case enumeration)

---

## v1 health guards

Ratchets that freeze known debt so it can't worsen while v2 progresses.

| Guard | Symptom | What it catches |
|-------|---------|-----------------|
| `#[must_use]` on `wire_callable_return_outputs()` | S34 | Ignored `Result` at new call sites |
| `ratchet_fail_open_types` | S23/S35 | New DSL types on ports without `ValueBacking` |
| `ratchet_identity_types_in_core_registry` | pre-existing | New identity/opaque types in registry |
| fidelity classification tests | S3 | `evaluate_fn_body()` regressions |

All ratchets are one-way (lists can only shrink).

---

## Resolved: Eval stack machine — continuation ordering during suspension bubbling (S52-EVAL)

**Root cause (found):** Continuation stack ordering was reversed during
suspension bubbling. When a sibling call suspends inside a block/match-arm:

1. `eval_block_s` pushes an **inner** continuation (block's remaining stmts)
2. `eval_stmts` catches `ExprResult::Suspend` and pushes an **outer**
   continuation (function's remaining stmts)
3. The outer continuation ends up **on top** of the stack (pushed last)
4. `pop_stack` pops top-first → resumes the outer context before the inner
5. The inner block's `Return`/`EarlyReturn` is never processed — execution
   falls through to wrong code (typically the function's fallback path)

**Concrete symptom:** `scan_token("f")` should hit the `is_ident_start`
branch and return via `scan_ident`. Instead, `pop_stack` resumed with
`scan_token`'s remaining 23 stmts (skipping the block's Return), falling
through to `match lookup(single_punct, ...)` → `Unknown { char: "f" }`.
This cascaded: broken tokens → parser sees wrong kinds → returns Unit.

**Fix applied:** Before calling `eval_expr_s`, record `stack_base =
stack.len()`. On `Suspend`, use `stack.insert(stack_base, cont)` instead
of `stack.push(cont)`. This places the outer continuation **below** any
inner continuations pushed by `eval_expr_s`, so `pop_stack` processes
inner (block) continuations first. Applied at 4 bubble-up sites:
`eval_stmts` (3 Suspend handlers) and `eval_block_s` (1 Suspend handler).

**Result:** 54/58 v2 tests pass (up from 41). Remaining 3 are Phase 5
tests that OOM on large source files (memory scaling, not correctness).
1 test intentionally ignored (needs self-hosting).

### Refactor note: shared-stack mutation violates design invariants

The `stack.insert(stack_base, ...)` fix is correct and tested, but it
is a positional compensation on shared mutable state. The design doc
(`DESIGN-eval-redesign.md` lines 18-21) specifies:

> **Clear interfaces.** Return values, not mutated shared state.

and the Step::Call type (line 128) carries `cont: Option<Continuation>`,
meaning the main loop should be the only code that touches the stack.

**Current violation:** `eval_expr_s` and `eval_block_s` mutate
`&mut Vec<Continuation>` as a side effect. Multiple layers push in an
order determined by call sequence, and `stack.insert` compensates.

**Preferred approach for a follow-up PR:**

`eval_expr_s` and `eval_block_s` should stop mutating the stack.
Instead, return inner continuations with the Suspend variant:

```rust
enum ExprResult {
    Value(Value),
    EarlyReturn(HashMap<String, Value>),
    Suspend {
        callee: FnId,
        inputs: HashMap<String, Value>,
        inner_conts: Vec<Continuation<'a>>,
    },
    Error(String),
}
```

Then `eval_stmts` pushes outer first, inner on top — correct by
construction, no positional tricks. This aligns with the design doc's
"return values, not mutated shared state" principle and eliminates the
class of ordering bugs entirely.

**Status:** Done. Public API switched to `evaluate_stack` (step 4),
old recursive evaluator deleted (step 5). The bubble-up sites are
exactly the 5 points that call `eval_expr_s` with a subsequent push
(4 in `eval_stmts` + 1 in `eval_block_s` for non-last Expr).

### S67 — Remaining eval_stack invariant violations

**File:** `daglang-eval/src/eval_stack.rs`

The stack machine works correctly for all tested workloads but has
structural issues that could cause subtle bugs as the DSL grows:

1. **`eval_expr` is not pure.** It handles `LoweredExpr::Call` (via
   `eval_non_sibling_call_raw`, which can re-entrantly call
   `evaluate_stack`), `LoweredExpr::Return` (produces early_return
   error signal), and `LoweredExpr::Block` with Return stmts. The
   doc claims "pure expression → Value" but calls and returns are
   control flow, not values.

2. **Two block evaluation paths.** `eval_block_s` (suspendable) and
   `eval_expr`'s Block arm (pure) evaluate blocks with different
   semantics. The pure path can't suspend on sibling calls; it
   handles them via re-entrant `evaluate_stack`.

3. **Two match evaluation paths.** `eval_match_s` (suspendable, used
   by `eval_expr_s`) and `eval_match_local` (pure, used by
   `eval_expr`). Guards in both use `eval_expr`, not `eval_expr_s`
   — this is correct because the continuation model can't represent
   guard truthiness checks, but it means guard evaluation uses
   native recursion for sibling calls.

4. **`wrap_value_as_output` flattens Maps.** When a fn's trailing
   expression is a Map, it becomes the output HashMap directly. The
   caller can't distinguish "returned a Map" from "returned multiple
   named fields." The `"value"` fallback in `extract_projection` is
   a partial repair for one case of this.

5. **`EvalError` conflates errors and control flow.** The
   `early_return` field uses the error path for non-error semantics.
   Return signals propagate as `Err(EvalError { early_return: Some(...) })`.

6. **`eval_call_args` and `eval_non_sibling_call_raw` drop positional
   args.** When `param_name` is `None`, the evaluated value is
   discarded. Intrinsic fn-reference paths create synthetic calls
   with `(None, Ident(...))` positional args that hit this.

7. **No tail continuation elimination.** Tail calls push identity
   continuations (`remaining: &[], binding: None`). Deep mutual
   recursion at N=40K allocates 40K continuations on the heap.

**When to fix:** Items 1-3 are the Phase 6 IR cleanup (move Return,
Block, For to statement forms). Items 4-5 are architectural. Item 6
is a latent bug (fn-reference callbacks in intrinsics don't pass
args). Item 7 is the main contributor to gist pipeline OOM.

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
| S11 | Collection ops require 5 edits each | `CollectionKind` registry: `emit_family()`, `typecheck_contract()`, `from_name_or_alias()`, `is_eval_intrinsic()` centralized in `ir/src/patterns/collection.rs`. Consumers delegate. | 2026-03-13 |
| S14 | Obligation classification by name prefix | Structural: output type shape (`Handle`/`Env` -> ResourceProvide, single String -> PureRender). `infer_fn_obligation` no longer reads `name`. | 2026-03-13 |
| S15 | Filesystem resource hardcoded by type name | `FILESYSTEM_HANDLE_TYPE` constant + `is_filesystem_resource_port()` helper in `ir/src/resource/mod.rs`. Resolve.rs migrated. | 2026-03-13 |
| S12 | Builtin type metadata split across functions | `BuiltinType` struct + 58-entry `BUILTIN_TYPES` registry in `types.rs` | 2026-03-13 |
| S16 | Transport dispatch duplicated | `TransportTripletSpec` + `build_transport_triplet()` in `transport.rs` | 2026-03-13 |
| S17 | Mock wrong-type 20-arm matrix | `ValueBacking::mock_value()` + `output_matcher_path()` | 2026-03-13 |
| S18 | No port TypeId validation after lowering | `validate_port_type_ids()` with builtin/generic fallbacks (advisory) | 2026-03-13 |
| S22 | Transport resolution by name prefix | `TransportObligation` structural dispatch in resolver | 2026-03-13 |
| S26 | LLM provider metadata split | Consolidated static registry in `transport/llm/provider.rs` | 2026-03-13 |
| S31 | OperationKey missing on transport nodes | All 12 nodes (prepare+execute+parse × 4 sites) stamped | 2026-03-13 |
| S32 | Executor auto-mocks missing inputs | `DryRunStrictness::Strict` (default) / `Lenient` on `ExecuteConfig` | 2026-03-13 |
| S33 | No cardinality validation on edges | `CardinalityIncompatibility` via `validate_coercions()`, in `verify_dag()` | 2026-03-13 |
| S40 | Optional type syntax not normalized | `normalize_optional_type_id()` for T?/OptionalT/Optional\<T\> | 2026-03-13 |
| S41 | Cardinality mock hides empty-list | `ValueBacking::mock_value()` returns empty list for List backing | 2026-03-13 |
| S44 | Shell output parsing by field shape | `output_parsing` DSL annotation + parser/lowerer support | 2026-03-13 |
| S45 | Response provider by name substring | `response_provider` DSL annotation + `Node.response_provider` field | 2026-03-13 |
| S46 | Transport class by operation substring | Already structural (no substring inference existed) | 2026-03-13 |
| S48 | Test class/fermi parsed from strings | Fermi enum variants in testgen | 2026-03-13 |
| S49 | Output matcher by type name string | `ValueBacking::output_matcher_path()` | 2026-03-13 |
| S57 | No runtime type enforcement at DSL boundaries | `check_call_inputs()`/`check_return_value()` + thread-local warnings | 2026-03-13 |
| S66 | gunbc-resolve depends on daglang-driver | `daglang-driver` optional via `compile` feature flag | 2026-03-13 |
| S67 | `ValueType::Unknown` sentinel | Renamed to `Inferred`, `TypecheckWarning` added, all sites updated | 2026-03-13 |
| S70 | Emit input types too permissive | `EmitInput` + `classify_for_emit()` + classified emitter variants | 2026-03-13 |
| Eval-8 | `LoweredExpr::Return` is an expression | Removed from `LoweredExpr`; return is statement-only | 2026-03-13 |
