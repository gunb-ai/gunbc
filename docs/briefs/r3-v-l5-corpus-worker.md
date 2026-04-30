# R3 T-Verification-L5-Corpus Worker Brief

**Status:** STANDBY. Sequentially gated on T-Verification-L4-L7-Direct.

**Owning manager:** R3 Verification Manager.

## Scope

Build the L5 corpus-driven cross-target equivalence suite. L5 checks that Shape A emitted targets produce algebraically equivalent runtime behavior for the same `.dag` program corpus.

This lane starts from the L4 corpus, then layers cross-target comparison on top. It does not define corpus completeness from scratch and does not own target grounding.

## Dispatch Preconditions

Dispatch only when:

1. `T-Verification-L4-L7-Direct` has landed a checked-in L4 corpus manifest.
2. Shape A target grounding is closed for the dispatch-time target set. The floor is Rust + Python per the R3 worker-dispatch precondition; include Go only if the closure ledger reports Go closed at dispatch.
3. The cross-target harness primitive supports comparing emitted outputs across the chosen target set, not only the current `DifferentialEquals` cost-lineage fixture.
4. Target execution environments are hermetic and deterministic enough for CI.

## Deliverables

- A frozen L5 target set derived from the closure ledger at dispatch.
- Cross-target execution harness over the L4 corpus.
- `TestClaim` rows named `l5_cross_target_consistency_holds_per_corpus_program`.
- Mismatch reports that identify the program, target pair, normalized observable output, and the predicate that failed.

## STOP Conditions

- The target set is hand-maintained independently of the closure ledger.
- A target runtime requires non-hermetic host state or network access for the verification corpus.
- A mismatch is papered over with target-specific normalization not declared by the substrate/language spec.
- The worker needs to change target grounding facts to make L5 pass; route that to the owning Grounding lane.

## Non-Goals

- L4 emit/eval direct comparison.
- L6 structural form coverage.
- Adding new Shape A targets beyond the closed grounding ledger.

## Acceptance

The lane closes when every materialized program in the inherited L4 corpus passes cross-target equivalence for every pair in the frozen dispatch-time target set, and the result is represented as checked-in `.dag` `TestClaim` data plus runner support.
