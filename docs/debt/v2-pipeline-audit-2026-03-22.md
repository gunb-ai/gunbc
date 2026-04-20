### v2 Pipeline Audit (2026-03-22)

Full line-by-line audit of all 14 v2 .dag files (~16,000 lines). ~100 violations
across 7 structural themes. Root Causes A/B/C above were v1-focused; these are
the v2-native counterparts. Execution order minimizes total work — each theme
unblocks or cheapens the next.

**Execution order:** 4 → 6 → 3 → 5 → 1 → 2 → 7 (interleaved)

#### Why These Exist — Three Root Causes

The 7 themes compress to 3 root causes. Understanding them prevents recurrence.

**I. The IR conflates domain facts with rendering strategy.**

The clearest example was shared semantics carrying Rust policy.
`MethodSemantics` used to carry `wrap_result_in_rc` and `pass_receiver_by_ref`,
and `CallSemantics` used to carry `needs_rc_wrap` — fields that only made sense
if compiling to Rust. Those fields are now gone from `00_core.dag`, but the
lesson remains: once target policy enters shared semantics, every downstream
consumer starts destructuring facts it doesn't own. Python and Go previously had
to pattern-match Rust-only fields they did not use.

Once the IR carries rendering hints, the boundary between "what the program is" and
"how to render it" blurs. Reconcile starts computing Rc decisions. Emit starts
re-resolving types. Dynamic becomes a catchall because the type system serves two
masters. This is the origin of Themes 3, 5, and partially 7.

Prevention: for every field on a core type, ask "would this field make sense if we
were compiling to VHDL?" If no, it doesn't belong in core.

**II. No structural fold over ExprData.**

The DAG language has sum types but no generic visitor. Every consumer writes its own
20-arm match. 5 consumers × 20 variants = 100 match arms that must stay in sync.
Adding one ExprData variant means editing 10+ functions across 6 files.

The "shared dispatch" for Theme 2 is building a `fold_expr` by hand. The reconcile
fusion (Theme 1) is manually combining two handler tables into one match. Both are
workarounds for the language lacking parametric types — you can't write
`fold_expr(handler: ExprHandler<A>, acc: A, expr: Node) -> A` without generics.

Prevention: the language design choice to omit generics forced copy-paste parallelism.
Until/unless the language gains parametric types, the mitigation is to minimize the
number of walks (target: 5) and never add a new one without deleting an existing one.

**III. Define-at-use-site instead of import-from-authority.**

When reconcile needed kernel types, it defined `is_primitive_name()`. When emit needed
them, it defined `build_primitive_set()`. When complexity needed method costs, it
defined `classify_method_cost()`. Each file solved its local problem by copying. Dead
stubs (artifact, trace) are the flip side: code written speculatively, never connected
to an authority. This is Themes 4 and 6.

Prevention: import-first discipline. Before defining a list or classifier, check if an
upstream module already has one. If not, define it in the lowest shared module, then
import.

**All seven themes are symptoms of one thing: the v2 compiler was built bottom-up.**
Each file solved its local problem correctly. Nobody enforced that shared facts flow
downward from a single authority. The fix is to invert the direction: define authorities
first, then build consumers that import from them.

#### Acceptance Criteria — End State

All themes done = every item below is checked. Organized by file so nothing
gets missed during cleanup. Items marked DELETE must not exist; items marked
GONE mean the surrounding function/field no longer exists in that file.

**`00_core.dag`**
- [x] `kernel_types: List<String>` exists (canonical list, 8 entries)
- [x] `is_kernel_type(name: String) -> Bool` exists, uses `kernel_types`
- [x] `LookupCallSemantics` has no `needs_rc_wrap` field
- [x] `IntrinsicMethodSemantics` has no `wrap_result_in_rc` field
- [x] `RuntimeBridgeSemantics` has no `wrap_result_in_rc` or `pass_receiver_by_ref` fields
- [x] `expr_self_call_info(...)` exists and computes both recursion facts in one walk
- [ ] `expr_has_self_call` GONE (compat wrapper removable after downstream imports stop depending on it)
- [ ] `expr_has_non_tail_self_call` GONE (compat wrapper removable after downstream imports stop depending on it)

**`03_resolve.dag`**
- [x] `kernel_type_names()` DELETE — callers import `kernel_types` from core

**`04_reconcile.dag`**
- [x] `is_primitive_name()` DELETE — callers import `is_kernel_type` from core
- [x] `build_type_env` kernel list (lines 3157-3164) replaced with `kernel_types` import
- [x] `build_type_env_unresolved` kernel list (lines 3260-3264) replaced with `kernel_types` import
- [x] `node_is_named_ref` inline kernel exclusion (lines 968-978) uses `is_kernel_type`
- [x] `type_needs_rc` GONE (moved to emit_rust)
- [x] `type_needs_rc_seen` GONE (moved to emit_rust)
- [x] `data_lookup_needs_rc_wrap` GONE (moved to emit_rust as `rust_lookup_receiver_needs_rc_wrap`)
- [x] `rc_wrapped: Bool` GONE from TypeSummary
- [x] `rc_wrapped_types: Map<String, Bool>` GONE from EmitGraphInfo
- [x] `rc_wrapped_types: Map<String, Bool>` GONE from EmitStateAccum
- [x] `emit_info_is_rc_wrapped_type` GONE (moved to emit_rust)
- [x] All `rc_wrapped_types` accumulation logic GONE from `build_emit_graph_info`
- [ ] `infer_expr` and type resolution fused into single walk (`infer_and_resolve_expr`)
- [ ] `collect_calls_in_expr` + `expr_has_self_call` + `expr_has_non_tail_self_call` fused into `analyze_expr_calls` returning `CallAnalysis`
- [ ] Dynamic sites audited: each classified as Correct/Lazy/Fixed, ≤5 justified remaining
- [ ] No string-based method dispatch downstream of the classifiers in reconcile

**`05_emit.dag`**
- [x] `build_primitive_set()` DELETE — callers import `kernel_types` from core
- [x] `ctx_is_rc_wrapped` GONE (Rc concern moved to emit_rust)
- [ ] Shared `emit_typed_expr` dispatch exists (single 20-arm match, target parameter)
- [ ] Shared TCO dispatcher with `TcoSyntax` config exists
- [ ] Shared service/transport traversal exists

**`05_emit_rust.dag`**
- [x] `build_module_vtoe` stub DELETE
- [x] `emit_record_lit` compat wrapper DELETE (tests updated to call `emit_record_lit_full`)
- [x] `resolve_expr_type_node` DELETE
- [x] Rc decision map is derived once in Rust emit entry/module wrappers from `type_summaries`
- [x] `type_needs_rc`, `type_needs_rc_seen` live here (moved from reconcile)
- [x] `rust_lookup_receiver_needs_rc_wrap` lives here (moved from reconcile)
- [x] All 6 Rc-probing heuristics consolidated into the Rust-side Rc pre-pass
- [ ] `emit_typed_expr` 20-arm match GONE (replaced by leaf functions called from shared dispatch)
- [ ] `emit_typed_tco_expr` parallel walk GONE (replaced by shared TCO dispatcher)
- [x] All 18 intrinsic methods handled (no fallback arms)

**`05_emit_python.dag`**
- [ ] `_unimplemented` placeholders (lines 1063, 1076) GONE — real emission or compile error
- [ ] `emit_py_typed_expr` 20-arm match GONE (replaced by leaf functions)
- [ ] All 18 intrinsic methods handled (currently 7)
- [ ] No silent fallback for unhandled methods

**`05_emit_go.dag`**
- [ ] `/* unhandled expr */` wildcard (line 592) GONE — match is exhaustive
- [x] Dead `if wrap_result_in_rc` identity branch (line 659) GONE
- [ ] `emit_go_typed_expr` 20-arm match GONE (replaced by leaf functions)
- [ ] All 18 intrinsic methods handled (currently 7)
- [ ] No silent fallback for unhandled methods

**`06_pipeline.dag`**
- [x] Artifact computation block (lines 170-179) DELETE — `_artifact_output`, `plan`, `artifact` locals all gone
- [x] Go arm wired: `Go => emit_go(typed: typed)` (not error diagnostic)
- [x] `import v2.compiler.emit_go { emit_go }` exists
- [x] `resolve_sources` refactored — shared tokenize→parse→resolve helper with `compile_sources`
- [x] Header comment (line 10) matches reality (mentions Go alongside Rust/Python)

**`07_complexity.dag`**
- [x] `intrinsic_method_cost_shape` is the only method→`CostShape` authority — no parallel `classify_method_cost` or inline string classifier remains
- [ ] `intrinsic_cost_shape` is exhaustive match on IntrinsicMethod — no Option, no None, no wildcard
- [x] `is_size_preserving_method(mname: String)` DELETE — replaced by `is_size_preserving(intrinsic: IntrinsicMethod) -> Bool`
- [x] `is_size_preserving` is exhaustive match — no string comparison
- [ ] `count_self_calls` (lines 1253-1323) DELETE — fused into `cost_of_expr` or uses `CallAnalysis` from reconcile
- [x] `cost_of_expr` reads `method_semantics` from Node, never matches on method name strings

**`07_ownership.dag`**
- [ ] Match arm patterns walk `VariantPattern` bindings (currently skipped)
- [ ] Destructuring patterns updated for any MethodSemantics field changes from Theme 5

**`08_artifact.dag`**
- [x] `plan_artifacts` ModuleBased stub arm (lines 86-88) DELETE
- [x] `plan_artifacts` ServiceBased stub arm (lines 89-91) DELETE
- [x] Only `Explicit` arm remains, or function deleted entirely

**`09_trace.dag`**
- [x] `import std.types { SourceSpan }` fixed to `import v2.std.core { SourceSpan }`
- [x] Interpreter-oriented header/comments reconciled with `src/v2/DESIGN.md` (compiler is a pure transform; no interpreter in the compiler)
- [x] Module connected to pipeline (called from `compile_sources`) or explicitly marked as future work

**Cross-cutting invariant: `00_core.dag` is target-agnostic.**

Every field on every type in `00_core.dag` must satisfy: "this field would make sense
if we were compiling to VHDL, C, or a hardware description language." Fields that
encode a specific target's memory model (Rc, borrow, GC, pointer), execution model
(async, coroutine), or syntax (indentation, braces) do not belong in core.

Rendering decisions are *computed* by emit from domain facts — never *stored* on core
types. If `type_needs_rc` is derivable from cycle detection on the type graph (it is),
it should never have been a field. If `pass_receiver_by_ref` is derivable from Rust's
borrow rules applied to the method's receiver type (it is), it should never have been
a field. The domain model records the facts; each backend derives its strategy.

After cleanup, this grep returns zero results:
```
rg -n '\b(needs_rc_wrap|wrap_result_in_rc|pass_receiver_by_ref|Rc|borrow)\b' src/v2/00_core.dag
```

**Cross-cutting invariant: no new ExprData walks without deleting an existing one.**

Until the language gains parametric types (enabling a generic `fold_expr`), the number
of full ExprData walks is capped at 5. Adding a 6th walk requires justification and
consolidation of an existing pair. The 5 allowed walks are:
1. `infer_and_resolve_expr` (reconcile) — type inference + resolution
2. `analyze_expr_calls` (reconcile) — call graph + recursion detection
3. `walk_expr` (ownership) — binding consumption classification
4. `cost_of_expr` (complexity) — symbolic cost computation
5. `emit_typed_expr` (emit, shared) — target-language rendering

**Cross-cutting invariant: import-from-authority, never define-at-use-site.**

Before defining a list, classifier, or predicate, check if an upstream module already
defines the same concept. If not, define it in the lowest shared module that all
consumers import, then import it. A fact defined at the use site will be copied to the
next use site.

**Cross-cutting verification:**
- [x] `rg -n '"String", "Int", "Bool"' src/v2/*.dag` returns only `00_core.dag`
- [x] `rg -n 'wrap_result_in_rc' src/v2/*.dag` returns 0 results
- [x] `rg -n 'pass_receiver_by_ref' src/v2/*.dag` returns 0 results
- [x] `rg -n 'needs_rc_wrap' src/v2/*.dag` returns only `05_emit_rust.dag`
- [x] `rg -n '\b(needs_rc_wrap|wrap_result_in_rc|pass_receiver_by_ref|Rc|borrow)\b' src/v2/00_core.dag` returns 0 results
- [ ] `grep -r '"Dynamic"' src/v2/` returns ≤5 results in `04_reconcile.dag`, all with justification comments
- [ ] `grep -r '_unimplemented\|/\* unhandled' src/v2/` returns 0 results
- [ ] `grep -rn 'ExprLiteral.*ExprVar.*ExprCall' src/v2/` — full 20-arm ExprData matches exist only in: `infer_and_resolve_expr`, `analyze_expr_calls`, `walk_expr` (ownership), `cost_of_expr`, `emit_typed_expr` (shared). Total: 5 walks, down from 11+.
- [ ] Adding a new ExprData variant requires editing ≤5 match arms (one per walk above)
- [ ] Adding a new intrinsic method requires editing 4 files: core (enum), reconcile (classifier), and one leaf function per target renderer

---

#### Theme 4: Kernel/Primitive Lists → Single Source of Truth

**Invariant:** No duplicate representations. Single-authority metadata.

**Problem:** 4+ copies of the same 6-8 type names (`kernel_type_names`,
`is_primitive_name`, `build_primitive_set`, `build_type_env` hardcoded list),
already drifting (`build_primitive_set` adds `"Char"`).

**Design:** Add to `00_core.dag`:
```
data kernel_types: List<String> = ["String", "Int", "Bool", "Float", "Secret", "Json", "Unit", "Bytes"]
fn is_kernel_type(name: String) -> Bool { kernel_types |> any(t => t == name) }
```

Delete `kernel_type_names()` in 03_resolve, `is_primitive_name()` in 04_reconcile,
`build_primitive_set()` in 05_emit, hardcoded list in `build_type_env`. All import
from core.

**Effort:** ~30 min. Zero risk.

---

#### Theme 6: Dead/Disconnected Infrastructure → Delete or Connect

**Invariant:** No fallbacks that fabricate. No parallel implementations.

| Dead code | Action |
|-----------|--------|
| Pipeline artifact stage (`_artifact_output`, lines 170-179) | Delete |
| `plan_artifacts` stub arms (ModuleBased/ServiceBased) | Delete |
| Artifact/Boundary types | Keep — forward-looking, types are cheap |
| Go pipeline dispatch (returns error despite emit_go existing) | Add `import emit_go`, wire `Go => emit_go(typed: typed)` |
| Trace `import std.types` | Fix to `import v2.std.core` |
| `build_module_vtoe` stub | Deleted |
| `resolve_sources` duplication | Extract shared tokenize→parse→resolve helper |
| `emit_record_lit` compat wrapper | Update tests, delete wrapper |
| Pipeline header comment | Fix to match reality |

**Effort:** ~1 hour. Low risk — mostly deletion.

---

#### Theme 3: String-Keyed Method Dispatch → Enum Everywhere

**Invariant:** No case enumeration for open sets. Single-authority metadata.

**Problem:** The enum pipeline is now mostly in place. `cost_of_expr` and
`receiver_size_var` dispatch on reconcile-provided `MethodSemantics`, and
reconcile now resolves known method semantics/result types once via
`resolve_known_method_node`. Residual string-based method logic still lives
in the source-to-semantics classifiers (`classify_reconciled_intrinsic_method`,
`classify_runtime_bridge_method`) and in per-target renderer leaf dispatch.

**Design:** After reconcile, every method call carries `MethodSemantics` with
`IntrinsicMethodSemantics { intrinsic: IntrinsicMethod, ... }`. Downstream phases
dispatch on the enum, never on strings.

Changes:
1. Keep `intrinsic_method_cost_shape(intrinsic) -> CostShape` as the single cost-shape authority
2. `resolve_known_method_node(...)` is the single reconcile authority for known method semantics and result types
3. `receiver_size_var(...)` reads `MethodSemantics`, not method-name strings
4. Delete the remaining string-based method dispatch downstream of the reconcile classifiers

Single authority chain:
```
string (source) → reconcile → IntrinsicMethod (enum)
                                  ↓
                    emit: rendering per intrinsic
                    complexity: CostShape per intrinsic
                    ownership: edge classification per intrinsic
```

**Effort:** ~2 hours. Medium risk — touches reconcile/emit/complexity.

---

#### Theme 5: Target-Specific Leakage → Ownership as Rendering Concern

**Invariant:** DAG nodes are facts, rendering is separate.

**Problem:** Rc wrapping still spans multiple places inside the Rust renderer
(`type_needs_rc`, pattern-deref heuristics, lookup-specific wrapping, DryRunMode),
even though the Rust-only fields and shared Rc indexes have now been removed from
core/reconcile/shared emit.

**Design:** Reconcile produces target-agnostic facts only. Rust-specific ownership
decisions move to a Rust-specific pre-pass within emit_rust.

Move FROM reconcile to emit_rust:
- `type_needs_rc`, `type_needs_rc_seen` → Rust renderer pre-pass [done]
- `data_lookup_needs_rc_wrap` → Rust renderer (`rust_lookup_receiver_needs_rc_wrap`) [done]
- `rc_wrapped` on TypeSummary → deleted from shared summaries; Rust derives Rc status locally [done]

Move FROM emit shared to emit_rust:
- `resolve_expr_type_node` → deleted; emitter now trusts typed nodes plus narrow local fallback [done]
- All 6 Rc-probing heuristics → compute once in a Rust-side pre-pass via `RcPatternAnalysis` / `RcMatchAnalysis` [done]

Clean up MethodSemantics:
- `wrap_result_in_rc` on IntrinsicMethodSemantics/RuntimeBridgeSemantics → Rust-only context [done in core semantics]
- `pass_receiver_by_ref` on RuntimeBridgeSemantics → same [done in core semantics]
- `needs_rc_wrap` on LookupCallSemantics → same [done in core semantics]

Target-agnostic facts reconcile SHOULD provide:
- "This type is recursive" (structural via children/connective)
- "This type participates in a cycle of depth N" (SCC analysis)
- "This value has N semantic consumers" (ownership analysis)

**Prerequisite:** None, but unblocks Theme 2.
**Effort:** ~3-4 hours. Higher risk — restructures reconcile/emit boundary.

---

#### Theme 1: Parallel ExprData Walks → Fuse Reconcile Passes

**Invariant:** No parallel implementations. No duplicate representations.

**Problem:** 8+ complete 20-arm ExprData walks. Reconcile still has 4. Adding one
expression kind edits 10+ match arms.

**Current reconcile walks:**
1. `infer_expr` — type inference (20 arms)
2. `resolve_expr_types` — type resolution (20 arms)
3. `collect_calls_in_expr` — call graph edges (20 arms)
4. `expr_self_call_info` — shared self-recursion + TCO eligibility walker in `00_core.dag` (20 arms, wrapped by `expr_has_self_call` / `expr_has_non_tail_self_call`)

**Fused design:**

Walk A (`infer_and_resolve_expr`): Combines (1) and (2). Infer each subexpression's
type, resolve it in the same traversal. Single 20-arm match.

Walk B (`analyze_expr_calls`): Combines (3) and the shared self-call analysis. Returns
`CallAnalysis { all_calls, has_self_call, has_non_tail_self_call }`.

Complexity module: `cost_of_expr` + `count_self_calls` → fuse into one walk.

Current reduction: the separate self-call/TCO walks have already been collapsed to one shared core walk. Final target remains 4 reconcile walks → 2. Cost of adding ExprData variant drops from 10+ to ~4.

**Effort:** ~4-5 hours. Medium risk — reconcile is the largest file.

---

#### Theme 2: Triple Renderer Parallelism → Shared Dispatch + Per-Target Leaves

**Invariant:** No parallel implementations.

**Problem:** 3× expression dispatch (60 match arms), 3× TCO, 3× services,
3× resources, 3× data. Python/Go only handle 7/18 intrinsic methods with
silent fabricating fallbacks.

**Design:** One shared expression dispatch in `05_emit.dag`, per-target leaf
functions in emit_rust/emit_python/emit_go.

Shared dispatch:
```
fn emit_typed_expr(texpr: Node, target: RenderTarget, ctx: EmitContext, ...) -> String {
  match texpr.expr_data {
    ExprLiteral { value: v } => target_literal(v, target)
    ExprMethodCall { ... } => target_method_call(recv_str, method, arg_strs, ms, target, ...)
    ...
  }
}
```

Per-target files shrink to leaf renderers only (`target_literal`, `target_call`,
`target_method_call`, `target_match`, etc.). TCO uses shared dispatcher with
per-target syntax config (`TcoSyntax { loop_open, break_prefix, continue_kw }`).

After Theme 3, each renderer provides `render_intrinsic(intrinsic, recv, args) -> String`
covering all 18 intrinsics.

**Prerequisite:** Theme 5 (move Rc to emit_rust) — otherwise shared dispatch
must thread Rust-specific state that Python/Go ignore.

**Size reduction estimate:**
- emit_rust: 3,571 → ~1,200 (leaves + Rust pre-pass)
- emit_python: 1,169 → ~400 (leaves only)
- emit_go: 1,196 → ~400 (leaves only)
- emit shared: 819 → ~1,500 (shared dispatch + helpers)
- Total: 6,755 → ~3,500 (~48% reduction)

**Effort:** ~8-10 hours. Highest risk — largest restructuring. Do last.

---

#### Theme 7: Fabricating Fallbacks → Fail Loud or Implement

**Invariant:** No fallbacks that fabricate. Correctness by construction.

**Problem:** Silent fallbacks produce valid-looking but wrong output.

| Fabrication | Action |
|-------------|--------|
| `Dynamic` as permissive wildcard (~25 reconcile sites) | Audit each: keep justified, fix lazy inference, convert error-masking to diagnostic. Target: <5 justified. |
| Python/Go intrinsic fallthrough (7/18) | Implement all 18 per language (Theme 2). Until then, emit `raise NotImplementedError` / `panic` with context. |
| Go `/* unhandled expr */` wildcard | Make match exhaustive — add remaining ExprData arms. |
| Transport panic/unimplemented | Emit language-specific compile error with context. |
| `plan_artifacts` ignoring config | Delete stubs (Theme 6). |

**Dynamic audit classification per site:**
- **Correct:** genuinely polymorphic position → keep
- **Lazy:** type could be inferred but isn't → fix inference
- **Error-masking:** inference failed silently → convert to diagnostic

**Effort:** Ongoing, ~1 hour per batch of 5 Dynamic sites. Interleave with Themes 1 and 5.

---

