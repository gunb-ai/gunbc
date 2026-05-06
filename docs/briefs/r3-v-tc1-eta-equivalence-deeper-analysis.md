# R3 TC1 Eta-Equivalence Deeper Structural Verification Analysis

**Status:** **ACCEPTED** (2026-05-06 — Brian Director countersign: Path **A** / **G1.a**).
**Canonical ratification:** [`docs/r3-program-plan.md`](../r3-program-plan.md) §10.3 — rows
**Q-PAFS**, **Q-Pattern-A-First-Slice-Subscope**, and **Q-EVAL-Lens-Fold-First-Slice**
(**ACCEPTED** 2026-05-06, Path **A**). **Sole authority** = the committed §10.3 table at
repo `HEAD` (not any PR description). PR [#1824](https://github.com/gunb-ai/gunbc/pull/1824)
is the **merge record** that landed the countersign row text on `main`; later PRs only
edit surrounding briefs/citations. **Path
check:** ratification lives in the committed file `docs/r3-program-plan.md`; from this
brief the relative link `../r3-program-plan.md` resolves to that path (same `docs/`
directory as `docs/briefs/`). This brief is the **engineering scope narrative** aligned
to that row. **Worker dispatch:**
[`r3-v-pattern-a-tc1-v1-worker.md`](r3-v-pattern-a-tc1-v1-worker.md).

**Lifecycle**

| Stage | Meaning |
| --- | --- |
| PROPOSAL | Exploratory surface map only (superseded). |
| DESIGN | Path A / B / C resolution recorded 2026-05-06 (superseded as policy stage by ACCEPTED). |
| **ACCEPTED** (here) | Path **A** ratified — implementation authorized in lockstep with Evaluator **E3** (**E6-G1.a**). |

This brief binds **Verification policy scope**; executable routing + carrier shapes land via coordinated PRs (see worker brief). **Do not** treat ACCEPTED as permission to widen `SubstrateResearchDeferredClaim` or activate strict-fire fixtures without those PRs.

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

**DESIGN resolution (which slice first):** concrete choice lives in **§DESIGN — Q-PAFS
first executable slice** below. Predicate/carrier *shape* inside strict-fire remains
Substrate-owned STOP+PING until ACCEPTED routing.

## DESIGN — Q-PAFS first executable slice (Pattern-A / TC1)

This section satisfies the Verification Mgr obligation called out in
`r3-program-plan.md` §10.3 **Q-PAFS**: PM default ("TC1 first") is not engineering
sign-off; **DESIGN** records the explicit tradeoff and recommended path.

### Path A — TC1 **static representative** via **E6-G1.a** (**recommended**)

**Shape:** Ratify a **finite**, Director-visible **representative set** (not universal
quantification over all `Lens<C>` for the first executable slice): fixed eta pair of
programs (`f` and `lambda x.apply(f, [x])`) plus a named list of lens instances (or
substrate-approved typed lens handles) that the evaluator can fold with **existing**
`fold_lens<C>` / `DimensionReport<C>` machinery.

**Runtime prereqs (Evaluator / E6, aligned with Evaluator Mgr F4 + program-plan
Q-EVAL-Lens-Fold-First-Slice / Q-Pattern-A-First-Slice-Subscope):**

- **E6-G1.a** static lens fold: two **structurally named** programs in `.dag` (or
  equivalent declared scope) each produce a **typed** `DimensionReport<C>` through the
  same evaluator-owned fold authority—no X1.b generic transform-dispatch requirement for
  this slice.
- **T-Substrate-Lens-Primitive** + lens producer retirement progress assumed per existing
  lane dependencies: TC1 strict-fire still does **not** rest on widening
  `SubstrateResearchDeferredClaim` (see **Carrier Scope Analysis**).
- **Pattern-A consumer envelope** unchanged: `BinaryDimensionReportEquals` over the two
  reports once producers exist (`docs/briefs/r3-v-pattern-a-coverage-rollup.md`).

**Substrate vs evaluator scope:** Substrate names the **representative lens set** and
any eta-pair **declaration refs**; Evaluator proves it can **execute** the fold twice and
lift reports without fixture-local producer identity or string bypass (Evaluator #1131
safe contract).

**Engineering justification:** Smallest executable surface that honors the PM default
(TC1 before TC2/TC3/RustDagIsomorphism ordering) **without** blocking on **E6-G1.b** /
**X1.b** S1/S3 generic dispatch. Unblocks **V1** “TC1 first slice” and **Evaluator E3**
(E6-G1.a) on the **same** Director **ACCEPTED** gate, per PM coordination note.

### Path B — TC1 **generic** via **E6-G1.b / X1.b** (**defer**)

**Shape:** Universal or parametric coverage over `Lens<C>` / programs requiring **X1.b**
transform dispatch and **G1.b** generic lens-fold substrate not required for Path A.

**Runtime prereqs:** Substrate **X1.b** S1/S3 + generic dispatch path; substantially
larger than G1.a; sequenced **after** Path A strict-fire is green or explicitly reprioritized
by Director.

**Scope delta vs A:** Shifts primary risk from “representative selection” to “generic
dispatch + substrate carrier completion.” **Not** chosen for first executable slice.

### Path C — **RustDagIsomorphism** first (**alternate, not recommended for Q-PAFS default**)

**Shape:** Prioritize shape-report producers for `DimensionReport<Dag>` before TC1 eta
pair (`docs/briefs/r3-v-pattern-a-coverage-rollup.md` §4 row **RustDagIsomorphism**).

**Runtime prereqs:** Still requires generic typed `DimensionReport<C>` production and two
structural shape reports; **fewer eta-specific** obligations than TC1.

**Why not for Q-PAFS default:** Conflicts with stated PM/Brian **Pattern-A ordering**
(TC1 first) unless Director explicitly reorders. Remains a **credible fallback** if Director
rules TC1 eta unblockers dominate calendar—would require explicit scope edit, not silent
Verification drift.

### DESIGN verdict

| Question | Decision |
| --- | --- |
| First Pattern-A executable slice for TC1? | **Path A** — static representative **E6-G1.a**. |
| Generic TC1 (Path B)? | **Deferred** after Path A closes or Director reprioritizes. |
| RustDagIsomorphism before TC1? | **Not** under Q-PAFS default; **Path C** only with Director reorder. |
| Universal vs representative for first fire? | **Representative finite set** for slice one; universal quantification is a **later** ratchet question (see Open Questions). |

**ACCEPTED (2026-05-06):** Path **A** countersigned by Brian ("approved path A countersign");
program-plan §10.3 rows updated; any deviation from Path A (e.g., Path C first) is a new
Director/Brian scope calibration — not silent Verification drift.

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

- **Representative set contents:** which built-in / staged lens instances and which
  single eta pair land in slice-one (Substrate + Director visibility)—DESIGN defers
  enumeration to implementation routing post-ACCEPTED.
- **Universal quantification:** remains **out of scope** for first executable slice (Path
  A); revisit after Path A strict-fire or if Director mandates broader coverage.
- Does eta-equivalence live as a function/program relation in substrate, or only inside a verification predicate?
- Should equality include witness-list ordering, or compare `DimensionReport<C>` modulo semantically irrelevant witness ordering?
- Does lens producer retirement provide enough reflection authority to construct eta-expanded forms structurally, or is a lambda-calculus grounding carrier still needed?

## Non-Goals

- No fixture activation or runner widening until **ACCEPTED** + implementation dispatch.
- No runner validity widening for `SubstrateResearchDeferredClaim`.
- No edits to `r2-closure-ledger.md`, `r3-v-formal-grounding-tc-bundle.md`, `r3-verification-manager.md`, or `r3-pb-t-fixedpoint-worker.md` from this DESIGN revision alone.
