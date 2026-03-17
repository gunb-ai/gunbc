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

## Rubric for new findings

Every heuristic is downstream evidence that an upstream stage dropped structure.
The standard template for documenting a finding:

1. **Current failure mode** — what goes wrong observably
2. **Missing fact** — what information the code is trying to guess
3. **Boundary that should carry it** — where the fact should become structural
4. **Smallest safe bridge** — the minimum fix that unblocks without violating invariants
5. **Terminal fix** — the design that eliminates the class of bug

Decision rules:
- If a fact is derivable from authoritative structure, compute it once and carry it forward.
- If a fact is not derivable, require it explicitly in syntax or the boundary type.
- Don't turn a derivable fact into a required annotation.
- Don't fix a missing fact by inventing a permissive fallback type, name, or placeholder.

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

### Branch 10: The compiler checks keywords, not graph consistency (v2)

The compiler's job is to ensure bits get where they need to — that every
edge in the graph connects compatible ports. This is constraint
satisfaction on the graph structure: classical logic. Either the edges
match or they don't.

But the v2 compiler doesn't work on the graph. It works on **keywords.**
The `Item` type has 8 variants (`TypeDef`, `FuncDef`, `FnDef`,
`ServiceDef`, `ResourceDef`, `DataDef`, `ExternFuncDecl`, `PatternDef`),
and every compiler phase dispatches on which keyword the author used:

- `build_type_env`: "if TypeDef, register shape. If ResourceDef, register
  shape. Else skip."
- `build_func_env`: "if FuncDef, register callable. If FnDef, register
  callable. Else skip."
- `infer_item`: "if FuncDef, infer body. If ServiceDef, skip."
- `emit_typed_item`: per-variant rendering for all 8.

This means the compiler's behavior depends on which magic word the author
wrote, not on the graph structure. A `service` with operations that have
typed inputs and outputs is invisible to expression checking — not because
its graph structure is different from a function, but because the keyword
says `service` instead of `func`.

**S86: Compiler dispatches on item keywords instead of graph properties.**

**Discovery context (2026-03-16):** After fixing S85, 25 errors remained
in the gist pipeline — all "undefined variable 'git'." `git.Core` is a
`ServiceDef` with typed operations (input/output port shapes). The
compiler has all the information it needs to check edge compatibility.
But the keyword `service` routes around the expression checker entirely.

**The 8 keywords analyzed:**

The minimum structural vocabulary for a DAG is 3 concepts:
- **Shape** — declares a data structure (what flows through edges)
- **Computation** — has I/O ports and a body (transforms inputs to outputs)
- **Boundary** — has I/O ports but no body (provided externally)

Every keyword maps to one of these + properties:

| Keyword | Structural concept | Differentiating property |
|---------|--------------------|--------------------------|
| `type` | Shape | — (this IS the shape definition) |
| `fn` | Computation | purity (derivable: no I/O refs in body) |
| `func` | Computation | has `uses` clause (I/O declaration) |
| `service` | Computation | transport binding (external I/O) |
| `resource` | Shape | capability mode (Read/Write) |
| `data` | Computation | no inputs, constant body |
| `extern func` | Boundary | no body (externally provided) |
| `pattern` | Computation | identical to `func` (pure documentation) |

The keywords are fine as **parse-time syntax dispatch** (telling the
parser what syntax to expect) and as **frontend validation sugar** (e.g.,
`fn` asserts purity — the compiler verifies no I/O refs in the body).
But they should not be the mechanism by which the compiler decides what
to check. After parsing, the compiler should work on graph properties:
does this node have ports? What shapes do they carry? Does it have a
body to check? Does it have edges to external providers?

**Design principle:** Encode the reason, not the label. The reason a
caller needs its callee to be pure (e.g., "this context provides no
Network resource") is stable — it comes from the graph structure and
doesn't change when code is refactored. The label (`fn` = pure) is
volatile — it must be updated every time the implementation changes,
leading to constant keyword churn.

The compiler should enforce the stable constraint: "node A calls node B,
A's context provides resources {R1, R2}, B's subgraph requires {R1, R2,
R3} — edge unsatisfiable, missing R3." This catches the real problem
regardless of keywords. The developer learns WHAT would break (R3 is
unavailable) and WHY (the caller doesn't provide it), not just "you
used the wrong keyword."

By contrast, a keyword-level check ("you wrote `fn` but your body calls
a service") is a hint — trivially bypassed by changing `fn` to `func`.
Nothing structurally broke. The developer learned nothing about what
depends on purity. The label changed, the graph didn't. This is the
same problem as Python type hints — they look like they're doing
something, but if they're not enforced by structure, they're noise
that drifts from reality.

The rubric applies directly: "if a fact is derivable from authoritative
structure, compute it once and carry it forward." Resource requirements
propagate through call edges. The compiler can derive the full I/O
profile of any node by walking its subgraph. No keyword needed.

**Missing fact:** the graph properties of each item — what ports it
introduces, what shapes they carry, whether it has a body, what edges
it requires. Currently the compiler derives these by keyword dispatch
(8-way match). The alternative: extract graph properties once after
parsing, then the checker works on properties uniformly.

**Terminal direction:** Rename `04_typecheck.dag` to `04_check.dag`
(graph consistency checker). The checker's job:

1. **Resolve names** — map string references to structural port shapes
2. **Extract graph properties** — for each item, derive: what names
   does it introduce? what port shapes? what resources does its
   subgraph require? This replaces the per-keyword `build_type_env` +
   `build_func_env` split.
3. **Propagate port shapes + resource requirements** — walk
   expressions, determine what each edge carries and what resources
   each subgraph requires (currently `infer_expr`, extended with
   resource propagation)
4. **Check edge consistency** — for every call edge, verify:
   (a) output port shape is compatible with input port shape
   (b) caller's resource set satisfies callee's resource requirements

The checker never asks "what keyword is this?" It asks "are all edges
satisfied?" Error messages say what would break: "node X calls node Y,
but Y requires Network which X doesn't provide" — not "you used `fn`
but your body calls a service."

Keywords (`fn`, `func`, `service`, etc.) remain as parse-time syntax
dispatch. They are NOT validation assertions — the graph structure is
the sole authority. If a developer changes `fn` to `func`, nothing
structurally changes (the resource requirements are the same either
way). The compiler catches the real issue: an unsatisfied edge.

**Terminal vision: keywords are namespacing, not magic.**

Every name in the system — `String`, `Int`, `Network`, `git.Core`,
`CredentialFlow` — carries meaning because of its DAG composition,
not because the compiler has a hardcoded case for it. `String` is a
node defined at the base of the composition chain (currently hardcoded
in `kernel_type_env()`). In the terminal state, it would be DAG-defined
in `dsl/std/types.dag` like any other node. The compiler doesn't know
what "String" means — it knows what a node with certain port shapes
and composition structure means.

This collapses the IR to a single concept: **a node is a name, optional
ports, optional body, optional properties, composed from other nodes.**
What we currently call keywords (`type`, `fn`, `service`, `resource`)
are syntactic entry points for defining nodes with different default
structures. After parsing, they're all the same thing. The checker
sees only nodes and edges.

The parser can keep varied syntax for readability (defining a record
looks different from defining a function body). But `00_core.dag`'s
`Item` / `TypedItem` types collapse toward a single `Node` with
properties. The 8-variant enum, the per-keyword dispatch, and the
separate type/function environments all disappear.

**Boundaries.** A node with ports but no body — its outputs must be
provided externally. The checker verifies every consumer's port is
compatible and a provider exists at link time. Replaces `extern func`
as a keyword with a structural graph property.

**Smallest safe bridge (current):** Per-keyword heuristics to register
`ServiceDef` namespace roots as expression-scope variables, and
permissive field/method access on service types. These are documented
debt — they work by inventing permissive fallback types (violating the
rubric) and will be removed when the checker works on graph structure.

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
**Test status:** 89 pass, 0 fail, 7 ignored (cargo gates + gist pipeline).

**What works today:**
- Module discovery, parsing, type codegen, fn codegen (records, match,
  if/else, for, lambda, string interp, intrinsics, `with()`, `concat()`)
- Rust rendering, crate assembly, runtime shims
- Target-agnostic emission architecture (Rust + Python renderers)
- v1 TCO pass for recursive .dag functions (tokenize_loop is iterative)
- O(1) lookups for type cache (Map), item registry (Map), module index (Map)
- Gist 11-module closure: typecheck passes with 0 errors in compiled v2 crate

**Generated v2 crate: cargo check + cargo build pass.**

**Gist pipeline status (2026-03-16, updated after Streams 1-4):**
- Resolve: passes (0 errors, 0.1s)
- Typecheck: passes (0 diagnostics, SCC cycle detection works correctly)
- Emit: still hangs on multi-module input despite Stream 1 clone fix.
  The fold accumulator clone overhead was one contributor, but other
  O(n²) patterns in the emit phase remain (repeated inference, list-based
  lookups, append-by-concat — see performance audit).

**Blockers and resolution status:**

1. **v1 codegen clone overhead** — FIXED (2026-03-16). `clone_if_needed()`
   in `fn_codegen.rs` now takes `fold_accum_name: Option<&str>` and skips
   `.clone()` for field accesses on the fold accumulator variable. This
   keeps Rc refcount at 1 so `Rc::try_unwrap` succeeds → O(1) in-place
   mutation instead of deep clone. All 4 call sites updated.

2. **S85: recursive type infinite recursion** — TERMINAL FIX APPLIED
   (2026-03-16). The `resolving: List<String>` threading is replaced by
   SCC-based cycle detection at `build_type_env` time. `detect_type_cycles`
   precomputes `recursive_types: List<String>` on `TypeEnv`. `resolve_type_expr`
   checks the precomputed set — Named references to cycle members are kept as
   Named; all others expand. `resolve_type_expr_with_resolving` and the
   `resolving` parameter are deleted from all helpers. No growing list
   allocation per resolution step.

**Path forward (5 independent streams):**

Stream 1: **v1 codegen clone overhead** — DONE. `clone_if_needed()` is
fold-accumulator-aware. All v2 compiler tests pass (89/89).

Stream 2: **S85 terminal fix** — DONE. SCC cycle detection replaces
`resolving` list. `recursive_types` precomputed on `TypeEnv`.
`resolve_type_expr_with_resolving` deleted.

Stream 3: **S86 terminal fix** — unified node IR + graph consistency
checker. Collapse `Item`/`TypedItem` (8 variants each) to a single
`Node` type. Checker enforces edge constraints on nodes, not keyword
constraints. Acceptance criteria:
(a) all S86 heuristics deleted;
(b) `Item` 8-variant enum replaced with `Node` post-parse;
(c) `build_type_env` + `build_func_env` replaced with unified graph
    property extraction from nodes;
(d) kernel primitives (`String`, `Int`, etc.) DAG-defined in
    `dsl/std/types.dag`, not hardcoded in `kernel_type_env()`;
(e) service operations typecheck through normal port-shape propagation;
(f) error messages say what would break structurally;
(g) keywords (`fn`, `type`, `service`, `resource`) are parse syntax
    only — changing them produces no diagnostic change if graph is
    unchanged.
Largest stream, most architecturally important. Touches `00_core.dag`,
`04_typecheck.dag` (rename to `04_check.dag`), emitters, and
`dsl/std/types.dag`. Parser keeps keyword syntax for readability.

Stream 4: **S84 v2 emitter TCO** — VERIFIED WORKING (2026-03-16). The v2
emitter already has TCO: `expr_has_self_call()`, `has_non_tail_self_call()`
classify functions; `emit_tco_body()` / `emit_tco_expr()` render loops.
Confirmed in generated code: `find_dot_index`, `normalize_access_type` etc.
emit as `loop { ... continue; ... break; }` patterns. S84 is closed.

Stream 5: **v1 structural closure** — the 5 findings from 2026-03-15
review (anonymous records, collection intrinsics, test contracts, TCO
backend contract, embedded metadata). Low priority, none block gist
or self-hosting.

Stream 6: **S87 emitter clone overhead** — the v2 emitter passes
`InferScope` (containing `type_cache: Map<String, TypeExpr>` with ~5K-10K
entries per module) by value through every recursive `emit_expr_in_scope`
call. Each call clones the full scope at multi-child AST nodes. For 10K
body nodes × 25K scope entries = 250M entry clones — hours in debug,
minutes in release. The emitter also converts `TypedExpr` back to `Expr`
(discarding attached types), then recovers types via span-keyed cache
lookups. See S87 finding below.

All 6 streams are independent — no code overlap, any ordering works.

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

**S84: v2 emitter TCO — CLOSED (2026-03-16).**
The v2 emitter has working TCO: `expr_has_self_call()`, `has_non_tail_self_call()`
classify functions; `emit_tco_body()` / `emit_tco_expr()` render iterative loops.
Verified in generated code: tail-recursive functions emit as `loop { ... continue;
... break; }` patterns (confirmed for `find_dot_index`, `normalize_access_type`).
Both Rust (`loop {}`) and Python (`while True:`) renderers handle TCO correctly.

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

**Missing fact:** whether a type definition participates in a cycle. This
is derivable from the type dependency graph (SCC analysis) but is never
computed or stored. The `.dag` language allows recursive types syntactically
but provides no structural signal to the compiler. The compiler's only
mechanism (the `resolving` list) is opt-in, manually threaded, and silently
droppable.

**Structural position:** Recursion is a property the compiler can derive
from the type graph — SCC analysis on the type dependency graph identifies
which types participate in cycles and which fields are back-edges. This is
a derivable fact, not one that should require a source-level annotation.
Requiring a `recursive` keyword would introduce a second authored fact that
can drift from the actual graph structure, violating the rubric ("don't
turn a derivable fact into a required annotation").

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

**Smallest safe fix (unblock gist):** APPLIED AND SUPERSEDED.

**Terminal fix (SCC-based cycle detection) — APPLIED (2026-03-16):**
The `resolving: List<String>` parameter and `resolve_type_expr_with_resolving`
are deleted. Replaced by:
- `type_expr_deps(expr: TypeExpr) -> List<String>` — extracts Named dependencies
- `reaches_self(root, current, bindings, visited) -> Bool` — DFS reachability
- `detect_type_cycles(bindings) -> List<String>` — finds all cycle participants
- `TypeEnv.recursive_types: List<String>` — precomputed at `build_type_env` time
- `resolve_type_expr` checks `env.recursive_types` — Named refs to cycle members
  are kept as Named; all others expand. No growing list, no opt-in threading.

The `resolving` parameter is removed from `resolve_field`, `resolve_variant`,
`resolve_param`, `resolve_resource_use`, and all their callers.

**Remaining work:**
- **`Box<T>` rendering** — the emitter should detect recursive fields and wrap
  them in `Box<T>` for Rust (based on `recursive_types` set). Currently not done.
- **Back-edge metadata** — the cycle set tells WHICH types are recursive but not
  WHICH fields are back-edges. Adding `back_edges: List<FieldPath>` to
  `TypeBinding` would make `Box` wrapping precise.

**Invariant this violates:** Invariant 9 ("Correctness by construction, not
by validation"). The current API makes it easy to construct the wrong call.
The correct call requires threading state through every intermediate helper.
There is no compile-time or runtime enforcement — just convention.

---

### S87: Emitter clone-dominated performance (v2)

**Current failure mode:** Self-compile (v2 crate processing its own 10
.dag source files) takes hours in debug, 15+ minutes in release. The
emitter is the dominant contributor after typecheck.

**Missing fact:** resolved types at each expression node. The typechecker
*computes* these (stored as `TypedExpr.resolved_type`) but the emitter
discards them via `typed_expr_to_expr` (O(B) tree clone per item), then
recovers them from a span-keyed cache (`infer_expr_type` → `map_get`).
The cache (`type_cache: Map<String, TypeExpr>`) has ~5K-10K entries per
module and is part of `InferScope`, which is cloned at every recursive
emit call.

**Boundary that should carry it:** the TypedExpr type already carries
`resolved_type` at every node. The emitter should read it directly.

**Root cause analysis (2026-03-17):**

The clone cost model for `emit_expr_in_scope(expr: Expr, registry:
Map<String, ItemInfo>, scope: InferScope)`:

| Component | Entries | Cloned per call | Total for self-compile |
|-----------|---------|-----------------|------------------------|
| scope.type_cache | ~5K-10K | yes (in scope) | dominant |
| scope.type_env | ~100 | yes (in scope) | moderate |
| scope.func_env | ~100 | yes (in scope) | moderate |
| scope.locals | ~0-30 | yes (in scope) | small |
| registry | ~300 | yes (separate) | moderate |

With B ≈ 10K recursive calls × S ≈ 25K scope entries = 250M entry
clones. At ~100ns per clone (release): ~25s for emitter alone.

**Four-task fix plan:**

| Task | Before | After | Improvement |
|------|--------|-------|-------------|
| T1: Emit from TypedExpr | O(B × 25K) clones | O(B × 600) clones | ~40× |
| T2: Single-pass analysis | O(3B) walks | O(B) walk | 3× |
| T3: Split scope (ctx+locals) | O(B × 600) | O(B × 200 + let×30) | ~2× |
| T4: v1 codegen refs | O(B × 200) | O(B) | ~200× |

**T1 is highest impact:** eliminating `type_cache` from the emit scope
shrinks clone cost from ~25K to ~300 entries per call (~40× reduction).
After T1, self-compile emitter should complete in <1s release.

**What T1 changes:**
- `emit_expr_in_scope(expr: Expr, ...)` → `emit_texpr(texpr: TypedExpr, ...)`
- All 17 expression emitters match on TypedExpr variants
- `.resolved_type` read directly (no span cache lookup)
- Deleted: `typed_expr_to_expr` (9 call sites), `infer_expr_type`,
  `lookup_type_by_span`, `missing_emit_type`, `scope_after_plain_expr`,
  `type_cache` on emit scope

**T4 is a v1 codegen change:** detect read-only parameters and generate
`&T` references. Applies broadly to all compiled .dag code, not just the
emitter. Stretch goal — may not be needed if T1-T3 are sufficient.

**Invariants this violates:**
- "No duplicate representations" — types exist as TypedExpr.resolved_type
  AND as span-keyed cache entries. The cache is a derived copy.
- "No fallbacks that fabricate" — cache misses produce
  `Named { name: "__EmitTypeCacheMiss" }`, a fabricated sentinel.

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
| `Box::new()` wrapping | SCC-derived cycle metadata on `TypeBinding`. Per-backend rendering. (See S85.) |
| `Some()`/`None` injection | `TypeExpr::Optional` in typed IR. Per-backend rendering. |
| `.as_str()` insertion | Rust emitter knows String match context. |
| `..Default::default()` | Emitter has all field types. Complete struct literals. |

### Phase B: Import resolution (eliminates S78, S79)

| Heuristic | What v2 resolver does instead |
|-----------|-------------------------------|
| `std_types_prelude()` (S78) | Types from resolved module graph. No hand-written defs. |
| `module_prelude()` (S79) | Per-module `use` derived from resolved imports. |

### Phase C: Graph consistency checker (S86 terminal direction)

Replace keyword-dispatched typechecker with edge constraint checker
on a unified node IR:

- **Unified node:** collapse `Item` (8 variants) and `TypedItem`
  (8 variants) to a single `Node` with properties. A node is a name,
  optional ports, optional body, optional properties, composed from
  other nodes. Keywords (`type`, `fn`, `service`, `resource`) are
  parse-time syntax that produces nodes — not dispatch targets.
- **No kernel builtins:** `String`, `Int`, `Bool`, etc. are
  DAG-defined nodes in `dsl/std/types.dag`, not hardcoded in
  `kernel_type_env()`. The compiler has no concept-specific names.
- **Edge constraint checking:** for every call edge, verify
  (a) port shapes compatible, (b) caller's resource set satisfies
  callee's requirements. Error messages say what would break.
- **Resource propagation:** walk subgraphs to derive I/O profiles.
  No purity keywords — purity is structural (empty resource set).
- The S86 heuristics (`__service_*`, `__field_*`, permissive access)
  and the `build_type_env` / `build_func_env` split all disappear.

### Phase D–G: Further modeling

- **D: Variant disambiguation** — checker resolves from context (expected type).
- **E: Optionality** — `TypeExpr::Optional` already modeled. Emitter reads source/target.
- **F: Ownership** — `.dag` has value semantics. Backend decides (Rust: clone/move, C: copy/refcount, Go: GC, Verilog: wire).
- **G: Boundaries** — nodes with ports but no body. Outputs must be provided
  externally. Replaces `extern func` keyword with a structural graph property.
  The checker verifies: every consumer of the boundary node's output ports is
  compatible, and a provider exists at link time.

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
| v2 type system | S85 (recursive types unmodeled) | First-class in resolved type IR: SCC-derived cycle metadata on TypeBinding |
| v2 graph model | S86 (keyword dispatch, not graph consistency) | Checker works on graph properties (ports, edges, shapes); keywords are parse sugar + validation, not dispatch |
