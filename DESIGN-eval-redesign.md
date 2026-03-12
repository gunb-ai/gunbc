# Evaluator Redesign: Pure Explicit-Stack Evaluation

## Problem

The DSL evaluator (`daglang-eval/src/eval.rs`) interprets `LoweredFnBody`
trees by recursing on the native Rust call stack. The v2 self-hosted
parser has ~80 mutually recursive functions; interpreting 12 modules with
150+ type definitions overflows even 128MB stacks (S52).

The current code has accumulated specialized trampoline logic for self
tail calls, mutual tail calls, and a `Return` fallback to preserve
wrapping semantics — signs that the retrofitted-TCO path is contorted.

## Governing Invariants

From `src/README.md`:

- **Pure core logic.** Deterministic functions from inputs to outputs.
- **Clear interfaces.** Return values, not mutated shared state.
- **No parallel implementations.** One evaluator, not two.
- **Explicit boundary contracts.** Preconditions enforced, not assumed.

## Core Principle: IR Structure Mirrors Evaluator Structure

The evaluator has two levels with fundamentally different properties:

| Level | What it does | Can it suspend? | Depth bound |
|-------|-------------|-----------------|-------------|
| **Expressions** | Arithmetic, field access, string interp, match | Never | AST depth (syntactic) |
| **Statements** | Sequencing, binding, fn calls, control flow | On fn calls | Input size (unbounded) |

**The fix:** enforce this separation in the IR as a lowering contract.
Fn calls only appear at statement level, never nested inside expressions.
Then `eval_expr` is a total pure function, and all suspension logic lives
in `eval_body` + the main loop.

Edge cases (short-circuit operators with calls, calls in branches, calls
in lambda bodies) are eliminated by the lowerer — not handled case-by-
case in the evaluator.

## The Lowering Contract

After lowering, `LoweredExpr::Call` appears only in statement position:
`LoweredStmt::Let(name, Call{...})` or `LoweredStmt::Expr(Call{...})`.
Never nested inside another `LoweredExpr`.

This is ANF restricted to calls. The lowerer hoists nested calls into
synthetic let-bindings:

```
Source:      let x = g(f(a))
Lowered:     let __t0 = f(a)
             let x = g(__t0)

Source:      is_valid() && check_db()
Lowered:     let __t0 = is_valid()
             if __t0 { check_db() } else { false }

Source:      map(list, x => f(x))
Lowered:     for x in list { let __t0 = f(x); __t0 }
```

**This is the foundation, not a pre-step.** The lowerer change is
moderate surgery (the current `lower_expr` preserves nesting — nested
calls in args, record fields, pipe chains, and lambda bodies all exist
in real .dag files). But it's a one-time change that permanently
simplifies the evaluator.

**Verification:** a structural walk after lowering asserts no
`LoweredExpr` contains a `Call` descendant. This is the boundary
contract between lowerer and evaluator.

## Data Structures

### Split by change frequency

```rust
/// Constant for the entire evaluation. The immutable code store.
/// Fn bodies are keyed by name; continuations reference them by FnId.
struct EvalContext {
    fns: Vec<LoweredFnBody>,               // indexed by FnId
    fn_index: HashMap<String, FnId>,        // name → FnId
    data_values: HashMap<String, Value>,
}

type FnId = usize;

/// Variable bindings. Changes every let-binding.
/// Current: Rc<HashMap> with make_mut (keep for now).
/// Future: im::HashMap or numeric local slots + Vec<Value>.
struct Env { ... }

/// Per-fn-call metadata.
struct FrameInfo {
    fn_id: FnId,
    call_depth: usize,
}
```

FnId + pc (program counter = statement index) replaces `&'a` references.
No lifetimes spread through the continuation type. `EvalContext` is the
immutable code store; runtime state is all indices and values.

**Env representation is decoupled from the control machine.** The
explicit-stack refactor works with the current `Rc<HashMap>` env. Whether
to switch to `im::HashMap` or numeric local slots is a separate decision
made after the machine works, informed by benchmarks.

### Return types

```rust
/// eval_expr: total function over call-free expression trees.
/// Never suspends. Always returns.
enum ExprResult {
    Value(Value),
    Return(HashMap<String, Value>),
    Error(String),
}

/// eval_body: runs statements until completion or suspension.
enum Action {
    Done(HashMap<String, Value>),
    Suspend {
        callee: FnId,
        inputs: HashMap<String, Value>,
        cont: Continuation,
    },
    TailCall {
        callee: FnId,
        inputs: HashMap<String, Value>,
    },
    Error(String),
}
```

`Suspend` vs `TailCall` is explicit in the type — not inferred from
"missing binding." The distinction is first-class: `TailCall` means
identity continuation (literally no residual work), `Suspend` means
there's a continuation to push.

### Continuation

```rust
struct Continuation {
    fn_id: FnId,
    pc: usize,                  // index of next statement to execute
    binding: Option<String>,    // what to bind the call result to
    projection: Projection,     // how to extract the return value
    env: Env,
}

enum Projection {
    /// Use the `return` field if present, else the `value` field.
    PrimaryReturn,
    /// Use the entire output map as-is.
    WholeMap,
}
```

**Projection makes return semantics explicit.** The current evaluator's
`sibling_fn_value` function is a runtime heuristic ("use `return` if it
exists, otherwise `value`, otherwise the whole map"). That policy should
be decided once at lowering time and stored in the continuation. The
evaluator just executes the plan.

**Tail call = identity continuation.** A call is a tail call only when
there is literally no residual work: no binding, no remaining statements,
no projection. `return { return: f() }` is NOT a tail call because it
has wrapping/projection work. This is exactly the bug the current mutual
TCO hit — blindly trampolining lost the field-name wrapping.

### Depth vs time limits

```rust
const MAX_STACK_DEPTH: usize = 10_000;   // suspended continuations
const MAX_STEP_BUDGET: usize = 1_000_000; // total eval_body invocations
```

Once tail calls stop pushing continuations, "stack depth" and "how long
we've been running" diverge. An infinite tail-call loop uses O(1) stack
but unbounded time. Both need limits.

## Main Loop

```rust
pub fn evaluate(
    fn_id: FnId,
    inputs: HashMap<String, Value>,
    ctx: &EvalContext,
) -> Result<HashMap<String, Value>, String> {
    let mut stack: Vec<Continuation> = Vec::new();
    let mut fn_id = fn_id;
    let mut pc: usize = 0;
    let mut env = Env::from_inputs(inputs, ctx);
    let mut steps: usize = 0;

    loop {
        steps += 1;
        if steps > MAX_STEP_BUDGET {
            return Err("step budget exceeded".into());
        }

        let body = &ctx.fns[fn_id];
        match eval_body(&body.stmts[pc..], &env, ctx) {
            Action::Done(result) => match stack.pop() {
                None => return Ok(result),
                Some(cont) => {
                    let value = cont.projection.extract(&result);
                    env = match &cont.binding {
                        Some(name) => cont.env.bind(name, value),
                        None => cont.env,
                    };
                    fn_id = cont.fn_id;
                    pc = cont.pc;
                }
            },
            Action::Suspend { callee, inputs, cont } => {
                if stack.len() >= MAX_STACK_DEPTH {
                    return Err("max call depth exceeded".into());
                }
                stack.push(cont);
                fn_id = callee;
                pc = 0;
                env = Env::from_inputs(inputs, ctx);
            }
            Action::TailCall { callee, inputs } => {
                // No continuation pushed. O(1) state replacement.
                fn_id = callee;
                pc = 0;
                env = Env::from_inputs(inputs, ctx);
            }
            Action::Error(msg) => return Err(msg),
        }
    }
}
```

Always the same shape: evaluate a body, handle the result. No separate
`resume` function. `Suspend` pushes, `TailCall` replaces, `Done` pops.

## Recursion Bounds

```
main_loop                      — iterative, heap-bounded by MAX_STACK_DEPTH
  └→ eval_body(stmts)         — iterative over stmts
       └→ eval_expr(expr)     — native-recursive, bounded by AST depth
```

**Structural invariant:** `eval_body` never calls `eval_body`. Only the
main loop does. `eval_expr` never sees a `Call` (IR contract). Native
stack = O(AST expr depth). Heap = O(call depth). Both bounded.

| Resource | Bound | At depth 10K |
|----------|-------|-------------|
| Native stack | O(expr_depth) ≈ 30 frames | ~6KB |
| Heap stack | ≤ MAX_STACK_DEPTH | ~800KB |
| Env snapshots | O(1) each (structural sharing) | shared |
| AST cloning | zero — FnId + pc into EvalContext | zero |
| Time | ≤ MAX_STEP_BUDGET | clean error |

## Migration Path

Each step compiles and passes all tests.

0. **Freeze the lowering contract.** Hoist calls to statement level in
   `daglang-lower`. Add structural verifier. This is the foundation.
1. **Introduce `EvalContext` and `FrameInfo`.** Extract from `Env`.
2. **Make `Action` explicit.** `Suspend`/`TailCall`/`Done`/`Error`
   with `Continuation` and `Projection`. Introduce alongside old types,
   then remove old types.
3. **Introduce the main loop.** FnId + pc based. TCO built in.
4. **Delete old recursive path** (`eval_fn_body_rc`, trampoline,
   `tail_call`/`early_return` error variants).
5. **Remove `with_parser_stack(32MB)`.** Un-ignore `phase6`.
6. **Benchmark env representation.** Decide `im::HashMap` vs current
   vs local slots based on measured performance.

## Verification

**Structural (static):** `eval_body` has one call site (main loop).
`eval_expr` has no path to `Call` (IR contract verifier).

**Bound tests:**
```rust
#[test] fn mutual_recursion_10k_default_stack()
#[test] fn heap_stack_bounded_by_max_depth()
#[test] fn step_budget_catches_infinite_tail_loop()
#[test] fn anf_no_nested_calls_after_lowering()
```

**Acceptance:** `phase6_gist_full_pipeline` on default 8MB stack.
