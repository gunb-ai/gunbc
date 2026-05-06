# R3 TC1 Eta-Equivalence Deeper Structural Verification Analysis

**Status:** **DESIGN** (2026-05-06). Verification Mgr engineering choice among Pattern-A
first-slice paths; **not** yet **ACCEPTED** (Director countersignature still required per
`docs/r3-program-plan.md` §10.3 **Q-PAFS** and Brian directive: PROPOSAL → DESIGN →
ACCEPTED).

**Lifecycle**

| Stage | Meaning |
| --- | --- |
| PROPOSAL | Exploratory surface map only (superseded by this revision). |
| **DESIGN** (here) | Chooses a concrete first-slice shape, defers alternatives, states runtime prereqs and scope deltas. |
| ACCEPTED | Director/Brian countersign; authorizes implementation dispatch (V1 TC1 executable slice + Evaluator **E6-G1.a** lens-fold first slice in lockstep). |

TC1 deferred fixture remains **as-is** until ACCEPTED + routed implementation; this brief
still does **not** authorize substrate, runner, or fixture edits by itself.

**Owning manager:** R3 Verification Manager, absorbed formal-grounding responsibility.

## Scope

TC1 is the substrate-lens eta-equivalence claim landed as an author-now-fire-later fixture:

`src/v3/compiler/tests/fixtures/tc1_substrate_lens_eta_equivalence_deferred.dag`

The live carrier is `SubstrateResearchDeferredClaim`, runner-valid only in that fixture. This carrier is not a generic staging predicate and must not be widened from this analysis.

## What TC1 Asserts

The fixture states:

`For any Lens<C>, applying it to f and to eta-expanded lambda x.apply(f, [x]) yields identical DimensionReport<C>.`

In substrate-lens terms, the theorem is not merely "two functions compute the same value." It says lens observation is invariant under eta expansion:

1. the reflected `Dag` / `Behavior` structure for a function and its eta-expanded wrapper may differ syntactically;
2. `fold_lens<C>` must nevertheless produce the same `DimensionReport<C>`;
3. equality includes both success and failure shape, because `DimensionReport<C>` is `DimensionOk { composed, witnesses } | DimensionFail { violations, witnesses }`.

This connects directly to `docs/design-lens-framework.md`: `Lens<C>` has `read`, `sequential`, `branch`, `iterate`, and `validate`; the fold returns the existing `DimensionReport<C>` carrier. TC1 therefore exercises the generic fold and its result carrier, not a single lens instance.

## Current Carrier Semantics

`SubstrateResearchDeferredClaim` currently checks only deferral shape:

- the claim lives in `tc1_substrate_lens_eta_equivalence_deferred.dag`;
- its three `DeclarationRef`s resolve to fixture-local markers;
- those markers inhabit `Tc1ResearchGateMarker`, `SubstrateLensPrimitiveTargetLaneMarker`, and `LambdaCalculusGroundingAuthorityDoc`.

That is a fail-closed staging receipt. It does not apply a lens, construct eta pairs, run `fold_lens<C>`, or compare `DimensionReport<C>`.

Clean outcome: TC1 strict-fire is not fully expressible by the existing deferred carrier alone. Activation needs additional structural work after T-Substrate-Lens-Primitive and lens producer retirement land.

## Strict-Fire Extension Surface

Likely strict form:

`tc1_substrate_lens_eta_equivalence_strict_fire`

Minimum semantics:

1. construct or reference a pair of eta-equivalent `.dag` terms: `f` and `lambda x.apply(f, [x])`;
2. quantify over `Lens<C>` instances, or at least over a Director-ratified representative set if universal quantification is deferred;
3. run `fold_lens<C>` on both reflected programs;
4. compare the two `DimensionReport<C>` values structurally.

The existing `LensOutputEquals` predicate is too narrow as-is. It applies one named lens to one named input and compares to one expected declaration. TC1 needs pairwise equality of two fold outputs, and probably polymorphic `C` / lens-instance coverage.

Potential substrate paths, for Director/Substrate ratification:

- add a TC1-specific predicate, e.g. `EtaEquivalentLensOutputsEqual { lens_ref, subject_ref, eta_ref }`;
- generalize `LensOutputEquals` into a binary structural equality predicate over two lens applications;
- model eta-equivalence as a substrate relation over reflected functions, then reuse a generic `DifferentialEquals`-style output comparison.

This analysis does not choose among those paths.

## Lens-Framework Anchors

TC1 shares ground with these lens-framework checks:

- I1/I2: `Lens<C>` declaration and generic `fold_lens<C>` machinery. Without this, TC1 has no strict evaluator.
- I4: worked lens fixtures prove ordinary `DimensionReport<C>` production for concrete instances.
- I8/I9: fail-closed read and aggregate-validate tests matter because eta-equivalence must preserve failures as well as successes; a missing fact or aggregate diagnostic cannot be fabricated away under eta expansion.

TC1 is deeper than I4/I9: those are instance examples; TC1 asks for invariance under a semantic program transformation.

## Carrier Scope Analysis

Do not widen `SubstrateResearchDeferredClaim`. It is fixture-scoped by runner code and `r2-closure-ledger.md`; widening would weaken the fail-closed staging boundary.

If strict-fire needs a generalized carrier for eta-equivalence, that is a substrate-fact-introduction candidate under `INVARIANTS.md` P1. Candidate shape should be reviewed by Substrate Manager / Director because it would add either:

- a new `TestPredicate` variant for eta-equivalence;
- a reusable binary lens-output equality predicate;
- or a substrate relation for eta-equivalent reflected functions.

## Cross-Claim Coordination

Shared with TC2:

- both likely compare `DimensionReport<C>` outputs;
- TC2 compares strategy/order outputs, while TC1 compares eta-related program forms;
- a generic structural equality surface for `DimensionReport<C>` could serve both.

Shared with TC3:

- all three depend on T-Substrate-Lens-Primitive;
- TC3 may need an evaluation-step witness shape; TC1 may need a fold-output witness shape. If those converge on a common proof-result carrier, coordinate before introducing parallel predicates.

## Open Questions

- Is TC1 universal over all `Lens<C>` instances, or strict-fire over the migrated built-in lens set plus user-authored lens fixture?
- Does eta-equivalence live as a function/program relation in substrate, or only inside a verification predicate?
- Should equality include witness-list ordering, or compare `DimensionReport<C>` modulo semantically irrelevant witness ordering?
- Does lens producer retirement provide enough reflection authority to construct eta-expanded forms structurally, or is a lambda-calculus grounding carrier still needed?

## Non-Goals

- No fixture activation today.
- No runner validity widening for `SubstrateResearchDeferredClaim`.
- No edits to `r2-closure-ledger.md`, `r3-v-formal-grounding-tc-bundle.md`, `r3-verification-manager.md`, or `r3-pb-t-fixedpoint-worker.md`.
