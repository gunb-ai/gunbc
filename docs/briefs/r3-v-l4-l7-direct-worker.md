# R3 T-Verification-L4-L7-Direct Worker Brief

**Status:** STANDBY. Planning artifact only; do not dispatch implementation until the preconditions below are green.

**Owning manager:** R3 Verification Manager.

## Scope

Build the Evaluator-direct runtime verification harness for:

- **L4 emit/eval match:** for each corpus program and target, emitted target output equals `.dag` evaluator output.
- **L7 algebraic-law witnesses:** algebraic-law claims are checked by executing the declared witness surface, not by engine assertion or target-private policy.

This lane is a runtime equivalence harness, **not** a `Lens<C>` instance. It may consume lens outputs and structural pre-checks, but the core comparison reads emitted artifacts and evaluator results, which is outside `Lens<C>.read: (Dag, Behavior) -> Witness<C>`.

## Dispatch Preconditions

Dispatch only after a single ledger/readiness check shows:

1. R2-Evaluator body execution is landed enough to execute bounded `.dag` function bodies for the intended corpus.
2. PR-A.3 strategy / memoization carriers are implemented or the body-evaluator slice names a closed eager-baseline carrier sufficient for deterministic comparison.
3. Witness construction is available for L7, including runner support for all required `AlgebraicLawKind` rows. Current runner support is partial: `Associativity` has an operational witness path; other law kinds remain not-yet-implemented.
4. `DifferentialEquals` / related harness primitives from `r2_evaluator_cross_target_equivalence_harness_primitives.dag` have reached the slice needed by runtime emit/eval comparison, not only the cost-lineage scaffold.
5. The initial L4 corpus authoring spec is locked. `design-emission-model.md` recommends a generated cross-product corpus plus user-program corpus; this brief does not re-litigate that choice.

## Deliverables

- A corpus manifest for L4 direct verification with per-program target materialization rules.
- Runner mechanics that execute `.dag` evaluator output and emitted target output under the same input record.
- `TestClaim` rows named `l4_emit_eval_match_holds_per_corpus_program_per_target`.
- L7 `AlgebraicLaw` rows for the laws whose witness constructors are implemented.
- Failure diagnostics that distinguish evaluator failure, target execution failure, emit failure, and semantic mismatch.

## STOP Conditions

- Need for a new `TestPredicate` variant. Escalate via `INVARIANTS.md` §P1; do not introduce substrate from this lane.
- Any attempted L4 comparison that uses source-text or target-private policy as authority instead of evaluator output plus emitted artifact output.
- A law kind has only a textual/theoretical assertion and no executable witness constructor.
- The corpus tries to include targets whose Shape A grounding is not closed at dispatch.

## Non-Goals

- L5 cross-target equivalence corpus authoring; that is the sequential Lane 2 brief.
- L6 structural form coverage; that moved to R2 T-Ground-CrossTarget-Meta.
- Cost-lens composition; Substrate owns that R3 continuation lane.

## Acceptance

The lane closes when:

- `l4_emit_eval_match_holds_per_corpus_program_per_target` passes for every materialized `(program, target)` pair in the frozen corpus.
- `l7_algebraic_law_witnesses_evaluate_structurally` passes for every materialized law witness.
- The corpus manifest and target set are represented in checked-in fixtures, not only in prose.
