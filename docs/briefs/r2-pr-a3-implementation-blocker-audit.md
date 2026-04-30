# R2 PR-A.3 Implementation Blocker Audit

**Status:** BLOCKED - implementation audit for PR-A.3 eager strategy /
memoization carriers. This is not a carrier declaration PR.

**Parent authority:** [`r2-pr-a3-strategy-memoization-audit.md`](r2-pr-a3-strategy-memoization-audit.md),
[`r2-pr-a-runtime-value-model.md`](r2-pr-a-runtime-value-model.md), and
[`docs/design-pb-runtime-interpreter.md`](../design-pb-runtime-interpreter.md)
sections 3.2-3.3.

## Blocked Carrier Spelling

PR-A.3 is supposed to land a closed eager-baseline strategy shape:

```dag
type EvalStrategy = ApplicativeOrder { input_order: InputEvaluationOrder }

type InputEvaluationOrder
  = LeftFirst

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

The load-bearing property is that `EvalStrategy` and
`InputEvaluationOrder` each have exactly one inhabitant in the first
implementation slice. `NormalOrder`, `RightFirst`, parallel input order,
`EvalThunk`, body evaluator logic, and TC2 strict equality are deferred by
the parent audit.

## Parser Gap

The current handwritten parser cannot express the desired one-inhabitant
record-payload sum in the intended spelling. In
[`src/v3/compiler/src/parse_generated.rs`](../../src/v3/compiler/src/parse_generated.rs),
`rhs_is_sum()` routes a `type X = ...` RHS to `TypeSum` only when it sees a
top-level `|` before the next item boundary. `parse_sum_variants()` then
parses variants after that route has already been selected; it does not
provide an alternate leading-pipe one-variant spelling.

As a result, this authored PR-A.3 spelling:

```dag
type EvalStrategy = ApplicativeOrder { input_order: InputEvaluationOrder }
```

is treated as a `TypeAlias` to `ApplicativeOrder`, and the following `{`
is reported as a top-level parse error. Bootstrap regeneration then records
the parse diagnostic instead of declaring `EvalStrategy`.

## Forbidden Workarounds

Do not add fake variants just to satisfy `rhs_is_sum()`:

- no `NormalOrder` until `EvalThunk` and an executable normal-order rule land;
- no `RightFirst` or parallel input order until their evaluation rule,
  memo-key inclusion, and TC2 comparison obligation land;
- no duplicate or placeholder strategy/input-order variants;
- no string strategy labels or name-only state fingerprints;
- no new `Value` variants, especially no `ClosureValue`.

Those workarounds would make illegal PR-A.3 states constructible merely to
pacify a parser heuristic. The correct result is to block implementation
until the surface syntax or carrier model can represent the closed
one-inhabitant semantics directly.

## Acceptable Unblock Paths

PR-A.3 implementation may resume after one of these lands:

1. Parser support for single-variant sums, including record-payload variants
   such as `ApplicativeOrder { input_order: InputEvaluationOrder }`, with a
   regression test covering this exact shape.
2. An approved product/alias carrier amendment that preserves the
   one-inhabitant closed strategy semantics and is explicitly accepted by the
   PR-A.3 / PB-Runtime authorities.
3. A broader substrate syntax decision that provides an equivalent
   one-inhabitant sum spelling without adding fake strategy inhabitants.

Until one of those gates lands, PR-A.3 remains blocked at implementation and
the existing TC2 `evaluation_order_independent_lens_results` fixture remains
deferred.
