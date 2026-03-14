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
**S72 (resolved):** `generated_types_are_not_stale` is `#[ignore]`d.
The anonymous-record naming bug is fixed — `gen-types` now emits
`TypeName { field: val }` for fold initializers via three-tier
inference: exact struct match, best-match by field count, or
synthesized helper struct. Test remains ignored because gen-types
output still has other codegen quality issues that prevent the on-disk
`mod.rs` from being regenerated. Delete when: gen-types output compiles
cleanly and the on-disk file can be regenerated.
**S73:** `cargo check --workspace --all-targets` is not a stable hygiene
ratchet because `gunbc-codegen` declares bins under
`target/codegen/bin/*/main.rs`. Why it persists: those entrypoints are
generated out-of-band, so a fresh checkout cannot lint all targets
until codegen has already run.
~~**S74:** Blanket `#[allow(dead_code)]` suppressions in lowering/emit
hid unused scope-analysis state and MIPS config plumbing.~~ **DONE.**
Removed the repo-local `dead_code` allows; test-only scope helpers now
compile only under `#[cfg(test)]`, and unused carried state was deleted
instead of suppressed.

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
(7 .dag files, 7,268 lines). The anonymous-record naming bug (S72) is
fixed. Assessment of the v1→v2 compilation path (2026-03-13) identified
concrete blockers — see gap analysis below.

**Decision:** Option B is preferred — the eval.rs refactor has zero
residual value. If stack sizes need bumping again before self-hosting
lands, Option A becomes justified as an interim fix.

#### V2 self-hosting gap analysis (2026-03-13)

**What works today:**
- Module discovery: `daglang.toml` multi-root config discovers all 7 v2 modules
- Parsing: all v2 .dag files parse to AST successfully
- Type codegen: `typedef_to_code_ir` handles structs, enums, aliases
- Fn codegen: handles records, match, if/else, for, lambda, string interp,
  intrinsics (map/filter/fold/any/contains/sum/join/last/enumerate)
- Rust rendering: `render_rust` produces valid Rust from code_ir

**Blocker 1 — `build_context` ignores `daglang.toml`:**
`build_context()` hardcodes `dsl/` as the only root. Must call
`resolve_default_roots()` to read `daglang.toml` multi-root config.
Fix is 1 line in `compile/context.rs`.

**Blocker 2 — v1 typechecker rejects v2 idioms (12 errors):**
- TC003 ×7: bare enum variant used where parent type expected
  (`UnterminatedString` vs `StringScanResult`, `Some` vs `Option<T>`)
- TC003 ×2: `Record` literal where typechecker expects `Map`
- TC021 ×3: named arguments not recognized (`ch:` in `code_point()`)

Fix: make typechecker accept variant-as-type in assignment/return
context, and recognize named args in the callable registry.

**Blocker 3 — fn_codegen missing constructs:**
- `with(state, { field: value })` — immutable record update, used
  pervasively for state threading (~100 call sites in v2)
- String builtins: `char_at()`, `substring()`, `string_length()`,
  `lookup()` — need Rust equivalents
- Recursive enum boxing: `Expr` contains `Expr` via variants;
  type codegen must detect cycles and insert `Box<>`
- Optional field patterns: `T?` → `Option<T>`, `Some { value: x }`
  match patterns

**Blocker 4 — end-to-end crate assembly:**
- No harness to emit a complete Cargo crate (Cargo.toml, lib.rs,
  per-module files, main.rs driver)
- Need v2-specific stdlib shims (string ops, list ops, Option helpers)

**Recommended sequence (next PR):**
1. Fix `build_context` to read `daglang.toml` (1 line)
2. Fix 12 typecheck errors (variant-as-type, named args)
3. Add `with()` and string builtins to fn_codegen
4. Add recursive type detection for `Box<>`
5. Build crate assembly harness
6. Iterative: generate crate, compile, fix errors

---

### Branch 8: Type-unaware codegen produces heuristic-dependent output

The v1 fn_codegen pipeline compiles .dag function bodies to Rust without
type information. This forces heuristic workarounds that guess behavior
from naming patterns — exactly the kind of string-based classification
that the sustainability invariants prohibit.

**Root cause:** fn_codegen receives raw AST but no type environment. It
doesn't know field types, return types, or whether a struct field is
recursive (`Box<T>`) or optional (`Option<T>`). Every decision that
requires type information is either wrong or heuristic.

**Terminal state:** Self-hosting. The v2 compiler has a typechecker and
emitter that work on typed IR. Once self-hosting is achieved, the v1
fn_codegen is dead code and all heuristics go with it.

#### S75: `+` operator overloaded (FIXED 2026-03-13)

The .dag language used `+` for arithmetic, string concatenation, and
list concatenation. The codegen had no way to determine which without
type information, producing six heuristic functions (`is_numeric_expr`,
`is_list_field`, `is_list_call`, `is_list_ident`, `is_likely_list_concat`,
`contains_string_literal`) that guessed meaning from field names and
AST structure.

**Fix:** Introduced `concat()` as an explicit intrinsic function for
sequence concatenation. `+` is now exclusively arithmetic. Migrated
~130 call sites across 8 .dag files. Added `Concat` trait to v2_rt
runtime shim. Deleted all six heuristic functions.

**Principle:** Operators must have syntax-local semantics. If you need
type information to determine what an operator does, the operator is
the wrong abstraction — use an explicit function instead.

#### S80: `PR.val: Map` — untyped parse results (FIXED 2026-03-13)

The v2 parser used `type PR { val: Map, state: ParserState, err: Diagnostic? }`
as a universal parse result. The untyped `Map` field forced the v1 codegen to
synthesize anonymous structs with String-defaulted field types, producing ~200
type errors in the generated Rust.

**Root cause:** Untyped dynamic container (`Map`) used where static types are
needed. This is the same pattern as TypeId (Branch 1) — a deferred lookup that
forces downstream code to guess.

**Fix:** Replaced `PR` with ~45 per-category typed result types
(`ExprResult`, `ItemResult`, `NameResult`, etc.). Each type has value fields
directly alongside `state` and `err` (flat, no nesting). Eliminated `ok()`,
`fail()`, `is_err()` helpers. Migrated ~108 function signatures and ~205
construction sites.

**Principle:** Parse results are typed data. Using an untyped container
(Map/HashMap) for typed data forces runtime field lookup instead of
compile-time field access. The extra work of defining ~45 types is repaid
immediately by the compiler catching field access errors statically.

**Impact:** Eliminates ~200 E0308 errors and reduces synthesized anonymous
structs to near zero in parse.rs. Also removes 5 materialized types from
`std_types_prelude()` and their hardcoded `struct_field_types` entries.

#### S81: fn_codegen emits Rust, not code_ir — CRITICAL (OPEN)

fn_codegen.rs was designed to produce target-agnostic `code_ir` that flows
through all backends (Rust, Go, C, MIPS). During v2 bootstrap, it has
accumulated ~15 Rust-specific heuristics that inject target-language
constructs directly into the IR:

- `clone_if_needed()` — Rust ownership (C/Go don't need this)
- `Box::new()` wrapping — Rust heap boxing (C uses pointers, Go implicit)
- `Some()`/`None` injection — Rust `Option<T>` (C uses NULL, Go uses nil)
- `.as_str()` insertion — Rust `String` vs `&str` (no other language has this)
- `..Default::default()` — Rust struct update syntax (no equivalent elsewhere)
- `LazyLock` for Map data — Rust static initialization
- `Deref`/`*` for Box unwrapping — Rust-specific dereference

**Root cause:** fn_codegen has no type information, so it compensates with
heuristics. Those heuristics are Rust-shaped because the only emission
target during bootstrap is Rust.

**Why this matters:** The code_ir layer exists so that one compilation
produces IR that all backends can render. If the IR already contains
`"clone"`, `"Box::new"`, `"Some"`, then the Go renderer must ignore
Rust-specific method calls, the C renderer must translate `Box::new` to
`malloc`, and every new backend must enumerate and strip the Rust idioms.
The IR has become a Rust AST with extra steps.

**Deeper framing:** DAG nodes are **facts** — tautological assertions
about computation (types, cardinality, data flow, semantics). These
facts are target-agnostic truths. **Rendering facts** (how to express
Optional, how to handle ownership, how to name types) are target-specific
decisions that belong in backends, not in the IR.

The heuristics in fn_codegen are **rendering facts mixed into computation
facts**. `clone_if_needed` is a Rust rendering fact ("ownership requires
copying here") injected into the computation DAG. The computation fact is
just "this value is used at these two sites" — the rendering decides what
that means (Rust: clone, C: nothing, Go: nothing, Verilog: fan-out wire).

**Structural test:** Can you swap rendering facts without changing
computation facts? If `Box::new()` is in the code_ir, a C backend must
translate it to `malloc` — the rendering fact leaked into the computation
layer. If the code_ir says "this field is recursive" and the Rust
renderer emits `Box::new()` while the C renderer emits a pointer, the
separation is correct.

**Design guide:** The DAG type system is compositional — similar to
protobuf message stacking. Products, sums, primitives, containers.
These should map naturally across all targets including hardware
description languages (Verilog, PSPICE). If a type construct or IR
node only makes sense for languages with heaps and garbage collectors,
it's a rendering fact masquerading as a computation fact.

**Fix path:** The v2 compiler's emitter reads computation facts (types,
cardinality, recursion, optionality) and applies rendering facts
per-backend. Backends are themselves compositional .dag models — rendering
strategies expressed as facts about how computation concepts map to
target idioms. No heuristics — the types tell the emitter what's true,
the backend decides how to express it.

**Interim principle:** Every Rust-specific construct added to fn_codegen
during bootstrap must be documented here and must NOT be migrated to
the v2 emitter. The v2 emitter should derive these from type information,
not from heuristic pattern matching.

#### S76: `clone_if_needed()` — blind ownership heuristic (OPEN)

fn_codegen adds `.clone()` to all variable/field expressions passed as
function arguments or struct fields. Produces correct but inefficient
generated code. ~300 unnecessary clones in the v2 crate.

**Root cause:** No ownership tracking in the codegen pipeline.

**Fix path:** v2 compiler's emitter should track variable liveness and
emit borrows or moves as appropriate.

#### S77: `infer_struct_name()` — field-name matching (OPEN)

Anonymous records `{ field: value }` in .dag must be mapped to named
Rust structs. The codegen guesses which struct by matching field names
against known struct definitions. This produces wrong matches when
multiple structs share field names (e.g., `BindingPower` vs `Expr::BinOp`
both have `left` and `right` fields).

**Root cause:** .dag allows anonymous records; Rust requires named types.
The codegen has no type context to resolve the intended type.

**Fix path:** Either require named records in .dag source, or carry
return type information from function signatures into the codegen.

#### S78: Materialized types in `std_types_prelude()` (OPEN)

Types imported from `std.types` (`SourceSpan`, `FilePath`) and types
invented for anonymous records (`BindingPower`, `ItemResult`,
`VariantResult`, `CapabilityResult`, `ParamResult`, `OperationResult`)
are hand-written as Rust struct definitions in the v2 crate assembly.

**Root cause:** The v1 emitter doesn't resolve `import` declarations
across modules. Types from external .dag modules aren't available.

**Fix path:** v2 compiler's resolve phase handles cross-module imports.

#### S79: Hardcoded cross-module imports (OPEN)

`module_prelude()` in v2_crate_emit.rs contains hardcoded `use`
statements (`use crate::v2_core::*`, `use crate::tokenize::*`, etc.)
rather than deriving them from `import` declarations in each .dag file.

**Root cause:** Same as S78 — no import resolution in the v1 emitter.

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

**Result (2026-03-11):** 54/58 v2 tests pass (up from 41). Remaining 3 are Phase 5
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

### S67 — Eval stack invariant violations (mostly resolved)

**File:** `daglang-eval/src/eval_stack.rs`

1. ~~**`eval_expr` is not pure.**~~ **Accepted permanent split.** `eval_expr`
   handles non-sibling calls via `eval_non_sibling_call_raw` (re-entrant
   `evaluate_stack`). This is correct: builtins/intrinsics are deterministic
   and don't suspend. Making all calls statement-level would require hoisting
   intrinsic args — not worth the ANF expansion.

2. ~~**Two block evaluation paths.**~~ **Accepted permanent split.**
   `eval_block_s` (suspendable, continuation stack) and `eval_block_pure`
   (pure, used by `eval_match_standalone` and lambda bodies). Both are
   documented with rationale. The pure path exists because standalone match
   evaluation has no sibling fns and no continuation stack.

3. ~~**Two match evaluation paths.**~~ **Accepted permanent split.**
   `eval_match_s` (suspendable) and `eval_match_local` (pure). Guards use
   `eval_expr` because the continuation model can't represent guard
   truthiness checks. Documented as intentional.

4. **`wrap_value_as_output` flattens Maps.** **Accepted permanent.** Map
   flattening is structurally necessary for the v2 evaluation model to
   return multi-field records. The `"value"` fallback in `output_value`
   cannot be removed without either breaking the v2 DSL model or adding
   unreliable signal detection (see DESIGN-eval-redesign.md Limitation 3).

5. ~~**`EvalError` conflates errors and control flow.**~~ **DONE.** `early_return`
   field removed from `EvalError`. `Return` is now a statement-level construct
   handled by `eval_block_s` / `eval_stmts` via `Step::EarlyReturn`.

6. ~~**Positional args dropped.**~~ **DONE.** `eval_call_args` preserves
   positional args as `__pos_0`, `__pos_1`, etc.

7. ~~**No tail continuation elimination.**~~ **DONE.** Full TCO: tail-position
   calls (including inside if/match/block) skip identity continuations.
   `is_tail_position` threaded through `eval_expr_s` / `eval_match_s` /
   `eval_block_s`. Deep mutual recursion at 40K uses O(1) heap for tail
   patterns.

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
| S11 | Collection ops require 5 edits each | `CollectionKind` registry: `emit_family()`, `typecheck_contract()`, `from_name()`, `is_eval_intrinsic()` centralized in `ir/src/patterns/collection.rs`. Former aliases (count/sum/filter_map/sort_by/append) promoted to first-class variants — `alias_contracts()` and `from_name_or_alias()` deleted. | 2026-03-13 |
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
| S67-5 | `EvalError` conflates errors and control flow | `early_return` field removed; `EvalError` is purely an error type | 2026-03-13 |
| S67-6 | Positional args dropped in `eval_call_args` | Preserved as `__pos_0`, `__pos_1`, etc. | 2026-03-13 |
| S67-7 | No tail continuation elimination | Full TCO with `is_tail_position` threading through eval_expr_s/match_s/block_s | 2026-03-13 |
| S74 | Blanket `#[allow(dead_code)]` suppressions | Removed; test-only helpers gated behind `#[cfg(test)]` | 2026-03-13 |
| BUG-6 | Collection aliases mapped to wrong arity | Count/Sum/FilterMap/SortBy/Append promoted to first-class `CollectionKind` variants | 2026-03-13 |
| E3.2 | Transport mock tests tautological | Replaced insert/get/assert with registry construction + entry count + well-formedness checks. `go_test_name` deleted (dead code). TestSpec API (E3.4-E3.6) is the proper replacement. | 2026-03-13 |
| Eval-9 | `extract_projection` indirection | Inlined to direct `output_value()` call; `PopResult::Error` variant removed (dead code) | 2026-03-13 |
| S75 | `+` operator overloaded for arithmetic, string concat, and list concat | Introduced `concat()` intrinsic; `+` is now exclusively arithmetic. Removed all heuristic disambiguation (`is_numeric_expr`, `compile_string_concat`, etc.). ~130 .dag sites migrated. | 2026-03-13 |
| S76 | v2 codegen: `clone_if_needed()` blindly clones arguments | OPEN — workaround for no ownership tracking in fn_codegen. Generates redundant `.clone()` calls. Remove when v2 compiler tracks ownership. | 2026-03-13 |
| S77 | v2 codegen: `infer_struct_name()` guesses struct type from field names | OPEN — workaround for anonymous records in .dag having no explicit type. Wrong inference causes E0308 errors. Remove when .dag records are named or typed. | 2026-03-13 |
| S78 | v2 codegen: hardcoded materialized types in `std_types_prelude()` | OPEN — `SourceSpan`, `BindingPower`, `ItemResult`, etc. manually defined in Rust because v1 emitter can't resolve cross-module imports. Remove when v2 compiler resolves imports. | 2026-03-13 |
| S79 | v2 codegen: hardcoded `module_prelude()` cross-module imports | OPEN — `use crate::v2_core::*` etc. should be derived from .dag `import` declarations. Remove when v2 compiler resolves imports. | 2026-03-13 |

---

## Heuristic elimination roadmap

Each v1 bootstrap heuristic (S81) exists because the codegen lacks
information that the v2 compiler will have. This roadmap maps each
heuristic to the **modeling decision or language feature** that makes
it unnecessary. Ordered by dependency — earlier items unblock later ones.

### Phase A: Type-aware emission (eliminates S76, S77, S78, S81 bulk)

The v2 emitter receives typed IR. Every emission decision that the v1
codegen guesses from naming patterns, the v2 emitter reads from types.

| Heuristic | What v2 emitter does instead |
|-----------|------------------------------|
| `clone_if_needed` (S76) | Emitter tracks variable liveness. Last use = move, earlier use = borrow. Per-backend: Rust emits `.clone()`, C emits nothing, Go emits nothing. |
| `infer_struct_name` (S77) | Typechecker resolves anonymous records to their structural type. Emitter knows the type — no guessing. |
| `Box::new()` wrapping | Emitter checks `TypeExpr` for recursion. Rust emitter emits `Box::new()`. C emitter emits `malloc`. Go emitter emits nothing (implicit). Verilog: N/A (no recursion in hardware types). |
| `Some()`/`None` injection | `TypeExpr::Optional` in the typed IR. Rust emitter emits `Some`/`None`. C emitter emits non-NULL/NULL. Go emitter emits value/nil. |
| `.as_str()` insertion | Rust emitter knows when a `String` is matched and emits `.as_str()`. Other backends don't need this — their string types are uniform. |
| `..Default::default()` | Emitter has all field types. Constructs complete struct literals per-backend. No default-filling needed. |
| `is_option_expr` double-wrap prevention | Typechecker resolves optionality. Emitter knows source and target types — no guessing whether a value is already `Option`. |

**Modeling decision:** No new language features needed. The v2 typechecker
already resolves types to `TypeExpr` values. The emitter reads those
values and makes per-backend decisions.

### Phase B: Import resolution (eliminates S78, S79)

The v2 resolver traces `import` declarations and builds a cross-module
type environment. Each module knows which types are defined locally vs
imported.

| Heuristic | What v2 resolver does instead |
|-----------|-------------------------------|
| `std_types_prelude()` (S78) | Imported types come from the resolved module graph. `SourceSpan`, `BindingPower` are defined in their .dag modules and imported by name. No hand-written Rust definitions. |
| `module_prelude()` (S79) | Per-module `use` statements derived from resolved imports. Each module imports exactly what its `import` declarations specify. |

**Modeling decision:** Already designed. `03_resolve.dag` builds a
`ModuleGraph` with `ResolvedImport` entries. The emitter reads these
to generate correct import statements per-backend.

### Phase C: Variant disambiguation (eliminates ambiguous LitStr/BinOpKind)

Multiple enums share variant names (`LitStr` in both `TokenKind` and
`LiteralValue`, `Add` in both `BinOpKind` and arithmetic). The v1
codegen resolves by "first definition wins" — positional, not structural.

| Decision | Options |
|----------|---------|
| **Option 1: Qualified variants in .dag source** | `LiteralValue.LitStr { value: s }` instead of bare `LitStr`. Requires parser change to support `Type.Variant` syntax. Explicit, no ambiguity, maps to every backend. |
| **Option 2: Typechecker resolves from context** | The typechecker knows the expected type from the field being assigned or the match scrutinee type. Resolves `LitStr` → `LiteralValue::LitStr` when the context expects `LiteralValue`. Already partially implemented in v1 — extend to full coverage in v2. |
| **Option 3: Rename to eliminate overlap** | Rename `LiteralValue::LitStr` to `StrLiteral` etc. Breaks the natural naming. Not recommended. |

**Recommended:** Option 2 for v2 (the typechecker handles it). Option 1
as a language feature for explicitness when disambiguation is needed.

### Phase D: Optionality as a structural property (eliminates double-wrapping)

The `.dag` language uses `T?` for optional types. The v1 codegen doesn't
track which values are optional vs required, so it guesses from field
names, parameter types, and expression shapes.

| Decision | Modeling |
|----------|---------|
| **Optional fields are typed** | `TypeExpr::Optional { inner: T }` is already in the type system. The emitter checks source and target optionality before wrapping. |
| **Optional parameters are typed** | Function params with `T?` produce optional bindings. The typechecker carries this through the body. |
| **Backend rendering** | Rust: `Option<T>`, `Some(v)`, `None`. C: `T*`, `&v`, `NULL`. Go: `*T`, `&v`, `nil`. Verilog: valid bit + data bus. |

**Modeling decision:** Already modeled in the type system. The issue is
purely that the v1 codegen doesn't read the types. v2 does.

### Phase E: Ownership as backend concern, not IR concern (eliminates S76)

The `.dag` language has value semantics — every binding is a value, not
a reference. How that maps to target memory management is a backend
decision:

| Backend | Strategy |
|---------|----------|
| Rust | Ownership analysis: last use = move, earlier = clone or borrow. Emit `.clone()` only when needed. |
| C | Copy semantics for small types, pointer + refcount for large. Or arena allocation. |
| Go | GC handles it. All values are implicitly shared. |
| Verilog | Wire assignment. No memory management. Values are signals. |
| PSPICE | Node voltages. No memory management. Values are circuit parameters. |

**Modeling decision:** The `.dag` language does not expose ownership,
borrowing, or memory management. These are backend concerns. The code_ir
should represent "this value is used here and here" — the backend decides
whether that means clone, borrow, share, or wire.

### Phase F: Static data as language-level constants

`data` definitions in `.dag` (e.g., `data keywords: Map<String, TokenKind>`)
are compile-time constant tables. The v1 codegen emits them differently per
container type (List → `static &[T]`, Map → `LazyLock<HashMap>`) with
Rust-specific patterns.

| Backend | Strategy |
|---------|----------|
| Rust | `static` or `lazy_static` depending on type complexity. |
| C | `const` arrays, or generated init functions for maps. |
| Go | Package-level `var` with init. |
| Verilog | ROM or lookup table. |

**Modeling decision:** `data` is a constant definition. The code_ir should
represent it as `ConstDef { name, type, value }`. Each backend renders
the constant in its idiom. The v1 codegen should not inject `LazyLock` at
the IR level — that belongs in `render_rust.rs`.

### Summary: what dies with self-hosting

When the v2 compiler achieves self-hosting, the following v1 code becomes
dead and should be deleted:

- `fn_codegen.rs` — entire file (replaced by v2 emitter)
- `v2_crate_emit.rs` — entire file (replaced by v2 pipeline)
- `v2_runtime_shim.rs` — entire file (replaced by proper stdlib)
- `synthesize_anonymous_structs()` — entire function
- `clone_if_needed()`, `is_option_expr()`, `is_none_expr()` — all heuristics
- `std_types_prelude()`, `module_prelude()` — all hardcoded scaffolding
- `infer_struct_name()`, `infer_field_type_from_expr()` — all type guessing
- `compile_intrinsic_call()` Rust-specific intrinsic handlers — replaced by
  per-backend intrinsic rendering
