# Sustainability Ledger

The governing metric for this codebase is **cost of change**: when the
language grows by one type, one expression form, or one transport, how
many files need editing? The sustainable compiler is one where that
number is 1.

This ledger tracks open violations. See `src/v1/README.md` for the invariants.

---

## Deep root: Incomplete compile-time resolution

The compiler doesn't fully resolve information at compile time. Types
are referenced by string names (`TypeId`) instead of embedded structure.
Classification uses string matching instead of structural queries.
Rust code redeclares facts the DSL already defines because it can't
read compiled DSL output.

**Terminal state:** The compiler fully resolves all references at
compile time. Ports embed type structure, not string names.
Classification derives from structure, not name patterns. Rust code
reads from DSL-compiled artifacts, not parallel declarations. Each
pipeline boundary validates that its output is fully resolved.

**v2 eliminates the deep root by design:** types are `TypeExpr` values,
not strings. No TypeRegistry. No deferred resolution.

---

## Open findings

### Branch 1: TypeId is deferred computation (eliminated by v2)

`Port.type_id` is a string that requires a `TypeRegistry` lookup to
resolve into structural information. Three representations of one fact.

**Terminal state:** Self-hosting. v2 types are structural `TypeExpr` values.

**S1:** `register_core_types()` duplicates .dag definitions. 2 places per type.
**S2:** `mock_element_expr` / `try_mock_element_value` enumerate types. ~240 lines.
**S4:** `port.cardinality` caches type info derivable from the type DAG.
**S5:** Emit pipeline uses core-only registry; DSL types invisible → string fallback.
**S13:** Semantic carrier classification by string match. 50+ match arms.

### Branch 3: Open-set enumeration by string

**S47:** Container type classification duplicated across emit functions.
Fix: `ContainerKind` enum at typecheck. (Eliminated by v2.)

### Branch 4: DSL/Rust boundary duplication

**S10:** Container types are compiler built-ins. 4+ places per new container.

### Branch 5: Permissive boundary types

**S30:** Testgen re-derives type info by parsing TypeId strings. Fix: query type DAG. (Partially done.)
**S62:** `[when]` guards on `func` body service calls not lowered into DAG IR.
Guards are silently dropped, making conditional service calls execute unconditionally.
Discovered via IAM preflight incident (2026-03-09); affected code deleted.

### Branch 7: Untyped runtime — accepted debt

The v1 evaluator works but is bounded by:
- **S52:** Parser mutual recursion not covered by TCO. — MITIGATED (stacker)
  `stacker::maybe_grow` handles deep recursion automatically. Not a correctness
  issue — bounded by AST depth.
- **Performance:** `Env::from_inputs` clones on every non-self call (partially
  mitigated with `Rc<HashMap>` COW). Map field flattening clones every field.

**Accepted permanent splits** in eval stack machine (all documented with rationale):
- `eval_expr` handles non-sibling calls via re-entrant `evaluate_stack`
- `eval_block_s` (suspendable) / `eval_block_pure` (pure, for standalone match/lambda)
- `eval_match_s` (suspendable) / `eval_match_local` (pure, guards can't use continuations)
- `wrap_value_as_output` Map flattening (structurally necessary for v2 multi-field records)

**Terminal state:** Self-hosting eliminates the evaluator entirely.

### Branch 8: Type-unaware codegen — TERMINAL (dies with self-hosting)

fn_codegen compiles .dag function bodies to Rust without type information.
Every decision requiring type info is heuristic. All findings in this branch
are terminal — v2 replaces v1, eliminating the entire codegen path.

**S81: fn_codegen emits Rust, not code_ir — TERMINAL.** ~15 Rust-specific
heuristics injected directly into IR: `clone_if_needed()`, `Box::new()`,
`Some()`/`None`, `.as_str()`, `..Default::default()`, `LazyLock`, `Deref`/`*`.

The code_ir layer exists so one compilation produces IR all backends can
render. DAG nodes are **facts** — target-agnostic assertions about computation.
**Rendering facts** (ownership, optional representation, type naming) belong
in backends, not IR. The structural test: can you swap the backend without
changing the IR?

**S76:** `clone_if_needed()` — blind ownership heuristic. ~300 unnecessary clones. — TERMINAL
**S77:** `infer_struct_name()` — field-name matching for anonymous records. Wrong on overlapping fields. — TERMINAL
**S78:** Materialized types in `std_types_prelude()` — hand-written because v1 can't resolve cross-module imports. — TERMINAL
**S79:** Hardcoded cross-module imports in `module_prelude()` — should derive from `import` declarations. — TERMINAL

### Standalone

**S8:** C backend discards Map key types. Intentional (C has no native map).
**S38/S39:** Test semantic strength — re-add collection/manifest/output-value assertions. (S38 partial.)
**S72:** `generated_types_are_not_stale` is `#[ignore]`d. Naming bug fixed; test
remains ignored pending full gen-types cleanup.
**S73:** `cargo check --workspace --all-targets` is not a stable hygiene ratchet
because `gunbc-codegen` declares bins under `target/codegen/bin/*/main.rs` (generated out-of-band).

### Branch 10: Item kinds are separate species, not DAG nodes (v2)

The `.dag` language's foundational principle is that all concepts are
expressed as DAG nodes — tautological/syllogistic modeling where every
construct is a node with I/O. A service, a function, a resource, a type
definition — these are all the same species: a named DAG node with inputs,
outputs, and properties. The differences (transport binding, computation
body, capability mode) are properties of the node, not reasons to create
separate compiler machinery.

But the v2 compiler's `Item` type has 8 distinct variants:
`TypeDef`, `FuncDef`, `FnDef`, `ServiceDef`, `ResourceDef`, `DataDef`,
`ExternFuncDecl`, `PatternDef`. Each variant is handled separately in
every compiler phase:

- **Parser:** different syntax productions per item kind
- **Resolver:** `get_exported_names` enumerates all 8, `get_variant_names`
  enumerates all 8 again
- **Typecheck:** `build_type_env` registers `TypeDef` in TypeEnv, ignores
  the rest. `build_func_env` registers `FnDef`/`FuncDef`/`PatternDef`,
  ignores the rest. `ServiceDef` and `ResourceDef` are invisible.
- **Emit:** `emit_typed_item` has a match arm per variant, each calling
  a dedicated rendering function

This means adding a new item kind or changing how an existing kind
participates requires editing every phase. More importantly, items that
should naturally participate in the same namespace don't — a `service`
can't be referenced in expressions, a `resource` can't appear in type
position — because each phase only knows about the specific variants
it was coded to handle.

**S86: Item variants are per-kind compiler machinery, not unified DAG nodes.**

**Discovery context (2026-03-16):** After fixing the gist pipeline OOM
(S85) and import resolution issues, 25 errors remain — all "undefined
variable 'git'" and cascading field-access failures. `git` is a
`ServiceDef` in `extdeps/git.dag`. The typecheck doesn't register
`ServiceDef` items in any environment, so service names are invisible
to expression typechecking. The heuristic fix (register service names
as variables) adds another per-kind special case to the growing pile.

**Current pattern:** Every new item kind requires:
1. A new `Item` variant in `00_core.dag`
2. A new `TypedItem` variant in `00_core.dag`
3. New match arms in resolver (`get_item_name`, `get_variant_names`)
4. New match arms in typecheck (`resolve_item_types`, `infer_item`)
5. New match arms in each emitter (`emit_typed_item`, `emit_py_typed_item`)
6. New registration logic in `build_type_env` / `build_func_env`

This is open-set enumeration (violates "no case enumeration for open sets")
on what should be a structural concept.

**Historical note:** The codebase has gone through several iterations of
this pattern — `resource`, `tool`, `service`, `pattern`, `extern func` —
each time adding a new variant and threading it through every phase. The
intent was always to collapse these to DAG nodes, but the per-kind pattern
keeps getting reintroduced because the compiler's `Item` type invites it.

**Missing fact:** the unified identity of a DAG node. A service IS a node
with I/O ports and a transport property. A function IS a node with I/O
ports and a computation body. A resource IS a node with capability
properties. These are properties on a node, not reasons for separate
compiler paths.

**Terminal direction:** Replace the 8-variant `Item` enum with a single
structural node type that carries:
- **Name** and **span** (identity)
- **Ports** — inputs and outputs (the I/O contract)
- **Properties** — transport binding, body expression, capability mode,
  etc. (what makes a service different from a function)
- **Kind tag** — optional, for syntax/diagnostic purposes, but NOT used
  for dispatch in compiler phases

Compiler phases would walk the node structurally: "does this node have a
body? emit it. Does this node have a transport? wire it." Rather than:
"is this a FuncDef? do the FuncDef thing. Is this a ServiceDef? do the
ServiceDef thing."

This is the same collapse that made `TypeExpr` work — v2 types are
structural values, not string-keyed registry entries. The same principle
applied to items would make `ServiceDef` naturally visible in the
expression namespace, `ResourceDef` naturally usable in type position,
and new item kinds zero-cost to add.

**Smallest safe bridge (current):** Register `ServiceDef` names as
variables in the expression scope during typecheck, with the service's
operation namespace as the "type." This is a heuristic — another per-kind
special case — but it unblocks the gist pipeline. Documented as debt.

---

## v2 impact classification

**Eliminated by v2 (no v1 fix needed):**
All of Branch 1 (S1, S2, S4, S5, S13), S47, Branch 2 parallels.

**Inherited by v2 (re-implement correctly):**
S34 (callable wiring), S38 (emitted code untested), S44 (shell output
parsing annotation), S45/S46 (provider/transport metadata stamping).

**v1 maintenance only:** S39, S48, S49, Branch 7 evaluator fixes.

---

## V2 self-hosting gap analysis (updated 2026-03-16)

**V2 source:** 10 modules (~9,600 lines), all parse with zero diagnostics.
**Test status:** 89 pass, 0 fail, 6 ignored (cargo gates + gist pipeline).

**What works today:**
- Module discovery, parsing, type codegen, fn codegen (records, match,
  if/else, for, lambda, string interp, intrinsics, `with()`, `concat()`)
- Rust rendering, crate assembly, runtime shims
- Target-agnostic emission architecture (Rust + Python renderers)
- Honest type system (no fabrication in lookup/resolve/emit paths)
- v1 TCO pass for recursive .dag functions (tokenize_loop is iterative)
- O(1) lookups for type cache (Map), item registry (Map), module index (Map)
- Gist 11-module closure: resolve passes in compiled v2 crate

**Generated v2 crate: cargo check + cargo build pass.**

**Gist pipeline blocker (2026-03-16):** Full pipeline OOMs on `std.types`
(item 55 of 101: `CredentialFlow`). Root cause: recursive sum type triggers
infinite recursion in resolver due to dropped cycle-detection state (S85).
Not an algorithmic scaling issue — a missing language feature.

**Path forward:**
1. Unblock gist: thread `resolving` through resolve helpers (S85 smallest fix)
2. v2 emitter TCO pass (S84) — required for self-hosting
3. Recursive type support as a language feature (S85 terminal direction)

### Risks identified during Track A–C integration (2026-03-15)

**S82: Flattened function namespace causes silent overwrites.**
All v2 modules' functions are merged into one `HashMap<String, LoweredFnBody>`
for the evaluator. Name collisions silently overwrite — the last module loaded
wins. `lookup_func_sig` was defined in both `04_typecheck.dag` and `05_emit.dag`
with different signatures. The emit version overwrote the typecheck version,
causing `unbound variable: scope` when the typechecker called it with the wrong
parameter names.
**Fix applied:** Renamed emit's version to `lookup_func_sig_in_scope`.
**Systemic risk:** Any future name collision will produce the same class of bug
with a misleading error message. The flattened namespace has no module isolation.
**Terminal state:** Self-hosting. The v2 compiler resolves imports structurally
and won't flatten namespaces.
**Mitigation until then:** `compile_all_modules()` should detect and reject
duplicate function names across modules.

**S83: Re-entrant evaluator stack overflow on deep call chains. — FIXED (stacker)**
`eval_non_sibling_call_raw` calls `evaluate_stack` re-entrantly for sibling
function calls inside intrinsic lambdas (map/filter/fold). Each re-entrant call
adds ~20 Rust stack frames. Processing 11 real .dag files through the v2
typechecker exceeds the default 8MB thread stack.
**Terminal state:** Self-hosting eliminates the evaluator.
**Fix:** `stacker::maybe_grow` added to re-entrant call sites, growing the
stack on demand. No manual stack size tuning needed.

**S84: v2 emitter has no TCO pass — CRITICAL for self-hosting.**
Track C added tail-call optimization to v1's `fn_codegen.rs` (Stmt::Loop +
parameter reassignment). This fixes the bootstrapping path: v1 compiles v2 .dag
files into iterative Rust. **But the v2 emitter (`05_emit_rust.dag`) does not
perform this transformation.** When v2 compiles itself, the generated Rust will
use recursive calls for functions like `tokenize_loop`, `resolve_imports`,
`collect_service_calls`, and every `fold` accumulator pattern. This will
stack-overflow at runtime, exactly as v1 did before Track C.
**Required:** Add a TCO analysis + transformation pass to the v2 emission
pipeline, analogous to what Track C added to v1. The v2 version should operate
on the typed IR (between typecheck and emit), detecting self-tail-recursive
functions and rewriting them to use a loop construct that the per-target
renderers can emit (`loop {}` for Rust, `while True:` for Python).

**v1 implementation note (2026-03-15):** Track C now uses a `TcoPlan`
intermediate in `fn_codegen.rs` rather than the earlier classify-then-rewrite
pair of passes. This is the smallest redesign that satisfies the v1 invariants:
tail position is modeled structurally instead of by threaded booleans, analysis
and rewriting share one representation, and unsupported recursive contexts fail
closed instead of partially transforming. We explicitly did **not** introduce a
full CFG / terminator IR in v1. A CFG would be cleaner long-term and would make
TCO just another edge rewrite, but the blast radius is too large for a bootstrap
compiler whose long-term future is still uncertain. If v1 becomes strategic,
promote control flow to a real block/terminator IR instead of extending
`TcoPlan`.

### Branch 9: Recursive types are representable but not modeled (v2)

The `.dag` language describes DAGs — acyclic computation graphs. Causality
flows forward; there are no real loops. But recursive type definitions like
`type T = ... | Variant { field: List<T> }` introduce cycles in the type
dependency graph, which is fundamentally not a DAG.

Recursion is a valid **modeling tool** — a virtual layer that helps describe
patterns that unwind through time. `CredentialFlow = ... | Chained { steps:
List<CredentialFlow> }` is a legitimate domain concept. But the language has
no structural position on it: the parser accepts it silently, the type
system doesn't distinguish it from non-recursive types, and every compiler
phase independently discovers (or fails to discover) the recursion.

**Discovery context (2026-03-16):** While investigating OOM in the gist
pipeline, profiling revealed that `typecheck_module` never returns for
`std.types` (523 lines, 101 type definitions). Root cause: `CredentialFlow`
is a recursive sum type. The v2 resolver has a cycle-detection mechanism
(`resolving: List<String>` threaded through `resolve_type_expr_with_resolving`)
but three helpers in the call chain — `resolve_field`, `resolve_variant`,
`resolve_param` — call the convenience wrapper `resolve_type_expr` which
creates a fresh empty `resolving` list, silently dropping cycle state. The
resolver enters infinite recursion, heap-allocating until OOM (SIGKILL after
60+ seconds in release mode on 16GB).

**The deeper issue is not the broken threading.** It's that the compiler has
no explicit model of recursive types. The fact "type T is recursive" is
never computed, never stored, never carried across phase boundaries. Instead:

1. `build_type_env` registers the type — doesn't notice the cycle
2. `resolve_type_expr_with_resolving` tries to insert cycle breakers via a
   threaded runtime parameter — but 3 of 14 call sites drop it silently
3. `validate_no_unresolved` sees surviving `Named` refs after resolution —
   assumes they are valid cycle breakers without verifying
4. `emit_type_expr` encounters `Named` — emits the bare name, no `Box<T>`
   indirection (Rust requires heap indirection for recursive types)
5. 7 additional functions (`type_expr_shape`, `type_expr_equals`,
   `collect_unresolved_in_type_expr`, `has_nested_records`, `emit_type_expr`,
   `emit_py_type_expr`, `normalize_access_type`) walk `TypeExpr` recursively
   with no cycle protection — correct for DAGs, infinite-loop for cycles

Each phase independently improvises. The `resolving` list is the same class
of bug as `TypeId` in v1 — a fact that should be structural but is instead
re-derived (or not) at every boundary.

**S85: Recursive types accepted without structural support.**

**Missing fact:** whether a type definition is intentionally recursive. The
`.dag` language allows it syntactically but provides no signal to the
compiler. The compiler's only mechanism (the `resolving` list) is opt-in,
manually threaded, and silently droppable.

**Philosophical position:** Recursive types are a virtual layer on top of
the DAG — a modeling convenience for patterns that unwind through time. The
DAG is the ground truth; recursion is a lens. The language should explicitly
support this lens rather than accidentally permitting it. A recursive type
should be declared, not discovered.

**Affected call sites (audit, 2026-03-16):**

Unsafe (reachable from recursive resolver, drop `resolving`):
- `resolve_field` (04_typecheck.dag:1614) — CRITICAL, called from Product + Coproduct cases
- `resolve_param` (04_typecheck.dag:1656) — moderate
- `resolve_resource_use` (04_typecheck.dag:1752) — moderate

No cycle protection (walk TypeExpr recursively, assume finite depth):
- `normalize_access_type` (04_typecheck.dag:443)
- `type_expr_shape` (04_typecheck.dag:471)
- `type_expr_equals` (04_typecheck.dag:490)
- `collect_unresolved_in_type_expr` (04_typecheck.dag:2641)
- `has_nested_records` (05_emit.dag:327)
- `emit_type_expr` (05_emit_rust.dag:330)
- `emit_py_type_expr` (05_emit_python.dag:268)

These 7 functions are correct for DAGs. They become bugs only because the
resolver fails to convert recursive types into structurally finite form
(cycle breakers). Fix the resolver, and these functions work as-is.

**Smallest safe fix (unblock gist):**
1. Thread `resolving` through `resolve_field`, `resolve_variant`,
   `resolve_param` and their callers (signature changes)
2. Add a recursive-type test case through the full pipeline:
   `type Node = Leaf | Branch { children: List<Node> }`
3. This ensures cycle breakers are inserted, making the downstream walks
   terminate. Does not address `Box<T>` rendering or the opt-in fragility.

**Terminal direction (language feature):**
Recursive types should be an explicit language concept — something like
`recursive type CredentialFlow = ...` — so the compiler can:
- **Reject unmarked cycles** at parse/resolve time (accidental self-reference
  is a compile error, not a silent OOM)
- **Carry the fact structurally** on `TypeBinding` or `TypeExpr` through all
  phases (no `resolving` list, no opt-in threading, no escape hatch)
- **Render correctly per-backend** (`Box<T>` for Rust, reference types for
  other targets) based on the structural marker, not heuristic detection

The exact syntax and semantics are open — this is a language design question,
not just a compiler fix. The `resolving` list and the `resolve_type_expr` /
`resolve_type_expr_with_resolving` split should both disappear in the
terminal state. See discussion in the heuristic roadmap (Phase A, `Box::new()`
wrapping row).

**Invariant this violates:** Invariant 9 ("Correctness by construction, not
by validation"). The current API makes it easy to construct the wrong call.
The correct call requires threading state through every intermediate helper.
There is no compile-time or runtime enforcement — just convention.

---

## Structural closure plan for current review findings (2026-03-15)

These are the remaining "the code had to guess because the boundary dropped a
fact" seams in the current v1 branch. Each entry names the missing fact, the
earliest boundary that should carry it, the smallest safe bridge in v1, and the
terminal fix.

### 1. Anonymous record target resolution

**Current failure mode:** `fn_codegen.rs` still uses `infer_struct_name()` to
guess which nominal struct an anonymous record literal should construct. On
miss it can fabricate an empty/unknown target instead of failing clearly.

**Missing fact:** the nominal target of the record literal (or an explicit
synthetic shape id when the record is intentionally anonymous).

**Boundary that should carry it:** typecheck output. By the time codegen sees
`{ a: ..., b: ... }`, the record should already be annotated with the resolved
target type, rather than only a field set.

**Smallest safe v1 bridge:** make record-target resolution return
`Option<RecordTarget>` instead of `String`; prefer explicit context
(`current_return_type`, expected field type, explicit record name) and emit
`compile_error!` on ambiguous or missing targets. Do not return `String::new()`.
Thread module-qualified names through synthesis so same-name structs in
different modules stop colliding in `struct_field_types`.

**Terminal fix:** replace anonymous-record guessing with a typed record form in
the checked IR, e.g. `TypedExpr::Record { target: RecordTarget, fields }`,
where `RecordTarget` is either `Nominal(TypeId)` or `Synthesized(ShapeId)`.
Codegen lowers that directly; `infer_struct_name()` and its duplicate ranking
logic disappear.

### 2. Collection intrinsic semantics in shared IR

**Current failure mode:** new fast paths like `count(filter(...))`,
`first(filter(...))`, `index_by`, and `map_values` are encoded with Rust-specific
`RawCode` and even by stringifying IR through the Rust renderer. Non-Rust
backends receive syntax, not semantics.

**Missing fact:** the operation itself. "Count the elements satisfying this
predicate" is not the same fact as "emit this Rust snippet".

**Boundary that should carry it:** lowering / typed-IR-to-code-IR boundary.
When the compiler recognizes a collection intrinsic pattern in `.dag`, it
should lower that to a structural operation node, not directly to Rust text.

**Smallest safe v1 bridge:** introduce explicit `code_ir` forms for the new
operations, or move the fast paths out of shared codegen and into the Rust
backend only. If a non-Rust backend cannot lower the operation, it must return
`UnsupportedConstruct`; it must not render `RawCode` verbatim. Pick one
authority for helpers like `index_by`: either emit a call to a runtime shim or
lower structurally, but not both.

**Terminal fix:** represent collection/map operations in IR explicitly, e.g.
`Expr::CollectionOp(CollectionOp::CountWhere { .. })`,
`Expr::MapOp(MapOp::Get { .. })`, `Stmt::MapInsert`, etc. Backends lower those
to target-specific syntax or reject them explicitly. `render_expr_inline()` and
Rust-renderer callbacks from `fn_codegen.rs` disappear.

### 3. Generated self-hosting tests and stage contracts

**Current failure mode:** generated tests in `v2_crate_emit.rs` hand-recreate
pipeline stages inline and mix stage-local data. That is how a resolve-only test
ended up referencing `result.files`, a value that only exists in the full
compile path.

**Missing fact:** which stage is being exercised, and which outputs belong to
that stage.

**Boundary that should carry it:** the public pipeline API used by generated
tests. The API should expose stage-specific result types, not a single
"everything" shape plus ad hoc reconstruction inside string templates.

**Smallest safe v1 bridge:** add stage-specific entry points like
`parse_sources`, `resolve_sources`, or `compile_sources_until(Stage::Resolve)`,
and make generated tests call those directly. The generated code should only
assert over fields that exist on that stage result type.

**Terminal fix:** define a small set of stage result structs with explicit
contracts (`ParsedSources`, `ResolvedGraph`, `TypedModules`, `EmittedCrate`).
Generated tests select a stage and receive only that stage's data. This also
lets the self-hosting ratchet speak precisely about what is proven.

### 4. TCO backend contract

**Current failure mode:** `TcoPlan` is structurally sound inside `fn_codegen`,
but shared `code_ir` still uses generic `Stmt::Loop`, `Stmt::Continue`, and
`Stmt::Break(expr)` to carry TCO control flow. Rust, Go, and C then recover the
meaning by convention.

**Missing fact:** that these control-flow nodes are specifically "exit or
continue the synthetic function-level tail-call loop", not ordinary source-level
loop control.

**Boundary that should carry it:** the boundary between TCO analysis and target
lowering. Backend-specific operational control flow should be introduced as late
as possible.

**Smallest safe v1 bridge:** keep `TcoPlan` as the cross-stage contract and
lower it separately per backend. Rust and Go can implement the lowering; C and
other backends should reject `TcoPlan` explicitly until they support it. Do not
rely on provenance-sensitive interpretations of plain `Break(expr)`.

**Terminal fix:** either add an explicit backend-neutral control-flow IR with
terminators, or keep `TcoPlan` as a first-class plan type consumed directly by
backend lowerers. In either design, "this exits the TCO loop with function
result X" is structural, not inferred.

### 5. Embedded source metadata

**Current failure mode:** the generated v2 crate currently duplicates the same
facts in multiple lists: which `.dag` files are embedded, which const names they
map to, which module names they should parse as, and which test cohorts they
belong to. Every new source requires editing multiple lists that can drift.

**Missing fact:** a single authoritative manifest for embedded compiler/test
sources.

**Boundary that should carry it:** crate assembly. The generated test module
should derive all constants, expected names, and test cases from one manifest,
not parallel arrays.

**Smallest safe v1 bridge:** define one `EmbeddedDagSource` table in
`v2_crate_emit.rs` with fields like `stem`, `rel_path`, `const_name`,
`expected_module_name`, and `cohort`. Use that table both to read files and to
emit generated tests.

**Terminal fix:** move embedded-source selection and test generation onto a
single metadata producer (compiler output or a shared manifest module). Adding a
source then means editing one structure, and every consumer derives from it.

---

## Heuristic elimination roadmap

Each v1 bootstrap heuristic (S81) maps to a modeling decision that makes
it unnecessary. Ordered by dependency.

### Phase A: Type-aware emission (eliminates S76, S77, S78, S81 bulk)

| Heuristic | What v2 emitter does instead |
|-----------|------------------------------|
| `clone_if_needed` (S76) | Tracks variable liveness. Last use = move, earlier = borrow. |
| `infer_struct_name` (S77) | Typechecker resolves anonymous records to structural type. |
| `Box::new()` wrapping | Checks `TypeExpr` for recursion. Per-backend rendering. (See S85.) |
| `Some()`/`None` injection | `TypeExpr::Optional` in typed IR. Per-backend rendering. |
| `.as_str()` insertion | Rust emitter knows String match context. |
| `..Default::default()` | Emitter has all field types. Complete struct literals. |

### Phase B: Import resolution (eliminates S78, S79)

| Heuristic | What v2 resolver does instead |
|-----------|-------------------------------|
| `std_types_prelude()` (S78) | Types from resolved module graph. No hand-written defs. |
| `module_prelude()` (S79) | Per-module `use` derived from resolved imports. |

### Phase C–F: Further modeling

- **C: Variant disambiguation** — typechecker resolves from context (expected type).
- **D: Optionality** — `TypeExpr::Optional` already modeled. Emitter reads source/target.
- **E: Ownership** — `.dag` has value semantics. Backend decides (Rust: clone/move, C: copy/refcount, Go: GC, Verilog: wire).
- **F: Static data** — `data` defs as `ConstDef { name, type, value }`. Per-backend rendering.

### What dies with self-hosting

- `fn_codegen.rs`, `v2_crate_emit.rs`, `v2_runtime_shim.rs` — entire files
- All heuristic functions: `clone_if_needed()`, `is_option_expr()`,
  `infer_struct_name()`, `std_types_prelude()`, `module_prelude()`
- `synthesize_anonymous_structs()`, `compile_intrinsic_call()` Rust-specific handlers

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

## Resolved summary (70 findings, 2026-03-10 through 2026-03-15)

| Theme | Findings | Resolution pattern |
|-------|----------|--------------------|
| Structural classification | S11, S12, S14, S15, S22, S44–S46, S48, S49, S67, S68–S70, S75, S80 | String match → structural dispatch, typed registries, explicit intrinsics |
| Fail-closed error handling | S3, S7, S23/S35, S24, S25, S34, S64 | `.ok()`/`let _`/`.unwrap_or(true)` → `Result` propagation, explicit `match` |
| Boundary contracts | S18–S20, S31–S33, S40, S41, S57 | Validation passes, cardinality checks, runtime type enforcement |
| Eval stack machine | S50–S56, S58–S61, S52-EVAL, Eval-8/9, S67-5/6/7 | Stack ordering, TCO, type enforcement, error/control-flow separation |
| Metadata consolidation | S16, S21, S26, S42, S43 | Single-authority registries, stamped classification |
| Code quality | S6, S9, S63, S65, S66, S69, S71, S74 | Purity fixes, crate deps, dead code removal |
| Foundation | R1–R6 | Deleted duplicates, structural walks, removed fabrication |
| Test quality | BUG-6, E3.2 | Promoted aliases, non-tautological assertions |
| v2 integration | S82 (namespace collision), G8/G12–G14 (fabrication) | Renamed, honest return types, sum types for sentinels |
| v2 type system | S85 (recursive types unmodeled) | Language feature: explicit recursive type declarations |
| v2 item model | S86 (items are separate species) | Unified DAG node with structural properties, not 8-variant enum |
