# Plan: Remaining eval_stack cleanup (parallel tracks)

## Track layout

Six independent tracks that can be implemented in parallel worktrees.
Tracks 1-2 share a dependency (both touch `eval_expr` and `EvalError`)
so they should be merged sequentially; all others are fully independent.

---

## Track 1: Move Block/For to statement forms (S67 items 1-3)

**Goal:** Eliminate the dual block/match evaluation paths by making
Block and For statement-level constructs in the IR.

**Files:**
- `daglang-eval/src/expr.rs` — add `LoweredStmt::Block(Vec<LoweredStmt>)`
  and `LoweredStmt::For { binding, iterable, body: Vec<LoweredStmt> }`
- `daglang-eval/src/eval_stack.rs` — remove `LoweredExpr::Block` and
  `LoweredExpr::For` arms from `eval_expr`; handle them in `eval_stmts`
  (which already has the continuation stack)
- `daglang-lower/src/anf.rs` — update ANF normalization for new stmt forms
- `daglang-lower/src/lib.rs` — update lowering to emit stmt forms
- `daglang-lower/src/expr.rs` — update `lower_fn_body_with_mode`

**Steps:**
1. Add `LoweredStmt::Block` and `LoweredStmt::For` variants
2. Update `eval_stmts` to handle them (using existing `eval_block_s` logic)
3. Remove `LoweredExpr::Block` and `LoweredExpr::For` from the enum
4. Update `anf.rs` normalizer and verifier
5. Update all lowering sites that emit Block/For expressions
6. Delete `eval_block_pure`, `eval_block_s`, `eval_match_local` —
   unify into the single `eval_stmts` path

**Note:** `eval_match_local` can be kept for `eval_match_standalone`
(DAG executor) since that context has no sibling fns and no continuation
stack. But document it as the standalone-only path.

## Track 2: Remove `early_return` from `EvalError` (S67 item 5)

**Depends on:** Track 1 (once Block is a statement form, Return stmts
only appear at statement level in `eval_stmts`, never inside `eval_expr`)

**Goal:** `EvalError` becomes a pure error type. No control-flow signals.

**Files:**
- `daglang-eval/src/eval_core.rs` — remove `early_return` field and
  `EvalError::early_return()` constructor
- `daglang-eval/src/eval_stack.rs` — remove `step_from_error` /
  `expr_from_error` conversion functions; `eval_expr` errors are always
  real errors

**Steps:**
1. After Track 1, verify no `eval_expr` call site can encounter a Return
2. Remove `early_return` field from `EvalError`
3. Remove `EvalError::early_return()` constructor
4. Simplify `step_from_error` → direct `Step::Error(e.message)`
5. Simplify `expr_from_error` → direct `ExprResult::Error(e.message)`

## Track 3: Fix positional arg dropping (S67 item 6)

**Goal:** `eval_call_args` and `eval_non_sibling_call_raw` preserve
positional args instead of silently discarding them.

**Files:**
- `daglang-eval/src/eval_stack.rs` — `eval_call_args` (line ~987),
  `eval_non_sibling_call_raw` (line ~998)

**Steps:**
1. Audit all call sites that produce `(None, expr)` positional args —
   these come from intrinsic fn-reference paths (e.g., `sort_by(list, f)`
   where `f` is a fn reference resolved to a synthetic call)
2. In `eval_call_args`: when `param_name` is `None`, assign a positional
   name (`"__pos_0"`, `"__pos_1"`, etc.) instead of dropping the value
3. In `eval_non_sibling_call_raw`: same fix for the sibling-fn-call path
   and the pre-evaluated args path
4. Add a test: intrinsic with fn-reference callback that passes positional
   args (e.g., `sort_by(list, compare_fn)` where `compare_fn` is a
   sibling fn)

## Track 4: Tail continuation elimination (S67 item 7)

**Goal:** Tail calls don't push identity continuations. Reduces heap
usage from O(call_depth) to O(non-tail call_depth).

**Files:**
- `daglang-eval/src/eval_stack.rs` — `eval_stmts`, `run_machine`

**Steps:**
1. Detect tail position in `eval_stmts`: a `Call` in the last statement
   with no remaining statements is a tail call
2. For tail calls, return `Step::Call` without pushing a continuation
3. In `run_machine`, `Step::Call` replaces `fn_id`/`stmts`/`env`
   directly (no stack push) — this is the design doc's `cont: None` path
4. Non-tail calls continue to push continuations as today
5. Verify: `deep_mutual_recursion_40k` test should now use O(1) stack
   for pure tail recursion
6. Add a test that verifies stack depth stays bounded for tail-recursive
   patterns

## Track 5: Eliminate `wrap_value_as_output` Map flattening (S67 item 4)

**Goal:** Functions always return `{"return": value}`. No Map flattening.

**Files:**
- `daglang-eval/src/eval_stack.rs` — `wrap_value_as_output`,
  `output_value`, `extract_projection`

**Steps:**
1. Change `wrap_value_as_output` to always wrap:
   `[("return".to_string(), value)].into_iter().collect()`
2. Change `output_value` to always extract from `"return"` key — remove
   the `"value"` fallback and Map reconstruction
3. Update `bind_let_result` callers that rely on Map flattening — these
   need explicit field projection at the call site (the lowerer should
   emit `name__field` bindings from Let stmts that destructure Maps)
4. Verify the v2 DSL evaluation model still works — this is the main
   risk area (Limitation 3 in the design doc)
5. If Map flattening is still needed for the v2 model, keep it in
   `bind_let_result` only (not in `wrap_value_as_output`), and document
   it as the single point of Map destructuring

**Risk:** This is the item the design doc says is "structurally
necessary." May need to keep `bind_let_result` flattening but remove
`wrap_value_as_output` flattening. Spike first.

## Track 6: Remove thread-local `TYPE_WARNINGS` (complete EvalOutcome migration)

**Goal:** All callers use `EvalOutcome`. Remove `take_type_warnings()`
and the thread-local `TYPE_WARNINGS`.

**Files:**
- `daglang-eval/src/eval_stack.rs` — remove `TYPE_WARNINGS`,
  `WarningState`, `WarningScope`, `take_type_warnings()`,
  `push_type_warning()`
- `daglang-eval/src/lib.rs` — remove `take_type_warnings` export
- All callers of `evaluate_stack` that need warnings → migrate to
  `evaluate_stack_with_diagnostics`

**Steps:**
1. Grep for all `take_type_warnings` and `evaluate_stack` call sites
2. Migrate callers that need warnings to `evaluate_stack_with_diagnostics`
3. For callers that don't need warnings, keep using `evaluate_stack`
   (which internally discards them — that's fine)
4. Replace `push_type_warning()` with direct `Vec` accumulation on a
   warnings parameter threaded through `check_call_inputs` /
   `check_return_value`
5. Remove `TYPE_WARNINGS` thread-local, `WarningState`, `WarningScope`
6. Remove `take_type_warnings()` from public API

---

## Merge order

```
Track 3 (positional args)  ─────────────────────────────→ merge
Track 4 (tail elim)        ─────────────────────────────→ merge
Track 5 (Map flattening)   ──── spike first ───────────→ merge if safe
Track 6 (thread-local)     ─────────────────────────────→ merge
Track 1 (Block/For stmts)  ─────────────────────────────→ merge
Track 2 (early_return)     ── depends on Track 1 ──────→ merge last
```

Tracks 3, 4, 5, 6 are fully independent of each other and of Track 1.
Track 2 depends on Track 1. All can start in parallel; Track 2 just
waits for Track 1 to land before its final step.
