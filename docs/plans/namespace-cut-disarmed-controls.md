# The cut stripped imports from the fixtures whose subject IS imports

**Found 2026-08-17, integrating main into `integration/namespace-cut`.** A
peer's finding (tidy-pike-117) generalised the mechanism; this note records the
population and one fully worked instance.

## The mechanism, stated first

The bulk import strip treated the corpus uniformly. But a FIXTURE is not
ordinary corpus — it is a deliberately authored specimen whose value is the
distinction it draws. Stripping an import from a fixture whose subject is
import-mediated name binding does not migrate the specimen; it can delete the
distinction the specimen exists to draw.

That matters more here than anywhere else in the tree, because the area the cut
changes — name binding — is exactly the area those fixtures control.

## Population

48 fixture specimens carried `import` lines on main. All now carry zero:

    dag/test/fixture      24 files
    src/v2/test/fixture   24 files

The subset whose SUBJECT is name binding, and therefore the at-risk set:

    cross_shard_seam/importer.dag                  (named for the role)
    module_graph_edge_source/consumer_imported.dag (the import arm of a
                                                    two-producer union control)
    module_graph_edge_source/provider_imported.dag
    leaf_key_collision/module_a.dag, module_b.dag  (+ cross_kind_* variants)
    decl_facts_reflection/ambiguous_specimen.dag
    decl_facts_reflection/ambiguous_b_specimen.dag
    record_construction_census/homonym.dag
    record_construction_census/qualified.dag

Wider context, not the same claim: ~1,100 files under `dag/test/claim` and
`src/v2/test/claim` also lost imports. Most of those are incidental dependency
declarations. The fixtures are called out separately because a specimen's
imports are frequently the specimen.

## Worked instance 1 — a two-producer control that can no longer discriminate

`dag/test/claim/module_graph_edge_source_witness_test.dag` exists because
`dependency_resolution_facts_live` unions an explicit-import producer with a
reference-derived producer, and a call site that passed the SAME producer into
both arms kept compiling and silently dropped every reference-only dependency
(review 50749). Its own note says a unit test of the fold would not catch that,
so it points the production function at a four-module fixture where one provider
is reachable ONLY by import and another ONLY by bare reference.

On this branch the fixture has no imports. Its own note also records that the
two producers are DISJOINT BY FILE — "reference_resolution_facts emits nothing
for a file that still carries import lines" — so with the import lines gone the
reference producer now serves both providers.

The witness therefore no longer distinguishes "both arms wired" from "reference
arm wired twice", which is the exact regression it was built to catch. Whether
it currently reds or greens is not the point: EITHER WAY it has stopped being
the control it was authored to be.

## Worked instance 2 — a bare annotation with two declarers, in the fixture for
## exactly that question

`record_construction_census/qualified.dag` is the positive control for
"a qualified construction cannot hide from a spelling census". Its BODY survived
the strip intact, because its distinguishing property is the full dotted spelling
in the construction expression:

    test.fixture.record_construction_census.specimens.CensusProbeSubject { weight: 3 }

But the file also lost `import test.fixture.record_construction_census.specimens
{ CensusProbeSubject }`, and its signature is still bare:

    fn census_probe_constructs_subject_qualified() -> CensusProbeSubject

Post-cut that spelling has TWO declarers in the pool, because the sibling
`homonym.dag` deliberately declares its own `CensusProbeSubject` — that is the
whole point of the homonym specimen. The binding-risk census names it exactly:

    file:        dag/test/fixture/record_construction_census/qualified.dag
    name:        CensusProbeSubject
    import_said: test.fixture.record_construction_census.specimens
    declarers:   [ ...census.homonym, ...census.specimens ]

So the fixture built to prove a census can tell two same-spelled declarations
apart now contains a bare annotation that cannot tell them apart. This is the
cleanest available specimen of the cut's central defect, and it is sitting inside
the control for that defect.

## Why this compounds the denominator problem

`namespace-cut-unqualified-reference-population.md` records that the diagnostic
count only ever saw ambiguous bindings unlucky enough to also be type errors — a
success arm that narrows. This note is the same failure one level up: the corpus
controls that would independently detect wrong binding were themselves stripped,
so the branch lost coverage in the same motion that created the risk.

Together they explain why the branch reads healthier than it is: the noisy
failures were repaired, and the instruments that would have reported the silent
ones were disarmed.

## What is owed

1. Restore the import-subject fixtures as SPECIMENS, not as corpus. A fixture
   demonstrating import-mediated binding must keep its imports for as long as the
   behaviour it specimens exists, or be deliberately retired with a receipt per
   DESIGN 4b — never silently normalised by a bulk pass.
2. Re-establish `module_graph_edge_source` as discriminating, or retire it and
   say so. A control that cannot fail is worse than an absent one.
3. Qualify `qualified.dag`'s return annotation to the declarer `import_said`
   names. Mechanical, and its own census row supplies the answer.
4. Exclude `**/test/fixture/**` from any future bulk rewrite, and treat a
   specimen edit as an authored decision.

## The general defense, which is the durable part

Two independent cuts hit this in one day — imports here, a resurrected v1 witness
file on the floor cut — and neither was caught by conflict handling, because
nothing conflicted. A one-sided add and a clean auto-merge both SATISFY git and
VIOLATE a cut:

    git's relation:  same path changed incompatibly, relative to the merge base
    a cut's relation: no member of the deleted class may re-enter after the base

On a deletion cut the CLEAN merge is the dangerous case; a conflict at least
summons a human. The only thing that caught either instance was an invariant
re-asserted after every merge, independent of whether anything conflicted --
here `^import ` count == 0 across the .dag corpus, there a derived emit-plan
path count. That habit is the mechanism, not caution.
