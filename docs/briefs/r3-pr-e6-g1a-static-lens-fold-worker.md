# R3 PR-E E6-G1.a — Static lens fold worker brief

**Status:** worker-ready implementation brief. Authorizes the Evaluator E3
slice after Q-PAFS Path A acceptance. This brief does not authorize generic
`fold_lens<C>` dispatch, X1.b / `Indirect`, parser/lowerer changes,
`lens_apply.rs` authority migration, or new runtime carriers.

**Parent authorities:**
[`r3-pr-e6-g1a-static-lens-fold-dispatch-packet.md`](./r3-pr-e6-g1a-static-lens-fold-dispatch-packet.md)
§Slice boundary and §Acceptance gates,
[`r3-v-tc1-eta-equivalence-deeper-analysis.md`](./r3-v-tc1-eta-equivalence-deeper-analysis.md)
§"DESIGN — Q-PAFS first executable slice (Pattern-A / TC1)",
[`../r3-design-schedule-2026-05-06.md`](../r3-design-schedule-2026-05-06.md)
§E3.

## Slice Contract

Implement **E6-G1.a** only: execute a single static top-level
`data ... : Lens<C>` representative through the body evaluator so the
fold produces typed `DimensionReport<C>` values for the TC1 static
representative pair. This is the Path A first executable slice accepted
for Q-PAFS on 2026-05-06.

The first slice is representative, not universal. It proves the evaluator
can run the static lens fold and construct honest report values without
the deferred generic dispatch surface.

## In Scope

- Static lens value consumption: a named top-level `data` binding of type
  `Lens<C>` supplies the lens record.
- Static function-field calls: lens function fields such as `read`,
  `sequential`, `branch`, `iterate`, and `validate` must reach
  `eval_transform_node` as `TransformTarget::Callable` resolved by the
  existing X1.a static-data-head path.
- Runtime record field access: non-function lens fields use the existing
  `TransformTarget::FieldProject` path over `Value::RecordValue`.
- Honest report construction: `Witness<C>`, `OptionalDiagnostic`, and
  `DimensionReport<C>` are constructed through declared substrate record
  and variant constructors using the live evaluator carriers
  `Value::RecordValue` and `Value::VariantValue`.
- Program traversal authority: use explicitly named structural scope
  such as whole-`Dag` or named declaration refs. Do not inherit
  `source_file` filtering from `lens_apply.rs` as fold authority.
- TC1 handoff surface: produce the two typed `DimensionReport<C>` outputs
  needed by Verification's `BinaryDimensionReportEquals` consumer for the
  static eta representative.

## Out of Scope

- Generic `fold_lens<C>(lens: Lens<C>, ...)` where `lens` is a parameter.
- X1.b runtime-sourced callee dispatch, `TransformDispatch::Indirect`, or
  any string/function-name fallback.
- Local lens registries, host-Rust mirrors for reports/witnesses, or
  fixture-local producer identity.
- `lens_apply.rs` / `lens_testgen.rs` rewrites.
- New `Value` variants, parser/lowerer syntax changes, or substrate
  carrier-shape changes.
- Verification consumer implementation beyond the minimal producer
  receipt needed to coordinate with V1.

## Implementation Guidance

Start from the current body-evaluator surface in
`src/v3/compiler/src/lib.rs`:

- `eval_transform_node` for `TransformTarget::FieldProject`.
- `eval_transform_node` for Arrow `TransformTarget::Callable`.
- constructor execution for declared record and variant targets.
- `eval_loop` for `LoopBound::Cardinality`, with `LoopBound::Descent`
  still fail-closed.

The worker should implement the smallest evaluator path that can execute
the static representative fold end-to-end. Prefer reusing existing
declaration walking, frame, callable, record, and variant construction
helpers. If the only path requires duplicating lowerer substitution logic
or introducing a second report/lens representation, stop and surface the
missing substrate authority instead.

Coordinate with Verification V1 on names and handoff shape only. The
Evaluator PR should not own the `BinaryDimensionReportEquals` consumer
unless a separate Verification dispatch explicitly asks for it.

## Acceptance Gates

1. A static top-level `Lens<C>` representative can be compiled and
   evaluated without host-Rust lens application.
2. Lens function-field calls in that representative execute through
   `TransformTarget::Callable`; tests fail if they route through
   `lens_apply.rs` or a local registry.
3. Non-function lens field reads execute through `FieldProject` over
   `RecordValue`.
4. The fold constructs `DimensionReport<C>` values through declared
   substrate constructors; tests inspect `Value::VariantValue` tags and
   `RecordValue` payloads where needed.
5. The TC1 static representative pair can produce two typed reports for
   the Verification `BinaryDimensionReportEquals` consumer envelope.
6. Cardinality loops used by the representative execute; any
   `LoopBound::Descent` dependency remains fail-closed and is surfaced as
   E5-blocked, not approximated.
7. Existing G0b/G0c/G0d ratchets remain green: static field-call lowering
   to `Callable`, public evaluator execution for static field calls, and
   constructor runtime execution.

## STOP + PING

- If the static representative lens value does not exist or cannot be
  authored with current substrate declarations, stop and request the
  lens-value authoring prerequisite.
- If the fold requires calling methods on a parameter-held lens, stop:
  that is E6-G1.b after X1.b S1/S3.
- If report construction requires new runtime carriers or host structs,
  stop and route the carrier gap to Substrate / Evaluator Mgr before
  coding.
- If `LoopBound::Descent` is required for the representative, stop and
  wait for the Substrate `descent_execution_proof` carrier plus the
  Evaluator E2 consumer.
- If Verification needs a different producer interface than two typed
  `DimensionReport<C>` outputs, stop and coordinate before landing a
  parallel predicate surface.

## Worker Output

Open one implementation PR against `main` with:

- the evaluator changes for E6-G1.a static lens fold;
- focused compile-to-evaluate tests for the static representative and
  report construction;
- no unrelated citation, runner, parser, lowerer, or substrate churn.

Report any STOP as a PR comment or inbox note with the exact symbol /
section anchor that blocked implementation.
