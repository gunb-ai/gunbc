# R3 T-V-L4-L7-Direct Worker Brief

**Status:** STANDBY. This is a planning brief, not a dispatch order. Dispatch waits on the R2-Evaluator and substrate prerequisites named below.

**Owning manager:** R3 Verification Manager.

## Scope

Build the Evaluator-direct verification harness for:

1. **L4 `l4_emit_eval_match`** — each certification-corpus `.dag` program evaluates through the `.dag` Evaluator and through each emitted target; the two results are algebraically equal.
2. **L7 `l7_algebraic_laws_witnessed`** — every relevant algebra declaration has runtime-constructed law witnesses.

The lane is a runtime-equivalence harness. It may consume lens outputs and witness substrate, but it is not itself a `Lens<C>` instance because it compares emitted target artifacts against evaluator results.

## Dispatch preconditions

- R2-Evaluator body evaluator lands with enough coverage to execute corpus programs.
- R2-Evaluator witness-construction surface lands for `AlgebraicLaw` claims.
- PR-A.3 strategy / memoization carriers are implemented, not only audited.
- Cross-target equivalence primitives from [`r2-pr-d-cross-target-equivalence-harness-primitives.md`](r2-pr-d-cross-target-equivalence-harness-primitives.md) are available where `DifferentialEquals`-style claim evaluation is reused.
- The L4 corpus authoring spec from [`docs/design-emission-model.md`](../design-emission-model.md) Q4 is materialized into checked-in fixture rows.

## Implementation outline

1. Author the certification-corpus manifest as `.dag` fixture data. Include generated cross-product seeds plus user-program seeds from `dsl/std/`, `src/v3/std/`, and `dsl/examples/` where they compile under the landed Evaluator.
2. Add L4 claim rows for every `(program, target)` pair in the materialized corpus and target set.
3. Implement or wire the runner path that invokes `.dag` evaluation and emitted-target execution, then compares algebraic output values rather than target byte strings.
4. Add L7 rows for algebra laws in `dsl/std/algebra.dag`; witnesses must be constructed by the Evaluator, not hard-coded host assertions.
5. Add ratchets that fail on target/corpus drift: new materialized corpus rows require corresponding L4 rows, and new algebra laws require corresponding L7 rows.

## Acceptance

- `l4_emit_eval_match` evaluates Pass for each materialized corpus row and target.
- `l7_algebraic_laws_witnessed` evaluates Pass for each materialized algebra-law witness row.
- Failure messages identify the program, target, entry point, and mismatched algebraic value.
- No host-only oracle silently replaces Evaluator execution; host code may orchestrate process execution, but the claim authority is `.dag` evaluation plus target runtime output.

## STOP conditions

- A needed corpus row requires a new substrate carrier or `TestPredicate` variant. Stop and escalate through `INVARIANTS.md` §P1.
- The Evaluator cannot execute a representative corpus program because the body evaluator is still fixture-scoped. Keep the lane in standby rather than narrowing the corpus to hide the gap.
- L7 witness construction would require host-only witness fabrication. Stop and return to the witness-construction prerequisite.

## Non-goals

- L5 cross-target consistency. That is [`r3-v-l5-corpus-worker.md`](r3-v-l5-corpus-worker.md) and starts after this lane.
- L6 structural-form coverage. That lives in R2 Grounding.
- Cost-lens authoring. Verification may consume cost-lens outputs, but Substrate owns `T-CostLens-Composition`.

## Cross-refs

- Manager: [`r3-verification-manager.md`](r3-verification-manager.md).
- R3 authority: [`docs/r3-structure.md`](../r3-structure.md) `T-Verification-L4-L7-Direct`.
- Design authority: [`docs/design-emission-model.md`](../design-emission-model.md) L4/L7 and Q4.
- Evaluator authority: [`r2-evaluator-manager.md`](r2-evaluator-manager.md).
