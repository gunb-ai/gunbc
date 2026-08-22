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



## 2. The deletion as the census: what re-scoring after the cut showed

DESIGN 3 says the deletion *is* the census. Run against the instrument rather than only
against the build, it produced one result worth recording, and it is the census's own island
warning arriving on schedule:

| | before | after | delta |
| --- | --- | --- | --- |
| modules | 3851 | 3780 | -71 |
| **reachable** | **3549** | **3549** | **0** |
| population | 302 | 231 | -71 |
| CONSUMED-DECISIVE | 89 | 89 | 0 |
| STILL-UNCONSUMED | 107 | 43 | -64 |
| DEAD-CONSUMER-ONLY | 32 | 25 | -7 |

**Reachable is unchanged and CONSUMED-DECISIVE is unchanged**: nothing that had a live
caller, and nothing any root could reach, was touched. That is the discriminating reading --
a deletion that had caught a consumed module would have moved one of those two numbers.

**Seven modules moved from DEAD-CONSUMER-ONLY to STILL-UNCONSUMED, and that is the island
mechanic, not noise.** Their only bare-symbol caller was itself residue and is now gone, so
they became newly eligible by the same rule that had been holding them ineligible. The
census predicted exactly this and warned that a per-module verdict over a mutually
referencing island is incoherent because each member looks consumed until its neighbours
go. Confirmed here as a measurement: **the residue list is not a fixed set, it is a fixed
point, and one pass does not reach it.** A follow-on pass over these seven is owed and is
not attempted in this change.

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

## 4b. The one strand the deletion actually produced

The precondition in section 1 covers `.dag`, `.rs`, `.yml`, `.yaml`, `.toml`, `.sh` and
`.md`. Widening it afterwards to `.txt` and `.json` found exactly one live consumer the
source-extension scan could not see, and it is worth the row because of *where* it was:

`docs/probes/census_extra_excludes.txt` and `docs/probes/census_extra_excludes_seeds.txt`
both listed `dag/examples/gunbhub_serve_program/gunbhub_serve_program.dag`, and
`v1_compiler.census_exclude_derive` loads both -- the seeds drive the derived exclude
closure and the pinned oracle is that closure's drift witness. A row naming a deleted path
therefore skews the symmetric diff between them in the direction that reads as *drift*
rather than as *staleness*, which is the wrong diagnosis by exactly one step. Both rows are
removed, with the two count literals that pin the files (83 to 82, 27 to 26).

The row is removed rather than the module restored: an exclusion exists to keep a module out
of a resolve walk, and a module that is gone needs no exclusion.

**What this says about the precondition.** A source-extension scan is not a complete
consumption surface -- data files carry references too, and this one was load-bearing.
Nothing else in the 71 has a `.txt`, `.json`, `.jsonl` or `.lock` reference outside historical
receipt transcripts, checked after the fact. The general point stands for the next pass: the
census's decoded-surface list should include authored data files, not only source.

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

## 4d. Instrument defect 7, found by the floor: variant constructors are declared symbols

**The floor refused this change, and the refusal was correct.** `required-ci` reported
`unresolved type 'MergeReadinessVerdict'` in `gunbc.code_change_workflow` and eight
`undefined variable 'Ready'` in its witness. `gunbc.pr_digests` was deleted and should not
have been.

**Root cause, in the instrument rather than in the tree.** The re-score's declared-symbol
extraction matched `fn`, `data`, `type` and `const` declarations. It did not match
**coproduct variant constructors** -- the `= Ready | NotReady { ... }` continuation lines
under `type MergeReadinessVerdict`. So `pr_digests` was credited with owning
`MergeReadinessVerdict` but not `Ready` or `NotReady`, and `code_change_workflow` names
`Ready` **bare, with no import** -- precisely the whole-pool resolution the defect-6 re-score
exists to detect. The instrument was decoding declarations and not their variants, which is
the census's own defect 6 one level down.

**Corrected numbers at this branch's base**, replacing those in section 1 rather than sitting
beside them:

| bucket | as first measured | with variants counted |
| --- | --- | --- |
| CONSUMED-DECISIVE | 89 | **94** |
| STILL-UNCONSUMED | 107 | 102 |
| DEAD-CONSUMER-ONLY | 32 | 31 |
| AMBIGUOUS-SHARED-ONLY | 74 | 75 |
| **residue** | 139 | **133** |

Five modules move from residue to consumed. Only one of them was in the deleted batch, and
it is the one the floor named.

**What this says about the evidence in section 2, which was real but not sufficient.**
`reachable` and `CONSUMED-DECISIVE` were unchanged across the cut, and I read that as
"nothing consumed was touched". Both counters were computed by the *same instrument that had
the defect*, so they could not see the case they were blind to -- a control derived from the
measurement it is controlling cannot discriminate the measurement's own blind spot. **The
floor could, and did.** That is the delete-first doctrine working exactly as DESIGN 3
describes it: the deletion is the census, the real dependent refused loudly, and the refusal
identified a load-bearing edge that three static surfaces and two reviews had all passed
over.

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

### Why the `src/v1` edit is admitted

`census_exclude_derive.rs` is in the frozen seed, so the edit needs its class named.
**The row the pin names is deleted by this PR**, so the choice is not *touch v1 or leave it
alone* — it is *move two count literals, or knowingly land a dangling citation in the seed*,
which is the staleness class this repository polices, landed deliberately. Under
`gunbc.v1_maintenance_standing`'s PURPOSE test this is maintenance of an existing pin
tracking a v2-side deletion: no new surface, no new capability, two literals following a row
that had to move. The coupled change is the smaller harm and it is caused by this PR rather
than imported into it.

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

## 6. The 68 held, with the reason each survived

**Held is not keep.** Every row here carries a typed reason and an owed next step; none is
dispositioned as "consumed". Reporting them is the point -- the census's instruction is that
a refusal is data, and 68 refusals over a 139-row sweep is the measurement this lane
produces.

### Named by a live `.dag` file (36)

A string mention from a reachable module. Some are prose in a receipt; several are live wirings the census's own decoded surfaces cannot see -- `v2.workflow.product_receipt_stage` is named by `product_receipt_transport` as an `--entry` path in a *variable*, which is exactly the dynamically-composed-argv limitation the census declares. Each needs its mention read before it moves; deleting on the unreachability score alone would delete a working transport.

| module | path | lines | why it is held |
| --- | --- | --- | --- |
| `direct_rust_door_ingest_fixture` | `src/v2/compiler/self_host/direct_rust_door_ingest_fixture.dag` | 6 | src/v2/compiler/self_host/direct_rust_door_fixture.dag |
| `extdeps.bmc.mock_corpus` | `dag/extdeps/bmc/mock_corpus.dag` | 71 | dag/extdeps/diagnostic_mock_corpus.dag, dag/gunbc/extdeps_scope_frontier.dag |
| `extdeps.ebay.mock_corpus` | `dag/extdeps/ebay/mock_corpus.dag` | 79 | dag/gunbc/extdeps_scope_frontier.dag |
| `extdeps.linux.mock_corpus` | `dag/extdeps/linux/mock_corpus.dag` | 15 | dag/extdeps/diagnostic_mock_corpus.dag, dag/gunbc/extdeps_scope_frontier.dag |
| `extdeps.tcgplayer.mock_corpus` | `dag/extdeps/tcgplayer/mock_corpus.dag` | 63 | dag/gunbc/extdeps_scope_frontier.dag |
| `gunbc.apply` | `dag/gunbc/apply.dag` | 118 | dag/gunbc/runner_capacity_realize.dag, dag/gunbc/runner_lifecycle.dag |
| `gunbc.auth.credentials` | `dag/gunbc/auth/credentials.dag` | 89 | dag/gunbc/tailscale_acl_phase2_credential.dag |
| `gunbc.auth.optional_impersonation` | `dag/gunbc/auth/optional_impersonation.dag` | 20 | dag/gunbc/tailscale_acl_phase2_credential.dag, dag/test/claim/tailscale_acl_phase2_design_witness_test.dag |
| `gunbc.ci_build_job_v1_compiler_unit_receipt` | `dag/gunbc/ci_build_job_v1_compiler_unit_receipt.dag` | 21 | dag/gunbc/ci_spec.dag |
| `gunbc.githooks_pre_push_cli` | `dag/gunbc/githooks_pre_push_cli.dag` | 10 | dag/gunbc/githooks_pre_push_fmt_transport_scaffold.dag, dag/std/emit_on_demand.dag |
| `gunbc.namespace_census_receipt` | `dag/gunbc/namespace_census_receipt.dag` | 74 | dag/gunbc/doc_graph_roots.dag |
| `gunbc.p1_retention_cohort_receipt` | `dag/gunbc/p1_retention_cohort_receipt.dag` | 8 | dag/gunbc/doc_graph_roots.dag |
| `gunbc.plans.fleet_subsumption_manual_gaps` | `dag/gunbc/plans/fleet_subsumption_manual_gaps.dag` | 196 | dag/gunbc/auth/materialized_secret.dag, dag/gunbc/build_cache_instance.dag, dag/gunbc/ci_floor_measurement.dag +9 |
| `gunbc.plans.host_convergence_circuit_residue` | `dag/gunbc/plans/host_convergence_circuit_residue.dag` | 75 | dag/gunbc/host_converge.dag |
| `gunbc.plans.transport_argv_anemia_dissolution` | `dag/gunbc/plans/transport_argv_anemia_dissolution.dag` | 89 | dag/extdeps/exec/command.dag, dag/extdeps/git/git.dag |
| `gunbc.site.interaction` | `dag/gunbc/site/interaction.dag` | 14 | dag/gunbc/design/interaction.dag |
| `gunbc.test_node_wall_clock_ratchet` | `dag/gunbc/test_node_wall_clock_ratchet.dag` | 99 | dag/gunbc/plans/structural_quadratic_wall_coverage_audit.dag |
| `gunbc.v1_maintenance_standing` | `dag/gunbc/v1_maintenance_standing.dag` | 83 | dag/gunbc/ci_layer_roots.dag, dag/gunbc/documentary_refs.dag, dag/gunbc/roadmap_serve.dag +3 |
| `std.exec_format` | `dag/std/exec_format.dag` | 37 | dag/extdeps/boot/emit.dag, dag/extdeps/boot/freestanding_payload.dag, dag/extdeps/boot/freestanding_witness.dag +3 |
| `std.import` | `dag/std/import.dag` | 2 | dag/gunbc/commit_workflow.dag |
| `std.methods` | `dag/std/methods.dag` | 67 | src/v1/compiler_tests_rust.dag |
| `std.rational` | `dag/std/rational.dag` | 5 | dag/gunbc/doc_graph_roots.dag, dag/gunbc/econ/acquisition.dag |
| `std.stack` | `dag/std/stack.dag` | 56 | dag/gunbc/witness_floor_workflow.dag |
| `std.verification` | `dag/std/verification.dag` | 34 | dag/gunbc/plans/realization_measurement_loop.dag, dag/gunbc/plans/resolver_type_name_collision_wall.dag, src/v2/test/fixture/frontier_probe_elision_boundary_overlay.dag |
| `tools.ci_heal_dispatch` | `dag/tools/ci_heal_dispatch.dag` | 48 | dag/gunbc/ci_spec.dag |
| `tools.gunbc_ci` | `dag/tools/gunbc_ci.dag` | 25 | dag/std/emit_on_demand.dag, src/v2/test/claim/host_language_transport_script/corpus/wall_residue_live_test.dag |
| `v2.extdeps.languages.ecmascript` | `src/v2/extdeps/languages/ecmascript.dag` | 1340 | dag/gunbc/language_target_registry.dag |
| `v2.extdeps.languages.ptx` | `src/v2/extdeps/languages/ptx.dag` | 223 | dag/gunbc/language_target_registry.dag |
| `v2.std.datetime` | `src/v2/std/datetime.dag` | 658 | dag/extdeps/pin.dag, src/v2/test/manual/parse_forensics_scaling_witness.dag |
| `v2.std.probe_selector` | `src/v2/std/probe_selector.dag` | 674 | dag/gunbc/non_fold_residue.dag, dag/gunbc/plans/dag_v2_defork_audit.dag |
| `v2.test.workflow.glob_discovery_law` | `src/v2/workflow/glob_discovery_law.dag` | 113 | src/v2/test/claim/complexity/accumulator_copy_roster_gate_test.dag |
| `v2.workflow.class_b_import_closure_transport` | `src/v2/workflow/class_b_import_closure_transport.dag` | 118 | dag/test/claim/long/rust_test_fixtures_import_closure_witness_test.dag, src/v2/workflow/class_b_import_closure_probe.dag |
| `v2.workflow.compiler_closure_ingest_transport` | `src/v2/workflow/compiler_closure_ingest_transport.dag` | 150 | dag/gunbc/ci_layer_roots.dag, dag/gunbc/explicit_witness_admission.dag, dag/tools/ci_gates.dag +1 |
| `v2.workflow.product_receipt_stage` | `src/v2/workflow/product_receipt_stage.dag` | 661 | src/v2/workflow/product_receipt_transport.dag |
| `v2.workflow.source_root_ingest_gate` | `src/v2/workflow/source_root_ingest_gate.dag` | 18 | dag/gunbc/explicit_witness_admission.dag, dag/test/claim/guarantee_rung_drop_witness_test.dag, dag/tools/ci_gates.dag |
| `v2.workflow.source_root_ingest_transport` | `src/v2/workflow/source_root_ingest_transport.dag` | 90 | dag/tools/ci_gates.dag, src/v2/test/claim/host_language_transport_script/corpus/migrated_transports_clean_test.dag |

### Named by the v1 seed's Rust (6)

Named from `src/v1/stage0`, which the instrument decodes only by *mirror filename*. A `.rs` file naming the module in a string or a comment is a host-side invocation surface the census lists as undecoded.

| module | path | lines | why it is held |
| --- | --- | --- | --- |
| `gunbc.auth.patterns` | `dag/gunbc/auth/patterns.dag` | 113 | src/v1/stage0/src/cli_run.rs |
| `gunbc.char_at_scaling_probe_support` | `dag/gunbc/char_at_scaling_probe_support.dag` | 58 | src/v1/stage0/src/bin/char_at_scaling_probe.rs |
| `gunbc.plans.affected_set_self_confirmation` | `dag/gunbc/plans/affected_set_self_confirmation.dag` | 29 | src/v1/stage0/src/cli_run.rs |
| `gunbc.seed_closed_vocabulary_wildcard_census` | `dag/gunbc/seed_closed_vocabulary_wildcard_census.dag` | 175 | src/v1/stage0/src/cli_run.rs |
| `std.behavioral` | `dag/std/behavioral.dag` | 51 | src/v1/stage0/src/bin/parse_witness.rs |
| `v2.workflow.phase_profile_proof_plan` | `src/v2/workflow/phase_profile_proof_plan.dag` | 22 | src/v1/stage0/tests/phase_profile_claim_executor.rs |

### Declares an `ExternalAuthority` anchor (21)

DESIGN 3's extdeps citation duty: the value of these may be the citation, not a call, so unreachability is not evidence of residue for them. All 21 are `DEAD-CONSUMER-ONLY` -- the island shape -- so they delete as connected components or not at all, which is the census's B6+ shape: one island per PR, each gated on whether the citation is the deliverable.

| module | path | lines | why it is held |
| --- | --- | --- | --- |
| `extdeps.boot.emit` | `dag/extdeps/boot/emit.dag` | 127 | CITED-AUTHORITY (citation may be the deliverable; census B6+, one island per PR) |
| `extdeps.boot.framebuffer` | `dag/extdeps/boot/framebuffer.dag` | 15 | CITED-AUTHORITY (citation may be the deliverable; census B6+, one island per PR) |
| `extdeps.boot.freestanding_payload` | `dag/extdeps/boot/freestanding_payload.dag` | 19 | CITED-AUTHORITY (citation may be the deliverable; census B6+, one island per PR) |
| `extdeps.boot.linux_x86_boot` | `dag/extdeps/boot/linux_x86_boot.dag` | 37 | CITED-AUTHORITY (citation may be the deliverable; census B6+, one island per PR) |
| `extdeps.cloud.gcp.sts` | `dag/extdeps/cloud/gcp/sts.dag` | 111 | CITED-AUTHORITY (citation may be the deliverable; census B6+, one island per PR) |
| `extdeps.colo.types` | `dag/extdeps/colo/types.dag` | 110 | CITED-AUTHORITY (citation may be the deliverable; census B6+, one island per PR) |
| `extdeps.container.oci.image_config` | `dag/extdeps/container/oci/image_config.dag` | 100 | CITED-AUTHORITY (citation may be the deliverable; census B6+, one island per PR) |
| `extdeps.container.oci.linux` | `dag/extdeps/container/oci/linux.dag` | 156 | CITED-AUTHORITY (citation may be the deliverable; census B6+, one island per PR) |
| `extdeps.container.oci.manifest` | `dag/extdeps/container/oci/manifest.dag` | 98 | CITED-AUTHORITY (citation may be the deliverable; census B6+, one island per PR) |
| `extdeps.container.oci.runtime_config` | `dag/extdeps/container/oci/runtime_config.dag` | 97 | CITED-AUTHORITY (citation may be the deliverable; census B6+, one island per PR) |
| `extdeps.ebay.ebay_contracts` | `dag/extdeps/ebay/ebay_contracts.dag` | 34 | CITED-AUTHORITY (citation may be the deliverable; census B6+, one island per PR) |
| `extdeps.formats.elf.encode` | `dag/extdeps/formats/elf/encode.dag` | 201 | CITED-AUTHORITY (citation may be the deliverable; census B6+, one island per PR) |
| `extdeps.formats.elf.hello_static_witness` | `dag/extdeps/formats/elf/hello_static_witness.dag` | 259 | CITED-AUTHORITY (citation may be the deliverable; census B6+, one island per PR) |
| `extdeps.formats.elf.primitives` | `dag/extdeps/formats/elf/primitives.dag` | 61 | CITED-AUTHORITY (citation may be the deliverable; census B6+, one island per PR) |
| `extdeps.formats.elf.segments` | `dag/extdeps/formats/elf/segments.dag` | 79 | CITED-AUTHORITY (citation may be the deliverable; census B6+, one island per PR) |
| `extdeps.formats.elf.types` | `dag/extdeps/formats/elf/types.dag` | 190 | CITED-AUTHORITY (citation may be the deliverable; census B6+, one island per PR) |
| `extdeps.github.mergeable_state_contracts` | `dag/extdeps/github/mergeable_state_contracts.dag` | 18 | CITED-AUTHORITY (citation may be the deliverable; census B6+, one island per PR) |
| `extdeps.posix.rusage` | `dag/extdeps/posix/rusage.dag` | 66 | CITED-AUTHORITY (citation may be the deliverable; census B6+, one island per PR) |
| `extdeps.runtime.local` | `dag/extdeps/runtime/local.dag` | 22 | CITED-AUTHORITY (citation may be the deliverable; census B6+, one island per PR) |
| `extdeps.tcgplayer.tcgplayer` | `dag/extdeps/tcgplayer/tcgplayer.dag` | 82 | CITED-AUTHORITY (citation may be the deliverable; census B6+, one island per PR) |
| `gunbc.hand_lens_host_bridge_scaffold_index` | `dag/gunbc/hand_lens_host_bridge_scaffold_index.dag` | 23 | CITED-AUTHORITY (citation may be the deliverable; census B6+, one island per PR) |

### Frozen against a named re-add (3)

DESIGN 3 frozen-X. Deleting these deletes what the re-add queue exists to re-attach.

| module | path | lines | why it is held |
| --- | --- | --- | --- |
| `tools.dag_compile_clean_seam_transport` | `dag/tools/dag_compile_clean_seam_transport.dag` | 124 | FROZEN-PENDING-RE-ADD (DESIGN §3 frozen-X; the re-add queue exists to re-attach it) |
| `tools.merge_admission_capture_transport` | `dag/tools/merge_admission_capture_transport.dag` | 32 | FROZEN-PENDING-RE-ADD (DESIGN §3 frozen-X; the re-add queue exists to re-attach it) |
| `tools.merge_admission_current_context` | `dag/tools/merge_admission_current_context.dag` | 134 | FROZEN-PENDING-RE-ADD (DESIGN §3 frozen-X; the re-add queue exists to re-attach it) |

### Deferred by the census, by name (1)

| module | path | lines | why it is held |
| --- | --- | --- | --- |
| `gunbc.scm.commit_closure_store` | `dag/gunbc/scm/commit_closure_store.dag` | 203 | the census named this row and declined to disposition it (replacement-migration leftover, #8820 author intent) |

### Owned by another lane (1)

| module | path | lines | why it is held |
| --- | --- | --- | --- |
| `gunbc.spark.managed_access_apply` | `dag/gunbc/spark/managed_access_apply.dag` | 464 | SPARK-AREA (census excludes dag/gunbc/spark/: fierce-lynx-647 owns it) |

## 7. What this change does NOT claim

- **Not that 71 was the maximum safe cut.** It is the subset provable clean on every
  decoded surface. The 36 `NAMED-IN-LIVE-DAG` rows are held on a *string mention*, which is
  weak evidence in both directions: several will be prose in a receipt and are ordinary
  residue, and at least one (`v2.workflow.product_receipt_stage`) is a live transport the
  instrument structurally cannot see. Only a per-row read separates them, and this change
  does not attempt it.
- **Not that the residue is now 68.** Section 2 measures it at 231 population / 68 residue
  *for the classes this lane touched*, and seven rows became newly eligible during the cut.
  The list is a fixed point reached by iteration, not a set.
- **Not that the census's other arm was served.** The directive has two: clean up what has
  no consumer, *or get it actually consumed*. Every row here was dispositioned under the
  first arm or held; **none was wired up**. The rows most likely to belong to the second arm
  are the 21 `CITED-AUTHORITY` islands, where the citation may be the deliverable, and the
  22 modules the census found *named* `v2.test.*` while declaring no test and sitting
  outside every test path -- a name implying enrolment while discovery keys on file suffix.
- **Not verified by typecheck alone.** The acceptance evidence is the required run on this
  branch, below.

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