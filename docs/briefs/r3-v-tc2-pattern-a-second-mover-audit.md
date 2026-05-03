# R3 Verification TC2 Pattern A Second-Mover Audit

**Status:** AUDIT RECEIPT - docs-only; no fixture, substrate, or runner edits.
**Dispatch:** Worker B TC2 second-mover audit after Worker C's deferred-test
conformance sweep.
**Scope:** `src/v3/compiler/tests/fixtures/tc2_evaluation_order_independence_deferred.dag`
structural TC2 claim semantics, not the already-audited `BinaryDimensionReportEquals`
fixture envelope.

## Contract Restatement

Evaluator's standby safe contract for `BinaryDimensionReportEquals` is:

1. Consumer refs resolve to declarations or arrows producing `DimensionReport<C>`
   for the same carrier `C`.
2. `BinaryDimensionReportEquals` remains the single comparison envelope.
3. Until typed `DimensionReport<C>` producer/evaluator paths exist, the runner
   performs shape validation and returns `NotYetImplemented`; it must not compare
   serialized reports, strings, bytes, debug output, or fixture prose.
4. Verification fixtures must not introduce new `TestPredicate` variants, local
   report shapes, producer identities, or ad hoc report comparison authority.

Worker C's conformance sweep records that TC2 satisfies this envelope. This
audit asks the next question: whether the same standby envelope composes cleanly
with TC2's evaluation-order independence semantics when Pattern A becomes
executable.

## TC2 Structural Claim Audit

The fixture's load-bearing claim is
`evaluation_order_independent_lens_results`. It declares two role types:

- `tc2_leftfirst_strategy_dimension_report = DimensionReport<Dag>`
- `tc2_rightfirst_strategy_dimension_report = DimensionReport<Dag>`

and compares those refs through:

```dag
BinaryDimensionReportEquals(
  tc2_leftfirst_strategy_dimension_report,
  tc2_rightfirst_strategy_dimension_report
)
```

The comments bind those role declarations to TC2's intended theorem: for the
same program and lens, valid evaluation strategies produce identical
`DimensionReport<C>` results. The named first executable contrast is
LeftFirst vs RightFirst input evaluation for n-ary `Transform`; the broader
semantic contrast is applicative-order vs normal-order.

The witness shape is therefore not a claim program in `TestClaim.source` and
not a serialized expected report. It is a pair of future report-producing refs
whose names currently stand in for strategy-conditioned roles. Strict-fire must
replace the role-name convention with producer-owned typed facts, while keeping
the fixture on the shared `BinaryDimensionReportEquals` envelope.

## Producer-Side Concepts

TC2 reaches into a producer-side concept that TC1 does not: evaluator strategy.
Current `src/v3/std/runtime.dag` intentionally exposes only:

- `EvalStrategy = ApplicativeOrder { input_order: InputEvaluationOrder }`
- `InputEvaluationOrder = LeftFirst`
- `EvalMemoKey { ..., strategy: EvalStrategy }`

The comments explicitly forbid placeholder `NormalOrder`, `RightFirst`, or
parallel-evaluation variants without executable evaluator behavior. That matches
the TC2 analysis brief: the first tractable second strategy is likely a second
`InputEvaluationOrder` under the same eager skeleton; full normal-order needs
thunk/state capture and is a larger residual.

So TC2's Pattern A strict-fire cannot be "two report aliases happen to compare
equal." It needs a typed producer path that can say:

1. this report was produced from program P and lens L;
2. through the same evaluator boundary;
3. under strategy S1 or S2, where S1/S2 are closed substrate strategy carriers;
4. with memo/evaluator state keyed structurally, not by names or strings.

Those are producer-side obligations. They should not be encoded in the fixture
as local producer identities or alternate predicates.

## Compositionality Verdict

**Verdict: composes, with a TC2-specific producer-side strategy modifier.**

Pattern A's executable shape can compose cleanly to TC2 evaluation-order
semantics if the eventual producer/evaluator surface supplies
strategy-conditioned `DimensionReport<C>` values. The shared comparison core
remains binary structural equality over the same `DimensionReport<C>` carrier,
so TC2 does not need a TC2-only predicate and does not need a local report
shape.

TC2 does, however, require one concept TC1 does not: a typed strategy-order
modifier over closed evaluator strategy/input-order carriers. TC1's eta leg
compares reports for two related program forms under the ordinary lens fold.
TC2 compares reports for the same program/lens under two evaluator strategies.
That difference belongs in the producer/modifier surface, not in a second
Verification fixture shape.

## Strict-Fire Preconditions

TC2 should stay deferred until all of these are live:

1. generic `DimensionReport<C>` production/evaluation for
   `BinaryDimensionReportEquals`;
2. a same-boundary evaluator entry that can produce reports for a program/lens
   pair;
3. at least two executable strategy/input-order inhabitants, not placeholder
   variants;
4. structural memo/state identity that includes the selected strategy;
5. a strategy-order modifier or equivalent producer metadata that binds each
   report to the selected strategy without string labels.

If any future author cannot express item 5 without adding fixture-local
producer identities or a TC2-only predicate, that is a Substrate/Evaluator
STOP+PING, not a Verification-side workaround.

## Debt Receipt

No debt touched. This audit does not pay down, introduce, or route a tracked
Debt-Paydown row; it records that TC2's deferred fixture can remain on the
shared `BinaryDimensionReportEquals` standby contract, while strict execution
waits on typed strategy-conditioned report producers.
