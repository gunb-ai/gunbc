# Evaluator Redesign: Pure Explicit-Stack Evaluation

## Problem

The DSL evaluator (`daglang-eval/src/eval.rs`) interprets `LoweredFnBody`
trees by recursing on the native Rust call stack. The v2 self-hosted
parser has ~80 mutually recursive functions; interpreting 12 modules with
150+ type definitions overflows even 128MB stacks (S52).

## Governing Invariants

From `src/README.md`:

**Pure core logic.** Deterministic functions from inputs to outputs.

**Clear interfaces.** Prefer returning values over mutating shared state.

**No parallel implementations.** One evaluator, not two.

**No duplicate representations.** Each fact lives in one place.

**Explicit boundary contracts.** Each pipeline stage's preconditions are
enforced, not assumed.

## Core Principle: IR Structure Mirrors Evaluator Structure

The evaluator has two levels of computation with fundamentally different
characteristics:

| Level | What it does | Can it suspend? | Recursion depth |
|-------|-------------|-----------------|-----------------|
| **Expressions** | Arithmetic, field access, string interp, pattern match | No — always completes | Bounded by AST depth (syntactic) |
| **Statements** | Sequencing, binding, fn calls, control flow | Yes — fn calls suspend | Bounded by input size (unbounded) |

The current evaluator conflates these levels — `eval_expr` can encounter
fn calls, which recurse into `eval_fn_body`, which runs statements, which
evaluate expressions. All on the native stack.

**The fix:** enforce the separation in the IR. The lowered IR must
guarantee that fn calls only appear at statement level, never nested
inside expressions. Then the evaluator's two levels are independent:
`eval_expr` is a total pure function (never suspends), and `eval_body`
handles all suspension via an explicit heap stack.

This is not a workaround — it's the explicit boundary contract between
the lowerer and the evaluator. The lowerer guarantees call-free
expressions; the evaluator relies on that guarantee for bounded native
stack usage. Edge cases (short-circuit operators with calls, calls in
if/else branches, calls in lambda bodies) are eliminated by the lowerer,
not handled by the evaluator.

## The IR Contract: Call-Free Expressions

After lowering, `LoweredExpr::Call` appears only in two positions:
- RHS of `LoweredStmt::Let(name, Call{...})`
- Bare `LoweredStmt::Expr(Call{...})`

Never nested inside another `LoweredExpr`. This is A-Normal Form (ANF)
restricted to calls.

The lowerer enforces this by hoisting nested calls into synthetic
let-bindings:

```
Source:      let x = g(f(a))
Lowered:     let __t0 = f(a)
             let x = g(__t0)

Source:      x |> f() |> g()
Lowered:     let __t0 = f(x)
             let __t1 = g(__t0)
             __t1

Source:      is_valid() && check_db()
Lowered:     let __t0 = is_valid()
             if __t0 { check_db() } else { false }

Source:      map(list, x => f(x))
Lowered:     for x in list { let __t0 = f(x); __t0 }
```

Short-circuit operators, conditional branches, and lambda bodies with
calls all reduce to the same pattern: calls at statement level inside
blocks. No special cases in the evaluator.

**Verification:** a single structural walk after lowering asserts that
no `LoweredExpr` contains a `Call` descendant. This is the boundary
contract.

### Lowerer feasibility

The current lowerer (`daglang-lower/src/expr.rs`) preserves nesting —
`lower_expr` recursively descends the AST without hoisting. Nested calls
in arguments, record fields, pipe chains, and lambda bodies all exist in
real .dag files.

The ANF transform is a well-understood compiler pass. The mechanical
cases (nested args, pipe chains, record fields) cover ~80% of
occurrences. Control-flow cases (short-circuit, branches) require
lowering to statement blocks, which is the natural representation anyway.

This is moderate surgery on the lowerer, not trivial. But it's a
one-time change that simplifies the evaluator permanently.

## Evaluator Design

### Data structures (DI-4: split by change frequency)

```rust
/// Constant for the entire evaluation.
struct EvalContext {
    sibling_fns: HashMap<String, LoweredFnBody>,
    data_values: HashMap<String, Value>,
}

/// Variable bindings. Changes every let-binding.
/// Immutable — bind() returns a new Env via im::HashMap.
struct Env {
    bindings: im::HashMap<String, Value>,
}

/// Per-fn-call metadata. Changes at fn-call boundaries.
struct FrameInfo {
    call_depth: usize,
    fn_name: Option<String>,
}
```

`im::HashMap`: O(1) clone (structural sharing), O(log n) insert/lookup.
Truly persistent — no hidden aliasing, no conditional cloning. Purity
is structural, not simulated.

### Return types (DI-3: no hidden output channels)

```rust
/// eval_expr: total function over call-free expression trees.
enum ExprResult {
    Value(Value),
    Return(HashMap<String, Value>),  // early return from enclosing fn
    Error(String),
}

/// eval_body: may suspend on fn calls.
enum Action {
    Done(HashMap<String, Value>),
    Call { callee: String, inputs: HashMap<String, Value> },
    Error(String),
}
```

### Continuation (zero-allocation suspension)

```rust
struct Continuation<'a> {
    binding: Option<String>,             // None = unbound call (not a tail call)
    remaining_stmts: &'a [LoweredStmt], // slice into EvalContext's fn body
    env: Env,
    frame: FrameInfo,
}
```

`remaining_stmts` is a slice — no index tracking, no bounds checking,
O(1) to create. `&'a` borrows from `EvalContext` which outlives
everything. `Env` snapshot is O(1) via `im::HashMap`.

**Tail call:** `remaining_stmts.is_empty() && binding.is_none()`. Don't
push — just replace the current state. Self-recursive and mutual tail
calls are identical.

**Unbound non-tail call** (e.g., `log("msg"); let x = f()`): binding
is `None`, remaining is non-empty. Continuation is pushed; result is
discarded on resume.

### Main loop

```rust
pub fn evaluate_fn_body<'a>(
    body: &'a LoweredFnBody,
    inputs: HashMap<String, Value>,
    ctx: &'a EvalContext,
) -> Result<HashMap<String, Value>, String> {
    let mut stack: Vec<Continuation<'a>> = Vec::new();
    let mut stmts: &'a [LoweredStmt] = &body.stmts;
    let mut env = Env::from_inputs(inputs, ctx);
    let mut frame = FrameInfo { call_depth: 0, fn_name: None };

    loop {
        match eval_body(stmts, &env, &frame, ctx) {
            Action::Done(result) => match stack.pop() {
                None => return Ok(result),
                Some(cont) => {
                    if let Some(name) = &cont.binding {
                        env = cont.env.bind(name, extract_return(&result));
                    } else {
                        env = cont.env; // discard unbound result
                    }
                    stmts = cont.remaining_stmts;
                    frame = cont.frame;
                }
            },
            Action::Call { callee, inputs } => {
                if frame.call_depth >= MAX_CALL_DEPTH {
                    return Err(format!("max call depth ({MAX_CALL_DEPTH}) exceeded"));
                }
                let callee_body = ctx.sibling_fns.get(&callee)
                    .ok_or_else(|| format!("unknown function: {callee}"))?;
                stmts = &callee_body.stmts;
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

## Recursion Bounds

```
main_loop                          — iterative, heap-bounded by MAX_CALL_DEPTH
  └→ eval_body(stmts)             — iterative over stmts, no recursion
       └→ eval_expr(expr)         — native-recursive, bounded by AST depth
```

**Structural invariant:** `eval_body` never calls `eval_body`. Only the
main loop does. `eval_expr` never encounters a `Call` (IR contract).
Therefore: native stack = O(AST expr depth). Heap = O(call depth).
Both bounded.

| Resource | Bound | At depth 10K |
|----------|-------|-------------|
| Native stack | O(expr_depth) ≈ 30 frames | ~6KB |
| Heap stack | O(call_depth) ≤ MAX_CALL_DEPTH | ~800KB |
| Env snapshots | O(1) each via structural sharing | shared |
| AST allocation | zero — slices into EvalContext | zero |

## Relationship to v2 Parser

The v2 parser's ~80 mutually recursive functions are the primary
consumer. Parser functions where the recursive call is in tail position
(common in recursive descent) get TCO for free — no continuation pushed.
Non-tail calls (`let field = parse_field()`) push a continuation bounded
by grammar nesting depth, not input size.

The parser .dag files go through the same ANF lowering. Pipe chains like
`tokens |> skip(1) |> parse_type()` hoist naturally. If any parser
pattern resists ANF, that pattern should be refactored in the .dag source.

## Migration Path

Each step compiles and passes all tests before the next.

0. **ANF lowering** — hoist calls to statement level in `daglang-lower`.
   Add structural assertion: no `LoweredExpr::Call` nested inside
   another `LoweredExpr`. This is the prerequisite.
1. **Introduce `EvalContext`** — extract constant state from `Env`.
2. **Introduce `FrameInfo`** — extract per-call state from `Env`.
3. **Replace `EvalError` control flow with `ExprResult`/`Action`** —
   introduce alongside old types, then remove old types.
4. **Introduce `Continuation` and main loop** — TCO built in from start.
5. **Switch Env to `im::HashMap`** — `bind()` returns new Env.
6. **Delete old recursive path.**
7. **Remove `with_parser_stack(32MB)`.** Un-ignore `phase6`.

## Verification

**Structural (static):** `eval_body` has exactly one call site (main
loop). `eval_expr` has no path to `Call` (IR contract assertion).

**Bound tests:**
```rust
#[test] fn mutual_recursion_10k_default_stack()   // no with_parser_stack
#[test] fn heap_stack_bounded_by_max_call_depth()  // clean error, not OOM
#[test] fn anf_no_nested_calls_after_lowering()    // structural walk
```

**Acceptance gate:** `phase6_gist_full_pipeline` passes on default 8MB
stack.
