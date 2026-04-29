# R2 PR-A Runtime Value Model

**Status:** PROPOSAL - Worker A design lock for the R2 Evaluator runtime-value lane. This is the work product for the dispatch sometimes called "PR-B runtime-value model" by the Director; the live Evaluator Manager brief still names this lane **PR-A**. This brief follows the live manager brief label and preserves the dispatch naming drift explicitly.

**Parent authority:** [`docs/briefs/r2-evaluator-manager.md`](r2-evaluator-manager.md) section "Owned deliverables" and section "Pre-dispatch design lock cadence". **PB-Runtime convergence authority:** [`docs/design-pb-runtime-interpreter.md`](../design-pb-runtime-interpreter.md) sections 2-3. **P1 authority:** [`INVARIANTS.md`](../../INVARIANTS.md) P1.

## Decision

R2-Evaluator's runtime-value model has two layers:

1. **Observable runtime values:** exactly the `Value` coproduct locked in `docs/design-pb-runtime-interpreter.md` section 3.2:
   - `LiteralValue(LiteralBits)`
   - `RecordValue(List<NamedField>)`
   - `VariantValue { tag: DeclarationId, payload: Value }`
   - `NodeRef(NodeId)`
   - `CardinalityValue(LoopBound)`
2. **Evaluator-internal state:** structural carriers that guide evaluation but are not `Value` inhabitants:
   - `EvalFrame` is a finite partial function from `PortId` to `Value` for one binding scope.
   - `EvalStateStack` is a stack of `EvalFrame`s for nested calls, branch bindings, and loop iterations.
   - `EvalThunk` is the optional lazy boundary carrier: an unevaluated `NodeId` plus an `EvalStateStack` snapshot, memoized by evaluator-owned state.

Closed-over environments therefore live in evaluator state, not in observable `Value`. This follows `docs/design-pb-runtime-interpreter.md` section 3.3 and passes `INVARIANTS.md` P1 locally: the underlying external fact is lexical environment / activation-frame semantics from lambda-calculus evaluation; the carriers are coordinates of evaluation state, not user-visible alternatives in the result domain; and the fact is runtime-substrate state, not a lens-extensible domain label.

## Substrate Targets

The first implementation slice should add a runtime module under the v3 std/runtime surface once that directory is introduced by the Evaluator or PB-Runtime lane. Suggested `.dag` declarations:

```dag
type NamedField {
  label: String
  value: Value
}

type Value
  = LiteralValue(LiteralBits)
  | RecordValue(List<NamedField>)
  | VariantValue { tag: DeclarationId, payload: Value }
  | NodeRef(NodeId)
  | CardinalityValue(LoopBound)

type EvalFrame {
  bindings: Map<PortId, Value>
}

type EvalStateStack {
  frames: List<EvalFrame>
}

type EvalThunk {
  node: NodeId
  captured_state: EvalStateStack
}

type EvalStrategy
  = ApplicativeOrder { input_order: InputEvaluationOrder }
  | NormalOrder

type InputEvaluationOrder
  = LeftFirst

type EvalStateKey {
  state: EvalStateStack
}
```

`EvalThunk` is not required for an eager-only first evaluator, but the carrier is the right lazy boundary if the evaluator chooses normal-order evaluation for a call or branch edge. A thunk captures state because evaluating a delayed node must use the lexical environment at thunk creation, not the caller's later frame stack.

`NamedField` is the `RecordValue` field carrier named by `docs/design-pb-runtime-interpreter.md` section 3.2. PR-A.1 must not introduce a second `NamedRuntimeField` authority unless it simultaneously amends the PB-Runtime design lock. `EvalFrame.bindings` uses `Map<PortId, Value>` because maps are finite partial functions: duplicate binding for one `PortId` is unrepresentable at the carrier boundary. If `Map<K, V>` is not yet available in the v3 runtime module when PR-A.2 lands, PR-A.2 must either port the existing `Map = PartialFunction` authority or add a fail-closed unique-binding carrier before mirroring evaluator state; it must not fall back to a duplicate-admitting `List<EvalBinding>`.

No `ClosureValue` variant should be added in R2. The live substrate has `TransformTarget::Callable(DeclarationId)` and `ArrowBody::UserDefined(NodeId)`, so callable identity is declaration/node identity plus the active frame stack at call time. First-class closures with captured environments are not expressible today; adding them would be a new substrate fact and must go through `INVARIANTS.md` P1 with the Substrate Manager.

## Evaluation Strategy

R2-Evaluator should implement a deterministic eager baseline first:

- `Transform` evaluates inputs left-to-right before applying `Callable`, `FieldProject`, or `Operator`.
- `Branch` evaluates the scrutinee before selecting a path, then evaluates only the selected path body in a fresh frame containing the path binding.
- `Loop` evaluates the init once, then each bounded iteration body in a fresh frame containing the accumulator binding for that iteration.
- `Bind` registers bindings in the current frame; function-form bodies are entered from `TransformTarget::Callable` by reading the `ArrowBody`.

Lazy evaluation is allowed only at explicit thunk boundaries. The result must be observationally equal to the eager baseline for pure `.dag` bodies; TC2 (`evaluation_order_independent_lens_results`) is the deferred structural claim for that theorem.

## Memoization Boundary

Memoization is evaluator-owned state, not an observable value. The safe key shape is:

```dag
type EvalMemoKey {
  program: DeclarationId
  node: NodeId
  state_key: EvalStateKey
  strategy: EvalStrategy
}
```

`strategy` is a closed carrier, not a string label. PR-A.3 may add variants only when it also defines their evaluation rule and TC2 comparison obligation. `state_key` is the structural reachable `EvalStateStack` identity for the node, not a string digest or name-only proxy; two calls to the same node with different bindings are distinct. PR-A.3 may replace `EvalStateKey { state: EvalStateStack }` with a digest carrier only if that carrier's construction and equality are the single declared authority over reachable-state identity. Memo entries cache `Value` results only after the evaluation completes successfully. Diagnostics and partial results are not cached as values.

## Acceptance Gates

This brief opens two runner-visible fixture hooks:

- `evaluator_runtime_value_model_landed` in `src/v3/compiler/tests/fixtures/r2_evaluator_runtime_value_model.dag`
- `evaluation_order_independent_lens_results` in `src/v3/compiler/tests/fixtures/tc2_evaluation_order_independence_deferred.dag`

Both are structural author-now gates. The first is `Compiles` until the runtime module exists and can be strengthened to a substrate-shape predicate. The second is explicitly gated on PB-Runtime spec landing and T-Substrate-Lens-Primitive landing; it must strengthen to a strict differential/lens equality claim when evaluator strategies are executable.

## Implementation Gates

Before body evaluation or reflection-projection consumes runtime values:

1. `Value`, `EvalFrame`, and `EvalStateStack` are declared in one runtime authority module.
2. Rust evaluator state mirrors those declarations without adding observable `Value` variants.
3. A call to `ArrowBody::UserDefined(NodeId)` pushes a fresh `EvalFrame`, binds parameters by `PortId`, evaluates the body node, and pops the frame.
4. Lazy strategy, if enabled, captures `EvalStateStack` in `EvalThunk` and memoizes only completed `Value` results keyed by node plus state.
5. TC2 remains deferred until both eager and lazy/order variants are executable through the same evaluator boundary.

## Follow-Up Split

PR-A.0 is design + structural gates only. It is not the runtime carrier implementation.

- **PR-A.1:** declare the observable `Value` surface in the runtime module, matching `docs/design-pb-runtime-interpreter.md` section 3.2 exactly, including the `NamedField` record payload carrier.
- **PR-A.2:** declare and mirror evaluator-internal `EvalFrame` / `EvalStateStack` carriers; wire closed-over environment lookup without adding `Value` variants and without duplicate-admitting frame bindings.
- **PR-A.3:** lock the executable strategy and memoization boundary (`EvalThunk` if lazy boundaries are enabled; eager baseline first).
- **PR-B / body evaluator:** implement body execution only after PR-A.1 through PR-A.3 provide the carrier substrate.

## Non-Goals

- Do not implement complete reflection in `lens_apply.rs`; PR-E owns reflected-program lens application.
- Do not add `ClosureValue` to `Value`.
- Do not introduce a second PB-Runtime value shape. PB-Runtime and R2-Evaluator share one structural model.
