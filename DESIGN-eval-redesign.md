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
contract between lowerer and evaluator. The runtime verifier in
`verify_anf_contract()` also checks this at the evaluator boundary
(fail-closed during migration).

## Pipeline Architecture

The evaluator is factored as three discrete stages:

```text
[1. Build EvalContext]  →  [2. Verify ANF contract]  →  [3. Run machine]

  sibling_fns             assert no nested Call         iterative main loop
  data_values             in any expression tree        heap continuation stack
  → fn_index                                           → Result<outputs, error>
```

Each stage has a clear input type and output type. The stages are
independently testable.

## Data Structures

### Immutable code store

```rust
struct EvalContext<'a> {
    fns: Vec<&'a LoweredFnBody>,       // indexed by FnId
    fn_index: HashMap<&'a str, FnId>,   // name → FnId
    data_values: &'a HashMap<String, Value>,
    sibling_fns: &'a HashMap<String, LoweredFnBody>,
}
type FnId = usize;
```

FnId + pc (program counter = absolute statement index) replaces `&'a`
references. No lifetimes spread through the continuation type.
`EvalContext` is the immutable code store; runtime state is all indices
and values.

**No FrameInfo.** The machine state is `fn_id`, `pc`, `env`, `stack`,
and `transitions`. `call_depth` is `stack.len()`. No duplicate
representations.

### Return types

```rust
/// eval_expr: total function over call-free expression trees.
/// Never suspends. Always returns.
fn eval_expr(expr, env, ctx) -> Result<Value, EvalError>

/// eval_body: runs statements until completion or suspension.
fn eval_body(fn_id, start_pc, env, ctx) -> Step

enum Step {
    Return(HashMap<String, Value>),
    Call {
        callee: FnId,
        inputs: HashMap<String, Value>,
        cont: Option<Continuation>,
    },
    Error(String),
}
```

`cont: None` means a true tail call (identity continuation — literally
no residual work). `cont: Some(...)` means there is a continuation to
push. The distinction is first-class in the type, not inferred.

### Continuation

```rust
struct Continuation {
    fn_id: FnId,
    pc: usize,                  // absolute index into ctx.fns[fn_id].stmts
    binding: Option<String>,    // what to bind the call result to
    projection: Projection,     // how to extract the return value
    env: Env,
}

enum Projection {
    /// Extract the "return" field. Falls back to single "value" field
    /// for compatibility with `return expr`. No other heuristic.
    ReturnField,
    /// Use the entire output map.
    WholeMap,
}
```

**Absolute pc.** `eval_body(fn_id, start_pc, env, ctx)` owns the full
statement loop. The continuation stores an absolute index, not a
relative offset. This makes resume points unambiguous.

**Projection is explicit.** The lowerer decides how to extract the
return value. The evaluator executes the plan. `ReturnField` matches
the current `sibling_fn_value` behavior. The `"value"` fallback is a
documented compatibility shim that will be removed once the return
convention is standardized.

**Centralized binding.** `bind_let_result(env, name, value)` is the
single helper for map-flattening into `name__field` entries. Used in
both normal statement processing and continuation resume — no
divergence.

### Limits

```rust
const MAX_STACK_DEPTH: usize = 100_000;   // heap continuations
const MAX_TRANSITIONS: usize = 10_000_000; // main-loop iterations
```

`MAX_STACK_DEPTH` bounds memory. `MAX_TRANSITIONS` bounds time. They
are independent: tail calls use O(1) stack but unbounded transitions.
Both limits produce clean errors.

## Main Loop

```rust
fn run_machine(entry_fn_id, inputs, ctx) -> Result<..., EvalError> {
    let mut stack: Vec<Continuation> = Vec::new();
    let mut fn_id = entry_fn_id;
    let mut pc = 0;
    let mut env = Env::from_inputs(inputs);
    let mut transitions = 0;

    loop {
        transitions += 1;
        if transitions > MAX_TRANSITIONS { return Err(...); }

        match eval_body(fn_id, pc, &mut env, ctx) {
            Step::Return(result) => {
                // Unwind: pop continuations until one resumes.
                loop {
                    match stack.pop() {
                        None => return Ok(result),
                        Some(cont) => {
                            let value = cont.projection.extract(&result);
                            if past_end && no_binding {
                                result = wrap_as_result(value);
                            } else {
                                bind_let_result(&mut env, ...);
                                fn_id = cont.fn_id;
                                pc = cont.pc;
                                break;
                            }
                        }
                    }
                }
            }
            Step::Call { callee, inputs, cont } => {
                if let Some(cont) = cont { stack.push(cont); }
                fn_id = callee;
                pc = 0;
                env = Env::from_inputs(inputs);
            }
            Step::Error(msg) => return Err(msg),
        }
    }
}
```

Always the same shape: evaluate a body, handle the result. `Call` with
`cont: Some(...)` pushes; `Call` with `cont: None` replaces; `Return`
pops and unwinds.

## Recursion Bounds

```
run_machine                    — iterative, heap-bounded by MAX_STACK_DEPTH
  └→ eval_body(fn_id, pc)     — iterative over stmts[pc..]
       └→ eval_expr(expr)     — native-recursive, bounded by AST depth
```

**Structural invariant:** `eval_body` never calls `eval_body`. Only the
main loop does. `eval_expr` never sees a `Call` (ANF contract). Native
stack = O(AST expr depth). Heap = O(call depth). Both bounded.

| Resource | Bound | At depth 100K |
|----------|-------|--------------|
| Native stack | O(expr_depth) ≈ 30 frames | ~6KB |
| Heap stack | ≤ MAX_STACK_DEPTH | ~8MB |
| Env snapshots | O(1) each (structural sharing) | shared |
| AST cloning | zero — FnId + pc into EvalContext | zero |
| Time | ≤ MAX_TRANSITIONS | clean error |

## Known Limitations

1. **`LoweredExpr::Return` is still an expression.** This means
   `eval_expr` is not fully pure — it can produce early-return signals.
   The clean fix: make `return` a statement form only, not an
   expression. Deferred — 23 match sites across 4 files (eval_stack,
   eval_core, anf, expr), high risk of breaking the ANF normalizer's
   call-hoisting logic and the suspendable evaluator's continuation
   handling for blocks.

2. **`LoweredExpr::Block` and `LoweredExpr::For` carry statement
   semantics.** The cleanest end state is `ForCollect` as a statement
   form with its own body-stmts, and blocks as statement sequences (not
   expressions). Deferred for the same reason as (1).

3. **`Projection::ReturnField` "value" fallback.** The parser now
   standardizes on `"return"` key (Phase 5a done). However, the
   `"value"` fallback cannot be fully removed because
   `wrap_value_as_output` flattens Map trailing expressions into the
   output HashMap. When a function's trailing expression is a variant
   like `Some { value: x }`, the Map `{"_variant": "Some", "value": x}`
   gets flattened, and the caller's `extract_projection` must
   reconstruct it. If the variant has a single payload field `"value"`,
   the flattened output has `{"value": x}` which needs the fallback to
   extract correctly. Removing this requires either:
   (a) making `wrap_value_as_output` always wrap with `"return"` (breaks
   the v2 DSL evaluation model which expects Map flattening), or
   (b) teaching `extract_projection` to distinguish "return-convention
   value" from "legitimate Map field named value" (no reliable signal).

## Migration Path

Each step compiles and passes all tests.

0. ✅ **Freeze the lowering contract.** Hoist calls to statement level
   in `daglang-lower`. Add structural verifier. (`anf.rs`)
1. ✅ **Introduce `EvalContext`.** Extract from `Env`. (`eval_stack.rs`)
2. ✅ **Make `Step` explicit.** `Call`/`Return`/`Error` with
   `Continuation` and `Projection`. (`eval_stack.rs`)
3. ✅ **Introduce the main loop.** FnId + absolute pc based.
   (`run_machine` in `eval_stack.rs`)
4. ✅ **Internalize non-sibling call handling.** Builtins moved to
   `eval_core::eval_builtin_call`. Intrinsics, `scan_while`, match
   evaluation, sibling-fn dispatch all self-contained in eval_stack.rs.
   Zero imports from eval.rs.
5. ✅ **Delete old recursive path.** eval.rs reduced from ~2300 lines
   to ~140 lines of thin wrappers. `eval_fn_body_rc`, trampoline,
   `tail_call` field, old `Env`, all intrinsics/builtins deleted.
6. ✅ **Remove `with_parser_stack(32MB)`.** 16 call sites unwrapped.
   `phase6_gist_full_pipeline` un-ignored (re-ignored as OOM, not
   stack overflow — 12 .dag files exceed 16GB heap in debug mode).
7. ✅ **Standardize return convention (partial).** Parser changed to
   `"return"` key. `"value"` fallback preserved (see Limitation 3).
8. 🔲 **IR cleanup.** Move `Return`, `Block`, `For` to statement forms.
   Deferred — see Limitation 1.
9. ✅ **Benchmark env representation (analysis).** `Rc<HashMap>` COW
   is correct for the workload. `im::HashMap` adds dependency for
   marginal benefit (3-10 bindings per scope). Local slots blocked
   by dynamic `name__field` binding names from Map destructuring.
   Recommendation: keep `Rc<HashMap>`.

## Verification

**Structural (static):** `eval_body` has one call site (main loop).
`eval_expr` has no path to `Call` (ANF contract verifier in both
`daglang-lower/anf.rs` and `eval_stack.rs`).

**Bound tests:**
```rust
#[test] fn deep_mutual_recursion_40k()       // heap stack, not native
#[test] fn anf_verifier_catches_nested_call() // contract enforcement
#[test] fn value_normalization()              // return key normalization
#[test] fn sibling_then_builtin()            // sibling → builtin chain
#[test] fn builtin_char_at()                 // eval_builtin_call path
#[test] fn builtin_scan_while_with_lambda()  // lambda in builtin
#[test] fn intrinsic_map_with_lambda()       // eval_intrinsic_call_s path
#[test] fn intrinsic_fold()                  // named-arg intrinsic
#[test] fn builtin_lookup()                  // Option-returning builtin
```

**Acceptance:** 54 v2 tests pass on default stack (no `with_parser_stack`).
`phase6_gist_full_pipeline` runs on default stack but OOMs in debug mode
(12 .dag files, >16GB heap — interpreter overhead, not a stack issue).
