# R3 PR-E E5 Loop Readiness Audit

**Status:** E5 readiness / implementation blocker. This is a docs-only receipt;
it does not implement `eval_loop`, change substrate carriers, widen strategy
carriers, or add runner behavior.

**Parent authorities:** [`r3-evaluator-dispatch.md`](r3-evaluator-dispatch.md)
E5, [`r2-pr-b-1-eager-evaluator-implementation-seed.md`](r2-pr-b-1-eager-evaluator-implementation-seed.md)
B.1.4, and [`r3-evaluator-e0-body-evaluator-api-scaffold.md`](r3-evaluator-e0-body-evaluator-api-scaffold.md).

## Live Loop Shape

The current Rust `LoopNode` carrier is:

```text
LoopNode {
    id: NodeId,
    source: PortId,
    init: PortId,
    body: NodeId,
    bound: LoopBound,
    output: PortId,
    span: SourceSpan,
}
```

`LoopBound` has two substrate inhabitants:

```text
LoopBound::Cardinality { count: PortId }
LoopBound::Descent { cluster: ClusterId }
```

`Dag::push_loop` already enforces that `source`, `init`, `body`, and the
`Cardinality.count` port exist, and the loop output inherits the resolved init
shape when available. E5 should consume this shape directly; it must not add a
parallel loop carrier or reinterpret `LoopBound` with string labels.

## Intended Eager Cardinality Rule

When the body evaluator can execute loop bodies, the smallest E5 implementation
should add:

```text
fn eval_loop(
    dag: &Dag,
    loop_node: &LoopNode,
    stack: &mut EvalStateStack<Value>,
    strategy: &EvalStrategy,
) -> Result<Value, EvalError>
```

For `LoopBound::Cardinality { count }`, the allowed eager rule is:

1. Evaluate `loop_node.init` through `eval_port`.
2. Evaluate `count` through `eval_port`.
3. Decode the count witness fail-closed. The first executable slice should
   accept only a runtime integer literal count; missing or non-integer counts
   must produce typed diagnostics rather than defaulting to zero.
4. For each iteration, push a fresh top frame, bind the accumulator value for
   that iteration to the loop accumulator port, evaluate `loop_node.body`
   through `eval_node`, pop the frame on success or failure, and thread the
   body result into the next iteration.
5. Return the final accumulator. Zero iterations returns the evaluated init
   value without evaluating the body.

For `LoopBound::Descent { cluster }`, E5 must return a named fail-closed
residual. Descent execution stays deferred to a later slice that consumes
`std.termination` evidence.

## Current Blocker

E1 and E2 are live on main, but E5 cannot yet meet the dispatch acceptance bar
without absorbing work owned by other PR-E slices.

- The live `eval_node` executes only `Behavior::Value`; `Transform`, `Branch`,
  `Loop`, and `Bind` still fail closed. A loop body that can consume the current
  accumulator and produce the next accumulator requires additional body
  behavior, not just the E1 literal path.
- The B.1.4 rule says to bind the accumulator for each iteration, but the
  executable binding port must be used consistently by later body evaluation.
  The candidate binding authority is `LoopNode.source`: it is the loop input /
  accumulator port and is already validated by `Dag::push_loop`. The first
  implementation PR should cite this audit and either use `source` as that
  binding port or STOP+PING if lowering authority says another port owns the
  accumulator.
- Count decoding needs a typed local diagnostic surface. The design allows
  "missing / non-integer count" to fail closed, but E1's current `EvalError`
  does not yet contain loop-specific variants. Adding those variants is
  implementation-local and allowed in E5, but it should happen with executable
  loop tests, not as dead API in this readiness PR.

A partial implementation that only handles zero iterations or a fixed literal
body would not prove accumulator threading. It would also create a misleading
`eval_loop` surface before the body evaluator can execute the body shapes loops
need. This slice therefore records the blocker instead of landing fake progress.

## Resume Gate

E5 implementation may resume when the body evaluator has enough behavior
coverage to run a loop body that reads the accumulator binding and returns a
new accumulator value without adding Transform, Branch, or Bind behavior inside
the E5 PR itself. The implementation PR should keep scope to `eval_loop`,
loop-specific diagnostics, and focused tests.

Required tests for the implementation PR:

- zero-iteration cardinality returns the evaluated init and does not evaluate
  the body;
- multiple-iteration cardinality evaluates the body deterministically;
- accumulator binding is visible to the body and threads the prior iteration's
  result;
- `LoopBound::Cardinality` with missing count fails closed;
- `LoopBound::Cardinality` with non-integer count fails closed;
- `LoopBound::Descent` returns the named residual;
- stack depth is restored on success and on loop-body diagnostics.

## Non-Goals

This readiness receipt does not authorize:

- `Transform`, `Branch`, or `Bind` implementation;
- strategy carrier widening, `NormalOrder`, `RightFirst`, or `EvalThunk`;
- substrate edits to `LoopNode`, `LoopBound`, `Value`, or `EvalStateStack`;
- runner or `test_runner.rs` changes;
- descent execution or `std.termination` evidence interpretation.
