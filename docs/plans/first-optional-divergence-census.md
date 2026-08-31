# The `first` interpreter/emitted divergence: census, root cause, and why the repair is two-sided

## The claim under census

`dag/std/algebra.dag` declares five methods with `return_type: OptionalOf { inner: ReceiverElement }`
— `first`, `last`, `get`, `lookup`, `map_get`. That row is the single authority for their result
type, and the Rust emit arm realizes it: `extdeps/languages/rust/emit.dag`'s method template row for
`first` is `{recv}.first().cloned()`, an `Option<T>`.

The interpreter arm did not. `v1_interpreter`'s `method_call.first` computed
`items.front().cloned().unwrap_or(Value::Null)` — the RAW ELEMENT, or `Value::Null` when the
collection was empty. `method_call.last` and `method_call.get` and `method_call.lookup` were the
same shape; only `map_get` already constructed the Optional (through `map_lookup_as_optional`, whose
doc comment states the construction-not-validation rule this census re-derives from the other side).

Two realizations of one declared signature that disagree are not a low rung on the §4b ladder. They
are DESIGN.md §5 silent wrongness — outside it.

## What is measured, and by what

Executed, not reasoned. A four-case probe run through `gunbc run` against `dag` + `src/v2`
source roots, on the seed as it stands on `main`:

| probe | interpreted | what `dag/std/algebra.dag` declares | |
|---|---|---|---|
| `[] \|> first`, outer/inner match | `OUTER-ABSENT` | `OUTER-ABSENT` | agrees |
| `[Absent] \|> first`, outer/inner match | `OUTER-ABSENT` | `OUTER-PRESENT-INNER-ABSENT` | **diverges** |
| `[Present { value: "x" }] \|> first` | `x` | `x` | agrees (positive control) |
| `(["x"] \|> filter(n => true) \|> first) == Present { value: "x" }` | `false` | `true` | **diverges** |
| `(["x"] \|> filter(n => true) \|> first) == "x"` | `true` | not well-typed | **diverges** |

Those five rows are now enrolled as executing evidence in
`dag/test/claim/first_optional_construction_witness_test.dag`, which is red against the raw-element
arm and green against the constructed Optional. The pre-existing
`dag/test/claim/branded_list_first_optional_witness_test.dag` is the regression control: it was
green BEFORE the repair and must stay green after.

## Why the pre-existing witnesses were green on a real divergence

`branded_list_first_optional_witness` eliminates `first` with
`match { Present { value: v } => .. Absent => .. }` over a NON-optional element type. The interpreter
carried compensating raw-unwrap arms in `match_pattern` — a `Present` pattern against a value that is
neither `Null` nor a `Variant` matches and binds the value itself — precisely so that shape would
agree. So the entire match-scrutinee population was green on a divergence it is structurally unable
to observe. **A witness whose RED is not authorable is a decoration**; for this class the
match-scrutinee shape is exactly that, and the two shapes in the table are the ones that are not.

## The census — 186 terminal `|> first` sites, 81 files, at identity grain

Population: every terminal `|> first` occurrence in `dag/**.dag` and `src/v2/**.dag` on `main`
(`git ls-files`, pipeline-terminal form). The parent lane reported 187 across 82 files; the extra
row is not on `main` and is most plausibly the known-red claim added on PR #9775's branch. The
count is reported for reconciliation only — **the roster below, not the count, is the deliverable.**

The three shapes, and what each does with the result:

- **S1 — eliminated by `match` (143 sites).** `match xs |> first { Present { value: v } => .. }`,
  including the `let x = .. |> first` then `match x` spelling. The compensating `match_pattern` arms
  make interpreter and emitted AGREE here, for every element type that is not itself an `Optional`.
  These sites are not victims; they are the regression population the repair must not break.
- **S2 — returned directly as the enclosing function's `T?` (37 sites).** The divergence is
  propagated, not resolved: the raw element leaves the function wearing the declared `Optional`
  type. Each of these is a victim exactly when one of its callers is S3-shaped.
- **S3 — flows into a value position (6 sites).** Compared, passed as an argument, or returned
  where a NON-optional type is declared. These are the sites where the two realizations produce
  different answers on inputs the corpus can actually reach.

The S2 and S3 rosters are below. S1 is not rostered individually: it is the complement, and its
membership test is mechanical (the occurrence is a `match` scrutinee).

## The finding that changes the shape of the repair

The census does not stop at `|> first`. The METHOD-CALL spelling `.first()` / `.last()` is a
separate and much larger population — 646 occurrences across 180 files in the same two trees — and
its dominant idiom is the value position, not the match:

    parse_int(s: fields.first())
    trim(tokens.skip(n: 1).first())
    percent(scalars.first())
    OpenBmcCollectionOne { value: values.first() }
    fn cache_facts_for_id(..) -> CacheInterfaceFacts { catalog |> filter(..) |> first() }

Every one of those passes an `Optional<T>` into a position declared `T`. They work TODAY in both
realizations, for two different reasons:

- **emitted**: `05_emit_rust`'s `rust_call_arg_fail_closed_unwrap` sees a `CardOptional` argument
  meeting a required parameter and emits `.expect("fail-closed: an optional value flowed into
  non-optional parameter N of F (empty Optional at runtime)")`. Typed, located, fail-closed.
- **interpreted**: nothing. There is no optional→required coercion in `v1_interpreter`'s argument
  binding. It works only because `first` handed back the raw element in the first place.

So the raw-element arm is not an isolated defect in one handler. **It is the compensation that the
interpreter's MISSING argument coercion has been leaning on**, corpus-wide. Repairing `first` alone
— making the interpreter construct the Optional its roster row declares — removes the compensation
without supplying what it was compensating for, and hundreds of value-position sites begin handing a
`Present { .. }` variant to `parse_int`, `trim`, `percent` and to record fields. That is a larger
silent wrongness than the one being fixed, in the same direction.

That is measured, not argued. With only the `first`/`last`/`get`/`lookup` construction in place,
`bmc_capability_solve_witness_test`'s `firmware_wire_version_is_parsed_before_track_matching` — an
ordinary corpus witness that names nothing about optionals — flips from PASS to
`FAIL (runtime error [type-error]: type error: parse_int expects a string argument, got Variant)`,
while the same witness is green on the unmodified seed. It reaches
`parse_int(s: fields.first())` in `extdeps/bmc/capability.dag`.

**The repair is therefore two-sided, and both sides derive from authorities that already exist:**

1. `v1_interpreter` constructs the `Optional` for every algebra row that declares `OptionalOf`,
   decided by call site rather than by value shape (the `RawMapLookup` rule, applied to the ordered
   collections). One authority: the `AlgebraFieldTemplate` rows.
2. `v1_interpreter` gains the optional→required argument coercion the Rust emit arm already has,
   decided by the callee's declared parameter cardinality — the same fact
   `rust_call_arg_fail_closed_unwrap` reads — and refusing, typed and located, where the emitted arm
   `.expect`s. One authority: the callee's parameter cardinality.

Neither half is landable without the other, and that is executed rather than predicted: the
capability-solve witness above is the discriminating control, red on half the repair and green on
both halves and on neither. Landing (1) alone is the absorbing-fallback shape read
backwards: it converts a silent agreement into a silent disagreement across a population the
change does not name.

## What blocks the second half, and why this lane stopped rather than improvised

Side (2) is implemented in this branch for `.dag`-declared callees: `call_function_inner` binds an
argument, reads the callee parameter's declared type node, and — where that parameter is a required
value parameter — unwraps `Present` and REFUSES on `Absent`, typed and located, exactly where the
emitted arm `.expect`s. Measured: the new witness is 7/7 green, and
`branded_list_first_optional_witness` (the regression population) stays 8/8 green.

It does not close the class, and the reason is a missing authority rather than a missing edit.
`parse_int(s: fields.first())` never reaches `call_function_inner`: `parse_int` is a BUILTIN, and
`04_method`'s `builtin_function_registry` maps a builtin name to a RETURN TYPE only — 04_sigs says
so in as many words. There is no declared parameter cardinality for a builtin anywhere the
interpreter (or anything else) can read, so the coercion cannot be DERIVED for a builtin call. The
capability-solve witness above stays red for that reason and no other.

Deciding it by the argument's value shape instead — "if it looks like an `Optional`, unwrap it" —
is available and is the wrong answer twice: it is validation standing where construction was
available (§5), and it is precisely the value-shape inference `map_lookup_as_optional`'s own doc
comment refuses, because a stored `V = Optional<T>` payload is then indistinguishable from a wrapped
result. So this lane stops here rather than shipping it.

**The grounding this class is waiting on**: builtin PARAMETER signatures, modelled beside the
return type the registry already carries, so that argument cardinality is a fact the substrate
holds rather than a fact only the Rust emitter's static types happen to know. That is
model-before-implement work in `std/` ahead of any further pipeline edit, and it is a routing
decision, not something to improvise inside this repair.

## Rung, ceiling, trigger

- **Class**: `first_optional_representation_divergence` — a collection projection declared
  `Optional<T>` realized as a raw element in one arm and as `Option<T>` in the other.
- **Rung found at**: below the ladder (silent wrongness). Both arms typecheck; neither warns.
- **Rung after part (1) + (2)**: mechanically preventable. The interpreter refuses when an algebra
  arm's result does not inhabit the optionality its roster row declares, and refuses when an absent
  optional reaches a required position. The invalid state stays writable — a hand-written arm in the
  seed can still compute the wrong thing — so this is rung 2, not 3.
- **Attainable ceiling**: structurally impossible (rung 4), reached when the arm BODIES stop being
  hand-written Rust in the seed and are projected from the same rows the emit arm reads. That is the
  §7 self-host frontier for `v1_interpreter`, not a local edit.
- **Next-rung trigger**: the capability that lets an interpreter primitive's body be derived from
  its `AlgebraFieldTemplate` row rather than authored beside it — the same `v1_interpreter` pure-eval
  dissolution named on the `method_call.map_keys` arm. Not an artifact; the capability.

## Roster

### S2-returned-as-optional (37 sites)

- `dag/extdeps/filesystem/linux.dag` · `linux_proc_mount_row_for_target`
- `dag/extdeps/git/object_store.dag` · `git_find_stored_object`
- `dag/extdeps/git/object_store.dag` · `git_find_unavailable`
- `dag/extdeps/languages/rust/emit.dag` · `rust_pair_completion_spelling_for`
- `dag/extdeps/languages/rust/representation.dag` · `rust_representation_realization_for`
- `dag/extdeps/llm/codex_auth.dag` · `codex_default_organization`
- `dag/extdeps/mercurial.dag` · `mercurial_first_cycle_node`
- `dag/extdeps/pijul.dag` · `pijul_delivery_dependency_issue`
- `dag/extdeps/pijul.dag` · `pijul_delivery_unavailable_issue`
- `dag/extdeps/pijul.dag` · `pijul_find_channel`
- `dag/extdeps/pijul.dag` · `pijul_missing_channel_member_issue`
- `dag/extdeps/pijul.dag` · `pijul_missing_conflict_change_issue`
- `dag/extdeps/pijul.dag` · `pijul_missing_context_issue`
- `dag/extdeps/pijul.dag` · `pijul_missing_dependency_issue`
- `dag/extdeps/pijul.dag` · `pijul_missing_tree_change_issue`
- `dag/extdeps/pijul.dag` · `pijul_present_change_declared_missing`
- `dag/extdeps/pijul.dag` · `pijul_present_path_declared_missing`
- `dag/extdeps/pijul.dag` · `pijul_present_vertex_declared_missing`
- `dag/extdeps/pijul.dag` · `pijul_self_dependency`
- `dag/extdeps/pijul.dag` · `pijul_state_identity_issue`
- `dag/gunbc/design/interaction.dag` · `detent_named`
- `dag/gunbc/design/material.dag` · `carrier_named`
- `dag/gunbc/design/state_response.dag` · `rest_member`
- `dag/gunbc/host/host_standup.dag` · `assimilation_obligation_for_input`
- `dag/gunbc/instruments/e0599_emitter_decision_census.dag` · `e0599_row_for_operation`
- `dag/gunbc/live_deploy/repository_convergence.dag` · `convergence_ref_at`
- `dag/gunbc/live_deploy/repository_convergence.dag` · `convergence_worktree_at`
- `dag/gunbc/namespace/namespace_clause_e_projection_law.dag` · `<module>`
- `dag/gunbc/roadmap/roadmap_belt_actuate.dag` · `belt_dispatch_result_for_label`
- `dag/gunbc/roadmap/roadmap_closing_contract_authoring.dag` · `closing_contract_target_node`
- `dag/gunbc/roadmap/roadmap_execution_contract.dag` · `dispatch_host_realization`
- `dag/gunbc/roadmap/roadmap_style.dag` · `swatch_named`
- `dag/gunbc/roadmap/roadmap_validation_oracle.dag` · `validation_oracle_first_incomplete`
- `dag/std/target_representation.dag` · `checkpoint_row_migration_for`
- `dag/std/target_representation.dag` · `representation_spelling_for`
- `dag/test/claim/algebra_carrier_roster_witness_test.dag` · `ascii_least`
- `dag/test/claim/roadmap/roadmap_program_view_witness_test.dag` · `fx_line_view`

### S3-value-position (6 sites)

- `dag/gunbc/generated_artifact_observation.dag` · `observe_generated_artifact_with`
- `dag/std/cache_interface.dag` · `cache_facts_for_id`
- `dag/std/orthogonal_geometry.dag` · `any_nonadjacent_edges_touch`
- `dag/test/claim/build_latency_actions_collect_witness_test.dag` · `witness_population_fold_groups_by_host_and_filters_job_name`
- `dag/test/claim/build_latency_actions_collect_witness_test.dag` · `witness_population_fold_groups_by_host_and_filters_job_name`
- `dag/test/claim/guarantee_probe_corpus_witness_test.dag` · `witness_harness_revision`