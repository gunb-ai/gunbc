# gunbc Roadmap

**Goal:** Self-hosted v2 compiler. The compiler is written in `.dag`, compiles
itself, and produces identical output when compiling itself again (fixed point).

**Thesis:** Explicit cause-and-effect relationships with basic primitives
(truth-valued structure, `Conj`/`Disj`, composition) are sufficient to express
any information concept. Named types are aliases for compositions; the compiler
should always be able to see through the name to the structure underneath.

---

## Current State (2026-03-20)

### What works

- **464 tests pass** — 368 daglang-emit + 96 v2-compiler-tests (12 ignored)
- **Generated crate compiles** — `v2_crate_cargo_check` passes in <7s
- **A4 ACHIEVED** — self-compiled output passes `cargo check` with 0 errors
  (error reduction: 7118 → 3917 → 58 → 0)
- **B3 Phase 1 complete** — TypedExpr (19 variants) eliminated. Expr carries
  `resolved_type: Node?` directly. TypedNode merged into Node. One AST
  instead of two — halves expression memory for self-compile.
- **A1 gist compile** — reconcile 0 errors, 30ms
- **Self-parse + self-resolve** — compiled v2 compiler tokenizes, parses, and
  resolves all 9 .dag modules with zero errors
- **R9 codegen ownership complete** — force_clone removed, V5 functional record
  update, SG-10 string comparison, type-directed clone, TCO param clone strip,
  fold-accum field extract via `Rc::make_mut`
- **Tokenizer ownership clean** — v1 emitter compiles helpers as returning `ScanResult`
  (single token), `tokenize_loop` is sole owner of token accumulator list
- **String operations are O(N) in current runtime** — char_at does O(1) byte read
  but callers convert to String for comparison (heap alloc per char). string_length,
  substring, scan_while all call is_ascii() O(N) per invocation. scan_while also
  heap-allocates per character even in ASCII path. See P0-d diagnosis and
  dsl/std/primitives.dag contracts for authoritative costs. P1a eliminates these
  by classifying value shapes so emission avoids String-for-codepoint entirely.
- **String params don't clone** — &str in generated code, Copy semantics
- **Node shrunk from ~544b to ≤176b** — transport/config boxed (176b test ceiling enforced)
- **Interpreter list_push is O(1)** — Arc COW via try_unwrap
- **TCO loops don't leak** — state moved, not cloned
- **Self-compile ratchets** — `self_compile_all_modules` runs by default (file
  count >= 9, all non-empty, source count >= 13, error ratchet <= 2700; needs tightening).
  `profile_self_compile` and `self_compile_cargo_check` are `#[ignore]` (opt-in
  via `--ignored`), not default CI ratchets
- **B3 Phase 2a contracts frozen** — DeclaredFuncSig/ResolvedFuncSig split,
  SCC-aware resolution, ResolvedGraph boundary type, retirement plans

### What's next

**P1a (v1 perf unblock), then P1b (v2 normalize stage), then Wave 1.**
A4 cargo check passes (0 errors), but the self-compiled binary cannot
self-compile — it hangs in the tokenizer. The root cause is not one bug
but a class of problems: the emitter applies blanket runtime gatekeeping
(Rc, stacker, String) instead of using information already available in
the typed AST. P1a fixes this within the v1 Rust emitter — no new IR,
no new pipeline stage. P1b builds the permanent v2 normalize stage
(EmitGraph, stable binding identity) after the perf gate is cleared.

**Performance journey (2026-03-14 → 2026-03-20):**

| Date | Bottleneck | Root cause | Fix |
|------|-----------|------------|-----|
| Mar 14 | Self-compile OOM | `node_type_deps` missed container deps → exponential type resolution | 2-line fix in reconcile |
| Mar 16 | Stack overflow | Recursive descent without stack growth | P0-a: stacker in v2 emitter |
| Mar 19 | Clone overhead / OOM | `force_clone` inflated all refcounts → O(N) every list_push | R9: V5, TCO clone strip, fold-accum extract |
| Mar 19 | Reconcile hang | O(N²) topo_resolve + detect_cycles | Kahn's algorithm, precomputed deps_map |
| Mar 20 | Tokenizer hang | Emitter representation: char-as-String, blanket Rc, blanket stacker, string_length O(N) | **P1a: v1 perf unblock** (emitter uses typed AST info it already has) |

Each previous fix was a localized bug. The tokenizer bottleneck is
structural — and the same representation issues will affect the parser,
reconciler, and emitter when they process 400K+ bytes of .dag source
during self-compile. Patching one phase at a time would continue the
pattern of "fix one bottleneck, reveal the next."

**P1a (v1 perf unblock)** is the immediate fix: the v1 emitter already
has typed AST nodes with scope, call graph, and type info. It needs to
use that information instead of applying blanket heuristics. No new IR
types, no new pipeline stage — changes entirely within `daglang-emit`
Rust code.

**P1b (v2 normalize stage)** is the permanent architecture: classify the
resolved graph's edges, produce a canonical EmitGraph with behavioral
facts and stable binding identity. Each backend's language model reads
the EmitGraph and makes target-specific rendering decisions. Requires
IR work (stable binding IDs or positional identity) that the current
codebase doesn't yet support. See P1a and P1b sections below.

```bash
# Verify current state:
cargo test -p v2-compiler-tests --lib --quiet     # 96 tests (12 ignored)
cargo test -p v2-compiler-tests v2_crate_cargo_check  # generated crate compiles
```

After P1a lands, retest self-compile. Then proceed to Wave 1.

### Baseline tests

```bash
cargo test -p daglang-emit --quiet               # 368 tests
cargo test -p v2-compiler-tests --quiet           # 96 tests (12 ignored)
cargo clippy --all-targets -- -D warnings         # clean
cargo test -p v2-compiler-tests v2_crate_cargo_check  # generated crate compiles
```

---

## Execution Model (2026-03-19)

### Critical path

**Critical path summary:** A1 (done) → R9 (done) → B3 Ph1 (done) →
A4 (done) → P0-a/P0-b (done) → **P1a (v1 perf unblock)** →
self-compile retest → **Wave 1** → **P1b (v2 normalize stage)** →
**B3 Ph2a** → A5 → A6 → A7

### Immediate next actions

1. **P1a (v1 perf unblock)** — changes entirely within `daglang-emit` Rust
   code. Uses existing typed AST to make per-function and per-binding
   decisions instead of blanket heuristics. No new pipeline stage, no new
   IR types. See P1a section for scope.
2. **Self-compile retest** — after P1a, verify the self-compiled binary
   can self-compile (`v2_crate_self_compile_cargo_check` in <5 min).
3. **Wave 1 (parallel lanes)** — reconcile/resolve/emit cleanup lanes,
   now unblocked by P1a's representation fixes.
4. **P1b (v2 normalize stage)** — new `04a_normalize.dag` module. Requires
   stable binding identity (positional IDs or scope-qualified names).
   Produces EmitGraph. This is the permanent architecture. See P1b section.
5. **Wave 2 (B3 Ph2a boundary)** — single ownership surface for
   DeclaredFuncSig/ResolvedFuncSig split, SCC resolution, ResolvedGraph
   boundary, retirement of `validate_no_unresolved` + `compile_sources_lenient`.
6. **SG-9 workaround revert** — after P1a confirms codegen fixes are
   sufficient, revert TokPos extraction and branch-aware use counting.

### Completed parallel lanes (2026-03-19)

All five A4 prep lanes implemented and merged:

| Lane | Branch | What was done |
|------|--------|---------------|
| M (measurement) | `wt/a4-measure` | `profile_self_compile` test: per-phase/per-module timing, RSS checkpoints via `mach_task_basic_info`, diagnostic counts, file size totals |
| R (codegen ownership) | `wt/a4-codegen-ownership` | Bug 1: `strip_tco_param_clones` post-pass strips `.clone()` from TCO params passed to non-TCO callees when not referenced later. Bug 2: `compile_fold_accum_field_extract` uses `std::mem::take(&mut Rc::make_mut(&mut acc).field)` for fold-accum field access across 7 intrinsics |
| P (tokenizer hedge) | `wt/a4-tokenizer-hedge` | New `ScanResult` type. All 6 helpers (`emit`, `scan_token`, `scan_ident`, `scan_number`, `scan_string`, `scan_str_cont`) return single token instead of accumulating. `tokenize_loop` sole owner of token list |
| T (ratchets) | `wt/a4-ratchets` | `self_compile_all_modules`: file count >= 9, all non-empty, source count >= 13, error ratchet <= 2700 (needs tightening). New `self_compile_cargo_check` + host-side `v2_crate_self_compile_cargo_check` tests |
| C (design) | `wt/b3-phase2a-design` | 4 boundary contracts frozen: DeclaredFuncSig/ResolvedFuncSig split, SCC-aware resolution, ResolvedGraph boundary type, retirement plans for validate_no_unresolved + compile_sources_lenient |

### Explicit deferrals

- full B3 Phase 2b Expr→Node pattern conversion — after Phase 2a
- B4 transport dissolution — after A4/A5
- C3/C4 emitter architecture work — after A7
- Result<T,E> — before A6, after A4 evidence

---

## R9 — Codegen Ownership (DONE)

### Done

- **V5 (functional record update):** Detects `Struct { f1: old.f1, f2:
  g(old.f2) }` and compiles to `Rc::try_unwrap(old) + std::mem::take +
  in-place assign`. Conservative: only fires when source type matches
  target, source is consumed, and ALL modified fields are List/Map (support
  Default for take). Debug assertion on sole ownership.
- **SG-10 (string comparison):** `.to_string()` stripped from Eq/Ne
  operands. `&str == &str` comparisons are now zero-allocation.
- **force_clone removed:** Replaced with type-directed clone (Rc-named,
  Rc-collection, match-bound). No more global flag inflating refcounts.
- **TCO param clone strip (Bug 1 fix):** `strip_tco_param_clones` post-pass
  after `lower_tco_plan`. Strips `.clone()` from call arguments when TCO
  param is not referenced in later statements — converts clone+move to
  direct move. Eliminates ~3.1GB clone overhead in tokenizer hot path.
- **Fold-accum field extract (Bug 2 fix):** `compile_fold_accum_field_extract`
  uses `std::mem::take(&mut Rc::make_mut(&mut acc).field)` to extract struct
  fields with refcount=1 before `Rc::try_unwrap`. Applied to all 7 mutating
  intrinsics (list_push, map_insert, concat, sort_by, replace_last, reverse,
  map_merge).
- **Tokenizer ScanResult refactor:** All 6 helpers return single `ScanResult`
  token instead of accumulating into a passed-in list. `tokenize_loop` is sole
  owner of token accumulator — no clone needed for helper calls.

### Remaining (cleanup, not blocking)

- **SG-11:** `stacker::maybe_grow(512KB, 2MB)` on every function — 530
  calls. Fix: only wrap genuinely recursive functions (reuse TCO detection).
- **SG-12:** Rc-wrapping Copy-sized types (SourceSpan = 16 bytes). Fix:
  extend `is_simple_enum` detection to small all-Copy structs.
- **Revert SG-9 workarounds:** After A4 evidence run, revert TokPos
  extraction and branch-aware use counting (may no longer be needed now
  that the codegen fixes handle the underlying ownership bugs).
- **Widen V5:** Currently limited to all-takeable modified fields. Extend
  to handle non-takeable fields (e.g., by substituting source ident with
  __owned in compile context).

### Performance Fallback Inventory

Every instance of `Rc::try_unwrap(x).unwrap_or_else(|rc| (*rc).clone())`
in the generated code is a **performance fallback**: correct on both paths,
but the clone path is O(N) and fires silently when refcount > 1. The
historical `force_clone` flag was the original reason these fallbacks fired
everywhere. That global flag is now removed, but the fallback mechanism still
exists and the remaining ownership bugs below still drive refcount > 1 on the
hottest self-compile paths.

**V5 policy:** new try_unwrap sites must use `expect("sole ownership")`
in debug/test builds. Release builds keep the fallback for safety. This
surfaces degradation as test failures instead of silent slowness.

**Terminal fix:** Track D cost algebra proves sole ownership statically.
`try_unwrap` with fallback is replaced by `Rc::into_inner` (panics on
failure) or direct moves. No runtime fallback exists.

#### Codegen sites (fn_codegen.rs → emitted into compiled .dag code)

| Site | Line | Operation | Status |
|------|------|-----------|--------|
| `rc_unwrap_stmts` | 4709 | Core template for all collection mutations | **Root pattern** |
| `list_push` | 2732 | `v.push(item)` | Always O(N) due to force_clone |
| `map_insert` | 2888 | `m.insert(k, v)` | Always O(N) due to force_clone |
| `append` | 2554 | `v.push(item)` (single-element) | Strips fold accum clone |
| `concat` fold | 2874 | `v.extend(other)` in fold bodies | Strips fold accum clone |
| `sort_by` | 2652 | `v.sort_by_key(f)` | Always O(N) due to force_clone |
| `replace_last` | 2674 | `*v.last_mut() = item` | Always O(N) due to force_clone |
| `reverse` | 2720 | `v.reverse()` | Always O(N) due to force_clone |
| `map_merge` | 2912 | `m.extend(other)` | Always O(N) due to force_clone |

#### Runtime sites (v2_runtime_shim.rs)

| Site | Line | Operation |
|------|------|-----------|
| `Concat for Rc<Vec<T>>` | 40-41 | try_unwrap both operands |
| `fold_list` | 114 | try_unwrap to iterate owned |
| `map_insert` | 129 | try_unwrap map for insertion |
| `map_merge` | 137-138 | try_unwrap both maps |

#### Root causes (FIXED 2026-03-19)

`force_clone` is removed. The two patterns that defeated try_unwrap have
been fixed:

| Pattern | Where | Fix | Status |
|---------|-------|-----|--------|
| Function-call clone | fn_codegen.rs `lower_tco_plan` | `strip_tco_param_clones` post-pass strips `.clone()` when TCO param not referenced later | **DONE** |
| Struct-field access | fn_codegen.rs intrinsic compilation | `compile_fold_accum_field_extract` uses `std::mem::take(&mut Rc::make_mut(&mut acc).field)` | **DONE** |

Additionally, the tokenizer was refactored (Lane P) so helpers return
`ScanResult` instead of receiving the token accumulator, eliminating
the clone path at the .dag source level.

#### .dag source workarounds (revert after A4 evidence run)

| Workaround | File | What it does | Status |
|-----------|------|-------------|--------|
| TokPos extraction | 01_tokenize.dag | Pulled `tokens` out of `TokenizerState` | Likely redundant after Lane P/R fixes |
| Branch-aware use counting | fn_codegen.rs | Max across branches instead of sum | Likely redundant after Lane R fixes |

### Acceptance

- [ ] `self_compile_all_modules` completes without OOM (evidence run pending)
- [ ] `self_compile_cargo_check` passes (emitted Rust compiles)
- [ ] SG-9 .dag workarounds reverted after evidence confirms they're redundant
- [ ] 464 tests pass, generated crate compiles clean

---

## Completed: A1 — Gist Compilation

Full pipeline (tokenize → parse → resolve → reconcile → emit) runs on gist
sources with 0 reconciler errors in 30ms. Emitted Rust compiles with
`v2_crate_cargo_check`.

---

## Invariant Audit (2026-03-18)

### Fixed in this session

| # | Violation | Fix |
|---|-----------|-----|
| V1 | Duplicate builtin name lists (reconciler + emitter) had diverged (`concat` missing from emitter) | Eliminated both lists. Reconciler bridges Call→MethodCall using `infer_method_call_type_node` (now returns `Node?`). Emitter handles `concat` as free function in `emit_typed_method_call`. |
| V2 | Dead `TypecheckAccum` type after fold→recursion rewrite | Deleted. |
| V3 | Dynamic field access fabricated `leaf_node(name: field_name)` — named types from field names | Changed to `leaf_node(name: "Dynamic")` — propagates "unknown" honestly. |
| F1 | `leaf_node(name: "Unknown")` catch-all fabrication in `infer_expr` | Changed to `leaf_node(name: "<error:unhandled>")` — can't be mistaken for real type. |
| F-fab | 5 error-path fabrication sites used variable names as types (`leaf_node(name: name)`, `leaf_node(name: func_name)`, etc.) | All changed to `<error:reason>` markers. Added missing diagnostic for unknown record type names. |
| F-range | `range(min: 1, max: 5)` parsed but min/max values discarded (pre-existing debt) | Parser now encodes min/max as structured RecordLit in the FieldInit value. |
| F-svc | Service namespace recovered by dot-splitting string heuristic (S86 debt) | Parser stores namespace root as FieldInit property. Reconciler reads property directly. `find_dot_index` deleted. |

### Tracked for R9 (verification + cleanup)

| # | Violation | Status |
|---|-----------|--------|
| V5 | `force_clone` blocks all single-use moves (SG-4) | **DONE** — functional record update with debug assertion |
| SG-10 | String comparisons heap-allocate both sides | **DONE** — `.to_string()` stripped from Eq/Ne |
| SG-9 | .dag workarounds for force_clone (TokPos, fold_accum, branch counting) | Revert after verification at scale |

### Tracked for future work

| # | Violation | When | Notes |
|---|-----------|------|-------|
| F2 | `ItemInfo.kind` is String (`"fn"`, `"func"`, `"other"`) | B3 Phase 2 | Should be a closed enum `ItemKind = Fn \| Func \| Other`. Adding a kind = updating match arms (feature, not cost). |
| F3 | `SemanticsCtx` uses all-String fields (backend, exec_model, etc.) | **DONE** | Fixed: `SemanticsCtx` fields are now closed enums (`BackendKind`, `ListModel`, `MapModel`, `StringModel`) in `07_complexity.dag:106-120`. |
| F4 | `PrimCost.op: String, model: String` in 07_complexity.dag | **DONE** | Fixed: `PrimCost.op` is now `PrimOp` (closed enum, `07_complexity.dag:51-60`), `ctx` is `SemanticsCtx` (closed enum). |
| B4-val | `validate_no_unresolved()` is post-hoc validation (Invariant 9) | B3 | Already tracked. Delete when pipeline boundary types make unresolved structurally unrepresentable. |
| F5 | `infer → reconcile` rename lacks contract justification | Documentation | Rename already done. Needs documented rationale tied to contract change (reconciliation = bidirectional, not just inference). |
| F6 | `05_emit_rust.dag` re-discovers structural facts through string heuristic lists (`known_opt_fields`, `types_with_value_field`, `known_struct_with_accessor_field`, `is_rc_exclude`) | Post-B3 emitter cleanup | The reconciler/boundary already knows these facts structurally. Push them through metadata or type summaries instead of maintaining name lists in the emitter. |
| F7 | `emit_typed_method_call` in `05_emit_rust.dag` is a growing string-dispatch ladder for special lowerings | Post-B3 emitter cleanup | Keep the fallback `.method(...)` path, but move special lowerings behind a clearer lowering table / metadata boundary so adding one lowering does not require editing a long `if method == ...` chain. |

---

## Completed: R8 — Rc-Wrap Generated Types (temporary bootstrap convergence)

**Status:** R8 was the bootstrap convergence strategy that made A4 possible.
It is **superseded by P1a/P1b**. R8 applied blanket Rc wrapping to all non-Copy
types — correct for convergence but exactly the kind of runtime gatekeeping
P1a eliminates. Per-binding consumer analysis determines ownership:
single semantic consumer → bare move (no Rc). Shared → Rc. R8's blanket
rule becomes a fallback that P1a progressively replaces.

**Original structural principle (for historical context):** The DAG language
has value semantics with no mutation. Every value is logically shared — using
a value twice doesn't require two copies. The codegen maps DAG values to
Rust ownership, where using a value twice requires `.clone()`. R8 chose
uniform Rc wrapping as the simplest correct mapping.

```text
DAG value semantics          R8 mapping (bootstrap)    P1a mapping (per-binding)
─────────────────           ─────────────────────     ────────────────────────
String                  →    &str param (Copy)         same
Int, Bool               →    i64, bool (Copy)          same
List<T>                 →    Rc<Vec<T>>                bare Vec<T> when sole owner
Map<K,V>                →    Rc<HashMap<K,V>>          bare HashMap when sole owner
struct Node { ... }     →    Rc<Node>                  bare Node when 1 semantic consumer
enum TokenKind { ... }  →    Rc<TokenKind>             bare when 1 semantic consumer
```

**R8 rule (active until P1a lands):** Every non-Copy generated type is
Rc-wrapped at all usage sites. `compile_ident` can emit `.clone()` freely
on any variable — it's always O(1). No type-specific checks, no special
cases. **P1a replaces this with per-binding decisions based on semantic
consumer count from the typed AST (v1) or EmitGraph (P1b, v2).**

**Types excluded from Rc-wrapping** (pure tag enums, already Copy):
Connective, BinOpKind, UnaryOpKind, OperationModifier, RenderTarget, Severity,
Certainty. The codegen already detects these (`is_simple_enum` → derives Copy).

### Callsite migration

**type_codegen.rs** (the structural change — one predicate, applied uniformly):

- **TC-1:** `type_expr_to_rust_with_registry` Named type resolution (~line 150).
  Thread a `HashSet<String>` of non-Copy generated type names. Emit `Rc<T>`
  instead of `T` for matching names. This single change controls field types
  in struct/enum definitions AND function signatures.
- **TC-2/TC-4/TC-5:** `typedef_to_code_ir` and `format_variant` field rendering.
  Inherits from TC-1 — field types become `Rc<T>` automatically.
- **TC-3:** `typedef_to_code_ir_boxed` Box-wrapping. Skip Box for Rc-wrapped
  fields — Rc already heap-allocates, breaking cycles. Extends the existing
  `TypeExpr::Generic` exclusion to Named types in the Rc set. This eliminates
  all current boxed fields (return_type, body, type_annotation, transport,
  config) since their types are all Rc-wrapped.
- **TC-6/TC-7:** `fndef_to_code_ir` param and return type rendering. Inherits
  from TC-1.

**fn_codegen.rs** (construction, matching, field access):

- **FN-1:** `compile_struct_field_value` (~line 1020). When target field is
  `Rc<T>` and value is freshly constructed `T`, wrap in `Rc::new(...)`.
  Mirrors existing Box-wrapping logic.
- **FN-2:** `compile_match_typed` (~line 3427). When scrutinee type is
  Rc-wrapped, emit `match &*x { ... }` instead of `match x { ... }`.
  Bindings become references; field access through Deref still works.
- **FN-3/FN-4:** Box deref logic (`collect_boxed_deref_stmts`, field access
  deref). Remove for fields that are Rc-wrapped (Deref handles automatically).
  Keep for any remaining Box-wrapped fields.

**render_rust.rs:**

- **RR-2:** `render_match`. If deref is inserted at IR level (FN-2), no
  change needed. Otherwise, insert `&*` on Rc-typed scrutinees.

**v2_runtime_shim.rs:**

- **RT-1:** `index_by` closure receives `&Rc<V>` instead of `&V`. Auto-deref
  handles field access in closures — verify but likely no change needed.

**v2_crate_emit.rs:**

- **V2-2/V2-3:** Hardcoded types (`SourceSpan`, `BindingPower`, `FilePath`)
  are NOT Rc-wrapped — they're small, Copy-compatible, not generated from .dag.

### Key simplification: Box-wrapping becomes unnecessary

Once all generated types are Rc-wrapped, `compute_recursive_fields` returns
empty — Rc already breaks all cycles. The R2 boxing of Node.transport/config
becomes redundant. The entire Box-wrapping infrastructure can be simplified
or removed after R8 lands.

### Acceptance

- [ ] `v2_crate_gist_resolve` passes in release mode in <60 seconds
- [ ] memory usage for gist resolve drops below 500MB
- [ ] all 464 tests pass
- [ ] generated crate compiles clean

---

## Completed Work

### Track R: Representation + Runtime (ALL DONE)

| Item | What | Impact |
|------|------|--------|
| R1 | Type size assertions in generated tests | Node ≤ 176b, Expr ≤ 800b enforced |
| R2 | Box Node.transport + Node.config | Node: ~544b → ~120b |
| R3 | String params → &str in generated code | Eliminates ~4GB string clone traffic |
| R4 | Interpreter list_push Arc COW | O(1) amortized append (30 files) |
| R5 | TCO clone leak fix | Loop state moved, not cloned |
| R6 | O(1) ASCII string intrinsics | char_at, string_length, substring, scan_* |
| R7 | Kernel primitive complexity contracts | dsl/std/primitives.dag |

### Track S: Stabilization (SUBSTANTIALLY DONE)

- **S1 (partial):** Parser builds Nodes directly; TypeExpr functions deleted
  from 04_infer.dag. Remaining: TypeExpr definition + helpers in 00_core.dag
  (see Blocker 3).
- **S4:** 96 v2-compiler-tests pass, v2_crate_cargo_check passes. Generated
  crate compiles. Gist resolve no longer OOMs.
- **S2/S3:** Emit hot paths typed, list builders use O(1) push, Kahn improved.

### Other completed items

- **B2:** `04_typecheck.dag` → `04_infer.dag` → `04_reconcile.dag`
- **C1:** `LanguageSpec` interface in `dsl/std/languages.dag`
- **C2:** Rust, Python, Go language extdeps in `dsl/extdeps/languages/`
- **D1:** Cost algebra types in `dsl/std/complexity.dag`
- **E0:** Monolith artifact wrapper defined
- **P0:** Stack overflow mitigated via stacker at re-entrant call sites
- **Streams 2+3:** PortContract dissolved, Shape → Connective, dead code removed
- **B3 Phase 1:** TypedExpr eliminated (19 variants + 5 helper types deleted).
  Expr carries `resolved_type: Node?`. TypedNode merged into Node. Self-call
  walkers rewritten. ~770 LOC across 8 .dag files.
- **Node convergence (partial):** Field/Param/ResourceUse/FuncSig type fields
  are Node; all three emitters have node-based readers

---

## Design Decisions (Decided)

### Blocker 1: Emitter walk triplication → Decision: A (shared walk with callbacks)

Deferred until v2 self-hosted. Requires function-as-data, available in v2 but
not v1 bootstrap. Until then, the bounded duplication (3 backends) is accepted.

### Blocker 2: Fabrication-on-error → Decision: A (add Result<T, E> to DSL)

The structural fix. Eliminates fabrication by making error paths return
`Err(diagnostic)` instead of `(fabricated_node, diagnostic_list)`.

**Design (decided 2026-03-18):** Result<T,E> is not a new mechanism — it's
what the Node model already does. `List<T>` is `Node { name: "List",
children: [T] }`. `Result<T,E>` is `Node { name: "Result", children:
[Ok_variant, Err_variant], connective: Disj }`. Generic type parameters
are structural holes in an anonymous DAG definition, filled at instantiation.

**Staging:**
- **Before A6:** Special-case Result like Option (hardcode Ok/Err in parser
  and reconciler). This unblocks structurally sound error handling.
- **After A7 (deferred):** General generic syntax (`type Foo<T> = ...`).
  Migrates Option and Result from special cases to the general mechanism.
  Requires v2 self-hosting (parser must parse its own parameterized types).

### Blocker 3: Delete TypeExpr from 00_core.dag → Scheduled for next tasks

Mechanical work: trace last callers of `field_to_node`/`variant_to_node` in
daglang-emit, migrate them, delete ~300 lines. Parallelizable with R8.

### Blocker 4: Node conflates resolved/unresolved → Deferred to B3

`validate_no_unresolved()` violates Invariant 9 (correctness by construction,
not by validation). The validation pass is marked for deletion when B3
(Expr→Node) reworks pipeline boundary types — that's the natural moment to
make resolved-vs-unresolved a type distinction rather than a runtime check.

**Concrete contracts frozen in B3 Phase 2a** (see that section for details):
- Contract 1 splits `DeclaredFuncSig` / `ResolvedFuncSig` so fabricated
  placeholders are structurally impossible at emit time.

### Blocker 5: Unicode/ASCII boundary mismatch → Decision: Unicode everywhere in the language, explicit adaptation at boundaries

The language contract is now fixed:

- `.dag` source is UTF-8
- strings are Unicode
- identifiers are Unicode
- `char_at`, `string_length`, `substring`, and `scan_*` operate on
  Unicode scalar values, not bytes

Interop contract:

- `SourceSpan` stays UTF-8 byte offsets for diagnostics/tooling
- ASCII-only target surfaces are adapters, not the language contract
- string data must be preserved exactly; if a boundary cannot carry
  Unicode directly, it must use an explicit encoding or return a clear
  error
- emitted identifiers may be mangled to backend-safe ASCII, but the
  mangle must be deterministic and injective

What this avoids:

- no silent ASCII downgrade
- no backend-specific Unicode semantics
- no late runtime panic on valid UTF-8 source just because a backend
  used byte indexing internally

Practical implication:

- Rust, Go, and Python backends must agree on Unicode-scalar string
  semantics
- the tokenizer/runtime split must be explicit: scalar indices for
  string operations, byte offsets for source spans
- direct native byte indexing/slicing is not valid lowering for
  language-visible string operations
- Contract 2 resolves return types in SCC topological order so callers
  never observe placeholder return types.
- Contract 3 introduces `ResolvedGraph` / `ResolvedFuncEnv` as the
  reconcile-to-emit boundary type.
- Contract 4 retires `validate_no_unresolved()` and
  `compile_sources_lenient()` once Contracts 1-3 land.

---

## Track A: Self-Hosting

**Dependencies:** R8 → A1 (done) → R9 (done) → B3 Ph1 (done) → A4 (done) → P0-a/b/c (done) → **P1a (v1 perf unblock)** → **Wave 1** → **P1b (v2 normalize)** → **B3 Ph2a** → A5 → A6 (Blocker 2) → A7

### A1: Gist compilation

Feed `gist.dag` and transitive dependencies through the v2 pipeline.

**Acceptance:**
- [ ] `v2_crate_gist_resolve` passes in release mode in <60s
- [ ] `v2_compile_gist_rust`: v2 compiles gist → Rust → `cargo check`

### A2: Runtime bridge

Generate entry point and runtime dependencies so compiled gist executes.

**Acceptance:**
- [ ] generated `main.rs` + `Cargo.toml` with runtime deps
- [ ] `cargo run -- gist --dry-run` produces correct output

### A3: Gist end-to-end

**Acceptance:**
- [ ] compiled Rust gist creates a real GitHub gist (manual gate)

### A4: Full self-compile pipeline

Extend self-compile from tokenize/parse/resolve to full pipeline including
infer and emit.

**Scope note:** A4 was initially exercised through the lenient bootstrap path.
The lenient path is a temporary convergence strategy (like R8) that B3 Phase 2a
retires before A5. The bootstrap chain targets `compile_sources` (strict).

**Acceptance:**
- [x] instrumented `self_compile_all_modules` reports per-phase and per-module
      timing with source/file/diagnostic counts (Lane M)
- [x] v2 crate processes its own .dag source through the full pipeline on the
      current bootstrap path (self-compile completes, 0 cargo check errors)
- [x] emitted Rust files compile with `cargo check` (0 errors as of 2026-03-20)
- [x] no OOM or stack overflow on any .dag file up to 4000 lines (Lane R fixes
      merged, codegen ownership fixes confirmed sufficient)
- [x] self-compile ratchet asserts semantic properties stronger than
      "non-empty file emitted" (Lane T)

### PERF gate: Self-compiled binary must not hang

The self-compiled binary must complete a level-3 self-compile (or at least
not hang). Without this, A5 bootstrap is impossible.

**P0-a (DONE):** v2 emitter wraps recursive non-TCO functions in
`stacker::maybe_grow`. Only genuinely recursive functions are wrapped
(better than v1's blanket SG-11 wrapping).

**P0-b (DONE):** v2 emitter derives `Copy` for simple enums (all unit
variants), avoiding unnecessary Rc wrapping and clone overhead.

**P0-c (DONE):** Reconcile O(N²) bottleneck fixed. `topo_resolve_types`
and `detect_type_cycles` replaced with precomputed deps_map and Kahn's
algorithm. 96/96 tests pass.

**P0-d (DIAGNOSED — blocked on P1a):** Self-compile still hangs. Profiling
shows the tokenizer (phase 1 of 5) never completes on ~400K chars of .dag
source. Root cause is not the tokenizer algorithm — it's the emitter's
representation model:

1. **char-as-String:** `char_at()` returns `String` (heap alloc per byte).
   Character predicates (`is_digit`, `is_ident_char`) do `ch.to_string()
   >= "0".to_string()` — 4+ heap allocs for a byte comparison. ~50 heap
   allocs per identifier token × ~10K tokens.
2. **string_length O(N):** `string_length(s)` calls `s.is_ascii()` which
   scans all bytes. Called 2-3× per token on the 200KB source string.
   Total: ~5 billion byte scans for bounds checking alone.
3. **blanket stacker (v1 only):** v1 emitter wraps ALL functions in
   `stacker::maybe_grow`, including leaf predicates called per character.
   Triple-nested stacker for `is_ident_char → is_ident_start + is_digit`.
4. **blanket Rc wrapping:** `Rc<TokPos>`, `Rc<ScanResult>`, `Rc<Token>`,
   `Rc<TokenKind>` — all created and immediately consumed. ~4 Rc heap
   allocs per token for values used exactly once.

These same issues affect parser, reconciler, and emitter during
self-compile — the tokenizer just hits the wall first because it processes
raw bytes. Patching one phase at a time would continue the "fix one
bottleneck, reveal the next" pattern from the last week. P1a is the
immediate fix; P1b is the permanent architecture.

**Acceptance:**
- [ ] `v2_crate_self_compile_cargo_check` completes in <5 minutes
- [x] 96+ tests pass, generated crate compiles clean

---

## P1 — Emission Representation (split: P1a perf unblock + P1b normalize stage)

### Problem

After reconcile, the resolved graph is **truthful but noisy**. Every
edge looks the same — but edges mean different things: destructive
consumption, read-only access, TCO threading, field projection,
pass-through to helpers. The current pipeline collapses all of this
into `rc_wrapped_types: Map<String, Bool>` — a per-type-name blanket
decision. Emission then re-derives semantic facts from raw structure
(or doesn't, and emits redundant runtime gatekeeping).

The result: every character comparison heap-allocates a String, every
struct wraps in Rc even when consumed once, every function gets a stack
guard even when it's a leaf. These aren't missing optimizations — they
are runtime re-checks of facts the graph already proved.

This is really two problems at different scopes:

- **P1a (v1 perf unblock):** The v1 emitter already has typed AST nodes
  with scope, call graph, and type info. It applies blanket heuristics
  instead of using this information. The fix is within existing Rust
  emitter code — no new IR types, no new pipeline stage.

- **P1b (v2 normalize stage):** The permanent architecture requires a
  new compiler stage + representation layer (EmitGraph, normalize phase,
  language model). The current IR (`Node` in `00_core.dag:308`) has no
  stable binding ID — only `name: String` and `span: SourceSpan`.
  Reconcile fabricates 22+ zero-span nodes via `leaf_node()`,
  `optional_node()`, `container_node()`. There are no def-use links.
  This IR work is a prerequisite for P1b, not for P1a.

### Principle: canonicalization, not optimization

The graph's purpose is **static intent analysis** — cardinality
constraints, set algebra, data flow proofs. All of that gatekeeping
is for the graph layer. By the time we reach emission, the proofs are
done. Emission should not need to redo them at runtime.

Every piece of runtime overhead in current generated code is a runtime
check that duplicates what the graph already proved:

| Runtime overhead | Re-checks at runtime | Graph already proved |
|---|---|---|
| `Rc::try_unwrap` | "Is refcount 1?" | Binding has one semantic consumer |
| `stacker::maybe_grow` | "Will I overflow?" | Function is a leaf |
| `String` for code points | "How long is this?" | Value is exactly one code point |
| `Vec<String>` + `join` | "Keep intermediates" | Intermediate never observed |

P1a uses the typed AST to eliminate these within the v1 emitter. P1b
makes the same facts first-class in the IR so all backends can use them.

The design rule: **if emit has to infer semantics from raw structure,
the boundary is wrong.** P1a enforces this within v1 Rust code; P1b
enforces it structurally across all backends.

### P1a: v1 perf unblock

Changes entirely within `daglang-emit` Rust code. The v1 emitter already
has typed AST nodes with scope, call graph, and type info from reconcile.
It needs to use that information instead of applying blanket heuristics.

**Scope:**
- Call graph SCC → classify functions (leaf / interior / recursive / TCO)
- Remove stacker from leaf functions
- Use type info to avoid Rc on single-consumer structs
- Use char_at return position to emit byte comparison instead of String
- Cache string_length outside loops

**No new IR types.** No new pipeline stage. No stable binding IDs needed.
The v1 Rust emitter already resolves identity from the typed AST — it
has scope, call targets, and type info. P1a uses what's already there.

**Concrete deliverable:** self-compile tokenizer completes.

**Files modified (P1a only):**

| File | Changes |
|------|---------|
| `src/v1/07_emit/daglang-emit/src/fn_codegen.rs` | Call graph SCC, function classification, per-binding consumer analysis |
| `src/v1/07_emit/daglang-emit/src/v2_runtime_shim.rs` | `char_at` → byte variant; length caching |
| `src/v1/07_emit/daglang-emit/src/render_rust.rs` | Gate stacker/inline/Rc on function and binding classifications |

**P1a acceptance criteria:**

- [ ] `v2_crate_self_compile_cargo_check` completes in <5 minutes
- [ ] All existing tests pass (96 v2-compiler-tests, 368 daglang-emit)
- [ ] Generated `tokenize.rs` verifiable:
  - [ ] No `stacker::maybe_grow` on leaf functions
  - [ ] No `.to_string()` in character predicate comparisons
  - [ ] No `v2_rt::string_length` inside `tokenize_loop` body
  - [ ] Single-semantic-consumer structs not `Rc`-wrapped (planned — requires P1b per-binding consumer analysis)
  - [ ] `scan_string_body.acc` is `String`, not `Vec<String>` (planned — requires build-reduce rewrite)

---

### P1b: v2 normalize stage (after P1a, before A5)

**Prerequisite:** IR work to support stable binding identity. The
current IR (`Node` in `00_core.dag:308`) has no stable binding ID —
only `name: String` and `span: SourceSpan`. Reconcile fabricates 22+
zero-span nodes via `leaf_node()`, `optional_node()`, `container_node()`.
There are no def-use links. Per-binding consumer counting from the
current graph would require re-resolving scope/identity first.

P1b requires either positional IDs or scope-qualified names before the
EmitGraph can be constructed. This is IR work that does not affect P1a.

### What we have after reconcile (P1b context)

The full resolved program graph — every binding, every producer, every
use site, every function body, every call target, every resolved type,
every loop/TCO/fold boundary, every metadata fact reconcile learned.

That is enough. The information is not missing.

But it is **undifferentiated**. A raw edge might mean:

- "this value is destructively consumed"
- "this value is only read"
- "this value is threaded into the next loop iteration"
- "this is a field projection"
- "this intermediate escapes its scope"
- "this intermediate does not escape"

These are very different for emission, but they all look like "an
edge exists."

### What P1b produces: EmitGraph

The job: **classify the graph's structure correctly, discard or relabel
administrative noise, surface the behavioral facts emission needs.**

The output is an **EmitGraph** — a derived canonical view (the full
`ResolvedGraph` is preserved for diagnostics/proofs/debugging).

**Per binding — behavioral facts:**
- Edge kind: **consumed** / **read** / **threaded** / **projected**
- Semantic consumer count (administrative edges excluded)
- Escape: does the value leave its defining scope?
- Accumulator: is this a loop accumulator?
- Loop-invariant: is this value constant across iterations?
- Value shape: single element vs general collection/text
- Materialization: must the intermediate exist, or is it internal
  to a build-reduce step?

**Per function:**
- Call position: leaf / interior / recursive / TCO
- Param classification: semantic input vs threaded state
- Stack growth relevance

**Per region (loop/fold/TCO body):**
- Build-reduce pairings
- Invariant computations
- Materialization boundaries

**Hard invariant:** The EmitGraph records only behavioral facts about
the computation. It never records target concepts (Rc, &mut, char,
StringBuilder, stacker). If any fact in the EmitGraph mentions a target
language, it is wrong.

**No-fallback rule:** Every non-minimal classification must have a
**forcing witness** — a specific edge or use site that forces the wider
classification. "Shared because we didn't analyze it" is a compiler
bug. "Shared because binding X is consumed at lines 42 and 67" is
correct.

**Witness compactness:** Witnesses are compact by default — only the
minimum needed to prove non-minimality. "Shared" needs 2 consume-site
IDs. "General text" needs 1 widening use. "Materialized" needs 1
escape witness. Full witness expansion (all sites) is lazy/debug-only.
This keeps the proof story strong without witness storage becoming the
new memory tax.

### How backends use the EmitGraph

The EmitGraph says **what happens to each value**. Each backend brings
**its own knowledge of its target language**. The rendering is the
intersection:

```
scan_string_body.acc:
  EmitGraph says: accumulator, build-by-append, single terminal
                  consumer, intermediate not observed, reduces to text

  Rust knows:  String is mutable       → String + push_str
  Java knows:  String is immutable     → StringBuilder + append
  Go knows:    string is immutable     → strings.Builder
  Python knows: str concat is O(n²)    → list + "".join()
  PSPICE:      accumulator + reduction → summing junction

is_digit.ch:
  EmitGraph says: read-only, single-element text, no mutation

  Rust:  u8, byte comparison
  Java:  char, char comparison
  Go:    byte, byte comparison
  PSPICE: single-bit-width signal line

tokenize_loop.source:
  EmitGraph says: read-for-duration, never consumed, threaded (admin)

  Rust:  &SourceRef (borrow)
  Java:  SourceRef (GC, immutable ref is fine)
  Go:    *SourceRef
  PSPICE: constant input port
```

Adding a new backend (Java, PSPICE) does not change the EmitGraph
layer. The backend brings its own rendering knowledge. The behavioral
facts are stable across all targets.

### Concrete trace: `01_tokenize.dag`

**`is_digit.ch`** — Raw graph: 2 edges, both comparisons. Classified:
2 read edges, 0 consume edges. Value shape: single-element (all callers
pass `char_at()` result). Function: leaf (no .dag calls). EmitGraph
says: read-only single-element text, leaf function. Rust renders
mechanically: `#[inline] fn is_digit(ch: u8) -> bool { ch >= b'0' }`.

**`scan_string_body.acc`** — Raw graph: 3 recursive edges (list_push),
3 terminal edges (join). Classified: accumulator (build-by-append in
TCO body), terminal consumer (join reduces to text), intermediate never
escapes. EmitGraph says: build-reduce, intermediate not materialized.
Rust renders mechanically: `String` + `push_str`, no `join` needed.

**`tokenize_loop.source`** — Raw graph: 5 edges. Classified: 1 TCO
threading (administrative), 2 pass-through reads to helpers, 2 field
projections. Semantic consumers: 0 destructive. EmitGraph says:
read-for-duration, never consumed. Rust renders mechanically:
`&SourceRef`.

**`tokenize_loop.tokens`** — Raw graph: 3 edges. Classified: 1 append
(list_push — accumulator), 1 TCO threading (administrative, IS the
list_push result), 1 terminal (returned in struct). Semantic consumers:
1 per iteration. EmitGraph says: sole-owner accumulator. Rust renders
mechanically: `Vec<Token>`, no Rc, push in place.

### Where this lives in the pipeline

```
tokenize → parse → resolve → reconcile → normalize → language model → emit
                                            ↑              ↑              ↑
                                     ResolvedGraph    EmitGraph      RenderPlan
                                     (truthful,       (canonical,    (target-
                                      noisy)           classified)    specific)
```

Normalization reads the full ResolvedGraph and produces a derived
EmitGraph. The language model (part of the language extdep) reads
the EmitGraph and produces target-specific rendering decisions.
Emission prints the RenderPlan mechanically.

**EmitGraph is an ID-keyed canonical view**, not a second graph. It
attaches normalized behavioral facts and witnesses to stable binding/
function identifiers. No topology cloning — just a derived view over
the existing structure. This avoids memory churn and boundary drift.

**IR prerequisite:** The current IR has no stable binding ID — only
`name: String` and `span: SourceSpan`. Reconcile fabricates zero-span
nodes without unique identity. Before P1b can construct an ID-keyed
EmitGraph, either positional IDs or scope-qualified names must be added
to the IR. This is P1b prerequisite work, not needed for P1a (which
uses the v1 typed AST where identity is already available).

The ResolvedGraph is preserved for diagnostics, proofs, and debugging.
The EmitGraph REPLACES `EmitGraphInfo.rc_wrapped_types` and similar
blanket maps.

### Complexity guarantees

Normalization is O(N) with all reference checks via map lookup (O(1) amortized
for HashMap in v1 Rust, O(log N) for BTreeMap in v2 emitted code — the O(N) total
holds either way since log factors are absorbed).

| Walk | What it does | Visits | Complexity |
|------|-------------|--------|------------|
| Call graph + SCC | Build adjacency list, Tarjan's | Each function + call edge once | O(F + E) |
| AST walk | Classify edges, record producers, track scope | Each AST node once | O(N) |
| Call-site producers | Collect what callers pass for each param | Each call site once | O(C) |
| TCO admin edges | Mark re-passed params as administrative | Each TCO recursive call | O(F_tco × P) |
| **Total** | | | **O(N)** |

For self-compile scale (N ≈ 100K nodes, F ≈ 660 functions, C ≈ 2000+
call sites): linear and comfortably below parse/reconcile cost. Actual
measured time to be ratcheted after implementation — allocation and
cache behavior affect constants. No walk visits all nodes for each
node. No quadratic loops.

**Implementation note:** Hot normalization data (per-binding facts,
per-function classifications, edge labels) should use dense integer
IDs and side tables (`Vec`/`IndexVec`/bitsets) for cache locality and
low allocation overhead. Reserve map lookups for boundary operations
(name→ID resolution at EmitGraph construction). v1 Rust uses HashMap
(O(1) amortized); v2 emitted code currently uses BTreeMap (O(log N))
— acceptable since log factors don't change the O(N) total, but
sub-millisecond constant-factor claims apply to v1 only. This also
gives more deterministic behavior for A6 fixed-point comparison.

**Invariant:** Every node is visited a bounded constant number of times.
Every reference check is a map lookup — O(1) amortized in v1 (HashMap),
O(log N) in v2 (BTreeMap), or indexed O(1) for dense tables. Producer
tracking is one level deep (direct producers only); language model does
bounded lookups for deeper tracing.

### Relationship to Track D

P1a/P1b's edge classification (consumed vs read vs threaded) and Track D's
ownership proofs are **one analysis at two maturity levels**. P1a/P1b
classify edges and surface behavioral facts. Track D strengthens
the same classifications into proof obligations that eliminate
`try_unwrap` runtime fallbacks entirely. Same edges, same
classifications — P1b is the front half, Track D is the back half.

### Boundary contracts (pull-forward from Phase 2a)

The **type shells** for `DeclaredFuncSig`, `ResolvedFuncSig`, and
`ResolvedGraph` should be frozen now, before Wave 1. This prevents
cleanup from accidentally deepening the old permissive shape.

**Boundary invariant:** If metadata is learned during reconcile and
used by emit, it must cross the boundary as data in the EmitGraph,
not be rediscovered by name heuristics.

### Relationship between P1a and P1b

P1a is a v1-only change within the existing Rust emitter. It uses
information already available in the typed AST (scope, call graph, type
info) to make per-function and per-binding decisions. It unblocks the
PERF gate without new IR types.

P1b is the permanent v2 architecture. It adds a new compiler stage
(`04a_normalize.dag`), requires IR work (stable binding identity), and
produces a canonical EmitGraph that all backends consume. P1b cannot
proceed until the IR prerequisite is resolved.

The v1 perf unblock (P1a) works because the v1 emitter already has
identity: the typed AST from reconcile gives it scoped bindings, call
targets, and resolved types. P1a teaches the pattern that P1b makes
structural.

| Component | P1a (v1 Rust) | P1b (v2 .dag) | Survives A7? |
|-----------|---------------|---------------|--------------|
| Function classification | `fn_codegen.rs` call graph SCC | `04a_normalize.dag` | **v2 version** survives |
| Edge classification | `fn_codegen.rs` consumer analysis | `04a_normalize.dag` | **v2 version** survives |
| Language model | `render_rust.rs` pattern matching | Per-language extdep | **v2 version** survives |
| Rendering tables | `fn_codegen.rs` | `05_emit_rust.dag` | **v2 version** survives |
| Other backends | Not needed for bootstrap | `05_emit_go/py.dag` | **v2 version** survives |

### P1b implementation stages

Three stages (P1b only — P1a is simpler, see P1a section above).
Stage 1 is a serial gate. Stage 2 runs three independent analyses in
a single AST walk. Stage 3 post-processes into EmitGraph.

```
Stage 1 (serial)          Stage 2 (parallel, one walk)       Stage 3 (post)
┌───────────────┐    ┌──────────────────────────────────┐    ┌──────────────┐
│ Call graph SCC │───▶│ 2a: Edge classification          │───▶│ Semantic     │
│ → function    │    │ 2b: Value shape (two-level)      │    │ consumer     │
│   class.      │    │ 2c: Loop invariance              │    │ count +      │
└───────────────┘    └──────────────────────────────────┘    │ build-reduce │
                                                             │ + witnesses  │
                                                             └──────────────┘
```

**Stage 1: Function classification** (serial gate)
- Build call graph SCC, classify every function as leaf / interior /
  recursive / TCO
- Separate semantic params from threaded state in TCO functions
- **Gate:** Stage 2 reads function classifications. Cannot parallelize.
- **Acceptance:** `is_digit`, `is_ident_start`, `emit` classified Leaf.
  No stacker on any Leaf. No function left unclassified.

**Stage 2: Single AST walk** (three independent analyses, one pass)

After Stage 1, these three analyses read function classifications but
are independent of each other — they can execute in a single combined
walk over the AST:

- **2a: Edge classification**
  - Classify every edge as consumed / read / threaded / projected
  - Tag administrative edges (TCO threading, pass-throughs) for removal
  - **Acceptance:** `tokenize_loop.source` has 0 consume edges, classified
    read-for-duration. `ScanResult`, `TokPos` have 1 semantic consumer
    where used once — not Rc-wrapped.

- **2b: Value shape (two-level)**
  - Level 1: **one Unicode scalar** vs **general text** (semantic,
    language contract remains Unicode scalars per Blocker 5)
  - Level 2: **ASCII-proven** vs **arbitrary scalar** (source-cursor-local;
    the language model chooses `u8`/`byte` only on ASCII-proven paths)
  - **Acceptance:** `is_digit.ch` classified one-scalar + ASCII-proven.
    No `.to_string()` in character predicates.

- **2c: Loop invariance**
  - Classify bindings in TCO/fold bodies: variant vs invariant
  - Invariant computations (e.g. `string_length(x)`) hoisted
  - **Acceptance:** `tokenize_loop` has no `string_length` inside loop.

**Stage 3: Post-processing** (depends on Stage 2)
- Compute semantic consumer count (excluding administrative edges)
- Identify accumulator + terminal reduction (build-reduce) patterns
- Classify intermediates: materialized (escapes) vs not materialized
- Attach forcing witnesses to every non-minimal classification
- Assemble EmitGraph
- **Acceptance:** `scan_string_body.acc` classified as not-materialized
  build-reduce. Rust renders as `String` + `push_str`.

### P1b acceptance criteria

- [ ] **Witnesses are first-class** in the EmitGraph output:
  - Every non-minimal classification carries a forcing witness
    (e.g., "shared because consumed at nodes {n42, n67}")
  - Testable: dump witnesses, assert specific bindings have expected
    witness shape
  - Debuggable: when normalization cannot certify a narrower classification,
    the witness explains why
- [ ] **Complexity:** normalization is O(N), all lookups via map or dense index
- [ ] **Memory:** normalization peak memory overhead is bounded relative
  to ResolvedGraph size (side tables, not cloned topology)
- [ ] EmitGraph is target-agnostic (no Rust/Go/Python/Java in normalization)
- [ ] Adding a new backend requires zero changes to normalization
- [ ] v2 emitter can consume the same EmitGraph
- [ ] **Guarantee:** no redundant runtime gatekeeping, canonical behavioral
  facts, and locally minimal target choices given those facts

**P1b files (.dag — permanent, required before A5/A6)**

| File | Changes |
|------|---------|
| `src/v2/04a_normalize.dag` | New module: graph normalization (edge classification, value shape, loop invariance, witnesses) |
| `src/v2/05_emit.dag` | Consume EmitGraph instead of raw ResolvedGraph |
| `src/v2/05_emit_rust.dag` | Language model reads EmitGraph facts, replaces blanket `rc_types` map |
| `src/v2/05_emit_go.dag` | Language model for Go (same EmitGraph interface) |
| `src/v2/05_emit_python.dag` | Language model for Python (same EmitGraph interface) |
| `src/v2/06_pipeline.dag` | Insert normalization between reconcile and emit |

### Estimated impact (P1a)

| Hot path | Before P1a | After P1a |
|----------|-----------|----------|
| Per-character predicate | stacker + `.to_string()` heap allocs | `#[inline]` byte comparison |
| Per-token bounds check | `string_length` O(200K) per call | `pos < cached_len` O(1) |
| Per-token ScanResult | `Rc::new(ScanResult{...})` heap alloc | Bare struct, moved |
| Per-string-char accumulation | `Rc<Vec<String>>` push + join | `String::push_str` |
| Estimated tokens/sec (200KB) | ~100 (hangs) | ~100K+ (completes in seconds) |

### Performance contract tests (all stages, v1 + v2)

Every pipeline stage returns **structural metrics** alongside its result.
Tests assert bounds on those metrics — no wall clocks, no machine
dependence. This catches algorithmic regressions (O(N²) introduced)
regardless of hardware speed.

**Metrics per stage:**

| Stage | Key metrics | Bounds |
|-------|------------|--------|
| Tokenize | char_visits, token_count | char_visits ≤ 2 × source_len, tokens ≤ source_len |
| Parse | node_visits, ast_nodes | node_visits ≤ 2 × token_count, ast_nodes ≤ token_count |
| Resolve | module_visits, import_lookups | module_visits ≤ modules × max_imports |
| Reconcile | type_visits, kahn_iterations | type_visits ≤ C × bindings, kahn_iters ≤ type_count |
| Normalize | node_visits, output_bindings, witness_ids | visits ≤ 4 × N, output ≤ input, witnesses ≤ 2 × non_minimal |
| Emit | node_visits, output_lines | visits ≤ C × N, lines proportional to nodes |

**Three structural properties tested:**

1. **Algorithmic complexity** — operation count bounded by `C × input_size`.
   If someone introduces a quadratic loop, the count blows up even on
   small inputs. Catches the class of bug, not the symptom.

2. **Allocation proportionality** — output size bounded by input size.
   Normalization doesn't produce more bindings than it consumes.
   Witnesses don't exceed `2 × non_minimal_count`. If a stage inflates
   data, it's a bug.

3. **Termination** — iterative algorithms (Kahn's, topo sort) assert
   `iterations ≤ input_elements`. Recursive walks assert
   `max_depth ≤ input_depth + K`. Each iteration/frame must make progress.

**Contract shape:**

```
// Returned alongside every stage result
struct StageMetrics {
    node_visits: usize,
    max_recursion_depth: usize,
    output_elements: usize,
    loop_iterations: usize,
}

// Tests — structural, not temporal
assert!(metrics.node_visits <= 4 * input_node_count);
assert!(metrics.output_elements <= input_element_count);
assert!(metrics.loop_iterations <= input_element_count);
assert!(metrics.max_recursion_depth <= input_depth + 2);
```

**Implementation:**
- **v1 (Rust):** `StageMetrics` struct returned in a tuple with stage output.
  Counts via `AtomicUsize` increment at visit sites (zero-cost in release
  builds behind `#[cfg(debug_assertions)]` or a feature flag).
- **v2 (.dag):** Metrics threaded as accumulator alongside stage output.
  Same bounds, same contract shape, testable by the v1 interpreter.
- **Auto-generation (future):** Test generation produces metric-bound
  assertions for all graph shapes, not just hand-picked examples.

**Acceptance:**
- [ ] Every pipeline stage in v1 returns `StageMetrics`
- [ ] Every pipeline stage in v2 returns metrics alongside result
- [ ] Bounds are asserted in unit tests for each stage
- [ ] Self-compile input passes all metric bounds
- [ ] Adding a new stage requires defining its metric bounds

### A5: Bootstrap stage 0 → 1

```text
v1 compiles v2 .dag → Rust → rustc → v2-stage0
v2-stage0 compiles v2 .dag → Rust → rustc → v2-stage1
```

**Design:** Library-first boundary. The primary contract is a library
entrypoint, with CLI as a thin wrapper:

- `compile_sources(sources, target) -> ArtifactSet` (already exists)
- `compile_dir(source_dir, output_dir, options) -> Report` (new)
- CLI `compile` parses args and calls `compile_dir`

This separation makes stage0→stage1 harnessing, stage1→stage2
comparison, and eventual A7 retirement cleaner. It also keeps
fixed-point testing about compiler semantics rather than command-line
plumbing.

**Acceptance:**
- [ ] v2-stage0 compiles v2 .dag → Rust → `rustc` → v2-stage1 builds
- [ ] v2-stage1 passes its own test suite (basic smoke tests)
- [ ] stage0→stage1 bootstrap harness exists as a test
- [ ] Primary contract is library (`compile_sources` / `compile_dir`)

### A6: Fixed point

```text
v2-stage1 compiles v2 .dag → Rust → rustc → v2-stage2
stage1 output == stage2 output
```

**Two gates:** Semantic equality first, byte-for-byte second:

- **A6a (semantic):** Normalized artifact-set equality — same modules,
  same items, same types, same function bodies. Formatting and ordering
  differences are tolerated. This is the debugging gate.
- **A6b (textual):** Byte-for-byte text equality — the final bar.

A6a lets you debug fixed-point failures without treating formatting
drift, ordering drift, and semantic drift as the same class of bug.

**Acceptance:**
- [ ] deterministic ordering of all emitted output (modules, items, fields)
- [ ] artifact normalization strips non-deterministic content (timestamps, paths)
- [ ] A6a: `normalize(stage1_output) == normalize(stage2_output)`
- [ ] A6b: `stage1_output == stage2_output` byte-for-byte comparison passes

### A7: v1 retirement

v2 builds and tests without v1 in the dependency chain.

**Acceptance:**
- [ ] v1 removed from build/test dependency chain
- [ ] all tests pass with only v2 compiler
- [ ] v1-era docs, workflows, ratchets cleaned up

### Remaining work: branchable lanes

The remaining roadmap should be executed in **waves of cleanly mergeable
branches**, not as one long serial refactor. The rule is simple: parallelize
by **write scope**, not by conceptual topic. If two tasks need the same top-level
types or the same orchestration function, they are not independent lanes.

#### Wave 0 — serial gate (DONE + P1a)

| Lane | Branch | Ownership | Files | Status |
|------|--------|-----------|-------|--------|
| G0 | `wt/a4-evidence` | Run/record the A4 evidence tests | tests + docs only | **DONE** — 0 cargo check errors |
| P0 | `v2-compiler-convergence` | Stacker wrapping (P0-a), Copy derive (P0-b), reconcile O(N²) (P0-c) | `src/v2/05_emit_rust.dag`, `src/v2/04_reconcile.dag` | **P0-a/P0-b/P0-c DONE** |
| P1a | `wt/p1a-perf-unblock` | v1 emitter: call graph SCC, function classification, per-binding consumer analysis | `fn_codegen.rs`, `render_rust.rs`, `v2_runtime_shim.rs` | **IN PROGRESS** |

#### Wave 1 — clean post-A4 lanes

These can run in parallel immediately after A4 because they have either
disjoint files or disjoint ownership zones within `04_reconcile.dag`.

| Lane | Branch | Ownership | Files | Merge notes |
|------|--------|-----------|-------|-------------|
| W1-A | `wt/mw-typecheck-module` | `build_scope_from_items`, `merge_scope_from_imports`, `typecheck_module`, per-module contribution/context types, item-registry construction | `src/v2/04_reconcile.dag` | Owns the module-orchestration zone only. Avoid edits to `infer_expr` / `resolve_expr_types` sections. |
| W1-B | `wt/mw-resolve-walks` | `resolve_*`, `resolve_expr_types`, `resolve_node_bounded`, `resolve_item_types` result-unpack removal | `src/v2/04_reconcile.dag` | Owns the resolution zone. Keep helper/result types local to that zone to reduce conflicts. |
| W1-C | `wt/mw-infer-walks` | `infer_expr`, infer helpers, block/list accumulation cleanup | `src/v2/04_reconcile.dag` | Owns the inference zone. Do not touch module boundary types in this lane. |
| W1-D | `wt/mw-resolve-graph` | `resolve_modules`, `kahn_step`, adjacency/indegree cleanup if profiling keeps it relevant | `src/v2/03_resolve.dag` | Fully independent of reconcile work. |
| W1-E | `wt/mw-emit-micro` | `order_typed_call_args`, cold emitter micro-walks, reserved-word lookup cleanup | `src/v2/05_emit.dag`, `src/v2/05_emit_rust.dag` | Independent of reconcile internals as long as boundary types stay unchanged. |
| W1-F | `wt/blocker3-core-cleanup` | Delete remaining `TypeExpr` helpers / last callers | `src/v2/00_core.dag`, v1 bootstrap callers | Explicitly parallelizable mechanical cleanup. |

#### Wave 2 — single boundary lane plus support lanes

This wave should **not** be split across multiple branches that all edit
`ResolvedGraph`, `TypedModule`, `DeclaredFuncSig`, `ResolvedFuncSig`,
`06_pipeline.dag`, and the emit entry points. Those are one ownership surface.

| Lane | Branch | Ownership | Files | Depends on |
|------|--------|-----------|-------|------------|
| W2-A | `wt/ph2a-boundary` | Declared vs resolved func-sig split, SCC resolution, strict `ResolvedGraph` boundary, `compile_sources_lenient` retirement path, `validate_no_unresolved` demotion | `src/v2/04_reconcile.dag`, `src/v2/06_pipeline.dag`, emitter entry points | W1-A merged |
| W2-B | `wt/ph2a-ratchets` | Acceptance tests, perf ratchets, strict-path test coverage, self-hosting assertions | tests / harness / docs | Can track W1/W2 APIs and merge after W2-A |
| W2-C | `wt/ph2a-docs` | Docs, roadmap, design notes, migration notes for the strict boundary | docs only | Independent support lane |

#### Wave 3 — A5/A6 parallel lanes after Phase 2a lands

Once the strict boundary is stable, the next self-hosting phases split fairly
cleanly again:

| Lane | Branch | Ownership | Files | Why independent |
|------|--------|-----------|-------|-----------------|
| W3-A | `wt/a5-stage-harness` | Stage0→stage1 harness, bootstrap runner, self-host test plumbing | pipeline/tests/harness | Mostly harness work; should not change emitter semantics. |
| W3-B | `wt/a6-determinism` | Deterministic file/module/item ordering, artifact normalization, stable output comparison helpers | emitters + artifact helpers | Needed for fixed-point equality; separate from harness mechanics. |
| W3-C | `wt/result-special-case` | Special-case `Result<T,E>` support before A6 | parser + reconcile + tests | Required for fail-closed error paths, but distinct from stage harness and determinism work. |

##### W3-A: A5 stage harness design

The bootstrap chain is:

```text
v1 (Rust) → assemble_v2_crate → v2-stage0 crate
v2-stage0 → compile_sources(.dag) → stage1 Rust files
stage1 Rust → cargo build → v2-stage1 binary
v2-stage1 → compile_sources(.dag) → stage2 Rust files           [A5]
stage1 output == stage2 output                                   [A6]
```

**Note:** The bootstrap chain targets the strict `compile_sources` entrypoint,
not the lenient path. B3 Phase 2a retires `compile_sources_lenient` before A5.
If the strict path has false positives, those are bugs to fix in reconcile,
not reasons to keep the lenient path.

**Current state:** `v2_crate_self_compile_cargo_check` proves stage0→stage1
produces valid Rust (0 cargo check errors). But stage1 output doesn't include
test infrastructure — it can't self-compile via tests.

**What A5 needs:**

1. **Stage1 must include self-compile capability.** Two options:
   - (a) Add a `self_compile` entry point to `06_pipeline.dag` that reads
     .dag files from disk and writes compiled output. The v2 emitter's
     `main.rs` emission already handles CLI — extend with a `compile`
     subcommand.
   - (b) The A5 test harness injects test infrastructure (embedded .dag
     sources + self-compile test) into the stage1 output before building.
     Simpler but less clean.

2. **File I/O for self-compile.** Currently `compile_sources_lenient` takes
   `List<SourceFile>` with in-memory content. Stage1 needs `filesystem_read`
   (already in `v2_rt.rs`) wired to a CLI that loads .dag files from a
   directory.

3. **Generated Cargo.toml must be complete.** The v2 emitter's
   `emit_cargo_toml` produces deps but no `[workspace]`. The stage1 output
   must compile standalone in a temp directory (not under the main workspace).

4. **Level-3 self-compile must complete.** This is gated on the PERF gate
   (P0). Without stacker wrapping and Copy derives in the stage1 output,
   the stage1 binary would overflow the stack on large .dag files.

**Recommended approach:** Option (a) — add a `compile` CLI subcommand to the
v2 pipeline. This is the clean path because stage1 becomes a real compiler
binary, not a test-only artifact. The subcommand reads `--source-dir` and
writes to `--output-dir`.

##### W3-B: A6 determinism requirements

Fixed-point equality (`stage1 output == stage2 output`) requires:

1. **Module emission order** — must be topological (by import deps), not
   iteration-order dependent. Currently `emit_rust` maps over `typed.modules`
   which comes from `resolve_modules` — verify this is deterministic.

2. **Item emission order within modules** — must follow source order. The
   v2 parser preserves source order; verify reconciler/emitter don't reorder.

3. **Map iteration order** — v2 uses `BTreeMap` in emitted Rust (sorted
   keys). The `.dag` `Map` type must also have deterministic iteration.
   `fold(map_values(...))` iterates in key order for BTreeMap.

4. **Artifact normalization** — strip anything non-deterministic from output:
   no timestamps, no absolute paths, no random seeds. Current output is
   likely already deterministic since the pipeline is pure.

5. **Byte-for-byte comparison** — the test compares `stage1_files` vs
   `stage2_files` content. Any whitespace or formatting difference fails.

#### Wave 4 — A7 retirement lanes

| Lane | Branch | Ownership | Files |
|------|--------|-----------|-------|
| W4-A | `wt/a7-runtime-retire` | Remove remaining v1 runtime/bootstrap dependency from build/test path | manifests, pipeline, harness |
| W4-B | `wt/a7-docs-retire` | Docs, workflows, cleanup of v1-era instructions/ratchets | docs + CI/workflow files |

##### W4-A: v1 retirement checklist

1. Remove `daglang-eval` (v1 interpreter) from workspace members
2. Remove `daglang-emit` v1 codegen paths (`fn_codegen.rs`, `type_codegen.rs`,
   `render_rust.rs`, `v2_crate_emit.rs`)
3. Remove `daglang-syntax` dependency on v1 AST types (or keep as shared parser)
4. Update `v2-compiler-tests` to use v2-stage1 binary instead of v1 interpreter
5. Remove v1-era `Cargo.toml` workspace members, CI workflows
6. Verify: `cargo test --workspace` passes with only v2 crate + shared parser

#### R9 cleanup (parallel with any wave)

- **SG-11 (v1):** Trim stacker to recursive-only in `render_rust.rs` — P0-a does
  the v2-side fix; this is the v1-side equivalent
- **SG-12 (v1):** Copy detection for small structs in `type_codegen.rs` — P0-b does
  the v2-side fix; this is the v1-side equivalent
- **SG-9 revert:** After PERF gate passes, revert TokPos extraction and branch-aware
  use counting if confirmed redundant

#### Deferred (post-A7)

- General generic syntax (`type Foo<T> = ...`)
- Full linear type checking in v2 compiler
- C3/C4 emitter extdep imports + CLI target selection
- B4 transport dissolution
- Track D complexity analysis (D2-D4)
- Track E artifact planning (E1-E4)
- Track F debuggability (F1-F4)

#### v1-safe lane policy

While self-hosting is blocked on P1a, other work can proceed in parallel
under v1. Explicit policy for what is safe:

**Safe now:**
- Tests, ratchets, measurement, profiling surfaces
- Stage harness scaffolding (A5 prep)
- Minimal debuggability/spans
- Deterministic ordering / normalization helpers (A6 prep)
- Docs and boundary design docs
- Data-only language/extdep work that does not widen compiler contracts

**Not safe now:**
- Anything that adds new lenient paths
- Anything that depends on current unresolved boundary types
- More global codegen switches/flags
- More backend-specific heuristics inside semantic phases

#### Temporary-debt ratchets

In addition to output ratchets, track degradation counts to prevent
"temporary forever":

- Count of `compile_sources_lenient()` callsites (target: 0)
- Count of `validate_no_unresolved()` callsites (target: 0)
- Count of emitted `try_unwrap(...).unwrap_or_else(...)` fallbacks
- Count of blanket stacker wrappers in generated code
- Count of heuristics/lists in emit that rediscover reconcile facts
  (e.g., `known_opt_fields`, `types_with_value_field`, `is_rc_exclude`)

#### Merge discipline

- During Wave 1, treat `04_reconcile.dag` as three owned zones:
  module orchestration, resolution, inference.
- Do not move functions across files during Waves 1-2. Optimize for merge
  cleanliness first; reorganize after the hot-path refactor lands.
- Reserve top-level boundary type edits (`TypedModule`, `TypedGraph`,
  `ResolvedGraph`, func-sig types) to the boundary lane.
- Every lane should land with its own tests/ratchets so branches merge on
  behavior, not just on text.

---

## Track B: Node Convergence

### B1: TypeExpr → Node (MOSTLY DONE)

All typed fields are Node. Remaining: TypeExpr definition + helpers in
00_core.dag (Blocker 3), parser boundary spray, bridge predicate lossy
conversion.

### B2: Rename typecheck → infer (DONE)

### B3: Expr → Node

#### Phase 1: Eliminate TypedExpr (DONE — unblocks A4)

Added `resolved_type: Node?` to every Expr variant. Merged TypedNode into
Node (added `is_self_recursive: Bool`, `has_non_tail_self_call: Bool`).
Deleted TypedExpr (19 variants), TypedNamedArg, TypedMatchArm, TypedFieldInit,
TypedStringPart, TypedNode. Rewrote self-call walkers to operate on Expr.

~770 LOC across 8 .dag files + v1 bootstrap. Parser fills `resolved_type: none`,
reconciler fills `Some { value: inferred_type }`. Emitters read `resolved_type.value`.

**Impact:** Halves expression memory (one AST, not two). Eliminates the 4GB OOM
root cause documented in the postmortem below.

#### Phase 2a: Boundary contracts for bootstrap (FROZEN — design ready, implementation after A4)

These contracts define the strict post-A4 boundary. Each makes a class of
fabrication structurally unrepresentable.

**Implementation shape fixed for Phase 2a:** keep the compiler in the current
functional/imperative style (explicit recursion, folds, local accumulators),
but widen boundary and helper result types wherever a single walk learns
multiple sibling facts. The goal is not to add a visitor/object layer; the
goal is to stop re-walking the same collections to rediscover adjacent data.

##### Contract 1 — Declared vs Resolved FuncSig

```
DeclaredFuncSig { return_type: Node? }   (parser output, None = must infer)
ResolvedFuncSig { return_type: Node }    (always concrete, never placeholder)
```

**Why:** `build_func_env()` currently fabricates placeholder return types for
unannotated functions. `infer_expr(Call)` can then observe the fake type
instead of the resolved one. Splitting the types makes it impossible to read
an unresolved signature where a resolved one is expected.

**Implementation sketch:**
- Parser produces `DeclaredFuncSig` with `return_type: Node?` (None when
  not annotated)
- `build_func_env` stores `DeclaredFuncSig` (no fabrication needed)
- Resolution phase produces `ResolvedFuncSig` with `return_type: Node`
  (always present)
- Emit boundary accepts only `ResolvedFuncSig`

##### Contract 2 — SCC-aware return type resolution

- Build call graph from resolved module, compute SCCs, process in
  topological order
- Non-recursive functions: infer return type from body
- Self-recursive without annotation: compile error (fail closed)
- Mutual recursion without annotations: compile error

**Why:** Current code fabricates placeholders because it doesn't process
functions in dependency order. SCC-aware ordering means every callee's
return type is resolved before any caller that depends on it.

##### Contract 3 — Resolved boundary type

- `ResolvedGraph` wraps `ResolvedFuncEnv` (not `DeclaredFuncEnv`)
- Emit accepts only structurally resolved graphs
- The type system enforces that unresolved state cannot reach emit

**Why:** This is the structural replacement for `validate_no_unresolved()`.
Instead of checking at runtime whether unresolved types leaked through,
the pipeline boundary type makes it unrepresentable. Satisfies Invariant 9
(correctness by construction, not by validation).

##### Contract 4 — Retirement plans

- `validate_no_unresolved`: debug-only assertion after Phase 2a lands,
  then delete after Phase 2b confirms all paths produce resolved graphs.
  Not a permanent validation pass.
- `compile_sources_lenient`: measure false positive rate, fix root causes,
  then delete. Not a permanent parallel code path.

**Why:** Both exist because the boundary type is too permissive. Once
Contracts 1-3 land, neither has a reason to exist.

##### Design constraint

Do **not** weaken `typecheck_module()` into a partial/best-effort mode.
The fix is stronger output types, not more lenient control flow.

##### Refactor rule set (updated 2026-03-19)

- **Prefer wider contribution/context types over sibling passes.** If one walk
  over items/imports learns multiple facts, return them together
  (`ItemContribution` / `ModuleContext` shape) instead of rebuilding them in
  separate passes.
- **Hot-path helpers must not return a list of compound results that callers
  immediately unpack with extra `map` / `flat_map` walks.** Either accumulate
  directly or return a wider batch result that is already split.
- **Use `fold_unpack` only as a transition tool.** It is acceptable for
  medium-priority cleanup, but true hot paths should avoid both the unpack
  walks and the intermediate results list.
- **When metadata is derivable during reconcile, carry it on the boundary
  type.** Emitters should consume shared summaries, not rediscover them.

**Acceptance:**
- [ ] `validate_no_unresolved()` deleted; replaced with structural type boundary
- [ ] declared vs resolved function signatures are distinct; no fabricated
      placeholder return types in `FuncEnv`
- [ ] call-graph/SCC return-type resolution fails closed for recursive
      unannotated functions
- [ ] `compile_sources_lenient()` deleted; bootstrap uses the strict
      resolved type boundary

#### Phase 2b: Expr → Node patterns (after Phase 2a lands)

Convert Expr variants to Node patterns. After this, "typed" just means
"return_type is filled in" and the pipeline shape is `Node → Node → Node → TextFile`.

Also: batch fix F2 (string-typed field):
- `ItemInfo.kind: String` → closed enum `ItemKind = Fn | Func | Other`
- (F3/F4 already resolved — see audit table)

**Acceptance (Phase 2a — boundary contracts):**
- [ ] `DeclaredFuncSig` and `ResolvedFuncSig` are distinct types
      (Contract 1)
- [ ] `build_func_env` produces `DeclaredFuncSig` with no fabricated
      placeholder return types (Contract 1)
- [ ] Emit boundary accepts only `ResolvedFuncSig` — compile error if
      any signature is still declared-only (Contract 1)
- [ ] Return types resolved in SCC topological order (Contract 2)
- [ ] Self-recursive function without return annotation: compile error,
      not placeholder (Contract 2)
- [ ] Mutual recursion without annotations: compile error (Contract 2)
- [ ] `ResolvedGraph` / `ResolvedFuncEnv` boundary type enforced at
      reconcile-to-emit handoff (Contract 3)
- [ ] `validate_no_unresolved()` demoted to debug-only assertion
      (Contract 4)
- [ ] `compile_sources_lenient()` false positive rate measured and
      root causes identified (Contract 4)
- [ ] 455+ tests pass, generated crate compiles clean

**Acceptance (Phase 2b — full convergence, after 2a verified):**
- [ ] `Expr` type deleted from `00_core.dag`
- [ ] `validate_no_unresolved()` deleted entirely
- [ ] `compile_sources_lenient()` deleted
- [ ] pipeline shape is `Node → Node → Node → TextFile`
- [ ] No String-typed fields where a closed enum is appropriate

### B4: Transport dissolution (NEEDS DESIGN DECISION)

`TransportBinding` should dissolve. Transport behavior should come from
structure rather than a fixed enum.

---

## Track C: Language Emission

### C1-C2: DONE

### C3: Emitters consult extdeps

Emitters import from language extdeps instead of inline data. Adding a new
target means writing an extdep, not editing compiler logic.

### C4: CLI target selection

`--target` flag loads the appropriate language extdep.

---

## Track D: Runtime Complexity Analysis

**Parallel with critical path. Urgent: eliminates performance fallbacks.**

### D1: Cost algebra (DONE — types defined in dsl/std/complexity.dag)

### D-ownership: Static ownership proof (eliminate try_unwrap fallbacks)

**Design (updated 2026-03-20):** Track D builds on P1a/P1b's edge classification.
P1a (v1) and P1b (v2) compute **semantic consumer count** (after removing
administrative edges like TCO threading and pass-through reads). Track D
strengthens this:
- If semantic_consumers == 1 at a try_unwrap site: emit `Rc::into_inner().expect()`
  (no fallback, panic on violation).
- If semantic_consumers > 1: compile error — "cannot guarantee O(1) mutation,
  restructure the code."
- If P1a/P1b hasn't classified a binding: that is a completeness bug, not a
  Track D gap.

This replaces ALL `Rc::try_unwrap(x).unwrap_or_else(|rc| (*rc).clone())`
instances (14 codegen + 4 runtime sites documented in Performance Fallback
Inventory) with statically verified moves. No runtime fallback exists.

**Relationship to P1a/P1b:** P1a (v1 emitter) and P1b (v2 EmitGraph) classify
edges and count semantic consumers. Track D uses those counts as proof
obligations. Same edges, same classifications — P1b is the front half,
Track D is the back half. Track D must NOT re-derive consumer counts from
raw use_count; it reads P1b's EmitGraph directly (or P1a's in-emitter
analysis for the v1 path).

**Staging:**
- **After P1a lands:** Most try_unwrap sites become provable because
  semantic consumer count correctly excludes administrative edges that
  inflated raw use_count.
- **After A7 (deferred):** Full linear type checking in v2 compiler.
  More robust than use counting but requires v2 self-hosting.

### D2: Typed summaries

Infer symbolic summaries from typed expressions/functions. Per-function
`ComplexitySummary` with `work`, `span`, `output_size` as symbolic `CostExpr`.

**CostExpr growth risk:** Symbolic formulas can grow faster than the
graph they summarize if callee summaries are inlined naively or
equivalent subexpressions are duplicated at every call site. Mitigation:
- `CostExpr` must be a shared DAG with interning/hash-consing
- Memoize summaries per function or SCC
- Prefer conservative upper bounds + local simplification over
  aggressive exact symbolic expansion
- **Ratchet:** CostExpr node count per function/module must be bounded;
  add growth ratchets to prevent silent blowup

F3/F4 already resolved: `SemanticsCtx` fields and `PrimCost.op` are now closed
enums in `07_complexity.dag` (see audit table).

### D3: DAG composition

Compose summaries over lowered DAG. Span = longest dependency path,
loop work = iteration count × body work.

**Exclusive branches:** "DAG work = sum of node work" is only exact
when all nodes in the region definitely execute. For exclusive choices
(match arms, short-circuit), plain summation is a worst-case upper
bound, not exact cost. The tighter rule: condition cost + max(branches),
not sum of all branches. The roadmap should be explicit that
composition produces **worst-case bounds**, not exact costs, when
exclusive structure is present.

**Recursive SCC summaries:** Acyclic composition is straightforward,
but recursive cost summaries need one of: (1) user annotations for
recurrence depth, (2) a restricted recurrence solver for common
patterns (linear recursion, divide-and-conquer), or (3) fail-closed
with "unbounded" classification. This mirrors the return-type
resolution strategy — infer what we can, error on what we can't.

### D4: Proofs and reporting

Surface complexity as proof/report. Policy checks can reject unbounded
workflows.

### Track D acceptance criteria

- [ ] `CostExpr` node count per function bounded by ratchet
- [ ] `CostExpr` uses hash-consing / interning (no duplicated subtrees)
- [ ] Recursive SCC summaries: infer linear/divide-and-conquer, annotate
  others, error on unresolvable — never silently produce unbounded
- [ ] Exclusive-branch composition uses `max(branches)`, not `sum`

---

## Track E: Artifact Planning

### E0: Monolith wrapper (DONE)

### E1-E4: Artifact model, target placement, boundary semantics, reporting

These define how one `.dag` graph is partitioned into multiple artifacts,
placed onto multiple targets, and emitted with explicit contracts between
pieces. Depends on B4/C3 being far enough along that target facts and
boundary structure are explicit.

---

## Track F: Debuggability

**Motivation:** A `.dag` program compiles to an intermediate language (Rust, Python,
JS) which compiles again to machine code or bytecode. When something goes wrong at
runtime, the user sees a Rust panic or Python traceback — pointing at generated code
they didn't write, in a language they may not know. The gap between "where the error
is reported" and "where the error was authored" can be two compilation steps wide.

But `.dag` has structural properties that most languages don't: every phase is pure,
every value is immutable, every node carries its source span, and an interpreter
already exists that can execute `.dag` directly. The debugger should exploit these
properties rather than trying to replicate what GDB/LLDB/pdb already do.

**Design principle:** The interpreter is the primary debugging surface. Users debug
their `.dag` logic in `.dag` terms. Cross-language source mapping is an optimization
for production tracing — same interface, different backend. We define the interface
now and build backends as needed.

**Core interface — TraceEvent:**

The contract between execution (however it happens) and debugging tools (however
they present). This is target-agnostic and defined in `.dag`:

```dag
type TraceEvent
  = Enter { node_id: String, span: SourceSpan, inputs: Map<String, String> }
  | Exit  { node_id: String, span: SourceSpan, output: String }
  | Error { node_id: String, span: SourceSpan, message: String }

type TraceFrame {
  func_name: String
  span: SourceSpan
  bindings: Map<String, String>
}

type Trace {
  events: List<TraceEvent>
  stack: List<TraceFrame>
}
```

Values serialized as strings in the trace — the trace is a diagnostic artifact,
not an execution artifact. This keeps the interface stable even as the value
representation evolves. A structured `TraceValue` can replace `String` later
without changing the event shape.

**Why this interface is sustainable:**

- The interpreter produces TraceEvents directly by instrumenting `eval_body`
- Generated code can produce them via inserted instrumentation calls
- Source maps are a way to reconstruct `span` from target-language positions
  without instrumentation — a different *producer*, same *consumer*
- All debugging tools (CLI, TUI, DAP adapter) consume TraceEvents regardless
  of how they were produced

**User scenarios this serves:**

1. *"My workflow failed. Where?"* — Error TraceEvent carries the `.dag`
   source span. No Rust/Python/JS knowledge needed.
2. *"What were the inputs when it failed?"* — TraceFrame.bindings at the
   error point. The interpreter already has this in its environment.
3. *"Let me step through it."* — Breakpoints + step commands against the
   interpreter, navigating by `.dag` source lines and node names.
4. *"I want a regression test for this."* — Snapshot the Enter event's
   inputs at a function boundary → hermetic test with captured state.

### F1: Span preservation + interpreter source locations

**Timing:** Alongside A1–A4. Small cost, high leverage.

The emitter currently discards all spans (`span: _` on every match arm).
The interpreter call stack has frames but no source locations. Errors say
`EvalError` with no `.dag` file:line.

**Work:**
- Stop discarding spans in emitters — carry SourceSpan through to output
- Thread source spans through interpreter stack frames (`eval_stack.rs`
  frame struct gains `source_span` + `func_name`)
- Errors format as `resolve.dag:142:5: type mismatch in field 'name'`
  instead of opaque error strings

**Acceptance:**
- [ ] Every `EvalError` includes `.dag` file:line:col
- [ ] Interpreter call stack is printable as `.dag`-level stack trace
- [ ] Emitted code retains source origin as comments or metadata

### F2: Interpreter debugger

**Timing:** A5–A6 timeframe. By bootstrap, you need to debug the
self-hosted compiler in `.dag` terms.

**Work:**
- Define TraceEvent + TraceFrame types in `dsl/std/`
- Instrument `eval_body`/`eval_stmt` to emit TraceEvents
- Breakpoints: by source location (`resolve.dag:142`) or node name
  (`reconcile_field`). Interpreter checks break condition at each
  Enter event.
- Step into/over/out: mapped to DAG node entry/exit boundaries
- State inspection: print TraceFrame.bindings at current position
- Trace recording: write TraceEvent stream to file for offline replay

**Acceptance:**
- [ ] Can set breakpoint by `.dag` file:line or function name
- [ ] Can step through `.dag` execution and inspect bindings
- [ ] Can record a trace and replay it without re-executing

### F3: Hermetic reproduction

**Timing:** Post-A6. Once the pipeline is stable, formalize replay into
a user-facing tool.

Because every phase is pure and values are immutable, any function
boundary is a potential isolation point. Capture the Enter event's
inputs → you have a self-contained test case.

**Work:**
- Snapshot mode: at a specified function boundary, serialize all inputs
- Test generation: emit a `.dag` test file that calls the function with
  the captured inputs and asserts the observed output
- Regression mode: on failure, automatically emit a snapshot test

**Sketch:**
```dag
# auto-generated from trace snapshot at reconcile.dag:87
# failure: "type mismatch: expected Node, got String"
test reconcile_field_regression {
  let input = { name: "status", children: [], connective: Conj }
  let scope = { bindings: { "Status": { name: "Status", ... } } }
  let result = reconcile_field(field: input, scope: scope)
  assert result.return_type.name == "String"
}
```

### F4: Cross-language source mapping

**Timing:** Post-A7. Extends C3/C4. May never be fully needed if the
interpreter debugger covers user needs.

This is where the interface pays off. TraceEvent consumers (debugger UI,
error reporters) don't change. Only the producer changes — instead of
the interpreter emitting events, a source map translates target-language
positions back to `.dag` spans.

**Work:**
- Source map emission: emitter writes `.dag.map` alongside generated code
  (format: `{target_line:col} → {dag_file:line:col}`)
- Error remapper: intercept target-language panics/tracebacks, translate
  via source map, format as `.dag`-level error
- *Optional, may not be worth it:* DAP (Debug Adapter Protocol) server
  that wraps the interpreter, enabling VS Code / IDE debugging

**Why this might not be needed:** If the interpreter debugger (F2) is
good enough for logic debugging, the only remaining need for cross-language
mapping is production error tracing — which the source map + error
remapper handles without a full debugger.

---

## Postmortem: Self-Compile OOM (2026-03-19)

### Symptoms
- `self_compile_all_modules`: SIGKILL at ~4GB, >60s
- `gist_compile_all_modules`: 40ms, <100MB
- Individual module compile: <3.5s each, no OOM
- 13 modules together via `compile_sources_lenient`: OOM

### Root causes found and fixed
1. **Tokenizer infinite loop on non-ASCII** — em dash (U+2014) in comments
   caused byte/char index mismatch in `char_at` runtime. Fix: replaced all
   non-ASCII in .dag comments with ASCII equivalents. The `char_at` function
   uses byte indexing for fast path but character indexing for fallback —
   this is a known bug class (see `feedback_byte_char_position.md`).

2. **Recursive types lost across imports** — `merge_envs` and `build_type_env`
   set `recursive_types: []` when merging parent environments. Imported
   recursive types (Node, Expr, TypedExpr) weren't in the merged
   `recursive_type_set`, so `resolve_node_bounded` expanded them up to
   depth 50 instead of stopping at the cycle marker. Fix: preserve
   `recursive_types` and `recursive_type_set` during merges.

3. **O(N^2) parent module traversals** — `typecheck_module` walked ALL
   entries in `parent_index` (every previously typed module) to build
   service_registry, service_locals, and variant_locals. With 13 modules,
   this was 78 full walks × hundreds of items each. Fix: scope walks to
   direct imports only (`resolved.resolved_imports`), matching the existing
   func_env pattern.

4. **Re-resolving already-resolved types** — `resolve_env_bindings` resolved
   EVERY binding including imports already resolved by their parent module.
   Fix: `local_names` parameter limits resolution to current module's types.

### Root cause FIXED (B3 Phase 1)
- **Cross-module reconcile memory explosion** — the reconciler created a
  parallel typed AST (TypedExpr) mirroring every Expr node but adding
  `resolved_type: Node` (~160 bytes) to each. For 13 modules with ~50K
  expressions, this doubled the AST memory → 4GB OOM.

  **Fix (2026-03-19):** Added `resolved_type: Node?` directly to Expr.
  Deleted TypedExpr entirely. The reconciler fills in resolved_type on Expr
  in-place. One AST instead of two. Memory drops from
  O(expressions × type_size × 2) to O(expressions × type_size).

  **Remaining risk:** algorithmic O(N^2) in .dag code (concat-accumulator
  pattern, filter-based lookups) may still cause hangs on self-compile.
  This is a CPU time issue, not a memory issue. Will be visible on first
  A4 attempt.

### A4 attempt 1: concat→list_push migration (2026-03-19)

**What was done:** Replaced all 13 `concat(acc, [x])` singleton-wrap
patterns with `list_push(acc, x)` across 6 .dag files (05_emit_rust,
05_emit_go, 05_emit_python, 05_emit, 06_pipeline, 04_reconcile). Also
replaced 1 `map |> fold(concat)` with `flat_map` in 05_emit_rust.dag.
All 3375 tests pass, clippy clean.

**Result:** Interpreter path is now O(1) per append (verified by tests).
Emitted crate compiles successfully (18.6s). But the emitted binary's
`self_compile_all_modules` test still gets SIGKILL'd after ~62s (OOM).

**Root cause: Rc::try_unwrap fails on the hottest emitted Rust paths.**
Three distinct patterns keep refcount >= 2 where self-compile spends most
of its time, defeating the O(1) path:

1. **Function-call clone (tokenizer — primary, ~3.1GB estimated).**
   `scan_token(source, tokens, pos, ch)` in the .dag source becomes
   `scan_token(source, tokens.clone(), pos, ch)` in emitted Rust because
   the TCO variable still holds a reference. Inside `emit()`,
   `Rc::try_unwrap` sees refcount=2 → full deep clone of the entire
   token Vec on every token. ~20K tokens for 02_parse.dag alone.
   The TokPos extraction (SG-9) was specifically designed to prevent this,
   but the codegen still clones `tokens` when passing to non-TCO helper
   functions (scan_token, scan_string, scan_str_cont, scan_ident, etc.).

2. **Struct-field accumulator access (reconcile, emit — secondary).**
   In fold bodies like `list_push(acc.text, line)` where `acc` is a
   struct (e.g., BlockEmitState), the emitted code does
   `acc.text.clone()` then `Rc::try_unwrap`. But `acc` still holds the
   original field reference → refcount=2 → full clone every iteration.
   Affects: `infer_block` (reconcile), all `emit_typed_block` folds.

3. **Standalone list accumulators work correctly.** When `acc` is a bare
   `Rc<Vec<T>>` (not a struct field), the TCO codegen moves `acc`
   directly into `Rc::try_unwrap` → refcount=1 → O(1) in-place push.
   The tokenizer's main loop (`tokenize_loop`) would work if it didn't
   also pass `tokens` to helper functions.

**Emitted crate stats:** 21,031 LOC Rust (excluding tests). Largest
modules: parse.rs (7,029 lines), reconcile.rs (4,382 lines),
emit_rust.rs (3,114 lines). 13 .dag files totaling 646KB of source text.

**Both patterns fixed (2026-03-19):**
- Pattern 1: `strip_tco_param_clones` post-pass in `lower_tco_plan`
  (codegen fix) + tokenizer `ScanResult` refactor (source-level fix).
- Pattern 2: `compile_fold_accum_field_extract` in all 7 mutating
  intrinsics uses `std::mem::take(&mut Rc::make_mut(&mut acc).field)`.
- Evidence run pending to confirm OOM is eliminated.

---

## Multi-Walk Refactor Program (2026-03-19)

This section turns the multi-walk audit into the concrete post-A4 plan.
"Lower bound" still means: one unavoidable walk over the owned input for a
stage, plus work proportional to emitted output. The audit sharpens the rule:
if a walk learns multiple sibling facts, widen the result type so it returns
those facts together. Do not rediscover them in follow-up passes.

**Execution rule:** this does **not** replace A4. Run the A4 evidence tests
first. Then use this list to remove the largest remaining multi-walk gaps in
the current compiler.

### Audit principle

- One O(N) walk is fine.
- K separate O(N) walks over the same collection is a scaling dimension.
- The two anti-patterns to remove are:
  1. **module-level multi-pass** — same items/imports walked in separate
     passes to extract sibling facts
  2. **result-unpack** — `map(process)` creates a results list, then 2-3 more
     walks split out fields/diagnostics

### Priority order after A4 evidence run

| Priority | Area | Confirmed extra work | Planned response |
|----------|------|----------------------|------------------|
| **P0** | `04_reconcile.dag:typecheck_module` | ~42 extra walks in the audit: 8 module/item passes plus unpack re-walks on `item_results` and `typed_item_results` | Fuse around wider `ItemContribution` / `ModuleContext`-style types. One env build, one contribution fold over items/imports, one infer pass. Carry shared registry/summary data on the boundary instead of rebuilding it later. |
| **P1** | `04_reconcile.dag:resolve_expr_types` | 8 extra result-unpack walks inside expression recursion | Rewrite collection-processing branches to accumulate split outputs directly, or return wider batch results that are already split. |
| **P2** | `04_reconcile.dag:infer_expr` | 5 systematic unpack sites on small lists, repeated across a very hot function | Use a mechanical `fold_unpack` / batch helper first, then inline the hottest sites if the next profile still points there. |
| **P3** | `04_reconcile.dag:resolve_item_types` and `resolve_node_bounded` | 7 extra unpack sites across params/uses/props/children/fields/variants | Convert to inline accumulation or standalone accumulator recursion. |
| **P4** | `05_emit*.dag` + `05_emit.dag` | emitters still walk `typed.modules` once to rebuild shared registry and again to emit; `order_typed_call_args()` re-walks args | Widen `TypedGraph` / `ResolvedGraph` to carry shared metadata once. Rewrite call-arg ordering around a single fold. |
| **P5** | `03_resolve.dag` | `resolve_modules()` / `kahn_step()` still re-walk modules and edges | Keep as explicit technical debt unless A4 profile says it moved onto the hot path. Rework to adjacency + indegree when worth it. |

**Estimated payoff:** P0-P3 remove roughly 130K unnecessary list traversals
in self-compile, plus the constant-factor savings from not allocating
intermediate results lists.

### Concrete next stages

1. **P0: reshape `typecheck_module()` around wider contributions.**
   Keep the functional/imperative style, but let one item walk produce all
   sibling outputs needed later: declared signatures, service registry,
   service locals, variant locals, item registry, and diagnostics. The module
   orchestration target is still 3 logical passes max:
   Pass A: build env + resolve item types
   Pass B: fold item/import contributions into one `ModuleContext`
   Pass C: infer items
2. **P1: rewrite `resolve_expr_types()` list branches.**
   Replace `map(...)` followed by `map(r => r.value)` and
   `flat_map(r => r.diagnostics)` with direct accumulation. This is the
   highest-value hot-path cleanup after `typecheck_module()`.
3. **P2/P3: remove systematic unpack helpers in reconcile.**
   For `infer_expr()`, start with a mechanical helper if it reduces churn.
   For `resolve_item_types()` and `resolve_node_bounded()`, prefer direct
   one-pass accumulation because the call sites are already localized.
4. **P4: widen the reconcile-to-emit boundary.**
   `item_registry` already exists per module. The next step is to carry the
   graph-wide shared registry/summaries on `TypedGraph` / `ResolvedGraph` so
   backends stop rebuilding them from `typed.modules`. Fold `order_typed_call_args()`
   into the same cleanup pass.
5. **P5 and below stay deferred unless profiling changes the order.**
   Parser brace rescans, resolver topo constants, and cold linear scans remain
   technical debt, but they should not pre-empt the reconcile refactor unless
   the A4 evidence run says otherwise.

### Acceptance

- [ ] A4 evidence run completes and produces a fresh hotspot ranking
- [ ] `typecheck_module()` performs one item-contribution walk and one
      import-contribution walk; no module/item collection is re-walked just to
      extract sibling facts
- [ ] `item_results` / `typed_item_results` unpack re-walks are eliminated
- [ ] hot-path reconcile helpers no longer use the result-unpack pattern
      (`resolve_expr_types`, `infer_expr`, `resolve_item_types`,
      `resolve_node_bounded`)
- [ ] `TypedGraph` / `ResolvedGraph` carries shared emitter metadata so
      backends do not rebuild it from `typed.modules`
- [ ] `order_typed_call_args()` no longer triple-walks the same arguments
- [ ] lower-priority resolver/parser scans stay explicitly deferred unless a
      new profile promotes them

---

## Backlog

- Anonymous record target resolution — ambiguous cases must fail closed
- Collection intrinsic semantics in shared IR
- Generated self-hosting tests and stage contracts
- TCO backend contract — no silent partial fallback

### Deferred to post-A7

| Item | What | Why deferred |
|------|------|-------------|
| General generic syntax | `type Foo<T> = ...` parameterized types | Requires v2 self-hosting (parser must parse its own parameterized type defs). Special-cased Result/Option sufficient until then. |
| Full linear type checking | Prove ownership flow statically in v2 compiler | Use-count-based proof (D-ownership) is sufficient for v1 bootstrap. Linear types need the v2 type system. |
| C3: Extdep imports | Emitters read from `dsl/extdeps/` instead of inline data | Blocked on 4 bootstrap limitations (name collision, no import renaming, data not bindable, hardcoded file lists). Inline duplication is stable. |
| Emitter metadata deduplication | Delete 75-line inline copy of language type maps in 05_emit.dag | Depends on C3. No drift yet — both copies identical. |
| Widen V5 | Handle non-takeable modified fields in functional record update | Current conservative V5 (all-takeable) covers hot paths. Widen when non-List/Map fields become bottlenecks. |

### Invariant to add

**No flags in codegen.** Boolean flags that change compilation behavior
globally (like `force_clone`) are forbidden. Every compilation decision must
be derived from the actual type and context of the expression being compiled,
not from a global "are there Rc types anywhere" check. Flags silently
degrade and are impossible to remove incrementally.

---

## The Fully Converged Node

After Tracks B and C complete:

```dag
type Connective = Conj | Disj

type Node {
  name: String
  span: SourceSpan
  children: List<Node>
  connective: Connective?
  params: List<Node>
  return_type: Node?
  body: Node?
  transport: Node?
  properties: List<Node>
}
```

### Why each field is irreducible

| Field | Logical role | Why separate |
|-------|--------------|--------------|
| `children` + `connective` | Composition | The core primitive |
| `params` | Obligations | Consumed, not composed |
| `return_type` | Guarantee | Flows out, not in |
| `body` | Proof / computation | How, not what |
| `transport` | I/O grounding | Must remain structural |
| `properties` | Extensible metadata | Domain facts |

### Pipeline

```text
source -> parse -> resolve -> infer -> emit
           |        |         |        |
         Nodes    Nodes     Nodes    TextFiles
          raw     linked    typed
```

One type flows through the pipeline; each phase enriches it rather than
translating into a parallel representation.

---

## The End State

- self-hosted
- structurally unified
- compositional
- target-polymorphic
- artifact-aware
- bootstrap-free
- fixed-point reproducible
- debuggable (errors trace to `.dag` source; failures reproduce hermetically)
