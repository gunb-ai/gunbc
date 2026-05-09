# PR-E E9 - Cross-target harness consumption readiness audit

**Status:** READINESS AUDIT - docs-only. This note does not authorize runner
implementation, new `TestPredicate` variants, substrate changes, target
enumeration, or L5 corpus execution.

**Parent authority:** [`r3-evaluator-dispatch.md`](r3-evaluator-dispatch.md)
E9 - Cross-Target Harness Consumption.

**Design lock:** [`docs/design-cross-target-equivalence.md`](../design-cross-target-equivalence.md)
defines semantic observations, corpus policy, oracle validity, float policy, and
side-effect normalization for strict L5 evidence.

**PR-D primitive surface:** [`r2-pr-d-cross-target-equivalence-harness-primitives.md`](r2-pr-d-cross-target-equivalence-harness-primitives.md)
records slice 0 (`Compiles`) and slice 1 (`DifferentialEquals`) as landed, and
keeps strict `ForAllTargets` receipts gated.

**Post-W1 delta:** [`r3-pr-e9-post-w1-lane1-consumption-readiness.md`](r3-pr-e9-post-w1-lane1-consumption-readiness.md)
records the #1499 state: the first post-W1 unblocked consumption slice is
Lane 1 / L4 direct Rust-Int `DifferentialEquals` evidence, not E9/L5
`ForAllTargets` execution.

## Current State

PR-D provides a structural import surface, not an executable cross-target
harness:

- `evaluator_cross_target_equivalence_harness_primitives_landed` exists as the
  stable R2 primitive gate in
  `src/v3/compiler/tests/fixtures/r2_evaluator_cross_target_equivalence_harness_primitives.dag`.
- `evaluator_cross_target_equivalence_harness_primitives_differential_scaffold`
  exercises the existing `DifferentialEquals` predicate on a fixture-local
  subject/oracle pair. This proves the fixture home and runner-visible shape; it
  does not prove multi-target emission or L5 corpus behavior.
- W1 `DifferentialEquals(rust_emit_output, dag_eval_output, ProgramOutputBind)`
  is now wired for the current Rust / Int slice. This narrows the Lane 1
  blocker, but it remains a transitional single-target producer path and does
  not supply LanguageSpec, Shape A, or L5 observation authority.
- `src/v3/compiler/tests/fixtures/r3_verification_l5_corpus.dag` already contains
  an R3 L5 skeleton using the existing `ForAllTargets` scaffold. Its file-level
  comment records the current runner state: `ForAllTargets` is not wired in the
  Rust runner.
- `src/v3/std/verification.dag` declares `ForAllTargets` as a scaffold with raw
  command / args / expected-exit fields. The design lock allows consumers to use
  that surface only under scaffold discipline; it is not the final typed
  observation model.

## Gates Before E9 Can Execute

E9 may move from readiness audit to implementation only when all of these gates
are cited together in the implementation PR:

| Gate | Required evidence |
|---|---|
| **LanguageSpec readiness** | Per-target primitive realization and typed capability edges are live, so a target run can be related to a declared language/target fact instead of a command string. |
| **All Shape A targets grounded** | Rust, Python, and Go Shape A emit/run paths are grounded for the same `Dag` program. Partial target coverage is a scaffold row, not strict L5 evidence. |
| **L4/L7 direct corpus seed** | `T-Verification-L4-L7-Direct` has produced the corpus rows L5 is meant to compare across targets. L5 consumes that corpus; it does not invent a parallel one here. |
| **Typed structural observation carrier** | Target results normalize into the `Value` / semantic observation domain from the cross-target design lock. Raw stdout bytes, emitted source, and diagnostic text are not equality authority. |
| **Runner capability dissolution path** | Any use of existing `ForAllTargets` command fields names the typed target capability / observation facts that will retire the raw command scaffold. |
| **Director-approved fixture home** | If PR-D slice 2 wires a `ForAllTargets` claim, it uses the existing PR-D/L5 fixture path approved for that claim instead of creating a parallel cross-target suite. |

If any gate is missing, the correct E9 output is a blocked row or docs update,
not runner code.

## Allowed Next Slices

Before the gates are live:

- Keep this readiness note and the cadence matrix current.
- Consume #1499 only through Lane 1 / L4 direct Rust-Int
  `DifferentialEquals` rows that stay inside the current W1 producer and
  evaluator surface.
- Add blocked-row wording for specific L5 skeletons only when the blocker is
  exact and traceable to the table above.
- Tighten cross-links from R3 Verification planning to the PR-D fixture and the
  cross-target design lock.

After the gates are live:

- Add a PR-D slice 2 receipt using the existing `ForAllTargets` predicate only if
  it compares typed semantic observations per the design lock.
- Keep the initial implementation to a single claim / corpus row so review can
  validate target selection, observation normalization, and fail-closed behavior.
- Treat any need for a new predicate, target enum, observation carrier, float
  tolerance policy, or effect policy as a P1 substrate/design escalation before
  coding.

## Explicit Non-Goals

- No L5 corpus execution in this audit.
- No target enumeration in Rust.
- No new `TestPredicate` variants.
- No stdout/string/byte equality as a semantic comparison shortcut.
- No new runtime `Value` inhabitants or evaluator body semantics.
- No changes to `src/v3/compiler/src/test_runner.rs`.

## Readiness Verdict

E9 is **not implementation-ready** on current main. The strict cross-target
receipt remains gated on LanguageSpec, all Shape A target grounding, corpus-home
approval, and a typed structural observation path. #1499 makes a narrow
Lane 1 / L4 Rust-Int consumption slice available, but that is not
`ForAllTargets` execution and must not be generalized into L5 target
enumeration or stdout equality.
