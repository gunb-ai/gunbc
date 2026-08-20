# `v2.lens.coverage` declares a failure-mode taxonomy with no detector

Found 2026-08-20 by `silent-bear-842` while measuring the cost of a proposed lens, during the
fabric CI binding work (#8576). **Recorded, deliberately not fixed** — it is not the CI/revenue
lane's work, and the taxonomy exists while the detector and observation plumbing do not, which is
the expensive half. This document is the deliverable; it exists so the finding is not lost and so
whoever picks it up starts from measurements rather than from a grep.

> **Both patterns are present in the same repo, and the difference between them is whether
> anything folds over the corpus.** `v2.lens.coverage` is a taxonomy with no detector;
> `gunbc.lifecycle_survivor_scan` derives its population from the corpus and gates it at zero.
> The second is the model to copy, not an exception to note.

## The constraint that must come first: provenance, not just observation

A detector for this class is **not** variant-observation coverage. It is
**provenance-qualified** variant-observation coverage, and the difference is not a detail —
it is the difference between a detector that works and one that launders the defect it was
built for.

Worked example, measured. `std.dissolution.DissolutionStatus` has a `DissolutionFired` arm.
All 14 rows of `v2.lens.inert_carrier`'s roster are `UnboundDissolution`, so **no production
row can ever reach `DissolutionFired`.** Yet the variant *is* constructed during the floor
run — `dag/test/claim/annotation_carrier_witness_test`
`bound_condition_pends_until_its_declaration_appears` builds a `bound_dissolution` itself,
feeds it in, and asserts on the fired arm.

So a naive "was this variant ever constructed?" lens reports `DissolutionFired` as **covered**
while the production population that can reach it is **empty**. That is the `VacuousArm` shape
one level down, inside the detector meant to catch `VacuousArm`, and it would have shipped
invisibly.

**Requirement:** the fold must record **how** a variant was reached — through a production
path, or constructed directly by the witness — not merely *that* it was reached. A test that
manufactures its own input is not evidence that anything reaches the variant.

## The finding

`v2.lens.coverage` declares `CoverageDefectKey`, a 13-variant vocabulary of DESIGN's
failure modes:

    DiscriminantPredicate, DegenerateType, HollowType, CarrierClone, Catamorphism,
    TemplateHole, OffSubstrateFact, WrongHome, VacuousArm, CanonicalCarrier,
    PlausibleFallback, ParallelAuthority, SkeletonCollapse

Each has one `data` acceptance row. **No fold anywhere produces a list of *detected*
defect keys from the corpus.** The module's generic set-difference helper
`missing_coverage` *is* consumed — by `v2.lens.mock_totality`, for published-vs-handled
mock totality, an unrelated concern.

Everything that consumes `VacuousArm`:

| consumer | what it does |
| --- | --- |
| `coverage_defect_vacuous_arm` (`v2.lens.coverage`) | a `data` row |
| `gunbc.doc_graph_roots` | a documentary binding |
| `near_miss_vacuous_not_parallel_claim_holds` | compares two constants |

## The receipt that makes it airtight

`v2.test.lens_coverage.near_miss_vacuous_not_parallel` declares

    data near_miss_vacuous_node: Node = Node {
      kind: TypeNode { connective: Atom { identity: ^near_miss_vacuous_symbol } },
      children: [],
      occurrence_id: SyntheticOccurrence
    }

**and the test never reads it.** Its whole body is:

    coverage_defect_vacuous_arm != coverage_defect_parallel_authority

A `Node` authored as a detector's input, beside a witness that only compares two
hand-authored constants. Intent is normally unobservable; here it is a **receipt of an
abandoned build** rather than an inference about one — someone modeled the shape of the
check and stopped.

Four sibling witnesses have the same shape (`hollow != degenerate`, etc.). Each cannot
fail unless someone edits a constant — the change-detector shape §5 names outright:
automating the literal's update collapses the assertion to `measure() == measure()`.

## Why the charitable reading fails

A fair reading is that these witnesses pin the keys as genuinely distinct *concepts*
rather than §3 nicknames — a real concern in this repo. It does not survive its own
test: comparing two constants cannot establish conceptual distinctness. What establishes
it is **each key having a distinct consumer that fires on different corpora**. So the
honest repair is not better witnesses; it is the detector.

## Rung

**Mitigatable at best, and arguably below it** — the module's *name* asserts coverage
while nothing detects, which is §6's "coverage by illusion" tier: the machinery exists
and nothing gates on it. This is itself an instance of the class the taxonomy names —
a richer name standing where a structural guarantee was needed, in the module whose job
is detecting exactly that.

## What a detector would need

Measured, not guessed. The taxonomy and acceptance rows already exist; **the detector and
the observation plumbing do not, and that is the expensive half.**

For `VacuousArm` specifically, the decidable form is *not* "this arm is unfireable"
(undecidable — a claim about all possible fixtures) but:

> **no witness in the floor run ever constructed this refusal variant.**

Coproduct-variant observation coverage — decidable by execution, no intent inference.
It must report **untested**, never **unfireable**; conflating them is ⊥-as-answer vs
⊥-as-ignorance (the empty-observation narrow) inside the mechanism built to avoid it.
That requires the floor run to record constructed variants, i.e. substrate
instrumentation rather than a pure `Node` reader.

## The method lesson, which generalises furthest

**Name plus green witness is the evidence shape that reads as done.** To establish a
class is covered, do not look for the *vocabulary* — look for **the fold that produces a
finding from the corpus**. Grep finds taxonomies; only a fold produces detections. A
13-variant enum with six hits and a passing test is exactly what full coverage and zero
coverage both look like from a grep.

## What is NOT claimed

- **Unbound is not the corpus default.** 14-of-14 holds for the inert-carrier roster
  specifically. `BoundDissolution` is used in production rows in `extdeps/dhcp/v4`,
  `extdeps/network/ipv6`, and `gunbc/self_host_promotion_obligations`. That roster is an
  outlier, and whether `unbound_dissolution` functions there as the path of least resistance
  rather than as the honest escape hatch its name suggests is an **open question, not a
  finding**.
- **The enforcement layer is not uniformly hollow.** See the opening quote.
- **No dissolution condition fired.** An earlier reading held that
  `inert_carrier_row_dissolve` fired mechanically when `Offer` gained a consumer. It cannot:
  `UnboundDissolution` carries only a `NonEmptyStr`, `dissolution_status` returns
  `DissolutionUnbound` for it, and `DissolutionFired` is reachable only from
  `BoundDissolution`. What deleted the row was `count_stale_roster`, a separate check whose
  behaviour the prose happened to describe correctly — which is exactly why it read as a
  trigger firing. `DissolutionTrigger`'s single form, `DeclarationAppears { ref }`, was the
  precise shape that row needed and was passed over for a string.

## The prior art to copy

`gunbc.lifecycle_survivor_scan` derives the in-class population from the corpus and gates it
at zero. Its sibling `gunbc.dissolution_migration_census` carries the lesson in its own
header: *"a hand-authored roster cannot ground its own completeness; it answers only what
someone wrote down, which DESIGN §5 rejects as an oracle."* That census once claimed
completeness falsely **in both directions**, and nineteen in-class survivors existed which it
had never seen — found by measurement and **migrated rather than reclassified**.
