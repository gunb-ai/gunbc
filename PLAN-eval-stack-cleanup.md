# Plan: eval_stack invariant fixes (S67 items 1-3, 5)

## Fixes

### Fix A: Unify block evaluation (S67 items 1-2)

**Problem:** Two block evaluation paths — `eval_block_s` (suspendable, pushes
continuations) and `eval_expr`'s Block arm (pure, uses re-entrant
`evaluate_stack` for sibling calls). Same computation, different semantics.

**Fix:** Remove the Block arm from `eval_expr`. Route all Block evaluation
through `eval_expr_s` → `eval_block_s`. Since `eval_expr` is called from
`eval_expr_s` as the fallback (`_ => eval_expr(...)`) and `eval_expr_s`
already intercepts `Block` before falling through, this means:

1. In `eval_expr` (line 932): replace the `LoweredExpr::Block` arm with
   an error: `Err(EvalError::new("Block must be evaluated via eval_expr_s"))`
2. The ANF verifier already ensures calls in blocks are at statement level,
   so `eval_block_s` handles them correctly via the continuation stack.
3. Verify: `eval_match_local` and any other pure-path callers that could
   pass a Block to `eval_expr` — these need to route through `eval_expr_s`
   or confirm blocks don't appear in their inputs.

**Risk:** `eval_expr` is called from `eval_non_sibling_call_raw` for
builtin/intrinsic arg evaluation. If a builtin arg is ever a Block, this
would break. ANF normalization should prevent this (blocks in arg position
would have been hoisted), but needs verification.

### Fix B: Unify match evaluation (S67 item 3)

**Problem:** Two match paths — `eval_match_s` (suspendable) and
`eval_match_local` (pure). Both do pattern matching + guard evaluation,
but `eval_match_local` can't suspend on sibling calls in arm bodies.

**Fix:** Remove `eval_match_local`. Route all match evaluation through
`eval_match_s`:

1. In `eval_expr` (line 914-916): replace the `Match` arm with an error
   like Block above.
2. `eval_expr_s` already intercepts `Match` and routes to `eval_match_s`.
3. `eval_match_standalone` (the public API for DAG executor) currently
   calls `eval_match_local` — update it to use `eval_match_s` with a
   temporary empty stack, or keep `eval_match_local` as the standalone-only
   path (since standalone match evaluation has no sibling fns to call).

**Complication:** Guard evaluation. `eval_match_s` comments (lines 728-736)
explain that guards use `eval_expr` (pure) because if a guard suspends,
the continuation model can't represent "check truthiness then maybe try
next arm." This is a real constraint. If guards never contain sibling
calls (ANF ensures calls are hoisted out of guard position), then the
pure path for guards is correct and not a parallel implementation — it's
the only path.

**Decision needed:** Is `eval_match_standalone` (used by DAG executor)
separate enough to justify keeping `eval_match_local`? It has no sibling
fns, so the suspendable path would never suspend. Keeping it avoids
dragging `&mut stack` into a context where there's no stack.

### Fix C: Separate early-return from errors (S67 item 5)

**Problem:** `EvalError { early_return: Option<...> }` conflates errors
with control flow. `return` statements propagate as `Err(EvalError)`.

**Fix:** Change `eval_expr` to return a type that distinguishes values,
early returns, and errors:

1. Option A: `eval_expr` returns `Result<Value, EvalError>` unchanged,
   but `LoweredStmt::Return` in block evaluation produces
   `ExprResult::EarlyReturn` (already exists) instead of using
   `EvalError::early_return`. This only works if Return stmts are only
   in Block context (which they should be after Eval-8 removed
   `LoweredExpr::Return`).

2. Option B: Change `eval_expr`'s return type to an enum:
   `enum ExprValue { Value(Value), EarlyReturn(HashMap<String, Value>) }`
   and remove the `early_return` field from `EvalError`.

**Preferred:** Option A. `LoweredStmt::Return` only appears inside
`LoweredExpr::Block`. After Fix A, all blocks go through `eval_block_s`,
which already returns `ExprResult::EarlyReturn`. So the `EvalError::early_return`
path in `eval_expr`'s Block arm (line 949) becomes dead code. Remove it
and remove the `early_return` field from `EvalError`.

---

## Remaining after these fixes (NOT done)

These items from the design doc / S67 would still be open:

1. **S67 item 4: `wrap_value_as_output` flattens Maps.** The design doc
   acknowledges this as Limitation 3 and says the `"value"` fallback is
   "structurally necessary — not a temporary shim." Not an invariant
   violation — accepted design trade-off.

2. **S67 item 6: positional args dropped in `eval_call_args`.** Latent
   bug for fn-reference callbacks in intrinsics. Low priority — only
   affects a path not yet exercised by real .dag files.

3. **S67 item 7: No tail continuation elimination.** Identity
   continuations (`remaining: &[], binding: None`) accumulate for tail
   calls. Deep mutual recursion at 40K allocates 40K continuations.
   The main contributor to gist pipeline OOM. Not an invariant violation
   (it's correct, just wasteful). Could be fixed by detecting tail
   position in `eval_stmts` and returning `Step::Call` without pushing
   a continuation.

4. **`eval_expr` still handles non-sibling calls.** After fixes A-C,
   `eval_expr` is pure except for `LoweredExpr::Call` → `eval_non_sibling_call_raw`.
   Builtins/intrinsics are deterministic and don't suspend, so this is
   functionally pure — but `eval_non_sibling_call_raw` can re-entrantly
   call `evaluate_stack`. The design doc says "eval_expr never sees a
   Call" but the ANF contract is scoped to sibling calls. Fixing this
   would require hoisting ALL calls (not just sibling calls) to statement
   level — a significant ANF expansion that affects lambda bodies,
   intrinsic args, etc. Not worth doing now.

5. **Slice-based vs absolute-pc continuations.** Known deviation,
   documented in design doc §6. Not an invariant violation.

6. **`Projection` enum.** Not needed (discussed and agreed).

7. **Tail call optimization in `Step::Call`.** Design doc's `cont: Option`
   distinction. Same as item 3 above.

8. **Thread-local `TYPE_WARNINGS` backward compat.** `take_type_warnings()`
   still exists, marked deprecated. Can be removed once all callers
   migrate to `EvalOutcome`.
