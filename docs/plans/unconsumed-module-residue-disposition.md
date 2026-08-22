# Disposition of the unconsumed-module residue

**This is the execution record for the population
[`unconsumed-module-census.md`](unconsumed-module-census.md) measured. That document is the
census and stays the authority on how the population is derived; this one records what was
done to it and, for every row not deleted, the typed reason it survived.** The two are
separate facts on separate clocks: the census answers *what is unconsumed*, this answers
*what was dispositioned, and what refused*.

Operator directive both serve (2026-08-21, verbatim): *"yes please make sure to clean up
anything without consumers that we don't need, or get them actually consumed."*

## 1. What was deleted, and what the number is

The census's reconciled cleanup list is **131 modules** -- 96 STILL-UNCONSUMED plus 35
DEAD-CONSUMER-ONLY, the two buckets that are residue on all three decoded surfaces. That
figure was measured on 2026-08-22 against a tree that has since moved. **Re-deriving the
instrument on this branch's base (`90986d19469`) reproduces it within the drift:**

| bucket | census (2026-08-22) | re-derived here | delta |
| --- | --- | --- | --- |
| population (unreachable) | 298 | 302 | +4 |
| CONSUMED-DECISIVE | 91 | 89 | -2 |
| DEAD-CONSUMER-ONLY | 35 | 32 | -3 |
| AMBIGUOUS-SHARED-ONLY | 75 | 74 | -1 |
| STILL-UNCONSUMED | 96 | 107 | +11 |
| **residue (the cleanup list)** | **131** | **139** | **+8** |

The instrument is re-derived rather than inherited on purpose. The census records that the
population *has a clock on it* -- one of its own appendix rows already pointed at a deleted
file -- so a disposition lane that trusts a four-day-old row list deletes against a tree
that no longer exists. Re-deriving also re-runs the census's controls: `v2.compiler.compile`,
`gunbc.spark.serving_desired`, `gunbc.clock_read`, `v2.std.node` and
`gunbc.accelerator_demo_gpu` all score reachable here, and RESIDUE-EMPTY again scores 0
consumed, so the instrument is neither finding consumption everywhere nor nowhere.

**Of the corrected 114-row residue, this change deletes 56.** The deleted set is the rows
that carry no obligation to anything outside themselves, on every surface the instrument
decodes *and* every surface it does not:

- unreachable from every root -- discovery path, entry row, or v1 seed mirror;
- no uniquely-owned symbol -- **including coproduct variant constructors and flat-namespace
  `operation`/`service`/`resource` declarations**, see 4d -- named bare by any `.dag` file;
- **no mention of the module name or its path in any `.dag`, `.rs`, `.yml`, `.yaml`,
  `.toml`, `.sh` or `.txt` file** in the tree;
- no `test fn` declared, so no assertion stops executing;
- no `src/v1/stage0/src` mirror, so the seed population is unchanged;
- **and its whole island is in the set** -- eligibility is computed as a fixed point over the
  deletion set, not per module (4g).



## 2. The deletion as the census

DESIGN §3 says the deletion *is* the census. Measured against the corrected instrument (§4d),
before and after the cut:

| | before | after | delta |
| --- | --- | --- | --- |
| modules | 3851 | 3795 | -56 |
| **reachable** | **3549** | **3549** | **0** |
| population | 302 | 246 | -56 |
| **CONSUMED-DECISIVE** | **91** | **91** | **0** |
| AMBIGUOUS-SHARED-ONLY | 97 | 97 | 0 |
| STILL-UNCONSUMED | 80 | 29 | -51 |
| DEAD-CONSUMER-ONLY | 34 | 29 | -5 |

Reachable, CONSUMED-DECISIVE and AMBIGUOUS are all unchanged; the whole movement is inside
the two residue buckets, and it equals the file count exactly.

**These counters are necessary and NOT sufficient, and §4d is why.** An earlier revision
offered exactly this table as proof the cut was safe. It was computed by an instrument
carrying an extraction defect, so it could not surface the case that defect was blind to --
and it read as reassurance precisely because it was consistent. The table is retained
because it still falsifies a whole class of error (a cut that caught a *reachable* module
would move the top row), and demoted because it cannot falsify the class that actually
occurred. The independent instrument is the floor, in §8.

## 3. The one row the census deferred, now answered by its author -- and the cause it gave is not the cause the census guessed

`gunbc.scm.commit_closure_store` was deleted by the batch rule, restored while the question
was open, and is **deleted again** on the answer from `gentle-eagle-360`, who authored #8820.
The verdict and the reasoning are recorded here rather than in the module, because the module
is going away and a citation into it dies with it.

**Both arms the census offered were built on a false premise, and the correction is
load-bearing.** The census framed this as *the envelope grew save/load and superseded it*
versus *the envelope should consume it as its persistence layer*.

- *Not superseded.* `gunbc.scm.repository_envelope` contains **zero `Filesystem` operations
  and not one `func`** -- it is a pure codec, `RepositoryEnvelope` to `JsonValue` and back.
  Nothing took over this module's job. Recording the deletion as "replaced by the envelope"
  would be false.
- *And the envelope should NOT consume it.* `commit_closure_store` persists **one root and
  its closure**. A repository is a different subject: an empty initialized state with **no
  root at all**, several commits over one shared node population, a checked-out selection.
  Wiring the envelope to a carrier that demands a root would force `init` to invent a phantom
  commit to satisfy it -- a grain mismatch dressed as a fix.

**The correct disposition row: DELETE, cause = STAGED ORPHAN AT THE WRONG GRAIN.** Not
superseded, not replaced, and not merely unconsumed. It was never wired, and the layer that
will do this job is repository-grain and will be written that way whether or not this module
stands.

**Why delete rather than hold it for that layer** -- the author's reasoning, and DESIGN §3's:
a surviving X is an **attractor**. While it stands, every nearby persistence question gets
answered in commit-closure vocabulary that is already scheduled to die. They declined to hold
a lane open against a speculative future "export one commit" operation for which they have no
consumer -- §6 experimental residue, applied by an author to their own module.

**A rung-honesty defect in its own carrier, volunteered by that author, which strengthens the
warrant.** The module's header claims its persistence is *"verified by direct execution in Wet
mode"* and points at `commit_closure_round_trip_probe`. That was true when it was run by hand
and is not true now: the probe is **enrolled nowhere and nothing executes it**. So the module
is not merely unconsumed -- it carries an overclaim about its own evidence, which is a
stronger reason to delete than unreachability ever was, and exactly the specification-without-
execution shape DESIGN §5 names.

**What must survive into the replacement, recorded because this deletion is what removes the
prompt for it** (not this lane's to carry, and named so a future reader does not have to
re-derive it): the host's success/content/error triple folded into a coproduct **at** the
boundary, so no downstream consumer picks which member to believe or reads content from a
failed read; and encode adjudicated **before** the write, so a successful write cannot upgrade
an encoding failure into `Saved`.

## 4. One finding the deletion surfaced, and it is not a deletion

**`gunbc.v1_maintenance_standing` is unreachable from every root and named bare by nothing,
and DESIGN.md names it as an authority.** DESIGN's 3 standing rule on the v1 seed states
plainly: *"The authority is `gunbc.v1_maintenance_standing` `v1_seed_standing`"* -- the
carrier for the entire semantics-frozen / maintenance-active reclassification, its admission
test, and its five-class refusal vocabulary.

It is held, and it is not proposed for deletion. The finding is the other direction: **the
carrier DESIGN designates as the authority for the v1 freeze standing executes nowhere and
is reached by nothing.** Its four in-tree `.dag` mentions (`ci_layer_roots`,
`documentary_refs`, `roadmap_serve`, `whole_corpus_compile_admission`, plus two witnesses)
are string citations, not calls. That is consistent with the standing's own declared rung --
it says the vocabulary *"is consumed by review diligence, not by any gate, so it sits at
mitigatable"* -- and this measurement is the independent corroboration of that sentence:
the module is not merely ungated, it is unreachable. Recorded as a rung-honesty datum for
whoever climbs it, not as a disposition.

## 4b. A strand that appeared, then dissolved when its module came back

Worth keeping as a record of the surface, not of a repair. Widening the mention scan past
source extensions to `.txt` found `docs/probes/census_extra_excludes.txt` and its seeds file
naming `dag/examples/gunbhub_serve_program/gunbhub_serve_program.dag`, which the first cut
deleted. `v1_compiler.census_exclude_derive` loads both -- the seeds drive the derived
exclude closure and the pinned oracle is its drift witness -- so a row naming a deleted path
skews the symmetric diff toward *drift* rather than *staleness*.

**Both the row removal and the two count literals it forced are reverted**, because
`gunbhub_serve_program` left the deletion set in §4d's re-derivation and the row is correct
again. **This PR touches no file under `src/v1`.**

The reusable finding stands and is what the episode was worth: **a source-extension mention
scan is not a complete consumption surface.** Authored data files carry references, and this
one was load-bearing. The census's decoded-surface list should include them. That check is
now part of the precondition in §1 and it stays there whether or not any row trips it.

## 4c. One deleted row was a committed copy of a DO-NOT-COMMIT artifact

Raised by review as two surviving references to a deleted filename
(`gunbc.ci_layer_roots`'s `WitnessExclusionRow` pattern and `discover_owned_data`'s
`exclude_subpaths` default), cleared there as harmless string patterns. That is correct, and
checking *why* it is correct turned up the sharper fact and reversed the tidy-up it invites.

`v1_compiler.cli_run` `discover_owned_data_decls` **generates** a module named
`v2.test.claim.workflow.host_discovered_owned_data_manifest` under
`src/v2/test/claim/workflow/`, and stamps it `GENERATED by discover_owned_data -- ephemeral
host transport. DO NOT COMMIT.` What this change deleted is
`v2.test.workflow.host_discovered_owned_data_manifest` at `src/v2/workflow/` -- a
**committed instance of that ephemeral artifact under a second module path, carrying all-zero
counts**. So the deletion removed a stale committed copy of a file the generator says must
not be committed, which is a better reason to delete it than unreachability was.

**And the two exclusion rows must stay.** `path_excluded` is a substring match over the full
path, so the pattern `host_discovered_owned_data_manifest.dag` still matches the *generated*
file whenever `discover_owned_data` produces it. They are live exclusions of an ephemeral
artifact, not dead rows left over from this deletion. Removing them as tidy-up -- which is
what "a pattern that no longer matches anything" invites -- would have re-admitted a
generated transport into discovery.

Recorded because the wrong reading is the natural one, and the next person to grep these
filenames will reach it.

## 4e. One row restored on disposition, not on measurement

`gunbc.generic_binder_field_projection_deficit` scores STILL-UNCONSUMED correctly and is
restored anyway. It is dispositioned KEEP-WITH-REASON in #8851 as a DESIGN 4b **deficit
filing** -- a declared rung, a ceiling, a next-rung trigger.

**An unconsumed deficit filing is 4b(2) working, not residue.** A class below its ceiling is
*required* to name its trigger, and nothing consumes that filing by design; being unreferenced
is its normal state, not evidence against it. Deleting it would have removed a safety-ledger
row and lowered a rung with none of the reason, bounded population and restoration trigger
4b(3) demands. Its sibling `gunbc.empty_decl_file_checkpoint_bypass` carries the same
disposition and was never in this batch.

The general point, which the census stated and this lane had to learn twice: **unreachability
is the wrong predicate for a row whose purpose is to be read by a human ledger.** Both this
row and section 3's `commit_closure_store` are that shape, and neither was caught by a
mechanical rule -- both came from a per-row disposition someone had already written down.

## 4d. Four extraction defects, one mechanism: under-extraction creates false uniqueness

**The floor refused the first cut**, with `unresolved type 'MergeReadinessVerdict'` and eight
`undefined variable 'Ready'` in `gunbc.code_change_workflow`. `gunbc.pr_digests` had been
deleted and should not have been.

The defect-6 re-score asks whether another `.dag` file names a candidate's declared symbol
*bare*. Answering needs to know what a module **declares** — and four ways of getting that
wrong were found, all under-extraction, all with the same consequence:

| # | defect | effect |
| --- | --- | --- |
| 7 | coproduct **variant constructors** not extracted (`= Ready \| NotReady`) | found here, by the floor |
| a | a **generic header** `type X<P> = A \| B` skips the block | relayed by `silent-deer-368` |
| b | a **multi-line variant record** truncates a start-of-line scan | relayed by `silent-deer-368` |
| c | `operation`/`service`/`resource` are **declarations**, read as references | relayed by `silent-deer-368` |
| d | a **same-line coproduct** `type X = A \| B` | found by their discriminator, in *my* extractor |

**Why this direction is the dangerous one.** Under-extraction means a module owns fewer
symbols than it does, which makes a *shared* name look **uniquely owned** — false uniqueness.
A module whose declarations all sit behind an unparsed header owns nothing, so a live consumer
naming its symbols bare is invisible and the module scores residue and **gets deleted**. That
is the opposite direction from the `pr_digests` case, which merely kept a dead-looking module
alive.

**Fixed at the root, not per bug.** Adopted `silent-deer-368`'s region-based extractor: a
type's region runs to the next top-level declaration (so (b) cannot truncate it), a leading
`<...>` is stripped before testing for `=` (so (a) parses), and `operation`/`service`/
`resource` names are declarations (so (c) does not misattribute). Verified against their four
discriminators — `Apply` in `std.upsert_decision`, `Select` in `v2.extdeps.languages.llvm_ir`,
`Ready` in `gunbc.pr_digests`, `Capability` in `std.behavioral` — **my extractor passed three
and failed the fourth**, which is how (d) was found. Their (c) tell also passes:
`extdeps.transports.sql` scores AMBIGUOUS rather than consumed-by-`filesystem_io`, which
linked to it only through `operation Delete {`.

**What re-deriving cost, and it was not nothing.** **13 modules left the deletion set** — nine
formatters, the `language_model` rust root, `generic_instantiation`, `roadmap_dispatch`,
`gunbhub_serve_program` — and **every one landed in AMBIGUOUS-SHARED-ONLY, not CONSUMED.**
That distinction is the whole reason the bucket exists: they are *no longer provably
unconsumed* and *not proven consumed*, so none may be deleted on this evidence, and reading
the shrinking residue as "more live modules found" would collapse exactly the gap the bucket
is there to hold.

**The general lesson, which outlives every row above:**

> **A control derived from the measurement it controls does not discriminate that
> measurement's blind spot.**

Both counters §2 offers came from the instrument carrying defect 7. The floor caught it
because the floor is an independent instrument that does not share the method. That is
delete-first's actual mechanism (DESIGN §3): not that the rule is trustworthy, but that the
substrate refuses on an authority the rule had no part in building. And defects (a)-(d) were
found by **hand-reading rows**, with counters that stayed consistent throughout — the same
lesson arriving a second time, from the other direction.

## 4g. Eligibility is a fixed point over the set, not a property of a module

31 residue rows are named bare only from *inside* the population. A per-module verdict over a
mutually-referencing island returns "consumed" for every member and the island never becomes
eligible; splitting one across batches reds the first batch.

So the deletion set is computed as a **fixed point**: repeatedly drop any module with a
surviving in-population consumer, until nothing drops. This ran twice for real, in both
directions, which is why it is a section rather than a footnote:

- **A deletion created a violation.** The first cut deleted `gunbc.pr_digests` while keeping
  `gunbc.code_change_workflow` — one island, split. That is the row the floor refused on.
- **A restore created one.** Putting the 13 AMBIGUOUS rows back re-created an in-population
  consumer for `v2.std.rust_leaf_model_claim` (via `v2.test.language_model.rust`), which had
  been eligible until its neighbour returned.

**This is the reason the change is one PR** and not six cluster PRs: the property is over the
set, so six PRs merging in arbitrary order re-open exactly this class unless each is
re-derived against the others' merged state — strictly more work and strictly more risk than
keeping the set closed. The reviewer's seams are the commits.

## 4f. A finding walked past while editing the pin, filed rather than fixed

Repairing `census_extra_excludes.txt` meant moving two count literals in
`v1_compiler.census_exclude_derive` (83 to 82, 27 to 26). **Those literals are a change
detector, not a check**, and this lane is in a position to say so with a receipt rather than
as a style objection.

A hand-maintained count beside a hand-maintained list measures whether someone remembered to
update both. It has no independent referent: it is not a controlled fixture, not an external
or versioned authority, not a policy budget, and not a monotone debt contract over a closed
universe — the four grounds DESIGN §5 admits for a numeric literal in a merge-blocking test.
The section's own review tell applies exactly: automating the literal's update collapses the
assertion to `measure() == measure()`, so the manual update was the assertion's entire
content.

The receipt is this change. One row moved for an unrelated reason, and both literals had to
be hand-followed; nothing about the list's *correctness* was checked by either. Had the row
moved without them, the failure would have reported as a count mismatch rather than as what
it was.

**Filed, not fixed.** The pin belongs to the exclude-closure lane — `census_exclude_derive`'s
own header names its dissolution (the pin retires when strict whole-tree resolve greens
without host fixed-point closure) — and repairing an oracle inside a deletion PR would be
this document's own §6 objection turned on itself. Recorded because the person who paid the
cost is the right person to report it, and the wrong person to fix it.

**Postscript: the edit that prompted this is reverted.** `gunbhub_serve_program` left the
deletion set in §4d, so the pin's row is correct again and the literals return to 83 and 27.
The finding is filed on its own merits, which is where it always belonged -- it is a property
of the pin, not of this deletion.

## 5. The 56 deleted

| module | path | bucket |
| --- | --- | --- |
| `gunbc.assimilate.bmc_wif_canary_bootstrap` | `dag/gunbc/assimilate/bmc_wif_canary_bootstrap.dag` | STILL-UNCONSUMED |
| `gunbc.cursor_sdk_secure_api_key` | `dag/gunbc/cursor_sdk_secure_api_key.dag` | STILL-UNCONSUMED |
| `gunbc.devboot.vertical_receipt` | `dag/gunbc/devboot/vertical_receipt.dag` | STILL-UNCONSUMED |
| `gunbc.floor_resolve_realization` | `dag/gunbc/floor_resolve_realization.dag` | STILL-UNCONSUMED |
| `gunbc.language_subject_scope_scaffold` | `dag/gunbc/language_subject_scope_scaffold.dag` | STILL-UNCONSUMED |
| `gunbc.p3a1_self_fork_homonym_disposition` | `dag/gunbc/p3a1_self_fork_homonym_disposition.dag` | STILL-UNCONSUMED |
| `gunbc.parse_allowlist` | `dag/gunbc/parse_allowlist.dag` | STILL-UNCONSUMED |
| `gunbc.provider_standing_live_probes` | `dag/gunbc/provider_standing_live_probes.dag` | STILL-UNCONSUMED |
| `gunbc.scm.commit_closure_store` | `dag/gunbc/scm/commit_closure_store.dag` | STILL-UNCONSUMED |
| `gunbc.site.register_principles` | `dag/gunbc/site/register_principles.dag` | STILL-UNCONSUMED |
| `gunbc.srv4_seeded_install_media_artifact` | `dag/gunbc/srv4_seeded_install_media_artifact.dag` | STILL-UNCONSUMED |
| `gunbc.tools.bmc_onboard_validate` | `dag/gunbc/tools/bmc_onboard_validate.dag` | STILL-UNCONSUMED |
| `gunbc.tools.grounding_confirm` | `dag/gunbc/tools/grounding_confirm.dag` | STILL-UNCONSUMED |
| `gunbc.tools.roadmap_spawn_request` | `dag/gunbc/tools/roadmap_spawn_request.dag` | STILL-UNCONSUMED |
| `gunbc.witness_family_fanout` | `dag/gunbc/witness_family_fanout.dag` | STILL-UNCONSUMED |
| `shared.dag_util` | `dag/shared/dag_util.dag` | DEAD-CONSUMER-ONLY |
| `std.binding` | `dag/std/binding.dag` | STILL-UNCONSUMED |
| `std.containers` | `dag/std/containers.dag` | STILL-UNCONSUMED |
| `std.list` | `dag/std/list.dag` | STILL-UNCONSUMED |
| `tools.build` | `dag/tools/build.dag` | STILL-UNCONSUMED |
| `tools.codegen` | `dag/tools/codegen.dag` | STILL-UNCONSUMED |
| `tools.readme` | `dag/tools/readme.dag` | STILL-UNCONSUMED |
| `v2.bin.main` | `src/v2/bin/main.dag` | STILL-UNCONSUMED |
| `v2.extdeps.formats.csv` | `src/v2/extdeps/formats/csv.dag` | STILL-UNCONSUMED |
| `v2.extdeps.formats.json_schema` | `src/v2/extdeps/formats/json_schema.dag` | DEAD-CONSUMER-ONLY |
| `v2.extdeps.formats.openapi` | `src/v2/extdeps/formats/openapi.dag` | STILL-UNCONSUMED |
| `v2.extdeps.formats.toml` | `src/v2/extdeps/formats/toml.dag` | STILL-UNCONSUMED |
| `v2.extdeps.formats.yaml` | `src/v2/extdeps/formats/yaml.dag` | DEAD-CONSUMER-ONLY |
| `v2.test.language_model.go_r1` | `src/v2/extdeps/language_model/go_r1.dag` | STILL-UNCONSUMED |
| `v2.test.language_model.go_r2a` | `src/v2/extdeps/language_model/go_r2a.dag` | STILL-UNCONSUMED |
| `v2.test.language_model.go_r2b` | `src/v2/extdeps/language_model/go_r2b.dag` | STILL-UNCONSUMED |
| `v2.test.language_model.go_r3_external` | `src/v2/extdeps/language_model/go_r3_external.dag` | STILL-UNCONSUMED |
| `v2.test.language_model.python_cross_runtime_drift` | `src/v2/extdeps/language_model/python_cross_runtime_drift.dag` | STILL-UNCONSUMED |
| `v2.test.language_model.python_l2_cross_target_parity` | `src/v2/extdeps/language_model/python_l2_cross_target_parity.dag` | STILL-UNCONSUMED |
| `v2.test.language_model.python_r2a` | `src/v2/extdeps/language_model/python_r2a.dag` | STILL-UNCONSUMED |
| `v2.test.language_model.python_r2b` | `src/v2/extdeps/language_model/python_r2b.dag` | STILL-UNCONSUMED |
| `v2.test.language_model.python_r3_external` | `src/v2/extdeps/language_model/python_r3_external.dag` | STILL-UNCONSUMED |
| `v2.test.language_model.rust_r2a` | `src/v2/extdeps/language_model/rust_r2a.dag` | STILL-UNCONSUMED |
| `v2.test.language_model.rust_r2b` | `src/v2/extdeps/language_model/rust_r2b.dag` | STILL-UNCONSUMED |
| `v2.test.language_model.rust_r3_external` | `src/v2/extdeps/language_model/rust_r3_external.dag` | STILL-UNCONSUMED |
| `v2.test.language_model.typescript_r2a` | `src/v2/extdeps/language_model/typescript_r2a.dag` | STILL-UNCONSUMED |
| `v2.test.language_model.typescript_r2b` | `src/v2/extdeps/language_model/typescript_r2b.dag` | STILL-UNCONSUMED |
| `v2.test.language_model.typescript_r3_external` | `src/v2/extdeps/language_model/typescript_r3_external.dag` | STILL-UNCONSUMED |
| `v2.extdeps.typecheckers.mypy` | `src/v2/extdeps/typecheckers/mypy.dag` | STILL-UNCONSUMED |
| `v2.extdeps.typecheckers.pyright` | `src/v2/extdeps/typecheckers/pyright.dag` | STILL-UNCONSUMED |
| `v2.test.algebra_laws.zip_eq_list_equality` | `src/v2/std/algebra_laws/zip_eq_list_equality.dag` | STILL-UNCONSUMED |
| `v2.std.inhabitant_bridge` | `src/v2/std/inhabitant_bridge.dag` | STILL-UNCONSUMED |
| `v2.test.nat_semiring.rung_l1_go_compiler_slice` | `src/v2/std/nat_semiring/rung_l1_go_compiler_slice.dag` | STILL-UNCONSUMED |
| `v2.test.nat_semiring.rung_l1_python_runtime` | `src/v2/std/nat_semiring/rung_l1_python_runtime.dag` | STILL-UNCONSUMED |
| `v2.test.qualified_name.from_node` | `src/v2/std/qualified_name/from_node.dag` | STILL-UNCONSUMED |
| `v2.std.type_expr_projection_row_schema` | `src/v2/std/type_expr_projection_row_schema.dag` | STILL-UNCONSUMED |
| `v2.workflow.ci_stage0_partition_compile_gate_emit` | `src/v2/workflow/ci_stage0_partition_compile_gate_emit.dag` | STILL-UNCONSUMED |
| `v2.workflow.ci_v1_compiler_test_targets_compile_gate_emit` | `src/v2/workflow/ci_v1_compiler_test_targets_compile_gate_emit.dag` | STILL-UNCONSUMED |
| `v2.workflow.floor2_prepared_subject` | `src/v2/workflow/floor2_prepared_subject.dag` | STILL-UNCONSUMED |
| `v2.test.workflow.host_discovered_owned_data_manifest` | `src/v2/workflow/host_discovered_owned_data_manifest.dag` | STILL-UNCONSUMED |
| `v2.workflow.probe_selector_host_health` | `src/v2/workflow/probe_selector_host_health.dag` | STILL-UNCONSUMED |

## 6. The 58 held, with the reason each survived

**Held is not keep.** Every row carries a typed reason and an owed next step; none is
dispositioned as "consumed". Reporting them is the point — the census's instruction is that a
refusal is data, and 58 refusals over a 114-row residue is the measurement this lane produces.

Separately from these, **97 rows sit in AMBIGUOUS-SHARED-ONLY** and are outside the residue
entirely: not provably unconsumed, not proven consumed, each needing a per-row read. 13 of
them arrived there from this lane's own re-derivation (§4d).

### Residue, but named somewhere this cut would strand (35)

Named by a live `.dag`, `.rs` or data file, or otherwise carrying an obligation the deletion precondition refuses to guess at. Weak evidence in both directions: several are receipt prose and are ordinary residue; at least one (`v2.workflow.product_receipt_stage`) is a live transport named via an `--entry` path held in a *variable*, the dynamically-composed-argv blind spot the census declares.

| module | path | bucket |
| --- | --- | --- |
| `direct_rust_door_ingest_fixture` | `src/v2/compiler/self_host/direct_rust_door_ingest_fixture.dag` | STILL-UNCONSUMED |
| `extdeps.bmc.mock_corpus` | `dag/extdeps/bmc/mock_corpus.dag` | DEAD-CONSUMER-ONLY |
| `extdeps.ebay.mock_corpus` | `dag/extdeps/ebay/mock_corpus.dag` | STILL-UNCONSUMED |
| `extdeps.linux.mock_corpus` | `dag/extdeps/linux/mock_corpus.dag` | DEAD-CONSUMER-ONLY |
| `extdeps.tcgplayer.mock_corpus` | `dag/extdeps/tcgplayer/mock_corpus.dag` | STILL-UNCONSUMED |
| `gunbc.auth.optional_impersonation` | `dag/gunbc/auth/optional_impersonation.dag` | STILL-UNCONSUMED |
| `gunbc.auth.patterns` | `dag/gunbc/auth/patterns.dag` | STILL-UNCONSUMED |
| `gunbc.char_at_scaling_probe_support` | `dag/gunbc/char_at_scaling_probe_support.dag` | STILL-UNCONSUMED |
| `gunbc.ci_build_job_v1_compiler_unit_receipt` | `dag/gunbc/ci_build_job_v1_compiler_unit_receipt.dag` | STILL-UNCONSUMED |
| `gunbc.generic_binder_field_projection_deficit` | `dag/gunbc/generic_binder_field_projection_deficit.dag` | STILL-UNCONSUMED |
| `gunbc.githooks_pre_push_cli` | `dag/gunbc/githooks_pre_push_cli.dag` | STILL-UNCONSUMED |
| `gunbc.namespace_census_receipt` | `dag/gunbc/namespace_census_receipt.dag` | STILL-UNCONSUMED |
| `gunbc.p1_retention_cohort_receipt` | `dag/gunbc/p1_retention_cohort_receipt.dag` | STILL-UNCONSUMED |
| `gunbc.plans.affected_set_self_confirmation` | `dag/gunbc/plans/affected_set_self_confirmation.dag` | STILL-UNCONSUMED |
| `gunbc.plans.fleet_subsumption_manual_gaps` | `dag/gunbc/plans/fleet_subsumption_manual_gaps.dag` | STILL-UNCONSUMED |
| `gunbc.plans.host_convergence_circuit_residue` | `dag/gunbc/plans/host_convergence_circuit_residue.dag` | STILL-UNCONSUMED |
| `gunbc.plans.transport_argv_anemia_dissolution` | `dag/gunbc/plans/transport_argv_anemia_dissolution.dag` | STILL-UNCONSUMED |
| `gunbc.seed_closed_vocabulary_wildcard_census` | `dag/gunbc/seed_closed_vocabulary_wildcard_census.dag` | STILL-UNCONSUMED |
| `gunbc.site.interaction` | `dag/gunbc/site/interaction.dag` | STILL-UNCONSUMED |
| `gunbc.test_node_wall_clock_ratchet` | `dag/gunbc/test_node_wall_clock_ratchet.dag` | STILL-UNCONSUMED |
| `gunbc.v1_maintenance_standing` | `dag/gunbc/v1_maintenance_standing.dag` | STILL-UNCONSUMED |
| `std.exec_format` | `dag/std/exec_format.dag` | DEAD-CONSUMER-ONLY |
| `std.import` | `dag/std/import.dag` | STILL-UNCONSUMED |
| `std.methods` | `dag/std/methods.dag` | STILL-UNCONSUMED |
| `std.verification` | `dag/std/verification.dag` | STILL-UNCONSUMED |
| `v2.std.datetime` | `src/v2/std/datetime.dag` | DEAD-CONSUMER-ONLY |
| `v2.std.probe_selector` | `src/v2/std/probe_selector.dag` | DEAD-CONSUMER-ONLY |
| `v2.std.rust_leaf_model_claim` | `src/v2/std/rust_leaf_model_claim.dag` | DEAD-CONSUMER-ONLY |
| `v2.test.workflow.glob_discovery_law` | `src/v2/workflow/glob_discovery_law.dag` | STILL-UNCONSUMED |
| `v2.workflow.class_b_import_closure_transport` | `src/v2/workflow/class_b_import_closure_transport.dag` | DEAD-CONSUMER-ONLY |
| `v2.workflow.compiler_closure_ingest_transport` | `src/v2/workflow/compiler_closure_ingest_transport.dag` | DEAD-CONSUMER-ONLY |
| `v2.workflow.phase_profile_proof_plan` | `src/v2/workflow/phase_profile_proof_plan.dag` | STILL-UNCONSUMED |
| `v2.workflow.product_receipt_stage` | `src/v2/workflow/product_receipt_stage.dag` | STILL-UNCONSUMED |
| `v2.workflow.source_root_ingest_gate` | `src/v2/workflow/source_root_ingest_gate.dag` | STILL-UNCONSUMED |
| `v2.workflow.source_root_ingest_transport` | `src/v2/workflow/source_root_ingest_transport.dag` | DEAD-CONSUMER-ONLY |

### Declares an `ExternalAuthority` anchor (21)

DESIGN §3's extdeps citation duty: the value may be the citation, not a call, so unreachability is not evidence of residue. Mostly `DEAD-CONSUMER-ONLY` — the island shape — so they delete as connected components or not at all (census B6+, one island per PR, each gated on whether the citation is the deliverable).

| module | path | bucket |
| --- | --- | --- |
| `extdeps.boot.emit` | `dag/extdeps/boot/emit.dag` | DEAD-CONSUMER-ONLY |
| `extdeps.boot.framebuffer` | `dag/extdeps/boot/framebuffer.dag` | DEAD-CONSUMER-ONLY |
| `extdeps.boot.freestanding_payload` | `dag/extdeps/boot/freestanding_payload.dag` | DEAD-CONSUMER-ONLY |
| `extdeps.boot.linux_x86_boot` | `dag/extdeps/boot/linux_x86_boot.dag` | DEAD-CONSUMER-ONLY |
| `extdeps.cloud.gcp.sts` | `dag/extdeps/cloud/gcp/sts.dag` | DEAD-CONSUMER-ONLY |
| `extdeps.colo.types` | `dag/extdeps/colo/types.dag` | DEAD-CONSUMER-ONLY |
| `extdeps.container.oci.image_config` | `dag/extdeps/container/oci/image_config.dag` | DEAD-CONSUMER-ONLY |
| `extdeps.container.oci.linux` | `dag/extdeps/container/oci/linux.dag` | DEAD-CONSUMER-ONLY |
| `extdeps.container.oci.manifest` | `dag/extdeps/container/oci/manifest.dag` | DEAD-CONSUMER-ONLY |
| `extdeps.container.oci.runtime_config` | `dag/extdeps/container/oci/runtime_config.dag` | DEAD-CONSUMER-ONLY |
| `extdeps.ebay.ebay_contracts` | `dag/extdeps/ebay/ebay_contracts.dag` | DEAD-CONSUMER-ONLY |
| `extdeps.formats.elf.encode` | `dag/extdeps/formats/elf/encode.dag` | DEAD-CONSUMER-ONLY |
| `extdeps.formats.elf.hello_static_witness` | `dag/extdeps/formats/elf/hello_static_witness.dag` | DEAD-CONSUMER-ONLY |
| `extdeps.formats.elf.primitives` | `dag/extdeps/formats/elf/primitives.dag` | DEAD-CONSUMER-ONLY |
| `extdeps.formats.elf.segments` | `dag/extdeps/formats/elf/segments.dag` | DEAD-CONSUMER-ONLY |
| `extdeps.formats.elf.types` | `dag/extdeps/formats/elf/types.dag` | DEAD-CONSUMER-ONLY |
| `extdeps.github.mergeable_state_contracts` | `dag/extdeps/github/mergeable_state_contracts.dag` | DEAD-CONSUMER-ONLY |
| `extdeps.posix.rusage` | `dag/extdeps/posix/rusage.dag` | DEAD-CONSUMER-ONLY |
| `extdeps.runtime.local` | `dag/extdeps/runtime/local.dag` | DEAD-CONSUMER-ONLY |
| `extdeps.tcgplayer.tcgplayer` | `dag/extdeps/tcgplayer/tcgplayer.dag` | DEAD-CONSUMER-ONLY |
| `gunbc.hand_lens_host_bridge_scaffold_index` | `dag/gunbc/hand_lens_host_bridge_scaffold_index.dag` | DEAD-CONSUMER-ONLY |

### Frozen against a named re-add (2)

DESIGN §3 frozen-X. Deleting these deletes what the re-add queue exists to re-attach.

| module | path | bucket |
| --- | --- | --- |
| `tools.dag_compile_clean_seam_transport` | `dag/tools/dag_compile_clean_seam_transport.dag` | DEAD-CONSUMER-ONLY |
| `tools.merge_admission_capture_transport` | `dag/tools/merge_admission_capture_transport.dag` | STILL-UNCONSUMED |

## 7. What this change does NOT claim

- **Not that 56 is the maximum safe cut.** It is the subset provable clean on every decoded
  surface, with its islands closed. The 35 rows held for a per-row read are held on a *string
  mention*, which is weak evidence in both directions.
- **Not that the residue is now 58.** Five modules moved DEAD-CONSUMER-ONLY to
  STILL-UNCONSUMED as their island neighbours went. **The residue is a fixed point reached by
  iteration, not a set reached in one pass**, and a follow-on pass is owed.
- **Not that the AMBIGUOUS bucket is cleared.** 97 rows are unresolvable at identity grain.
  13 of them got there from this lane's own correction, and reading that as "13 live modules
  found" would be exactly wrong: they are *not proven consumed either*.
- **Not that the extractor is now correct.** Four defects were found in it, three of them by
  someone else hand-reading rows while every counter stayed consistent. The honest claim is
  that it passes four discriminators it previously failed one of, not that a fifth defect
  does not exist.
- **Not that the directive's second arm was served.** Every row was deleted or held; **none
  was wired up.** The candidates are the 21 `CITED-AUTHORITY` islands and the 22 modules the
  census found named `v2.test.*` while declaring no test and sitting outside every test path
  — a name asserting enrolment that floor discovery, which keys on the file suffix, never
  grants.

## 8. Evidence

The deletion's discriminating check is not "the tree still parses" -- 71 modules nothing
imports would parse-clean whether or not they were load-bearing. It is the pair of counters
in section 2 that must not move (`reachable`, `CONSUMED-DECISIVE`) beside the one that must
(`population`, by exactly the number of files removed), plus the required run:

`claim_executor --required-ci --source-root dag --source-root src/v2` -- the three-phase
required mode (src/v1 `.dag` parse sweep, `--required-regen`, witness floor). Its result on
this branch is reported on the PR.

**The instrument is not committed, and that is a repository rule rather than a choice:**
`.gitignore` excludes `*.py` tree-wide, so the audit script that produced these numbers has
no home here. What it does is fully specified instead -- it implements the census's own
section-2 method without deviation, and the two arms that matter are stated exactly:

- **Roots** are the discovery paths (`/test/`, `*_test.dag`, `/lens/`, `/manual/`,
  `/fixture*`), every `--entry` argv literal and `*entry*:` path field found in any `.dag`,
  `.rs`, `.md`, `.yml`, `.sh` or `.toml` file, and every module whose name-with-underscores
  matches a `src/v1/stage0/src/*.rs` seed mirror. Edges are `import` lines plus
  fully-qualified `module.symbol` references resolved by longest-prefix against the module
  index. The population is what no root reaches.
- **The defect-6 re-score** takes each population member's declared symbols -- not its name,
  not its path -- keeps only those declared by exactly one module corpus-wide, and asks
  whether any other `.dag` file names one bare. Comments and string literals are removed
  with a character scanner, never a regex, because a regex terminates early on the `\{`
  interpolation escapes real `.dag` prose contains. An identifier preceded by `.` or
  followed by `:` is not a reference.

That specification is what makes the numbers disagreeable: re-implementing it is a
half-page, and any reader who does so and gets a different answer has found a defect in one
of us. Beside it, the two counters in section 2 are checkable without the instrument at all
-- `reachable` and `CONSUMED-DECISIVE` not moving is a claim about the tree, not about the
script.