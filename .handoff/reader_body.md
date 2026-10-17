# std.qualified_name: the ONE declaration-keyed reading of a type node

Part of the cardinality-absence program (coordinator deep-hawk-258), node `adhoc-1f73ff56-7b6`.
Stacks on #14225 (`optional_declaration_identity`, which this reads).
Plan approved by tidy-ant-251 (conditions a–d in `msg_cc6838c0`, accepted in `msg_7daf158c`).

## Defect

`dag/std/optional.dag:16` — `match left`, where `left: Optional<T>` — is the largest native-seven infer
root: a fan-out of ~144 modules (131 measured at #14087's head). The `T?` / `Optional<T>` sugar is
rewritten in resolve (#13990) into an **Occupancy** node, so by the time v2 infer reads the scrutinee it
is no longer a declaration reference. `infer_match_coproduct_of_type` read only declaration references,
so it refused the scrutinee as not-a-coproduct, and every module reachable through that root failed with
it.

## The ruling this honours

`gunbc.design_ruling optional_sugar_is_declaration_bound`: an Occupancy node must **denote** the
`std.optional.Optional` declaration wherever a declaration-keyed consumer reads it. That is a property of
the v2 Occupancy model and the Occupancy→declaration link, **not** a special case in the match checker.

So: **no `Occupancy` arm was added to `infer_match_coproduct_of_type`.** Its shape logic (Disj ⇒ read the
coproduct, Atom ⇒ maybe uninhabited, anything else ⇒ not declared, arity check over the declared type
params) is untouched. What changed is the one line that asks the type node which declaration it denotes:
the hand-read of a reference node became a call to the shared reader. The checker has no arm for any
type-node shape, and none is wanted, because an arm here would be a second place the sugar's meaning is
stated.

## The change

One shared reader in `src/v2/std/qualified_name.dag`, beside `optional_declaration_identity()` from
#14225:

```dag
type DeclarationDenotation { declaration: QualifiedName, type_arguments: List<Node> }
fn declaration_denotation_of_type(t: Node) -> Optional<DeclarationDenotation>
fn positional_slot_targets(node: Node) -> List<Node>     // the shared head ++ args slot walk
```

It answers, for any type node, "which declaration does this denote, with which type arguments?" Three
node forms, one answer:

| node form | reading |
| --- | --- |
| marked / bare `DeclarationReference` | `declaration_reference_path_optional`, unchanged semantics |
| `Instantiation` over a reference head | the head's path + the positional argument nodes (`positional_slot_targets`, moved verbatim out of `resolve_type_head_and_args`) |
| `Occupancy` — the Optional sugar | `optional_declaration_identity()`, the single QualifiedName from #14225, with the Occupancy payload as its sole type argument |

Per condition (a) the answer is a **QualifiedName identity**, never a bare spelling, and per condition
(b) `v2.std.qualified_name` is the **one** authority for Optional's declaration identity — #14087's
`optional_declaration_ref` row reads it from there (agreed with lively-fox-74, who is rebasing #14087
onto this stack). There is no second copy in the tree.

`optional_declaration_identity()` is exported so the reader and consumers can use it; nothing else about
the identity moves.

## Per-call-site table (condition c)

Census of every production call of the hand-read `declaration_reference_path_optional(node:` in `src/v2`
and `dag` (test/claim files and `recurring_failure_mode.dag` excluded): **60 at main, 48 at this head**.
14 production sites were converted; 2 new sites are the reader's own internals in `qualified_name.dag`
(which grows 1 → 3 because it now *hosts* the reading, it does not use it).

### Converted — the consumer was asking "which declaration does this type denote" (14)

| file:line (at this head) | function | what it reads |
| --- | --- | --- |
| `compiler/infer.dag:1198` | `infer_projection_receiver_of_type` | a projection's receiver type |
| `compiler/infer.dag:2915` | `infer_match_coproduct_of_type` | **the match scrutinee** — this defect |
| `compiler/infer.dag:4317` | `infer_node_mention_contested` | the mentioned type |
| `compiler/infer.dag:4329` | `infer_type_alias_unfolded_at` | the alias target |
| `compiler/infer.dag:4417` | `infer_kernel_value_type_optional` | the kernel value type |
| `compiler/infer.dag:4488` | `infer_type_declaration_ref` | a declared type reference |
| `compiler/infer.dag:4578` | `infer_string_literal_at` | the literal's target type |
| `compiler/infer.dag:4657` | `infer_produced_open_type_params` | produced open type params |
| `compiler/resolve.dag:4045` | `resolve_instantiation_argument_kinds` | instantiation argument kinds |
| `compiler/resolve.dag:4127` | `resolve_type_application_refusal_optional` | the applied **head**, read through the reader |
| `compiler/resolve.dag:4193` | `resolve_kind_inhabitance` | the kinded declaration head |
| `compiler/resolve.dag:5242` | `resolve_optional_application_contract_refusal_optional` | the whole **application** node |
| `std/inhabitance.dag:283` | `inhabitance_declaration_head` | the inhabitance head (file now has **0** hand-reads) |
| `compiler/fold_lowering.dag:1277` | `fold_collection_member_type` | the collection member type |

Per-file hand-read movement, main → head: `infer.dag` 18→11, `resolve.dag` 10→6,
`fold_lowering.dag` 3→2, `std/inhabitance.dag` 2→0, `std/qualified_name.dag` 1→3 (reader internals).

### Kept on the hand-read — authored spelling or node shape, not denotation (48)

These ask **how the source was written**, or walk a node's shape for a non-denotation purpose; converting
them would change meaning. Named by file (function) so review can check the judgement:

- **constructor / arm / pattern spelling** (13): `infer.dag` `infer_declaration_reference_facts`,
  `infer_variant_arm_reference_value_optional`, `infer_nullary_zero_param_coproduct_arm_use`,
  `refinement_declaration`, `infer_coproduct_arm_pattern_constructed`, `infer_sole_constructor_head_path`;
  `resolve.dag` `resolve_construct_walk_with_tag`, `resolve_construct_tag_names_constructor`,
  `resolve_node_walk_entered`, `resolve_requirement_member_refusal_optional`;
  `target_serialize.dag` `nullary_coproduct_arm_binding_spelling`; `std/node_query.dag`
  `construct_tag_path_optional`; `std/arrow_signature.dag`
  `arrow_signature_order_head_is_list_introduction`.
- **callee / call-target identity — not a type at all** (13): `infer.dag`
  `infer_application_callee_is_primitive_authority`, `infer_callee_declaration_from_declarations`,
  `infer_callee_is_roster_primitive_call`, `infer_record_construct_read`, `infer_gather_fold_init`;
  `eval.dag` `eval_node_is_callee_reference`, `eval_callee_declaration_optional`, `eval_transform_node`
  (×2), `eval_constant_member_body`; `name_resolve.dag` `closure_collect_declaration_callees`;
  `service_realization.dag` `argv_is_concat_operator`;
  `lens/complexity_accumulator_copy/analyze.dag` `lowered_callee_path_optional`;
  `std/list_introduction.dag` `list_introduction_elements_optional`.
- **census / lens / reference-graph tooling over authored text** (11): `type_declaration_use_census.dag`
  `type_ref_text`, `where_clause_references`, `as_callee_param_type`, `module_referenced_paths`;
  `dependency_demand_census.dag` `demand_fact_node`, `demand_census_requirement_members`,
  `demand_census_authored_reading`, `demand_census_reference_path_optional`; `reference_closure.dag`
  `reference_closure_walk_ownership`; `emit_semantic_decl.dag` `produced_type_binding_over`;
  `std/compilers/semantic_decl_emission.dag`
  `target_semantic_decl_field_surface_from_edge_otherwise`;
  `std/compilers/target_model.dag` `target_value_expr_record_construct_type_name_optional`.
- **fold iteration shape** (2): `fold_lowering.dag` `fold_realized_iteration_direction`,
  `fold_realized_member_role`.
- **the writer of the sugar** (1): `resolve.dag` `resolve_type_application_normalized` — this is the
  function that *produces* the Occupancy node from an authored `T?`; it must see the authored head, so it
  reads the reference itself.
- **`compiler/translate.dag:1304` `translate_algebra`** (1) — kept, and the reason is not "out of scope".
- **the reader's own internals** (3): `std/qualified_name.dag` `declaration_reference_path_optional` plus
  the two reference reads inside `declaration_denotation_of_type`.

If any kept site is in fact a type-denotation read, that is the same miss left in place and review should
say so; the table exists precisely for that check.

### Why `translate.dag` is the one that cannot be converted

The Occupancy connective is **also** the target-model collection carrier:
`target_collection_type_node_is_set_carrier` (`std/compilers/target_model.dag:12290`) reads the same
wrapper's slot for `^target_collection_set_carrier`, and those carriers are minted in
`std/collections.dag:1249/1253`. A blanket "every Occupancy denotes `std.optional.Optional`" at the
translate fold would emit a `std.optional.Optional` type expression for a Set carrier. That is the
concrete reason the fix is a **reading boundary that consumers opt into**, not a global rewrite of the
node: each declaration-keyed consumer asks the reader, and the carrier path is untouched. It is also
evidence for the retirement note below being stated honestly rather than aspirationally.

## Occupancy's retirement

Occupancy is on the retirement list — the sugar should eventually dissolve into the declaration. This PR
**does not grow it**: no new field, no new construction site, no new predicate over its shape, and the
Occupancy arm of the reader is three lines that consult `optional_declaration_identity()` and pass the
payload through as the sole type argument.

What it adds is exactly one edge *out* of Occupancy, and it routes every declaration-keyed question
consumers ask through that single edge. That is the shape of the retirement: once every declaration-keyed
reading goes through `declaration_denotation_of_type`, deleting the Occupancy node means deleting **one arm
of one function** rather than auditing the tree. The remaining load keeping Occupancy alive is the
target-model Set-carrier coupling named above, and this PR makes that coupling explicit instead of
implied — which is the thing that has to be dismantled for the node to go.

## Evidence

### Controls — executed on the native route (`--claim-run`), `src/v2/test/claim/optional_match_denotation.dag`

| row | before (identity head, no reader) | after (this head) |
| --- | --- | --- |
| `omd_a_match_on_an_optional_parameter_is_accepted` — the **RED** | `<!-- -->` | `<!-- -->` |
| `omd_the_arms_are_checked_against_absent_and_present` | `<!-- -->` | `<!-- -->` |
| `omd_a_non_coproduct_scrutinee_still_refuses` — control | `<!-- -->` | `<!-- -->` |
| `omd_an_authored_coproduct_still_types` — control | `<!-- -->` | `<!-- -->` |

The before state is produced inside the same dispatch and against the same binary by dropping the five
reader-touched files to their identity-head content, so the two columns differ by exactly the reader.

### Identity claim, re-run at this head

<!-- -->

### Fan-out re-measure of `dag/std/optional.dag:16` at this head

`gunbc test //gunbc/instruments:v2-native-census` — the emitted compiler's own `census-infer` verb
(`prepare_emitted_compiler`, then `census-infer`), which is what ranks `native_census_roots_ranked` by
`fan_out_count`:

```
BEFORE: <!-- -->
AFTER:  <!-- -->
```

### Native-route evidence — executed

`gunbc test //gunbc/instruments:v2-native-cli`, the native compile of the `v2.cli.compile_cli` closure,
`GITHUB_ACTIONS=true`, build and run in one dispatch:

```
<!-- -->
```

### Per-entry resolve probe, full pool before/after (condition d)

All 59 unique `--entry` paths in `.github/workflows`, each probed with
`gunbc run --entry X --function zz_no_such_fn` — `NoSuchFunction` means the entry **resolved**, so the
probe catches a resolve change breaking modules outside native-seven's closure. Run in both states:

```
BEFORE: <!-- -->
AFTER:  <!-- -->
```

## Files

`src/v2/std/qualified_name.dag` (the reader + the slot walk), `src/v2/compiler/infer.dag` (8),
`src/v2/compiler/resolve.dag` (4), `src/v2/std/inhabitance.dag` (1),
`src/v2/compiler/fold_lowering.dag` (1), `src/v2/test/claim/optional_match_denotation.dag` (new).

No `Occupancy` arm in the match checker. No growth of the Occupancy node.
