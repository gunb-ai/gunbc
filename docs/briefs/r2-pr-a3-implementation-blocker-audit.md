# R2 PR-A.3 Implementation Blocker Audit

**Status:** PARSER PREREQUISITE RESOLVED - PR #1286 landed parser support for
the one-variant sum syntax. The blocker audit remains as the historical
receipt; Substrate now owns the PR-A.3 carrier declarations. This is not a
carrier declaration PR.

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

## Historical Parser Gap

Before PR #1286, the handwritten parser could not express the desired
one-inhabitant record-payload sum in the intended spelling. In that earlier
state,
[`src/v3/compiler/src/parse_generated.rs`](../../src/v3/compiler/src/parse_generated.rs),
`rhs_is_sum()` routes a `type X = ...` RHS to `TypeSum` only when it sees a
top-level `|` before the next item boundary. `parse_sum_variants()` then
parses variants after that route has already been selected; it does not
provide an alternate leading-pipe one-variant spelling.

As a result, this authored PR-A.3 spelling:

```dag
type EvalStrategy = ApplicativeOrder { input_order: InputEvaluationOrder }
```

was treated as a `TypeAlias` to `ApplicativeOrder`, and the following `{`
was reported as a top-level parse error. Bootstrap regeneration recorded
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
pacify a parser heuristic. PR #1286 resolved the parser prerequisite; the
remaining discipline is to keep carrier implementation in Substrate territory
and preserve the closed one-inhabitant semantics directly.

## Resolved Unblock Record

The accepted unblock path was parser support for single-variant sums:

1. Record-payload variant support for
   `ApplicativeOrder { input_order: InputEvaluationOrder }`.
2. Bare single-variant atom support for `InputEvaluationOrder = LeftFirst`.
3. Regression coverage for both shapes in the parser prerequisite PR.

After PR #1286, parser syntax is no longer the live blocker. Substrate owns the
carrier declarations and generated bootstrap updates; Worker A owns the
follow-on verification surface. The existing TC2
`evaluation_order_independent_lens_results` fixture remains deferred until
multiple executable strategies exist.
