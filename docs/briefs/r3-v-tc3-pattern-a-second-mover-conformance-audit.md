# R3 Verification TC3 Pattern A Second-Mover Conformance Audit

**Status:** AUDIT RECEIPT - no fixture edits required.
**Dispatch:** Worker B' TC3 strong-normalization Pattern A second-mover audit,
Director-directed on #828 after TC1/TC2 Pattern A coverage.
**Companion authority:** `docs/briefs/r3-v-formal-grounding-tc-bundle.md`.

## Contract

This audit checks TC3 below the fixture-envelope layer already covered by
`docs/briefs/r3-v-deferred-test-conformance-sweep.md`.

1. The TC3 consumer remains `BinaryDimensionReportEquals`; it compares two
   typed report references only after producer-side `DimensionReport<C>` values
   exist.
2. The left and right report producers must resolve to the same carrier `C`;
   for the current fixture that carrier is `Dag`.
3. TC3 semantics live in the producer-side roles: one report represents a
   baseline evaluation-step view, and the other represents bounded-step /
   termination-evidence comparison.
4. The consumer must not compare serialized reports, debug text, fixture prose,
   source strings, bytes, or a TC3-local theorem encoding.
5. The fixture must not add a TC3-specific `TestPredicate`, quantifier carrier,
   producer identity enum, carrier field, or local Verification report shape.

## TC3 Structural Claim Audit

The fixture commits to one consumer comparison:

```text
BinaryDimensionReportEquals(
  tc3_evaluation_step_baseline_dimension_report,
  tc3_evaluation_step_compare_dimension_report
)
```

Both refs are declarations resolving to `DimensionReport<Dag>`. The carrier is
deliberately broad: TC3 is a theorem over typed `.dag` programs, not over a
single lens-specific observation carrier. The fixture keeps the claim source
empty because the predicate is structural and should not smuggle a sample
program into the theorem surface.

The fixture prose names the intended producer responsibilities:

- `tc3_evaluation_step_baseline_dimension_report` is the baseline evaluation
  projection.
- `tc3_evaluation_step_compare_dimension_report` is the bounded-step /
  termination-evidence projection.

Those names are role declarations, not executable producers. This is the same
author-now/fire-later discipline as TC1 and TC2: the current runner validates
shape and returns `NotYetImplemented` until generic `DimensionReport<C>`
production/evaluation lands.

## Witness-Shape Commitments

The safe second-mover shape is:

1. Producer A emits a baseline `DimensionReport<Dag>` for evaluation-step
   semantics over the well-typed fragment.
2. Producer B emits a comparison `DimensionReport<Dag>` from bounded forward
   execution / termination evidence.
3. Both reports use the same `Dag` carrier and the same structural report
   envelope.
4. `BinaryDimensionReportEquals` performs structural equality once report
   values are real.

The current fixture does not commit to a serialized normal form, step trace
string, byte snapshot, or per-program sample corpus. Equality remains over the
existing `DimensionReport<C>` carrier:
`DimensionOk { dimension_name, composed, witnesses }` or
`DimensionFail { dimension_name, violations, witnesses }`.

## Pattern A Compositionality Verdict

Pattern A composes cleanly at the consumer layer: the existing
`BinaryDimensionReportEquals` standby contract is sufficient for comparing the
two TC3 reports once producer-side report declarations exist. TC3 does not need
a TC3-specific consumer predicate merely to express equality of two
`DimensionReport<Dag>` values.

TC3 does reach producer-side concepts beyond TC1 and TC2:

- a universal quantifier over the well-typed `.dag` fragment;
- bounded forward execution / evaluation-step evidence;
- termination evidence grounded in `LoopBound`, B5 loop construction-closure,
  and T-FixedPoint semantics;
- a proof/coverage decision for whether strict-fire is a structural induction
  proof, a generated exhaustive producer over a typed fragment carrier, or a
  ratified bounded representative harness.

Those are not consumer-envelope defects. They are activation prerequisites for
the producer side and remain outside Verification's local fixture authoring in
this sweep.

## Strict-Fire Preconditions

TC3 should not strict-fire until all of these are true:

1. B5 loop construction-closure remains green, so `Loop` construction is
   structurally bounded and not an open marker question.
2. T-Substrate-Lens-Primitive / generic report production can emit real
   `DimensionReport<Dag>` values for TC3's two role declarations.
3. T-FixedPoint termination semantics land, because the full theorem witness
   needs the fixed-point / bounded-forward-execution horizon.
4. Evaluator exposes the evaluation-step / bounded-step producer surface needed
   by the two role reports.
5. Director/Substrate/Evaluator ratify the universal-fragment coverage shape:
   structural induction over `Behavior` x `LoopBound`, generated exhaustive
   producer, or a bounded representative harness.

## Non-Drift Findings

- No serialized report, string, byte, debug-output, source-text, or prose
  comparison is present.
- No new `TestPredicate` variant or TC3-specific quantifier predicate is
  introduced.
- No fixture-local `DimensionReport`, theorem-report, or termination-report
  carrier is introduced.
- `TestClaim.source` remains empty; the fixture does not downgrade TC3 into a
  per-program runtime termination check.

## Debt Receipt

Debt found + routed: none. This audit does not close a Debt-Paydown row
directly. It records that TC3's second-mover consumer shape conforms to the
Evaluator standby contract, while universal-fragment, evaluation-step, and
termination-evidence producer surfaces remain future Substrate/Evaluator/PB
activation work.
