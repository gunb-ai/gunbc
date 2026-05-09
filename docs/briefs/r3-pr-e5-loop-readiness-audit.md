# R3 PR-E E5 Loop Readiness Audit

**Status:** **Cardinality** `Behavior::Loop` execution is live on **`main`**;
**`LoopBound::Descent`** execution remains deferred. This is a docs-only receipt;
it does not implement descent execution, change substrate carriers, widen strategy
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
   accept only a non-negative runtime integer literal count. Missing,
   non-integer, or negative counts must produce typed diagnostics rather than
   defaulting to zero or wrapping into an unsigned iteration count.
4. For each iteration, push a fresh top frame, bind the accumulator value for
   that iteration to the loop accumulator port, evaluate `loop_node.body`
   through `eval_node`, pop the frame on success or failure, and thread the
   body result into the next iteration.
5. Return the final accumulator. Zero iterations returns the evaluated init
   value without evaluating the body.

For `LoopBound::Descent { cluster }`, E5 must return a named fail-closed
residual. Descent execution stays deferred to a later slice that consumes
`std.termination` evidence.

## Remaining gaps (post-cardinality landing)

- **`LoopBound::Descent`:** execution stays a named **`LoopBoundDescentResidual`**
  until a later slice consumes `std.termination` evidence (per E5 **STOP+PING**
  in [`r3-evaluator-dispatch.md`](r3-evaluator-dispatch.md)).

- **Generic lens fold / full `Lens.iterate` honesty** over every `LoopBound`
  inhabitant is **E6** scope and still depends on **`Behavior::Bind`** and other
  items in [`r3-pr-e6-lens-fold-readiness-audit.md`](r3-pr-e6-lens-fold-readiness-audit.md)—not
  additional cardinality-loop work in this receipt.

## Resume Gate

**Cardinality E5** has met its dispatch bar on **`main`** (`evaluator::eval_loop`
with accumulator binding per B.1.4). Further **E5** work resumes for
**`LoopBound::Descent`** when a slice can execute descent against termination
evidence without widening loop-bound substrate in that same PR. Keep **Bind**
and generic lens-fold concerns in **E6**, not mixed into the descent slice.

Required tests for the implementation PR:

- zero-iteration cardinality returns the evaluated init and does not evaluate
  the body;
- multiple-iteration cardinality evaluates the body deterministically;
- accumulator binding is visible to the body and threads the prior iteration's
  result;
- `LoopBound::Cardinality` with missing count fails closed;
- `LoopBound::Cardinality` with non-integer count fails closed;
- `LoopBound::Cardinality` with negative integer count fails closed;
- `LoopBound::Descent` returns the named residual;
- stack depth is restored on success and on loop-body diagnostics.

## Non-Goals

This readiness receipt does not authorize:

- `Transform`, `Branch`, or `Bind` implementation;
- strategy carrier widening, `NormalOrder`, `RightFirst`, or `EvalThunk`;
- substrate edits to `LoopNode`, `LoopBound`, `Value`, or `EvalStateStack`;
- runner or `test_runner.rs` changes;
- descent execution or `std.termination` evidence interpretation.
