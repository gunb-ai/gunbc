# PR #3510 Distinct-Node Zip-Fold Blocked Audit

Date: 2026-05-21
Work item: `node://adhoc-e059ca50-f67`
PR under review: [#3510](https://github.com/gunb-ai/gunbc/pull/3510)
Relevant revert: `678c3db8bfe355b1087edc9626f17b02b5991ec1`

## Summary

sunny-newt-545's final analysis on PR #3510 is correct.

This document is a bounded work-item audit receipt, not a new substrate authority. The live technical authorities remain the inline `.dag` scaffold markers and the ratified task/design rows they cite. This receipt exists only to close the resurrected PR #3510 question and route the follow-up correctly.

The exact structural preservation path is still the identity-MVP scaffold. `src/v4/std/find_witness.dag` dispatches `preservation_rule_exact_structural_equality_zip_fold` to `identity_mvp_preservation_holds` in `src/v4/std/constraint_satisfaction_predicate.dag`. That helper requires:

- `well_formed(source_facts)`
- `well_formed(algebra)`
- `well_formed(candidate)`
- `algebra == source_facts`
- `candidate == source_facts`

So current acceptance is source-identity only. It does not implement distinct-node coincidence.

PR #3510's attempted local fixes failed for the right reason: a local recursive rebuild followed by `==` adds no authority beyond raw equality, while atom-erasure fabricates leaf coincidence without a modeled source/target declaration witness. Exact distinct-node preservation needs the W-T-10 canonical Node grounding / edge-wise zip-fold substrate, or an equivalent modeled source/target coincidence authority. No such alternative authority is present in the current files.

## Evidence

`src/v4/std/find_witness.dag` owns `find_witness` and `verify_witness`, but it does not own leaf coincidence. The exact structural rule currently delegates through `preservation_predicate_holds`:

```text
preservation_rule_exact_structural_equality_zip_fold =>
  identity_mvp_preservation_holds(...)
```

`src/v4/std/constraint_satisfaction_predicate.dag` names the tracked scaffold:

```text
scaffold:identity-mvp-preservation
dissolve-on: each rule's real predicate semantics land
constraint_satisfaction via T-9/T-25-tail
exact_structural_equality_zip_fold via W-T-10 zip-fold predicate algebra
```

That is the decisive split: T-25-tail is relevant to constraint/refinement predicate proving, while exact structural equality is explicitly assigned to W-T-10's zip-fold predicate algebra.

`src/v4/std/coercion.dag` consumes the exact rule only by invoking `find_witness` with `preservation_rule_exact_structural_equality_zip_fold`. It lifts accepted `FindWitnessResult` into `CoercionResult`; it does not add an independent source/target coincidence check. Its `CoercionQuality` still carries `Identity | Exact`, with `Identity` used under the current scaffold until the zip-fold predicate distinguishes the quality.

The design docs agree with this routing:

- `docs/design-v4-compiler-homomorphism.md` describes the coercion fold as a catamorphism that zip-walks two canonical Node groundings.
- The same doc's MVP section places the exact-structural-equality predicate in `std/coercion.dag` / `std/find_witness.dag` and names `W-T-10-impl` as the translate body invoking `find_witness` with that predicate.
- `src/v4/TASKS.md` defines T-25-tail as the refinement predicate prover that erases proven refinements. It does not supply canonical source/target leaf coincidence for exact structural coercion.

## PR #3510 Provenance

PR #3510 tried these shapes:

1. Root kind / arity comparison.
2. Recursive zip-walk with unconditional atom coincidence.
3. `fold_node` canonicalization with atom identity erasure.
4. `fold_node` canonicalization preserving atom identity.

The blocking reviews correctly identified the failure modes:

- Unconditional atom erasure accepts too much: it treats unrelated atoms as coincident without modeled declaration evidence.
- Preserving atom identity restores fail-closed behavior, but then the fold rebuild is behaviorally equivalent to comparing the original Nodes for equality.
- The later distinct-node receipt stopped proving distinct-node behavior because its target fixture used the same leaf identities.

Commit `678c3db8b` removed the invalid helper and misleading receipt, returning PR #3510 to an empty diff against `main`.

## Decision

Disposition: **A — confirm blocked.**

The original distinct-node zip-fold implementation should not proceed inside `find_witness` as a local helper. PR #3510 should be closed or abandoned as an empty/reverted implementation PR.

The unblocking work should be a separate W-T-10 substrate dispatch that models:

- canonical source grounding as the authority for source-side leaves,
- target declared inhabitants / fact bundles as the authority for target-side leaves,
- a typed leaf-coincidence relation over those authorities,
- an edge-wise zip-fold predicate that walks canonical Node groundings and produces a checkable witness or fail-closed diagnostic,
- quality classification that can distinguish `Identity` from `Exact` only after the predicate supplies that evidence.

T-25-tail should not be treated as the unblocker for this PR. It may help prove/refine predicates in the constraint/refinement lane, but the current exact-structural rule explicitly dissolves through W-T-10's zip-fold predicate algebra.

## Recommendation

Close PR #3510 with a comment pointing at this audit. Keep current `find_witness` identity-MVP behavior unchanged until W-T-10 lands the modeled coincidence authority.

Route follow-up as W-T-10 substrate work, not as another PR #3510 implementation attempt.

## Lifecycle

Owner for dissolution: the future W-T-10 canonical Node grounding / edge-wise zip-fold predicate dispatch.

Dissolution trigger: once W-T-10 lands a modeled exact-structural zip-fold predicate with typed source/target leaf-coincidence evidence and updates the canonical task/design authority, this audit should be treated as historical PR disposition only. It must not be used as the active source of truth for `find_witness`, `coercion_fold`, T-25-tail, or W-T-10 scope after that point.
