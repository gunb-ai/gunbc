# gunbc Roadmap

**Goal:** Self-hosted v2 compiler. The compiler is written in `.dag`, compiles
itself, and produces identical output when compiling itself again (fixed point).

**Thesis:** Explicit cause-and-effect relationships with basic primitives
(truth-valued structure, `Conj`/`Disj`, composition) are sufficient to express
any information concept. Named types are aliases for compositions; the compiler
should always be able to see through the name to the structure underneath.

---

## Current State (2026-03-19)

### What works

- **455 tests pass** — 363 daglang-emit + 92 v2-compiler-tests (10 ignored)
- **Generated crate compiles** — `v2_crate_cargo_check` passes in <7s
- **B3 Phase 1 complete** — TypedExpr (19 variants) eliminated. Expr carries
  `resolved_type: Node?` directly. TypedNode merged into Node. One AST
  instead of two halves expression memory for self-compile.
- **A1 gist compile** — reconcile 0 errors, 30ms
- **Self-parse + self-resolve** — compiled v2 compiler tokenizes, parses, and
  resolves all 9 .dag modules with zero errors
- **R9 codegen ownership done** — force_clone removed, V5 functional record
  update, SG-10 string comparison, type-directed clone
- **String operations are O(1)** — char_at, string_length, substring, scan_*
- **String params don't clone** — &str in generated code, Copy semantics
- **Node shrunk from ~544b to ~120b** — transport/config boxed
- **Interpreter list_push is O(1)** — Arc COW via try_unwrap
- **TCO loops don't leak** — state moved, not cloned

### What's next

**A4 (full self-compile pipeline)** is the next item on the critical path.
B3 Phase 1 eliminated the TypedExpr parallel AST that caused the 4GB OOM
during self-compile. The self-compile pipeline should now be testable:

1. **Verify self-compile completes** — run `self_compile_all_modules` to
   confirm the OOM is fixed. If it still hangs, profile for the algorithmic
   bottleneck (O(N^2) concat-accumulator patterns in .dag code, documented
   in the postmortem below).
2. **Verify emitted output** — the self-compiled Rust should `cargo check`.
3. **If perf is acceptable:** proceed to A5 (bootstrap stage 0→1).

### Known remaining risks for A4

- **Algorithmic O(N^2) in .dag code** — `concat(acc, [x])` in folds,
  `filter` lookups instead of `map_get`. These cause quadratic behavior
  on the 15K-line self-compile workload. Documented in postmortem.
- **Tokenizer 5.3s per module** — known slow path, separate from OOM.
- **SG-9 workarounds not yet reverted** — TokPos extraction, branch-aware
  use counting. Can be reverted after A4 verification.

### Baseline tests

```bash
cargo test -p daglang-emit --quiet               # 363 tests
cargo test -p v2-compiler-tests --quiet           # 92 tests (10 ignored)
cargo clippy --all-targets -- -D warnings         # clean
cargo test -p v2-compiler-tests v2_crate_cargo_check  # generated crate compiles
```

---

## Dependency Chart

```text
                    ┌─────────────────────────────────────┐
                    │         Self-Hosted Compiler         │
                    │     (A6: fixed point, A7: retire)    │
                    └──────────────┬──────────────────────┘
                                   │
                    ┌──────────────▼──────────────────────┐
                    │       A4-A5: Full Self-Compile       │
                    │   v2 compiles itself → stage 0 → 1   │
                    └──────────────┬──────────────────────┘
                                   │
              ┌────────────────────┤
              │                    │
   ┌──────────▼──────────┐  ┌─────▼───────────────────────┐
   │  Result<T,E> in DSL │  │  B3 Ph1: DONE ✓             │
   │  (Blocker 2)        │  │  TypedExpr eliminated,      │
   │  ← before A6        │  │  Expr has resolved_type     │
   └─────────────────────┘  └──────────────┬──────────────┘
                                           │
                              ┌────────────▼────────────┐
                              │  R9: DONE ✓              │
                              │  V5+SG-10+force_clone rm │
                              └──────────────┬──────────┘
                                             │
                              ┌──────────────▼──────────┐
                              │  A1: DONE ✓              │
                              │  Gist compile 0 err 30ms │
                              └──────────────┬──────────┘
                                             │
                              ┌──────────────▼──────────┐
                              │  R8: DONE ✓              │
                              │  Rc-wrap + SG-6/7/8 fix  │
                              └─────────────────────────┘

Parallel (no dependencies on critical path):

   ┌──────────────────────┐  ┌──────────────────┐  ┌──────────────────┐
   │ B3 Phase 2:          │  │ C3/C4: Language  │  │ D2-D4: Cost      │
   │ Expr→Node patterns   │  │ emission from    │  │ analysis on real │
   │ (after A4 verified)  │  │ extdeps + CLI    │  │ code             │
   └──────────────────────┘  └──────────────────┘  └──────────────────┘

   ┌──────────────────────────────────────────────────────────────────┐
   │ F1: Span preservation (parallel w/ A1-A4)                       │
   │ F2: Interpreter debugger (A5-A6 timeframe)                      │
   │ F3: Hermetic reproduction (post-A6)                             │
   │ F4: Cross-language source maps (post-A7, extends C3/C4)         │
   └──────────────────────────────────────────────────────────────────┘

Deferred (decided, waiting on prerequisites):

   ┌──────────────────────┐
   │ Blocker 1: shared    │
   │ emitter walk (needs  │
   │ v2 self-host)        │
   └──────────────────────┘
```

**Critical path:** A1 (done) → R9 (done) → B3 Ph1 (done) → **A4 (NOW)** → A5 → A6 (needs Blocker 2) → A7

---

## R9 — Codegen Ownership (MOSTLY DONE)

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

### Remaining (parallel with A4, not blocking)

- **Verify at scale:** Run `self_compile_all_modules` with V5 + B3 Phase 1.
  B3 eliminated the OOM root cause; V5 + force_clone removal should give
  O(1) collection mutations. First A4 run will verify both.
- **SG-11:** `stacker::maybe_grow(512KB, 2MB)` on every function — 530
  calls. Fix: only wrap genuinely recursive functions (reuse TCO detection).
- **SG-12:** Rc-wrapping Copy-sized types (SourceSpan = 16 bytes). Fix:
  extend `is_simple_enum` detection to small all-Copy structs.
- **Revert SG-9 workarounds:** After A4 verification, revert TokPos
  extraction, branch-aware use counting.
- **Widen V5:** Currently limited to all-takeable modified fields. Extend
  to handle non-takeable fields (e.g., by substituting source ident with
  __owned in compile context).

### Performance Fallback Inventory

Every instance of `Rc::try_unwrap(x).unwrap_or_else(|rc| (*rc).clone())`
in the generated code is a **performance fallback**: correct on both paths,
but the clone path is O(N) and fires silently when refcount > 1. The
`force_clone` flag (fn_codegen.rs:1754) currently ensures refcount ≥ 2 at
ALL sites, meaning every fallback fires 100% of the time.

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

#### Root causes (why fallbacks always fire)

| Hack | Line | What it does | Why it exists |
|------|------|-------------|---------------|
| `force_clone` | 1754 | Clones ALL variable refs when Rc types exist | Avoids borrow-checker errors in generated code |
| `fold_accum_name` exemption | 1757-1762 | Exempts fold accumulators from force_clone | So fold bodies get O(1) list_push |
| Clone stripping | 2343, 2367, 2556 | Removes `.clone()` from fold accumulator args | So try_unwrap sees refcount 1 |
| `is_var_with_rc_type` | 1806 | Returns true for ANY Var when Rc types exist | Heuristic — no actual type lookup |

#### .dag source workarounds (should be reverted after V5)

| Workaround | File | What it does |
|-----------|------|-------------|
| TokPos extraction | 01_tokenize.dag | Pulled `tokens` out of `TokenizerState` so TCO gives refcount 1 |
| Branch-aware use counting | fn_codegen.rs | Max across branches instead of sum — still defeated by force_clone |

### Acceptance

- [ ] `self_compile_all_modules` completes in <5 seconds
- [ ] All SG-9 .dag workarounds reverted (TokPos extraction, fold_accum
  exemption, branch-aware use counting)
- [ ] 455 tests pass, generated crate compiles clean

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
| F3 | `SemanticsCtx` uses all-String fields (backend, exec_model, etc.) | D2 timeframe | The comment "so new backends can be added without modifying this type" is exactly what the invariants warn against. Should be enums/newtypes. |
| F4 | `PrimCost.op: String, model: String` in 07_complexity.dag | D2 timeframe | Typo in op name = silent wrong cost. Should be typed references. |
| B4-val | `validate_no_unresolved()` is post-hoc validation (Invariant 9) | B3 | Already tracked. Delete when pipeline boundary types make unresolved structurally unrepresentable. |
| F5 | `infer → reconcile` rename lacks contract justification | Documentation | Rename already done. Needs documented rationale tied to contract change (reconciliation = bidirectional, not just inference). |

---

## Completed: R8 — Rc-Wrap Generated Types

**Structural principle:** The DAG language has value semantics with no mutation.
Every value is logically shared — using a value twice doesn't require two
copies. The codegen maps DAG values to Rust ownership, where using a value
twice requires `.clone()`. That's the mismatch. The fix is uniform: **DAG
values are shared; Rust represents sharing as `Rc`.**

```text
DAG value semantics          Rust representation
─────────────────           ────────────────────
String                  →    &str param (Copy)         ← R3 done
Int, Bool               →    i64, bool (Copy)          ← already free
List<T>                 →    Rc<Vec<T>>                ← already done
Map<K,V>                →    Rc<HashMap<K,V>>          ← runtime ops done
struct Node { ... }     →    Rc<Node>                  ← THIS IS R8
enum TokenKind { ... }  →    Rc<TokenKind>             ← THIS IS R8
```

**Rule:** Every non-Copy generated type (struct or non-unit-variant enum) is
Rc-wrapped at all usage sites. `compile_ident` can emit `.clone()` freely on
any variable — it's always O(1). No type-specific checks, no special cases.

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
- [ ] all 455 tests pass
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
- **S4:** 92 v2-compiler-tests pass, v2_crate_cargo_check passes. Generated
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

**TODO in B3:** delete `validate_no_unresolved()` and replace with structural
type distinction (e.g., `ResolvedNode` wrapper or equivalent).

**Review follow-up (2026-03-19):** two current bugs are symptoms of this
same boundary problem, not separate one-off fixes:
- `FuncSig` requires a concrete `return_type` too early, so
  `build_func_env()` fabricates placeholders for unannotated functions and
  `infer_expr(Call)` can observe the fake type instead of the resolved one.
- `compile_sources_lenient()` exists to push a graph with unresolved
  typecheck state across the typecheck → emit boundary during bootstrap.

The sustainable fix is **not** best-effort typing. B3 Phase 2 should split
declared-vs-resolved function signatures, resolve function return types
before call inference can consume them, and make the emit boundary accept
only a structurally resolved graph.

---

## Track A: Self-Hosting

**Dependencies:** R8 → A1 (done) → R9 (done) → B3 Ph1 (done) → **A4 (NOW)** → A5 → A6 (Blocker 2) → A7

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

**Acceptance:**
- [ ] v2 crate processes its own .dag source through the full pipeline
- [ ] emitted Rust files compile with `cargo check`
- [ ] no OOM or stack overflow on any .dag file up to 4000 lines
- [ ] self-compile ratchet asserts semantic properties stronger than
      "non-empty file emitted"

### A5: Bootstrap stage 0 → 1

```text
v1 compiles v2 .dag → Rust → rustc → v2-stage0
v2-stage0 compiles v2 .dag → Rust → rustc → v2-stage1
```

### A6: Fixed point

`stage1 output == stage2 output`

### A7: v1 retirement

v2 builds and tests without v1 in the dependency chain.

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

#### Phase 2: Expr → Node patterns (after A4 verified)

Convert Expr variants to Node patterns. After this, "typed" just means
"return_type is filled in" and the pipeline shape is `Node → Node → Node → TextFile`.

Also: delete `validate_no_unresolved()` (Blocker 4). When pipeline boundary
types are reworked here, make resolved-vs-unresolved a type distinction
rather than a runtime check. The validation pass violates Invariant 9.

Concrete boundary work required here:
- Split function signature state into declaration-time vs resolved-time
  representations. `infer_expr(Call)` must not read from a signature that
  can still contain a fabricated or placeholder return type.
- Resolve function return types in dependency order over the call graph
  (SCC-aware). Recursive SCCs without explicit return annotations must fail
  closed instead of guessing.
- Introduce a resolved reconcile/emit boundary type so emit only accepts
  graphs where every item/expr type is already structurally resolved.
- Delete `compile_sources_lenient()` once the remaining bootstrap false
  positives are fixed. It currently exists only because the boundary type is
  too permissive.
- Do **not** weaken `typecheck_module()` into a partial/best-effort mode.
  The fix is stronger output types, not more lenient control flow.

Also: batch fix F2/F3/F4 (string-typed fields):
- `ItemInfo.kind: String` → closed enum `ItemKind = Fn | Func | Other`
- `SemanticsCtx` all-String fields → enums/newtypes
- `PrimCost.op: String, model: String` → typed references

**Acceptance:**
- [x] `Typed*` family deleted (Phase 1)
- [ ] `Expr` type deleted from `00_core.dag`
- [ ] `validate_no_unresolved()` deleted; replaced with structural type boundary
- [ ] Declared vs resolved function signatures are distinct; no fabricated
      placeholder return types in `FuncEnv`
- [ ] Call-graph/SCC return-type resolution fails closed for recursive
      unannotated functions
- [ ] `compile_sources_lenient()` deleted; bootstrap uses the strict
      resolved type boundary
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

**Design (decided 2026-03-18):** The v1 compiler already counts variable
uses. The ownership proof tightens this:
- If use_count == 1 at a try_unwrap site: emit `Rc::into_inner().expect()`
  (no fallback, panic on violation).
- If use_count > 1: compile error — "cannot guarantee O(1) mutation,
  restructure the code."

This replaces ALL `Rc::try_unwrap(x).unwrap_or_else(|rc| (*rc).clone())`
instances (14 codegen + 4 runtime sites documented in Performance Fallback
Inventory) with statically verified moves. No runtime fallback exists.

**Staging:**
- **Now (parallel):** Design the ownership analysis pass. Identify which
  try_unwrap sites can be converted to use_count == 1 checks today.
- **After force_clone removal:** Most sites become statically provable
  (use_count drops to 1 when force_clone stops inflating it).
- **After A7 (deferred):** Full linear type checking in v2 compiler.
  More robust than use counting but requires v2 self-hosting.

### D2: Typed summaries

Infer symbolic summaries from typed expressions/functions. Per-function
`ComplexitySummary` with `work`, `span`, `output_size` as symbolic `CostExpr`.

Also: batch fix F3/F4 string-typed fields in 07_complexity.dag:
- `SemanticsCtx` all-String fields → enums/newtypes
- `PrimCost.op: String, model: String` → typed references

### D3: DAG composition

Compose summaries over lowered DAG. DAG work = sum of node work, span =
longest dependency path, loop work = iteration count × body work.

### D4: Proofs and reporting

Surface complexity as proof/report. Policy checks can reject unbounded
workflows.

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
