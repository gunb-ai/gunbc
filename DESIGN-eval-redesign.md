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
- Variable lookup: O(1) amortized (HashMap)
- Let-binding: O(1) amortized (HashMap insert)
- Env creation for a child scope: O(1) (not O(n) clone)
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
/// Passed as `&EvalContext` — no cloning, no Rc.
struct EvalContext {
    sibling_fns: HashMap<String, LoweredFnBody>,
    data_values: HashMap<String, Value>,
}

/// Variable bindings. Changes every let-binding.
///
/// Immutable value type — `bind()` returns a new Env (DI-1).
/// Uses persistent/CoW map for O(1) child-scope creation (DI-2).
struct Env {
    bindings: HashMap<String, Value>,
}

/// Per-fn-call metadata. Changes at fn-call boundaries only.
struct FrameInfo {
    call_depth: usize,
    fn_name: Option<String>,
}
```

**Efficiency note (DI-2):** `Env::bind()` must not clone the entire
HashMap. Options:
- `im::HashMap` (persistent data structure, O(log n) insert, O(1) clone)
- `Rc<HashMap>` with `Rc::make_mut` (current approach, O(1) amortized)
- Scope-chain (Vec of small maps, O(depth) lookup)

The `Rc::make_mut` approach is acceptable IF we make the sharing explicit
— i.e., `Env::child()` documents that it shares the backing map, and
`bind()` documents that it clones-on-write. The current code does this
but the type doesn't communicate it.

### Return types, one per outcome (DI-3)

```rust
/// Result of evaluating a single expression.
enum ExprResult {
    /// Normal: produced a value.
    Value(Value),
    /// The expression is a call to a sibling fn that should be
    /// evaluated by the outer loop, not by native recursion.
    /// Carries the callee name, evaluated inputs, and a continuation
    /// describing how to use the result.
    Call {
        callee: String,
        inputs: HashMap<String, Value>,
        continuation: Continuation,
    },
    /// Early return from the enclosing fn body.
    Return(HashMap<String, Value>),
    /// Evaluation error.
    Error(String),
}

/// Result of evaluating a fn body to completion.
enum BodyResult {
    /// Normal completion with output fields.
    Done(HashMap<String, Value>),
    /// Need to call a sibling fn before continuing.
    Call {
        callee: String,
        inputs: HashMap<String, Value>,
        continuation: Continuation,
    },
    /// Evaluation error.
    Error(String),
}
```

No `Result<_, EvalError>` — the return type IS the outcome.

### The continuation (suspended computation)

When a fn body hits a non-tail sibling call (`let x = f(...)`), it
cannot proceed without f's result. The continuation captures "what to
do next" as pure data:

```rust
struct Continuation {
    /// Name of the callee (for extracting the return value).
    callee_name: String,
    /// How to bind the call result.
    binding: Option<String>,          // Some("x") for let, None for expr
    /// Remaining statements after the call.
    remaining_stmts: Vec<LoweredStmt>,
    /// The env at the point of suspension.
    env: Env,
    /// Frame metadata.
    frame: FrameInfo,
    /// Whether this is the fn body's top-level stmts.
    is_fn_body: bool,
}
```

**Key property:** Continuation is owned data. No references, no
lifetimes. It clones the remaining stmts (which are cheaply cloneable
AST nodes). This avoids all borrow-checker issues from the first attempt.

**Efficiency (DI-2):** Cloning `Vec<LoweredStmt>` for the remaining
stmts is O(remaining) per suspension. For the common case (call in the
last statement), remaining is empty or 1 element. For mid-body calls,
it's proportional to the number of remaining statements — acceptable
since fn bodies are typically short (5-20 stmts).

### The main loop

```rust
pub fn evaluate_fn_body(
    body: &LoweredFnBody,
    inputs: HashMap<String, Value>,
    ctx: &EvalContext,
) -> Result<HashMap<String, Value>, String> {
    let mut stack: Vec<Continuation> = Vec::new();
    let mut current = RunState::new(body, inputs, ctx);

    loop {
        match eval_body_pure(&current.body, &current.env, &current.frame, ctx) {
            BodyResult::Done(result) => {
                match stack.pop() {
                    None => return Ok(result),
                    Some(cont) => current = resume(cont, &result, ctx),
                }
            }
            BodyResult::Call { callee, inputs, continuation } => {
                stack.push(continuation);
                current = RunState::for_call(&callee, inputs, &current.frame, ctx);
            }
            BodyResult::Error(msg) => return Err(msg),
        }
    }
}
```

The entire fn-call stack is `Vec<Continuation>` on the heap. No native
recursion for fn calls. Expression evaluation within a single fn body
stays recursive on the native stack (bounded by AST depth, not input
size).

## Open Questions

1. **Nested calls in expressions.** `let x = g(f(...))` — when we hit
   `f(...)`, we're mid-way through evaluating `g`'s arguments. The
   continuation needs to capture "I was evaluating g's args, I have
   some done, here's what's left." This enriches the continuation type.
   Alternative: keep expression-level evaluation recursive (native
   stack) since expression depth is bounded. Only fn-call→fn-call
   transitions go through the heap stack.

2. **Tail-call optimization.** In the current design, tail calls
   (self-recursive or mutual) can be detected when `remaining_stmts`
   is empty and `binding` is None. The main loop can then skip pushing
   a continuation — just replace `current` with the callee. This
   subsumes the existing trampoline mechanism.

3. **Env efficiency.** Whether to use `Rc<HashMap>` with make_mut,
   `im::HashMap`, or scope chains. The choice affects DI-2 for
   child-scope creation vs. lookup cost.

## Migration Path

Incremental, test-by-test:

1. Introduce `EvalContext` — extract constant state from `Env`.
2. Introduce `FrameInfo` — extract per-call state from `Env`.
3. Replace `EvalError` control flow with `ExprResult` / `BodyResult`.
4. Introduce `Continuation` and the main loop.
5. Delete old recursive `eval_fn_body_rc` path.
6. Remove `with_parser_stack(32MB)` from v2 tests.
7. Un-ignore `phase6_gist_full_pipeline`.

Each step compiles and passes all tests before the next begins.
