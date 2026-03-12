# Evaluator Redesign: Pure Explicit-Stack Evaluation

## Problem

The DSL evaluator (`daglang-eval/src/eval.rs`) interprets `LoweredFnBody`
trees by recursing on the native Rust call stack. The v2 self-hosted
parser has ~80 mutually recursive functions; interpreting 12 modules with
150+ type definitions overflows even 128MB stacks (S52).

## Governing Invariants

From `src/README.md` — these constrain the solution space:

**Pure core logic.** Deterministic functions from inputs to outputs. The
evaluator is pure computation — no I/O, no side effects. The redesign
must preserve and strengthen this property, not weaken it.

**Clear interfaces.** Every public module should have a small,
well-defined API surface. Prefer returning values over mutating shared
state.

**No parallel implementations.** When the same computation exists in two
forms, they diverge. The redesign must not introduce a second evaluator
alongside the first — it replaces the recursive core.

**No duplicate representations.** If the same state is stored in two
places (e.g., call depth in both Env and CallFrame), they will diverge.

## Design Invariants (this redesign)

**DI-1: Purity.** Every eval function is a pure function: immutable
inputs, explicit outputs, no hidden mutation. The function signature
tells you everything it can do.

**DI-2: Efficiency.** O(1) operations stay O(1). Specifically:
- Variable lookup: O(log n) with persistent map (n = bindings in scope,
  typically < 100)
- Let-binding: O(log n) (persistent map insert)
- Env creation for a child scope: O(1) (structural sharing)
- Literal evaluation: O(1)

**DI-3: No hidden output channels.** A function's return type must
describe ALL its possible outcomes. No encoding control flow (tail calls,
early returns) as error variants.

**DI-4: Separation by change frequency.** Data that changes at different
rates lives in different structures. This prevents needless copying and
makes mutation boundaries explicit.

## Current State (what's wrong)

```rust
fn eval_expr(
    expr: &LoweredExpr,
    env: &Env,                                    // mutable Rc internals
    sibling_fns: &HashMap<String, LoweredFnBody>, // threaded manually
) -> Result<Value, EvalError>                      // EvalError = 4 outcomes
```

Purity violations:
1. `Env` has `Rc<HashMap>` bindings — copy-on-write with hidden aliasing.
   Two Envs may share a backing map; `bind()` clones conditionally based
   on refcount. Caller cannot reason about cost or sharing.
2. `EvalError` encodes four distinct outcomes (value, error, tail-call,
   early-return) in one type. Functions lie about their behavior.
3. `sibling_fns` and `data_values` are threaded through every call but
   never change. They're constant context, not per-call state.
4. `Env` bundles four fields that change at different rates: bindings
   (per statement), data_values (never), call_depth (per fn call),
   self_name (per fn call).

## Proposed Decomposition

### Data structures, split by change frequency (DI-4)

```rust
/// Constant for the entire evaluation. Created once, read everywhere.
/// Passed as `&EvalContext` — no cloning, no Rc. Continuations borrow
/// fn bodies directly via `&'a LoweredFnBody`.
struct EvalContext {
    sibling_fns: HashMap<String, LoweredFnBody>,
    data_values: HashMap<String, Value>,
}

/// Variable bindings. Changes every let-binding.
///
/// Immutable value type — `bind()` returns a new Env (DI-1).
/// Uses `im::HashMap` for O(1) clone, O(log n) insert/lookup.
/// Semantics match the API: bind() returns a new Env, the old one
/// is unchanged, no hidden sharing to reason about.
struct Env {
    bindings: im::HashMap<String, Value>,
}

/// Per-fn-call metadata. Changes at fn-call boundaries only.
struct FrameInfo {
    call_depth: usize,
    fn_name: Option<String>,
}
```

**Env uses `im::HashMap`** (DI-1 over micro-optimization). The current
`Rc<HashMap>` with `Rc::make_mut` simulates persistence through interior
mutation — exactly the kind of hidden-aliasing-with-conditional-cloning
that DI-1 aims to eliminate. `im::HashMap` is actually persistent: O(1)
clone via structural sharing, O(log n) insert and lookup. With n < 100
bindings in a typical scope, log n < 7 — negligible.

### Return types (DI-3)

One result type, not two. `eval_expr` returns `ExprResult`. `eval_body`
is the layer that processes a sequence of statements and constructs
continuations from `ExprResult::Call` — it returns `Action`.

```rust
/// Result of evaluating a single expression within a fn body.
/// With the ANF invariant, expressions never contain fn calls.
/// eval_expr is a total function over pure expression trees —
/// it always terminates and never suspends.
enum ExprResult {
    /// Normal: produced a value.
    Value(Value),
    /// Early return from the enclosing fn body.
    Return(HashMap<String, Value>),
    /// Evaluation error.
    Error(String),
}

/// What the main loop should do next.
/// Returned by `eval_body`, which runs statements until it finishes
/// or hits a call it can't resolve locally.
enum Action {
    /// Fn body completed with output fields.
    Done(HashMap<String, Value>),
    /// Need to call a sibling fn before continuing.
    Call {
        callee: String,
        inputs: HashMap<String, Value>,
    },
    /// Evaluation error.
    Error(String),
}
```

`eval_expr` never returns `Call` — the ANF invariant guarantees calls
only appear at statement level. `eval_body` is the only layer that sees
calls, and it produces `Action::Call` with a `Continuation` pushed to
the heap stack.

### The continuation (suspended computation)

Slim — just the data needed to resume the caller after the callee returns.

```rust
struct Continuation<'a> {
    /// What to bind the call result to.
    binding: String,
    /// The fn body being evaluated (borrowed from EvalContext).
    body: &'a LoweredFnBody,
    /// Index of the statement to resume at (the one after the call).
    resume_index: usize,
    /// The env at the point of suspension.
    env: Env,
    /// Frame metadata for the suspended fn.
    frame: FrameInfo,
}
```

No `callee_name` — the main loop already knows who it called.
No `is_fn_body` flag — the structural position (top of stack vs not)
determines top-level-vs-nested semantics. No `Option<String>` on
binding — if a call result isn't bound to anything, it's a tail call
(no continuation pushed; see below).

**No AST cloning, no Rc.** The continuation borrows `&'a LoweredFnBody`
directly from `EvalContext`, which outlives the entire evaluation. The
main loop, stack, and all continuations share the lifetime `'a` tied to
`&'a EvalContext`. Creating a continuation is O(1) — a pointer and an
integer. No reference counting, no allocation.

This works because the main loop is an iterative `loop {}` — not
recursive. The `Vec<Continuation<'a>>` lives on the heap alongside the
loop's local variables. Every `&'a` reference points into `EvalContext`,
which is immutable and outlives the loop. No borrow checker conflicts.

**Heap cost per continuation:** O(1) fixed overhead + the `Env` snapshot.
With `im::HashMap`, the Env snapshot is O(1) (structural sharing). So
total heap for N suspended calls = O(N). At MAX_CALL_DEPTH=10,000, each
continuation is ~80 bytes (pointer + index + Env pointer + FrameInfo),
total ~800KB. Bounded and predictable.

### The main loop

```rust
pub fn evaluate_fn_body<'a>(
    body: &'a LoweredFnBody,
    inputs: HashMap<String, Value>,
    ctx: &'a EvalContext,
) -> Result<HashMap<String, Value>, String> {
    let mut stack: Vec<Continuation<'a>> = Vec::new();
    let mut body: &'a LoweredFnBody = body;
    let mut start_index: usize = 0;
    let mut env = Env::from_inputs(inputs, ctx);
    let mut frame = FrameInfo { call_depth: 0, fn_name: None };

    loop {
        match eval_body(&body.stmts[start_index..], &env, &frame, ctx) {
            Action::Done(result) => {
                match stack.pop() {
                    None => return Ok(result),
                    Some(cont) => {
                        let value = extract_return_value(&result);
                        env = cont.env.bind(&cont.binding, value);
                        body = cont.body;
                        start_index = cont.resume_index;
                        frame = cont.frame;
                    }
                }
            }
            Action::Call { callee, inputs } => {
                let callee_body = ctx.sibling_fns.get(&callee)
                    .ok_or_else(|| format!("unknown function: {callee}"))?;
                body = callee_body;
                start_index = 0;
                env = Env::from_inputs(inputs, ctx);
                frame = FrameInfo {
                    call_depth: frame.call_depth + 1,
                    fn_name: Some(callee),
                };
            }
            Action::Error(msg) => return Err(msg),
        }
    }
}
```

The loop always does the same thing: evaluate a body from a given index.
No separate `resume` function — resuming IS binding a value and
continuing the loop. Fewer concepts, fewer code paths. Zero allocation
per iteration — all references borrow from `&'a EvalContext`.

### Tail-call optimization

Falls out naturally from the loop structure. When `eval_body` encounters
a sibling call as the last expression in the last statement with no
remaining work:

- `remaining_stmts` would be empty
- `binding` would be meaningless (nothing to bind to)

So `eval_body` simply doesn't push a continuation — it returns
`Action::Call` directly. The main loop replaces `body`/`env`/`frame`
with the callee's. No special trampoline mechanism needed.

This handles both self-recursive (`A→A`) and mutual (`A→B→A`) tail
calls identically. The stack doesn't grow because nothing is pushed.

### Expression evaluation stays recursive — via ANF invariant

Expression evaluation within a single fn body uses the native Rust stack.
This is safe IF expressions cannot contain fn calls — which we enforce
structurally.

**ANF (A-Normal Form) lowering:** The `daglang-lower` phase hoists all
nested fn calls into synthetic `let` statements. After lowering,
`LoweredExpr::Call` only appears as the RHS of a `Let` statement or as
a bare `Expr` statement — never nested inside another expression.

```
// Source:      let x = g(f(a))
// Lowered ANF: let __tmp0 = f(a)
//              let x = g(__tmp0)
```

With this invariant, `eval_expr` never encounters `ExprResult::Call`. It
evaluates pure expression trees (arithmetic, field access, string interp,
match, etc.) and always returns `ExprResult::Value`. Only `eval_body`
sees calls, and it handles them via the heap stack.

**This makes the structural invariant trivially true:** `eval_expr` cannot
trigger fn-call recursion because it never sees a call. The native stack
is bounded by expression tree depth (operators, field access, conditionals)
which is syntactic, not input-dependent.

**Verification:** After lowering, assert that no `LoweredExpr::Call`
appears nested inside another `LoweredExpr`. This is a structural
property of the IR, checkable with a single walk.

**Migration note:** If the current lowerer does not enforce ANF (calls
can appear nested in expressions), this must be added as a pre-step
before the evaluator redesign. The lowerer change is mechanical — hoist
calls into `let __tmpN = call(...)` — and does not change semantics.

## `eval_body` internals

`eval_body` walks statements sequentially. For each statement:

```
Let(name, expr):
    result = eval_expr(expr, env, frame, ctx)
    match result:
        Value(v)  → env = env.bind(name, v); continue
        Call{..}  → push Continuation{binding: name, remaining, env, frame}
                    return Action::Call{..}
        Return(m) → return Action::Done(m)
        Error(e)  → return Action::Error(e)

Expr(expr):        // last stmt: its value is the fn's return
    result = eval_expr(expr, env, frame, ctx)
    match result:
        Value(v)  → return Action::Done(wrap_return(v))
        Call{..}  → if is_last && no_remaining:
                        return Action::Call{..}    // tail call, no continuation
                    else:
                        push Continuation{..}
                        return Action::Call{..}
        Return(m) → return Action::Done(m)
        Error(e)  → return Action::Error(e)

Return(fields):
    evaluate each field expr, collect into HashMap
    return Action::Done(result)
    (if a field expr returns Call, push continuation)
```

## Recursion Bounds Analysis

The evaluation call chain has distinct levels, each with a different
recursion type and bound:

```
main_loop                            — iterative, heap stack
  └→ eval_body(stmts, env, frame)    — iterative loop over stmts
       └→ eval_expr(expr, env)       — native-stack recursive
            ├→ eval_expr(sub_expr)   — native-stack recursive
            ├→ eval_binop(lhs, rhs)  — leaf, O(1)
            ├→ eval_intrinsic(...)   — iterates over list, calls eval_expr per item
            │    └→ eval_expr(body)  — native-stack recursive
            └→ [sibling fn call]     — returns ExprResult::Call, does NOT recurse
```

### Per-level bounds

| Level | Type | Bounded by | Practical max | Heap cost |
|-------|------|------------|---------------|-----------|
| main_loop | iterative (heap) | MAX_CALL_DEPTH | 10,000 | O(N) continuations, ~1MB at 10K |
| eval_body | iterative (stmt loop) | stmt count per fn | ~20 | O(1) — no allocation |
| eval_expr | native recursive | AST expression depth | ~15-30 | O(1) — native stack only |
| eval_intrinsic | iterative (item loop) | list length | unbounded* | O(1) — no allocation |
| eval_expr in intrinsic | native recursive | AST depth of lambda | ~5-10 | O(1) — native stack only |

### The structural invariant

**`eval_body` never calls `eval_body`.** This is the key property that
makes the heap stack work. `eval_body` calls `eval_expr`; `eval_expr`
may recurse into itself but returns `ExprResult::Call` instead of
recursing into `eval_body`. Only the main loop calls `eval_body`. This
is statically verifiable by inspecting the code — grep for call sites.

If `eval_body` never calls itself (directly or transitively through
`eval_expr`), then the native stack has at most one `eval_body` frame
at any time. All fn-call stacking is in `Vec<Continuation>` on the heap.

### The intrinsic loophole (resolved by ANF)

**Original concern:** Intrinsics like `map(list, lambda)` iterate over
items and call `eval_expr` for the lambda body. If the lambda contains
a sibling fn call, it could bypass the heap stack.

**ANF resolves this.** After ANF lowering, lambda bodies cannot contain
fn calls — calls are hoisted to statement level. Intrinsic lambdas are
pure expression trees (`x => x.name`, `x => x + 1`). `eval_expr` on a
lambda body always returns `ExprResult::Value`, never suspends.

If a DSL program writes `map(list, x => f(x))`, the lowerer transforms
it into a `for` loop with explicit statements:
```
for x in list {
    let __tmp = f(x)    // call at statement level
    __tmp               // pure expression
}
```

The `for` loop's body is evaluated by `eval_body`, which handles the
call via the heap stack. No special intrinsic suspension needed.

**Verification:** assert that no `LoweredExpr::Call` appears inside
`LoweredExpr::Lambda` after lowering.

### Total resource bounds

At MAX_CALL_DEPTH = 10,000:
- **Heap stack:** 10,000 continuations × ~100 bytes = ~1MB
- **Env snapshots:** O(1) each via `im::HashMap` structural sharing.
  Total unique data = O(total bindings created), not O(depth × bindings).
- **Native stack:** O(max_expr_depth) ≈ 30 frames × ~200 bytes ≈ 6KB.
  Well within default 8MB stack.
- **No AST cloning:** continuations hold Rc + index, not cloned stmts.

The heap stack is bounded by MAX_CALL_DEPTH (explicit limit, clean
error on exceeded). There is no path to unbounded heap growth.

## Migration Path

Incremental, test-by-test. Each step compiles and passes all tests
before the next begins.

0. **ANF lowering** — ensure `daglang-lower` hoists all fn calls to
   statement level. After this step, `LoweredExpr::Call` never appears
   nested inside another expression or inside a lambda. Add a structural
   assertion to verify. This is a prerequisite — without it, the
   evaluator redesign cannot guarantee bounded native stack.

1. **Introduce `EvalContext`** — extract `sibling_fns` and `data_values`
   from `Env`. Thread `&'a EvalContext` through all eval functions. `Env`
   still has `Rc<HashMap>` bindings for now.

2. **Introduce `FrameInfo`** — extract `call_depth` and `self_name` from
   `Env`. Thread alongside `&EvalContext`.

3. **Replace `EvalError` with `ExprResult`** — two sub-steps:
   a. Introduce `ExprResult` alongside `EvalError` with conversion
      functions. Get everything compiling with both types coexisting.
   b. Remove `EvalError` control-flow variants (`tail_call`,
      `early_return`). Replace all call sites. `EvalError` becomes a
      plain error type (just `message: String`).

4. **Introduce `Continuation` and the main loop** — `eval_body` returns
   `Action`. The main loop manages `Vec<Continuation<'a>>`. Tail-call
   optimization built in from the start (don't push when remaining is
   empty).

5. **Switch Env to `im::HashMap`** — replace `Rc<HashMap>` with
   persistent map. `bind()` returns a new Env. Drop `Env::child()`.

6. **Delete old recursive `eval_fn_body_rc` path.**

7. **Remove `with_parser_stack(32MB)` from v2 tests.** Un-ignore
   `phase6_gist_full_pipeline`.

## Relationship to v2 Parser

The v2 self-hosted parser (`src/v2/`) is the primary consumer that
motivates this redesign. The parser's ~80 mutually recursive functions
are the workload that overflows the stack. Two considerations:

1. **Parser function structure matters.** If parser functions are
   structured so that recursive calls are in tail position (common in
   recursive-descent parsers), the evaluator's TCO eliminates stack
   growth entirely. Non-tail calls (e.g., `let field = parse_field()`
   mid-body) still push a continuation, but the depth is bounded by
   the grammar's nesting depth, not the input size.

2. **ANF compatibility.** The v2 parser .dag files will be lowered
   through the same ANF pass. Verify that parser patterns like
   `parse_type(tokens |> skip(1))` lower correctly — the `skip` call
   must be hoisted. If the parser uses patterns that resist ANF
   lowering, those patterns should be refactored in the .dag files.

## Verification

### Standard gates (every migration step)

- `cargo test -p daglang-eval` — unit tests
- `cargo test -p v2-compiler-tests` — integration tests
- `cargo clippy --all-targets -- -D warnings` — zero warnings

### Structural invariant (static, checkable by inspection)

**`eval_body` never calls `eval_body`**, directly or transitively
through `eval_expr`. Verify with:
```
grep -n 'eval_body(' src/05_graph/daglang-eval/src/eval.rs
```
The only call site should be in the main loop.

### Bound verification tests

```rust
/// Native stack depth stays bounded regardless of fn-call depth.
/// Instrumented with a thread-local depth counter.
#[test]
fn native_stack_depth_bounded() {
    // Mutual recursion (is_even/is_odd) at depth 10,000.
    // Run on the DEFAULT thread — no with_parser_stack.
    // Assert: max native eval_expr depth < 50.
    // Assert: heap stack depth reached 0 (all tail calls).
}

/// Heap stack is bounded by MAX_CALL_DEPTH.
#[test]
fn heap_stack_bounded_by_limit() {
    // Mutual recursion at depth MAX_CALL_DEPTH + 1.
    // Assert: clean error message, not OOM.
}

/// Intrinsic lambda bodies do not bypass the heap stack.
#[test]
fn intrinsic_lambda_does_not_bypass_heap_stack() {
    // map(list_of_1000, x => recursive_fn(x))
    // where recursive_fn triggers deep mutual recursion.
    // Run on default thread — no with_parser_stack.
    // If intrinsics bypass the heap stack, this overflows.
}
```

### Final acceptance gate

`phase6_gist_full_pipeline` passes without `with_parser_stack(32MB)`,
on the default 8MB thread stack.
