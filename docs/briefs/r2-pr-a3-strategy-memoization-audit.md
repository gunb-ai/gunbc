# R2 PR-A.3 Strategy / Memoization Audit

**Status:** AUDIT - docs-only decision surface for PR-A.3. PR-A.2 has
landed `EvalFrame` / `EvalStateStack`, but the PR-A.3 implementation slice is
blocked by the single-variant sum parser gap recorded in
[`r2-pr-a3-implementation-blocker-audit.md`](r2-pr-a3-implementation-blocker-audit.md).
This brief does not declare substrate carriers, does not edit evaluator Rust,
and does not strengthen TC2.

**Parent authority:** [`r2-pr-a-runtime-value-model.md`](r2-pr-a-runtime-value-model.md),
[`r2-evaluator-manager.md`](r2-evaluator-manager.md), and
[`docs/design-pb-runtime-interpreter.md`](../design-pb-runtime-interpreter.md)
sections 3.2-3.3.

## Purpose

Lock the PR-A.3 decision surface before implementation workers add strategy or
memoization state. PR-A.1 has landed the observable `Value` / `NamedField`
carrier in [`src/v3/std/runtime.dag`](../../src/v3/std/runtime.dag). PR-A.2
owns the evaluator-internal closed-over environment carriers:
`EvalFrame { bindings: Map<PortId, Value> }` and
`EvalStateStack { frames: List<EvalFrame> }`.

PR-A.3 depends on those state carriers. PR-A.2 has now landed them, but
PR-A.3 implementation remains blocked until the parser/substrate surface can
represent the closed one-inhabitant strategy carriers without fake variants.

## Decisions Locked For PR-A.3

### Eager Baseline First

The first executable strategy is deterministic eager evaluation:

- `Transform` evaluates input ports left-to-right before applying a callable,
  field projection, or operator.
- `Branch` evaluates the scrutinee first, then only the selected path body in a
  frame extended with that path binding.
- `Loop` evaluates the init once, then each bounded iteration body in a fresh
  frame containing the current accumulator binding.
- `Bind` adds bindings to the active frame; function bodies are entered from
  `TransformTarget::Callable(decl)` through the resolved `ArrowBody`.

This baseline is the comparison authority for later lazy or alternative input
order strategies. PR-A.3 implementation must not make lazy evaluation the first
or only executable semantics.

### Optional Lazy Boundary

Lazy evaluation is allowed only through an explicit `EvalThunk` boundary after
`EvalStateStack` exists:

```dag
type EvalThunk {
  node: NodeId
  captured_state: EvalStateStack
}
```

`captured_state` is load-bearing. A delayed node must evaluate in the lexical
state present when the thunk was created, not in the caller's later frame stack.
`EvalThunk` is evaluator-internal state. It is not a `Value` inhabitant and
does not authorize `ClosureValue`.

If PR-A.3 chooses not to enable lazy execution in the first implementation, it
may defer `EvalThunk`; it must still keep TC2 deferred and keep memoization
keyed to the eager strategy carrier.

### Closed Strategy Carrier

Strategy identity is a closed carrier, not a string:

```dag
type EvalStrategy
  = ApplicativeOrder { input_order: InputEvaluationOrder }

type InputEvaluationOrder
  = LeftFirst
```

`LeftFirst` is the only input order PR-A.3 should land with the eager baseline.
The initial carrier is therefore eager-only unless the same implementation slice
also lands `EvalThunk` and the normal-order evaluation rule. `NormalOrder` is a
future strategy inhabitant, not part of the baseline carrier by itself.

Adding `NormalOrder`, `RightFirst`, parallel input evaluation, or additional
strategy variants is a semantic expansion. Such a variant must land with:

1. A stated evaluation rule.
2. A memo-key inclusion rule.
3. A TC2 comparison obligation that says which two strategies are compared.
4. Any state carrier that makes the strategy executable; for `NormalOrder`,
   that means `EvalThunk` with captured `EvalStateStack`.

No PR-A.3 implementation may use string labels such as `"normal"` or
`"left-first"` as strategy authority.

### Structural State Identity

Memoization keys must include structural evaluator state identity:

```dag
type EvalStateKey {
  state: EvalStateStack
}

type EvalMemoKey {
  program: DeclarationId
  node: NodeId
  state_key: EvalStateKey
  strategy: EvalStrategy
}
```

`EvalStateKey` is the reachable `EvalStateStack`, not a name-only fingerprint.
Two evaluations of the same `node` in the same `program` with different
bindings are different memo keys. PR-A.3 may replace
`EvalStateKey { state: EvalStateStack }` with a digest carrier only if the
digest carrier's construction and equality are the single declared authority
over reachable-state identity. A free `String` fingerprint is forbidden.

Memo entries cache completed `Value` results only. Diagnostics, partial
results, in-progress thunks, and failed evaluations are not cached as values.

## Post-PR-A.2 Implementation Artifacts

After PR-A.2 lands `EvalFrame` / `EvalStateStack`, PR-A.3 should produce a
small substrate/runtime slice:

- [`src/v3/std/runtime.dag`](../../src/v3/std/runtime.dag): add
  `EvalStrategy`, `InputEvaluationOrder`, `EvalStateKey`, and `EvalMemoKey`.
  The first `EvalStrategy` carrier should contain only
  `ApplicativeOrder { input_order: LeftFirst }` unless the same slice enables
  an explicit lazy boundary. Add `EvalThunk` and `NormalOrder` together, or keep
  both deferred with this audit as the named dependency.
- `src/v3/compiler/src/bootstrap_generated.rs` and
  `src/v3/compiler/src/bootstrap_generated_without_parse_surface.rs`:
  regenerate after the `.dag` carrier declarations land.
- `src/v3/compiler/tests/integration/m2_substrate_inhabitance_test.rs`: add
  structural carrier tests, named along these lines:
  - `runtime_eval_strategy_has_closed_baseline_shape`
  - `runtime_eval_memo_key_uses_structural_state_key`
  - `runtime_eval_thunk_captures_eval_state_stack` if `EvalThunk` lands
- `src/v3/compiler/tests/fixtures/r2_evaluator_runtime_value_model.dag`:
  strengthen `evaluator_runtime_value_model_landed` only after the available
  verification predicate can validate `Value`, PR-A.2 state, and PR-A.3
  strategy/memo carriers without collapsing back to `Compiles`.
- `src/v3/compiler/tests/fixtures/tc2_evaluation_order_independence_deferred.dag`:
  keep the current deferred claim until at least two executable strategies run
  through the same evaluator boundary.

Rust evaluator implementation remains a later body-evaluator slice unless a
manager dispatch explicitly asks PR-A.3 to mirror carrier declarations in Rust.
If a Rust mirror is dispatched, it must mirror the `.dag` carriers without
adding observable `Value` variants and without string strategy labels.

## TC2 Boundary

`evaluation_order_independent_lens_results` is not proven by this audit and is
not proven by merely declaring `EvalStrategy`. It can strengthen only when:

1. PR-A.2 state carriers exist.
2. PR-A.3 has at least two executable strategies or input orders.
3. Both strategies evaluate the same program/lens boundary to comparable
   `DimensionReport<C>` results.

Until then, TC2 remains an author-now/fire-later fixture with `Compiles` as its
placeholder predicate. This audit may be cited as the reason strict equality is
deferred, not as evidence that Church-Rosser / evaluation-order independence
already holds.

## Non-Goals

- No edits to `src/v3/std/runtime.dag` in this audit.
- No `EvalFrame` / `EvalStateStack` carrier changes; PR-A.2 owns them.
- No body evaluator and no `lens_apply.rs` reflection-projection work.
- No new `Value` variants, especially no `ClosureValue`.
- No memoization of diagnostics or partial evaluator state as `Value`.
- No strategy strings or name-only state fingerprints.

## Resume Gate

PR-A.3 implementation may start after the blocker in
[`r2-pr-a3-implementation-blocker-audit.md`](r2-pr-a3-implementation-blocker-audit.md)
is resolved by an approved substrate/parser path. The first implementation PR
must cite this audit, the live PR-A.2 carrier path, and the specific subset it
is landing: closed eager strategy only, memo-key carriers, optional
`EvalThunk`, or a strict TC2 strengthening slice.
