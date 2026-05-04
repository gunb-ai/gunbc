# R3 Verification TC1 Pattern A Second-Mover Conformance Audit

**Status:** AUDIT RECEIPT - no fixture edits required.
**Dispatch:** Worker B' TC1 eta-equivalence Pattern A second-mover audit,
Director-directed on #828 after the Worker C deferred-fixture sweep.
**Complements:** `docs/briefs/r3-v-tc1-eta-equivalence-deeper-analysis.md`.

## Contract

This audit does not re-check the fixture envelope already covered by Worker C
PR #1578 (`docs/briefs/r3-v-deferred-test-conformance-sweep.md`): the fixture
uses `BinaryDimensionReportEquals(tc1_subject_f_report,
tc1_subject_eta_expanded_report)`, and both refs resolve to
`DimensionReport<Tc1EtaLensObservation>`. This receipt checks the deeper TC1
claim-structure commitment behind that envelope:

1. The TC1 consumer remains `BinaryDimensionReportEquals`; it compares two
   typed report references only after producer-side `DimensionReport<C>` values
   exist.
2. The left and right report producers must resolve to the same observation
   carrier `C`; for the current fixture that carrier is
   `Tc1EtaLensObservation`.
3. TC1 semantics live in the producer-side roles: one report represents
   `fold_lens<C>` over `f`, and the other represents `fold_lens<C>` over the
   eta-expanded form `lambda x.apply(f, [x])`.
4. The consumer must not compare serialized reports, debug text, fixture prose,
   source strings, bytes, or a TC1-local report shape.
5. The fixture must not add a TC1-specific `TestPredicate`, producer identity
   enum, carrier field, or local Verification report carrier.

## TC1 Structural Claim Audit

The current fixture commits to exactly one semantic comparison:

```text
BinaryDimensionReportEquals(
  tc1_subject_f_report,
  tc1_subject_eta_expanded_report
)
```

Both report refs are declarations resolving to
`DimensionReport<Tc1EtaLensObservation>`. That carrier is intentionally narrow:
it marks the eta-equivalence observation slot without encoding a second report
shape or a TC1-specific predicate.

The fixture prose names the intended producer responsibilities:

- `tc1_subject_f_report` represents applying `fold_lens<C>` to `f`.
- `tc1_subject_eta_expanded_report` represents applying `fold_lens<C>` to
  `lambda x.apply(f, [x])`.

Those names are not executable producers yet. They are role declarations that
reserve the two report positions for the future producer surface. This matches
the existing deeper-analysis brief: TC1 strict-fire is not expressible by
`SubstrateResearchDeferredClaim` alone and still waits on
T-Substrate-Lens-Primitive, lens producer retirement, and generic
`DimensionReport<C>` production/evaluation.

## Witness-Shape Commitments

The safe second-mover shape is:

1. Producer A constructs or references the subject form `f`.
2. Producer B constructs or references the eta-expanded form
   `lambda x.apply(f, [x])`.
3. Both producers run the same lens fold family and return the same report
   carrier, `DimensionReport<Tc1EtaLensObservation>`.
4. `BinaryDimensionReportEquals` performs the structural equality check once
   report values are real.

No current fixture field commits to witness-list ordering, a serialized
normal form, or a string representation of eta-equivalence. Equality remains
over the existing `DimensionReport<C>` carrier:
`DimensionOk { dimension_name, composed, witnesses }` or
`DimensionFail { dimension_name, violations, witnesses }`.

## Pattern A Compositionality Verdict

Pattern A composes cleanly at the consumer layer: the existing
`BinaryDimensionReportEquals` standby contract is sufficient for the second
mover to compare the two TC1 reports once producer-side report declarations
exist. TC1 does not need a TC1-specific consumer predicate merely to express
"the two reports are equal."

Pattern A does still need producer-side concepts outside the consumer contract:

- an eta-pair producer or substrate relation that can name `f` and
  `lambda x.apply(f, [x])` without relying on fixture prose;
- a `fold_lens<C>` producer surface that emits real `DimensionReport<C>` values
  for both forms;
- a coverage decision for whether TC1 is universal over all `Lens<C>` instances
  or strict-fire over a ratified representative lens set.

Those are not defects in the current fixture. They are the producer-side
activation surface already identified by the deeper-analysis brief. The current
fixture should remain an author-now/fire-later consumer receipt until those
producer surfaces land.

## Non-Drift Findings

- No serialized report, string, byte, debug-output, or prose comparison is
  present.
- No new `TestPredicate` variant is introduced.
- No fixture-local `DimensionReport` or lens-report shape is introduced.
- `SubstrateResearchDeferredClaim` is not widened; it remains fixture-scoped
  staging history rather than the strict-fire path.

## Debt Receipt

Debt found + routed: none. This audit does not close a Debt-Paydown row
directly. It records that TC1's second-mover consumer shape conforms to the
Evaluator standby contract, while producer-side eta-pair and lens-fold
activation remain future Substrate/Evaluator work.
