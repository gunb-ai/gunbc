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
