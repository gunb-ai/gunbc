# R3 Verification Deferred-Test Conformance Sweep

**Status:** AUDIT RECEIPT - no fixture edits required.
**Dispatch:** Worker C sweep, Director-cleared on #828; Evaluator safe contract from #1131 reply 4365081919.
**Scope:** TC1, TC2, TC3, and RustDagIsomorphism deferred-test fixtures.

## Contract

The audited fixtures must stay inside the Evaluator standby contract:

1. Consumer refs are declarations or arrows whose output/inhabitance resolves
   to `DimensionReport<C>` for the same carrier `C` on both sides.
2. `BinaryDimensionReportEquals` is the comparison envelope; structural
   equality occurs only after real typed report values exist.
3. Fixtures must not compare serialized reports, strings, bytes, debug output,
   or fixture prose.
4. Fixtures must not introduce new `TestPredicate` variants, producer
   identities, carrier fields, or local Verification report shapes.

## Runner Surface

The live runner surface matches the safe contract:

- `src/v3/std/verification.dag` defines one
  `BinaryDimensionReportEquals { left_report_ref, right_report_ref }` predicate.
- `src/v3/compiler/src/test_runner.rs` resolves both payload fields as
  `DeclarationRef` edges.
- The runner accepts either a declaration whose connective is
  `DimensionReport<C>` or an arrow whose output resolves to
  `DimensionReport<C>`.
- The runner rejects mismatched carriers before evaluation.
- The runner returns `NotYetImplemented` after shape validation and explicitly
  does not compare serialized reports.

## Fixture Results

| Fixture | Result | Audit notes |
|---|---|---|
| `src/v3/compiler/tests/fixtures/tc1_substrate_lens_eta_equivalence_deferred.dag` | conforms | Uses `BinaryDimensionReportEquals(tc1_subject_f_report, tc1_subject_eta_expanded_report)`. Both refs are declarations resolving to `DimensionReport<Tc1EtaLensObservation>`. No serialized/report-text comparison and no fixture-local predicate shape. |
| `src/v3/compiler/tests/fixtures/tc2_evaluation_order_independence_deferred.dag` | conforms | Uses `BinaryDimensionReportEquals(tc2_leftfirst_strategy_dimension_report, tc2_rightfirst_strategy_dimension_report)`. Both refs resolve to `DimensionReport<Dag>`. Strategy-order semantics remain in future producers, not in a new predicate variant. |
| `src/v3/compiler/tests/fixtures/tc3_strong_normalization_deferred.dag` | conforms | Uses `BinaryDimensionReportEquals(tc3_evaluation_step_baseline_dimension_report, tc3_evaluation_step_compare_dimension_report)`. Both refs resolve to `DimensionReport<Dag>`. Evaluation-step semantics remain producer-owned and strict-fire still waits on T-FixedPoint. |
| `src/v3/compiler/tests/fixtures/rust_dag_isomorphism_consumer.dag` | conforms | Uses `BinaryDimensionReportEquals(RustEnumExtractionDagShapeReport, DagReflectionDagShapeReport)`. Both refs resolve to `DimensionReport<Dag>`. The fixture is a consumer instance over the shared envelope, not a new predicate family. |

## Existing Coverage

Current integration coverage exercises the same shape-valid/NYI path:

- `tc1_substrate_lens_eta_equivalence_deferred_test::tc1_substrate_lens_eta_suite_reaches_unified_predicate_shape`
- `m1_5_verification_test::tc2_evaluation_order_independence_suite_passes`
- `tc3_strong_normalization_deferred_test::tc3_strong_normalization_suite_shape_valid_nyi_at_head`
- `m1_5_verification_test::rust_dag_isomorphism_consumer_reaches_binary_report_shape_gate`

## Debt Receipt

Debt found + routed: none. This sweep does not close a Debt-Paydown row; it
records that the four existing deferred fixtures remain aligned with the
Evaluator safe contract while generic `DimensionReport<C>` production and
evaluation remain NYI.
