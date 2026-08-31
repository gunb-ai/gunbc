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

## The census — a disposition roster, not a shape tally

Population: every terminal `|> first` occurrence in `dag/**.dag` and `src/v2/**.dag` on `main`.
186 occurrences resolve to **181 real sites over 81 files**, plus 5 that are not sites at all
(4 inside `//` annotations, 1 inside a string literal that carries a probe program). The parent lane
measured 187 across 82; that reconciles exactly as 186/81 plus PR #9775's own known-red row, which
is not on `main`. **Cite this census as the producer of the roster; the count is not the
deliverable and should not be transcribed.**

Each site carries a DISPOSITION — whether the divergence actually harms it — not merely a shape.
A shape says where the value goes; only the disposition says whether the two realizations answer
differently on an input the corpus can reach.

| disposition | sites | what it means |
|---|---|---|
| `AgreesUnderCompensation` | 142 | eliminated by `match`. The interpreter's raw-unwrap arms in `match_pattern` bind a `Present { value: v }` pattern to any value that is neither `Null` nor a `Present`/`Absent` variant, so interpreter and emitted agree for **every element type except `Optional` itself**. |
| `Propagates` | 36 | returned onward as the enclosing function's declared `T?`. The declared type is honest; only its REPRESENTATION differs, so the disposition is the caller's. |
| `HarmedNow` | 3 | the value reaches a position that reads the representation directly. |
| `NotASite` | 5 | annotation or string-literal text. |

**`AgreesUnderCompensation` is a measured disposition, not an assumption.** Its failure condition is
an element type that is itself `Optional`, and the corpus declares exactly five list-of-optional
carriers in total (`List<T?>` / `List<Optional<T>>`), all in witness tests, none of them reaching a
`first`. So **zero** of the 142 are harmed today. That is precisely why this class stayed invisible:
the shape that dominates the corpus is the one shape the compensation covers.

**`Propagates` resolves the same way, one level out.** Following all 36 functions to their call
sites: 72 callers eliminate by `match` (unharmed), 2 tail-propagate into another `T?`
(`mercurial_first_changeset_cycle` / `_file_revision_cycle`), and 4 compare `== none`
(`rust_representation_realization_for` in `self_host_symbol_identity_binding_witness_test`) — which
AGREES, because a miss is `Null` on one side and `Absent` on the other and both compare equal to
`none`. It agrees by the representation happening to line up on that one constructor: the same site
spelled `== Present { value: .. }` is the parent lane's discriminator and diverges. **Zero harmed
today, and the margin is one constructor wide.**

`HarmedNow`, in full — this is the whole victim list for the pipeline spelling:

- `dag/std/cache_interface.dag` · `cache_facts_for_id` — declares `-> CacheInterfaceFacts` and
  returns `catalog |> filter(..) |> first()`, an `Optional<CacheInterfaceFacts>`. Its three callers
  (`cache_reach_candidate_probe`, `cache_layer_cost_justified`,
  `cache_layer_ids_respect_locality`) then read `.locality` and pass it to `read_latency_cost`.
  Sibling defect in the same module: `cache_layer_plan_primary` / `cache_layer_plan_fallback` both
  declare `-> CacheInterfaceId` over `.first()`.
- `dag/test/claim/build_latency_actions_collect_witness_test.dag` ·
  `witness_population_fold_groups_by_host_and_filters_job_name`, twice —
  `measure_count(m: srv1.durations |> skip(n: 0) |> first)` passes an `Optional` into a required
  parameter. The two realizations agree while the list is non-empty and diverge on empty, where the
  emitted arm's `.expect(..)` stops and the interpreter carries `Value::Null` onward.

**Three of 181.** Read alone that number argues the class is not worth repairing. It is the wrong
denominator, and the next section is why.

## The finding that changes the shape of the repair

The census does not stop at `|> first`. The METHOD-CALL spelling `.first()` / `.last()` is a
separate and much larger population — 646 occurrences across 180 files in the same two trees by this
census's filter, 655 across 178 by the parent lane's independent one. Same magnitude, different
filter boundary; neither number should be transcribed, and the disagreement is itself the reason to
name the producer rather than the figure. Its
dominant idiom is the value position, not the match:

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

## Step 1 re-scoped: the signature carrier already exists, and the gap is its denominator

The routing decision on this lane was to model builtin parameter signatures in `std/` first. Reading
`std/` before authoring turned that into a different task, and the difference is worth recording
because it is the second time on this class that the obvious repair was the wrong one.

**`dag/std/primitive_identity.dag` already models it, and already executes.** It carries
`PrimitiveSignatureGrounding`, `PrimitiveSemanticContract`, `PrimitiveSignatureResolution` (whose
`SignatureResolved` arm is `{ parameters: List<AlgebraTypeTemplate>, result: AlgebraTypeTemplate }`),
`primitive_signature`, and `primitive_arity`, green by execution in
`dag/test/claim/primitive_signature_grounding_witness_test.dag`. Its own doc comment refuses the fork
this lane was about to commit, in as many words: *the contract carries a KEY into the one authority,
never its contents.* It even keys the lookup on `(canonical_name, profile)` rather than name alone,
precisely because `get` is `[ReceiverSelf, NamedTemplate { name: "Int" }]` on the List profile and
`[ReceiverSelf, ReceiverKey]` on the Map-shaped ones.

So there is no new carrier to mint, and authoring one would have been exactly the §3 nickname.
**What is missing is coverage, and it is measurable**: of `builtin_function_registry`'s 131 names,
20 have an `AlgebraFieldTemplate` row and resolve through `primitive_signature`; the other 111
answer `SignatureNotGrounded`. `parse_int` — the name that reds the corpus control — is one of the
111.

**The 111 are two populations, and nothing in the substrate separates them.** Some are language
primitives that should carry a signature (`parse_int`, `char_at`, `code_point`, `chars_to_string`,
`string_length`, `string_contains`, `scan_while`, `scan_to_eol`, `set_insert`, `set_union`,
`sorted_map_keys`, `hash_combine`). Most are host or lens transports whose parameter shape is a
Realization fact and not a language one (`doc_graph_orphan_count`, `fallback_arm_census_facts`,
`emit_host_run_transport`, `non_fold_residue_count`, `witness_layer_roots_compile_clean_check`) —
§3 puts those with their transport, not in the language's primitive surface.

The obvious discriminator does not discriminate. `gunbc.v1_interpreter_primitive_surface` enumerates
an arm for BOTH populations by construction, so joining on it classifies `doc_graph_orphan_count`
and `parse_int` identically — measured, not assumed. Splitting them on a naming convention instead
would be the smuggled heuristic §5 names, so this lane does not.

**The step-1 modeling question is therefore not "what shape does a builtin signature have" — that is
answered — but "what closes the language-primitive population", i.e. the denominator over which a
signature is obligatory.** That is a routing decision, and this lane has raised it rather than
picking one.

## The registry and the algebra rows already disagree

Independent of the `first` class, and worth its own row wherever primitive-surface debt is tracked:
the 20 overlapping names are two authorities for one operation's type, and they do not agree today.
`builtin_function_registry` is receiver-BLIND, so it collapses distinctions the algebra rows carry:

- `reverse` — registry `List<collection_element>`; algebra `ReceiverSelf` on both a scalar and a
  collection profile. On a `String` receiver the two answer with different types.
- `map_keys` and `map_values` — registry gives both `List<collection_element>`, one type variable
  for two different element positions; algebra gives `ContainerOf { .., element: ReceiverKey }` and
  `.. ReceiverValue`. For any `Map<K, V>` with `K /= V` the registry cannot be right about both.
- `concat` — registry `String`; algebra `ReceiverSelf`, so a list concat types as a String.
- `get` — registry one `Optional<collection_element>`; algebra distinguishes the List reading from
  the Map reading.

Every one of these is the same §3 fork as the `first` divergence, one layer up: not two
realizations of one declaration, but two declarations of one operation.

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

`HarmedNow` and `NotASite` are listed in full above. `Propagates` (36 sites) is rostered here at
identity grain; `AgreesUnderCompensation` (142) is the complement and its membership test is
mechanical — the occurrence is a `match` scrutinee, directly or through a `let` bound one line up.

### Propagates — returned onward as the enclosing function's `T?`

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
- `dag/gunbc/roadmap/roadmap_belt_actuate.dag` · `belt_dispatch_result_for_label`
- `dag/gunbc/roadmap/roadmap_closing_contract_authoring.dag` · `closing_contract_target_node`
- `dag/gunbc/roadmap/roadmap_execution_contract.dag` · `dispatch_host_realization`
- `dag/gunbc/roadmap/roadmap_style.dag` · `swatch_named`
- `dag/gunbc/roadmap/roadmap_validation_oracle.dag` · `validation_oracle_first_incomplete`
- `dag/std/target_representation.dag` · `checkpoint_row_migration_for`
- `dag/std/target_representation.dag` · `representation_spelling_for`
- `dag/test/claim/algebra_carrier_roster_witness_test.dag` · `ascii_least`
- `dag/test/claim/roadmap/roadmap_program_view_witness_test.dag` · `fx_line_view`
