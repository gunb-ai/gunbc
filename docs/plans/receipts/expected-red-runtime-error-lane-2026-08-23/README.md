# The 143 live runtime-errored enrolments, split by what the witness loader can reach

**Subject:** the expected-red roster (`v2.workflow.floor_expected_red` `floor_expected_red_roster`)
at main `faf6583461a`, 2026-08-23. This is the runtime-errored lane of the roster-deletion
program; the causal partition it builds on is
[floor-runtime-error-partition-2026-08-23](../floor-runtime-error-partition-2026-08-23/README.md).

## Re-measuring 143 without using the number

The prior receipt's subject is run `32633501354` at head `907f19c2cc`, where the ledger reported
`known_red_runtime_errored=164`. Joining those 164 identities against the tree at `faf6583461a`:
21 of them name a missing symbol that the referring file **already imports today** (the
`*_published_mock_corpus` names repaired by `gunbc#9006`). 164 − 21 = **143**, which reproduces
the reported live count from the tree rather than from the earlier count. The causal partition is
otherwise unchanged: 105 `R1`, 11 `T1-arity`, 9 `R6`, 8 `R2`, 5 `R3`, 2 `T2`, and one each of
`R5`, `C1`, `T3`.

Every one of the 143 names a declaration that EXISTS in the corpus. None is a typo, and the join
finds exactly one declaring module for each of the 113 `R1`/`R2` names — the repair is never a
choice between candidates.

## The split this receipt adds: the bare-reference closure has an eligibility gate

`cli_run` `build_both_closure_edge_index` computes the dotted-reference pull set for every source
and then, before the bare half:

    if source_declares_import_lines(&sf.content) { continue; }

So `extend_with_bare_reference_closure` — the mechanism that resolves an unimported bare name
through the tree census and pulls the module declaring it — is switched **off entirely** for any
file that declares even one `import` line. Eligibility is a property of the FILE, not of the name.

Joining the 143 against their referring files by that predicate:

| loader eligibility | rows | what can repair them |
|---|---|---|
| import-less file (bare closure eligible) | 120 | a root fix in the loader/evaluator can reach these |
| file already declares imports (bare scan never runs) | 23 | only the missing import line can reach these |

`live_143_by_loader_eligibility.tsv` is that join, one row per identity: eligibility, identity,
family/subclass, referring file, missing name.

The 23 are 19 reference rows across six files plus the four `bootstrap_footprint_anchor`
`atom_identity_hash` rows (which are not a reference failure at all).

**The consequence for per-site repair, stated as a hazard rather than a law.** Adding an import to
an import-LESS file moves it across that gate and switches its bare closure off, so every OTHER
ambient name in that file loses its census-derived pull. Whether that breaks the file depends on
whether its remaining names are reachable by import edge, dotted reference, or the entry closure —
`gunbc#8992` added one import to a previously import-less file and its six witnesses passed, so
the hazard is file-specific and not a certainty. It is still the reason the 120 must not be
repaired file-by-file by default: the unit of work is the root, and a per-site import there
changes the loader's treatment of the whole file to fix one name.

## What this PR changes

The 19 reference rows in the six already-importing files, and nothing else. For those files the
bare scan is already off, so the missing import is the only mechanism that can bind the name and
the change cannot disable anything that was working:

| file | import added |
|---|---|
| `src/v2/lens/vacuity_test.dag` | `v2.test.algebra_laws.nat_semiring { nat_add_left_identity_input }` |
| `src/v2/test/claim/intent_linearity/lens_unit/runtime_axis_test.dag` | `v2.lens.cost { symbolic_cost_of_node }`, `v2.lens.simulated_relationship { chain_is_simulated }` |
| `src/v2/lens/mutation_adequacy_test.dag` | `v2.lens.discrimination { unit_is_discriminating }` |
| `dag/test/claim/host_reach_identity_probe_witness_test.dag` | `gunbc.network_identity_subsumption { srv3_post_install_lease_table_fixture }` |
| `dag/test/claim/fleet_convergence_verdict_witness_test.dag` | `gunbc.network_identity_subsumption { srv3_post_install_lease_table_fixture }` |
| `src/v2/test/claim/manual/value_null_split_witness_test.dag` | `v2.lens.testgen { cross_repr_native_value_null }` |

**No roster row is removed by this PR**, and that is deliberate rather than incomplete: a row
leaves the roster only when its identity RUNS and its outcome is honest, and main is red at
PREPARATION (`dag/gunbc/runner_slot_provision.dag:240`, the `ArgvCommand` seal; `gunbc#9031` is
the fix in flight), so no witness executes on any PR cut from main. Unenrolling on an unexecuted
run would be exactly the stale-quarantine hazard the roster exists to surface. The removals land
in the same lane once a run observes these identities passing.

## Round 1 executed: the measurement, and what it changed

**Subject:** required-floor run `32660721426` on this branch (PR `gunbc#9039`, with `gunbc#9031`'s
seal fix merged in so the fold could be reached at all), 2026-08-23. Ledger line:

    planned=10695 executed=10695 terminal=10695 passed=10414 known_red_held=36 failed=0
    stale_quarantine=8 known_red_now_passing=8 known_red_runtime_errored=135 route_gap_held=101

So the runtime-errored family went **143 -> 135**, and the roster's own removal path fired: the
eight identities below reported STALE-QUARANTINE — enrolled as expected-red and PASSED — naming
themselves and asking to be removed. They are removed from the roster in this PR, which is the
only reason any row leaves it. Five are `v2.test.lens_vacuity.vacuity_test`, two are
`v2.test.lens_mutation_adequacy.mutation_adequacy_test`, one is
`v2.test.manual.value_null_split_witness.optional_null_straddle_rostered_until_phase_e`.

**The other eleven repaired rows did not flip, and the run says exactly why.** Their reported
missing name CHANGED: `test.claim.fleet_convergence_verdict_witness` and
`test.claim.host_reach_identity_probe_witness` advanced from
`srv3_post_install_lease_table_fixture` to `srv3_install_hang_no_router_lease_ms`, and the six
`v2.test.intent_linearity.lens_unit.runtime_axis` rows advanced from `symbolic_cost_of_node` /
`chain_is_simulated` to `cost_is_lowerable`. That is the first-failure frontier: the floor reports
only the FIRST unresolved name, so clearing one advances a row to its next blocker without
guaranteeing a verdict. Round 2 in this PR imports those three names in the same three files.

`runtime_errored_135_after_round_1.tsv` is the post-round-1 population, one row per identity:
loader eligibility, identity, referring file, missing name, declaring module, message.

The split holds at the new population: **120 BARE / 15 HASIMPORT**, the 15 being the eleven
frontier rows above plus the four `bootstrap_footprint_anchor` rows.

## The tail families, named so they are not mistaken for reference failures

Each of these is a distinct defect with its own root, and none of them is repaired by an import:

- `T1-arity` (11) — `content_hash`'s atom fold reaches the v1 intrinsic `atom_identity_hash` with
  an `Int` identity. `v2.std.compilers.semantic_decl_emission`
  `semantic_decl_string_to_bundle_node` folds a `String` into per-character atom nodes and passes
  the char code point as `identity`, where `v2.std.node` `Atom` declares `identity: Symbol`. The
  compiler accepts it; the intrinsic refuses at evaluation. The test file for four of these
  already records the diagnosis in place.
- `C1` (1) — `v2.test.emit.produced_decl_support_preserved` binds `render` from a
  `ProducedDeclWired` match arm and calls it; the interpreter's `eval_call` shadows the LEXICAL
  tier over builtins but consults `ctx.lookup_fn` BEFORE the lexical `Value::Fn` arm, so
  `std.layout` `render(doc, proto)` answers the call and the arity refusal follows. The law the
  code states beside that check ("a lexical binding shadows every name-keyed tier") holds for
  `Value::Closure` and not for `Value::Fn`.
- `T2` (2) — `v2.test.claim.staging` writes `Hit { value: 42 }` in an import-less file, and `Hit`
  is corpus-AMBIGUOUS (`std.cache_interface` `CacheLookupResult` carries `Hit { receipt: … }`;
  `v2.std.staging` `CacheProbe` carries `Hit { value: … }`). This is the census-ambiguous
  resolution hole DESIGN §4b already names.
- `T3` (1) — `v2.test.fixture.walk_plan_stage.recursion_refusal_member` is `fn f() { f() }`
  declared as a `test fn`, so discovery executes it and the depth wall refuses it. The refusal is
  correct; the enrolment is the wrong carrier for it.
- `R6`/`R5`/`R3` (15) — bare coproduct variants and cross-test-module `test data` in import-less
  files; same loader eligibility question as `R1`, not a separate mechanism.

## Round 2: the BARE-file discriminator, run as an experiment rather than a rollout

The 120 BARE rows cannot be diagnosed outside the floor. Two things are now measured and both
point away from the loader:

- the per-entry witness loader RESOLVES the R1 shape. Executed at main head on a release binary
  built from that tree, `gunbc run --claim-run` on
  `test.claim.ci_deploy_target_host_witness.witness_deploy_job_not_on_ubicloud_runner` — a row the
  floor reports as `no such function: gunbc_ci_deploy_srv1_stage` — **PASSES**, and
  `GUNBC_BARE_PULL_TRACE=1` prints
  `[bare-pull] dag/test/claim/ci_deploy_target_host_witness_test.dag -> 'gunbc_ci_deploy_srv1_stage' -> gunbc.ci_spec`.
  So the closure resolves the name and pulls the module.
- the floor's subject already holds every module. `assemble_prepared_subject` takes EVERY module
  under the source roots minus a small exclusion list, so "the module was not loaded" cannot be
  the floor's cause either.
- and no CLI reproduces the floor's subject: a whole-tree `gunbc run` REFUSES
  (`--entry <file.dag> is required … refused rather than approximated`).

So the floor is the only instrument, and the honest way to use it is one bounded experiment
rather than a 120-file rollout. Three import-LESS files are given their explicit imports here,
chosen to vary the thing in question — the density of OTHER ambient names in the file, which is
what moving the file across the bare-scan gate puts at risk:

| file | rows | names imported | other ambient names at risk |
|---|---|---|---|
| `dag/test/claim/ci_deploy_target_host_witness_test.dag` | 2 | 1 | several |
| `dag/test/claim/design_argument_witness_test.dag` | 9 | 3 | few |
| `dag/test/claim/emit_host_gate_witness_test.dag` | 6 | 6 | several |

Both outcomes are informative and neither is a rollout decision on its own. If the 17 rows flip to
PASS and nothing else in those three files regresses, the per-site import works for BARE files too
and the remaining question is only unit-of-work. If any of them REGRESSES a currently-passing
witness in the same file, that is the bare-scan gate biting, measured, and per-site repair is
excluded for the other 117 by evidence rather than by argument.

## Round 2 executed: the experiment answered, and it answered against my own hazard

**Subject:** required-floor run `32664512304` on this branch at `42d0a8502` (`origin/main` merged
after `gunbc#9031` landed), 2026-08-23. Ledger:

    planned=10693 executed=10693 terminal=10693 passed=10421 known_red_held=36 failed=0
    stale_quarantine=24 known_red_now_passing=24 known_red_runtime_errored=111 route_gap_held=101

`planned == executed == terminal` and all three are nonzero, so the fold actually ran and the
counts are readable. **143 -> 135 -> 111.**

**The BARE-file experiment passed on both arms, which is the load-bearing result.** All 17 rows in
the three import-LESS files flipped to PASS — `ci_deploy_target_host_witness` 2,
`design_argument_witness` 9, `emit_host_gate_witness` 6 — and `failed=0`, so **nothing else in
those three files regressed**. The bare-scan gate did not bite. That retires the hazard this
receipt raised in its own first section as a MEASURED negative rather than an argued one: moving a
file across `source_declares_import_lines` is survivable in practice, at least where the file's
remaining ambient names are reachable by the entry closure. It does not license a blind 117-file
rollout — the three files were chosen to vary exactly the property at risk, and a file whose other
names are reachable ONLY by the census could still break — but it removes the objection that
per-site repair is structurally unsafe for the BARE class.

The seven remaining frontier rows from round 1 also flipped (`fleet_convergence_verdict_witness`
2, `host_reach_identity_probe_witness` 2, `runtime_axis` 3 of 6). All 24 are removed from the
roster here by the same rule as before: each reported STALE-QUARANTINE, enrolled and PASSED.
Roster 195 -> 171 entries, deletions only.

**Three `runtime_axis` rows advanced a second time** — `cost_is_lowerable` cleared, and the
reported name is now `type_decls_anti_unify` (`v2.lens.structural_similarity`). Imported here.
That file has now advanced through three successive frontier names, which is the clearest single
demonstration in this lane that the floor reports the FIRST unresolved name and clearing one
guarantees an advance, not a verdict.

`runtime_errored_111_after_round_2.tsv` is the post-round-2 population as the run printed it.

## Round 3: the resolvable remainder, in one batch because the run is the check

With the gate hazard retired by round 2's measurement, the remaining reference rows are batched
rather than trickled. Of the 111, **90 name a symbol with exactly one declaring module** that the
referring file does not import: 48 files, 70 imports. Three of those needed the declaring module
decided by the FIELD's type rather than by uniqueness, because the name is corpus-ambiguous and
the census resolves ambiguity silently:

- `Dag` -> `v2.lens.fact_cardinality` (the `FactCardinalityDeclFact.tree` field's own coproduct),
  not `v2.lens.affected_set`'s type of the same name;
- `SourceFile` (x2) -> `v2.std.artifact` (the `Artifact.kind` field's `ArtifactKind`), not
  `v1.compiler.compile`'s type of the same name.

The remaining 21 are the tail families below plus the three `runtime_axis` rows already imported
in round 2: seven `srv3_subsumption` and one `host_phase_status` and four `bootstrap_footprint`
on `atom_identity_hash`, one `produced_decl_support` on the `render` shadowing, two `staging` on
`.raw on Int`, and one divergent fixture. None of them is an import.

Batching is safe here in the specific sense the floor makes it safe: every row still executes,
`failed=0` is asserted over the whole corpus, and a repair that breaks a sibling witness shows up
as a FAILURE naming it rather than as a silent pass. The batch is checked by the fold, not by me.
