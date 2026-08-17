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

## The merge driver's refusal is a policy, not an observation

    PolicyRefusedBeforeContentMerge    !=    ContentConflict

The generated-artifact driver refusing is a valid automerge POLICY. It is not an
observation of content overlap, and "no merged content and no conflict markers,
after no attempt was made" is not evidence about mergeability. A three-way
preview establishes content standing separately -- and does NOT by itself
authorize applying that merge, since a generated artifact still owes
regeneration from the merged authority.

I read the refusal as a finding during this integration and aborted the merge on
it. That was wrong; the plain three-way showed one twin merged completely clean
and the other had a single hunk.

## The general defense, which is the durable part

Two independent cuts hit this in one day — imports here, a resurrected v1 witness
file on the floor cut — and neither was caught by conflict handling, because
nothing conflicted. A one-sided add and a clean auto-merge both SATISFY git and
VIOLATE a cut:

    git's relation:  same path changed incompatibly, relative to the merge base
    a cut's relation: no member of the deleted class may re-enter after the base

    CONFLICTING IS LOUD, NOT SAFE.
    CLEAN IS SILENT, NOT GREEN.
    Only the post-merge invariant decides.

An earlier revision of this section said "a conflict at least summons a human",
implying the conflicting path is the safe one. That is FALSE and is corrected
here rather than quietly reworded: a human resolves conflicts incorrectly all the
time, and summoning a reviewer is not a receipt. Treating a
conflicted-then-resolved merge as self-verifying is the worse practice the
sentence licensed. BOTH paths owe the same check.

The only authoritative sequence, and it runs unconditionally on all four cases
(clean merge, conflicted-then-resolved, generated regeneration, forward merge
from main):

    merge event -> construct resulting tree -> re-evaluate the exact cut
    invariant -> mint held-or-violated

Merge status is PROVENANCE. Cut preservation is a SEMANTIC RECEIPT over the
resulting tree. The only thing that caught either instance today was that receipt
run regardless of whether anything conflicted -- here `^import ` count == 0 over
the .dag population, there a derived emit-plan path count. That habit is the
mechanism, not caution.

WHEN A COUNT STOPS BEING A COMPLETE ANSWER. The general wall is not "the class is
empty", it is

    observed old-class population on the merged tree == DECLARED ALLOWED RESIDUE

with emptiness as the common special case. This cut's declared residue is
genuinely {}, so `count == 0` is a complete answer TODAY. The moment a cut
carries a bounded residue the receipt must carry IDENTITIES, not a count -- two
different six-member populations must never compare equal because both count to
six. That is the same identity-join-not-count-equality rule DESIGN already
applies to completeness claims, and it is the rule this branch's own census
violated at (file, name) grain.

A NOTE ON SUBJECT CHOICE, because the two detectors were not equally good. The
floor cut's detector was an emit-plan path count disagreeing 8-vs-7 -- accidental,
because its subject is NARROWER than the rule, so a future re-entrant absent from
that emit plan passes silently. `^import ` over the exact *.dag population takes
the deleted class ITSELF as its subject. A general wall should be modelled on the
latter shape.

## The seed's own import machinery, disarmed by the cut (2026-08-17)

Separate from the 48 fixtures above: `v1_compiler.cli_run` `extract_import_paths`
matches lines beginning `import `, and the corpus now contains ZERO of them. So
every surviving caller returns empty, permanently. Enumerated at 3e493652f7a:

    resolve_virtual_source_with_imports        walks imports to build a closure
    resolve_transitively_bfs_legacy            BFS over imports
    (one further closure walk in the same file)

      -> these now compute an EMPTY dependency closure rather than refusing.
         They are the empty-observation narrow: cannot-express-what-changed
         rendered as nothing-is-affected. Not currently load-bearing for the
         gate, because the reference-derived closure replaced regen's use, but
         they are live functions that will answer "no dependencies" to whoever
         calls them next.

    four import-layer lens sites (layer prefix / declared-target checks)

      -> these guard on `extract_import_paths(..).is_empty()` and skip, or
         iterate an empty list and pass. They are INERT: vacuously green, unable
         to fail, still reporting. DESIGN §6 names this exact tier -- "beware the
         tier where the machinery exists but nothing gates on it -- coverage by
         illusion; an inert lens is itself a lie."

WHY THIS IS RECORDED RATHER THAN FIXED NOW. The operator's bar is compilation and
tests, and none of these breaks either -- which is precisely why they need
writing down: they are invisible to the bar in force. They are also exactly what
DESIGN §3's replacement rule says comes out at cutover ("production machinery
whose only purpose was to implement or compare X is deleted"), so this is a
deletion population, not a repair population.

THE ORDER MATTERS AND IS DELIBERATE. Deleting them touches cli_run.rs, a seed
file under the regen fixed point, and regen is not yet restored. Doing it before
regen is green would mean changing the generator and its output in the same
motion with no oracle to check either against. So: regen first, then this
deletion, and it should be one commit that removes the function and every caller
together rather than leaving a partially-dead surface.

NOT CLAIMED: that this list is complete. It is the `extract_import_paths` caller
set only. Other import-era machinery (the layer-prefix helpers, the
rel_path_for_layer_import projection) is reached through these and has not been
separately enumerated.
