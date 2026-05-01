# R3 Lane 3 — T-Free-Consequences-Demonstration Worker Brief

**Status:** PROPOSAL — standby. Brief authoring is allowed now; implementation
dispatch waits on `Lens<C>` substrate primitive landing plus R2-Evaluator
witness construction. This brief does not authorize substrate edits.

**Parent:** [`r3-verification-manager.md`](r3-verification-manager.md) — Lane 3
owned by R3 Verification Manager.

**Authority:** [`docs/r3-structure.md`](../r3-structure.md) §"Lane structure"
row **T-Free-Consequences-Demonstration** (NEW 2026-04-30). `r3-structure.md`
§"Manager structure" Item 2 names Verification Manager owner of 3 lanes + 1
ledger gate; TC1/TC2/TC3 remains absorbed cross-cutting responsibility, not a
fourth lane.

## Goal

Operationalize the user directive: "what guarantees does the compiler ACTUALLY
provide - i expect a small doc/testcases." This lane is deliberately small and
testcase-driven. It demonstrates which consequences fall out of the substrate
and lens framework once the required producers exist, without rebranding
heuristics as guarantees.

## Deliverables

1. **`docs/design-free-consequences.md`** — 1-2 page per-consequence guarantee
   analysis grounded in the 5 substrate behaviors and `Lens<C>` framework.
   This analysis can start before implementation dispatch because it is
   research/design-tier scoping.
2. **10-gate `.dag` `TestClaim` suite** — strict proof-of-claim testcases.
   Authoring waits on `Lens<C>` substrate primitive + R2-Evaluator witness
   construction; cost-related gates also consume T-CostLens-Composition.

## Lens Algebra Framing

- **Auto-parallelism** = `Lens<Bind-Independence>·Lens<Effect-Commutativity>·Lens<Cost>`.
  Independent binds may emit parallel structure only when the independence
  lens witnesses the dependency relation, effect/commutativity evidence proves
  the effects commute or are absent, and the cost lens makes parallelization
  meaningful. `docs/v3-modeling-analysis.md` §"Parallelism safety" makes the
  effect lens load-bearing, and `src/v3/std/effects.dag` keeps concurrent-write
  commutativity fail-closed until a value/merge witness exists.
- **Auto-memoization** = `Lens<Purity>·Lens<Cost>`. Repeated pure work may be
  cached when purity and cost jointly witness benefit. Runtime memoization of
  lens fold is one instance of this same primitive, not a separate framework
  feature.
- **Cross-target optimization** = `Lens<Cost>·LanguageSpec`. The claim is not
  "each target emits identical syntax"; it is that target-specific choices
  preserve the same cost/semantic relation under the language-spec facts.
- **Space-bound CX** = status/reference section only. Cite the closed R1 CX
  baseline where relevant, but explicitly preserve the active analysis state:
  `docs/v3-modeling-analysis.md` still marks space bound proofs **NOT
  STARTED** because the space lens is not modeled. Do not present space-bound
  proofs as a current compiler guarantee in this lane.

## Loop-Iteration Parallelism

Director-ratified design call: sequential default, opt-in only through
`Lens<Iteration-Independence>` plus the same effect/commutativity evidence
required for bind parallelism. There is no heuristic loop parallelization path.
The shape mirrors `Lens<Bind-Independence>`: the compiler may emit parallel
loop-iteration structure only when the lens produces a witness that iterations
are independent under the relevant substrate facts and the effect lens proves
parallel execution is observationally safe.

This design call anchors three gates:

- `auto_loop_parallelism_provable_independence_emits_parallel`
- `auto_loop_parallelism_unproven_falls_back_sequential`
- `auto_loop_parallelism_dependence_emits_sequential`

## Memoization Framing

Memoization is a free consequence only when the `Lens.read` purity invariant
and cost witness make repeated work safe and useful to cache. Treat lens-fold
caching as one instance of the same auto-memoization primitive. Do not create a
parallel "lens framework caching" lane or a bespoke cache policy gate.

This framing anchors:

- `auto_memoization_repeated_pure_call_cached`
- `auto_memoization_no_caching_for_one_shot`

## 10-Gate TestClaim Suite

Preserve these gate names from `r3-structure.md`, with gate 10 following the
`cross_target_optimization_cost_structurally_derived` rename queued in PR #1341
until that structure-doc update merges:

1. `auto_parallelism_independent_binds_emit_parallel`
2. `auto_parallelism_dependent_binds_emit_sequential`
3. `auto_parallelism_branch_arms_serialize`
4. `auto_loop_parallelism_provable_independence_emits_parallel`
5. `auto_loop_parallelism_unproven_falls_back_sequential`
6. `auto_loop_parallelism_dependence_emits_sequential`
7. `auto_memoization_repeated_pure_call_cached`
8. `auto_memoization_no_caching_for_one_shot`
9. `cross_target_optimization_constant_fold_consistent`
10. `cross_target_optimization_cost_structurally_derived`

## Standby State

Design authoring can start now:

- define the consequence vocabulary;
- map each consequence to its lens algebra inputs;
- identify fixture shapes and expected diagnostics;
- cite which existing R1/R2 evidence is consumed rather than re-proved.

TestClaim authoring waits:

- `Lens<C>` substrate primitive lands;
- R2-Evaluator can construct witnesses for the relevant lens outputs;
- T-CostLens-Composition lands enough cost facts for the cost-related gates.

## Acceptance

Lane 3 closes only when both deliverables are complete:

- `docs/design-free-consequences.md` landed with the per-consequence guarantee
  analysis; and
- all 10 named `.dag` gates pass.

Partial design analysis, seed fixtures, or a subset of green gates do not close
the lane.

## Discipline

- No substrate edits in this lane brief.
- Substrate-fact introduction routes through `INVARIANTS.md` §P1 and the
  Substrate Manager.
- Cross-validate shared substrate questions with the active substrate queue:
  `Lens<DagShapeReport>` producer + `BinaryDimensionReportEquals` consumer,
  and BridgeLedger pair work.
- If `Lens<Bind-Independence>`, `Lens<Iteration-Independence>`,
  `Lens<Effect-Commutativity>`, `Lens<Purity>`, or `Lens<Cost>` reveal shared
  `DimensionReport<C>` framing questions with TC1/TC2/TC3 unified-predicate
  authoring, surface that to the Verification Manager / Director before
  proposing a substrate shape.

## Explicitly Out Of Scope

- Authoring `docs/design-free-consequences.md` in this PR.
- Authoring or wiring the 10 `.dag` gates before prerequisites land.
- Adding new `TestPredicate` variants or lens carriers from this brief.
- Reclassifying TC1/TC2/TC3 as a fourth Verification lane.
