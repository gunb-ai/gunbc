# Construct tag as a declaration reference (qualified variant construction and patterns)

Status: PR1 of two (model and migration census). PR2 is the cut. Owner lane: lively-eagle-657, parent
gentle-koi-724, reviewer neat-boar-16.

## Symptom and the chain (DESIGN 6b)

`src/v2/compiler/01_tokenize.dag` (`v2.compiler.tokenize`) is file-refused at native ingest with
`body_lowering_reason_unsupported_form` at `v2.std.diagnostic.Rejected { diagnostics: .. }` in
`lex_walk_artifact`, so none of its declarations enter the index and every importer refuses
`tokenize` as unbound. That is the single root of the native seven. quiet-hawk-702 established this
by execution on D1: the native resolve walk names only `tokenize` for all seven, while explicit
imports of admitted files bind.

The chain behind it:

- **Unqualified `R { f: e }`** lowers through `v2.compiler.body_lowering_fold`
  `body_lower_try_record_literal` to `v2.std.node_query` `construct_node(tag: Symbol, ..)`, which
  builds `Conj { R: Atom(R), f: e }`. A fielded pattern lowers to the same shape.
- **Dotted head `a.b.R { .. }`** never reaches that route. `body_lower_record_literal_tag_optional`
  reads only one branded atom. The postfix read classifies the brace suffix as
  `PostfixChainSuffixCarried` and declines it. Its own comments name the reason: a construct tag
  is a `Symbol`, so a module-qualified tag has no representation.
- **The earliest unjustified boundary is therefore the construct carrier**, not body lowering.

The same boundary is already wrong on the unqualified path, one stage later. `v2.compiler.resolve`
`resolve_pattern_node_walk` and `resolve_node_walk` resolve the tag atom through `resolve_atom`,
then `resolved_reference_node`, into `declaration_reference_node(path)`. The edge label stays the
leaf `Symbol`. After resolve, the readers that key on "the label equals the atom's identity" answer
Absent for every user constructor: `construct_tag_optional`, `v2.std.node`
`arrow_body_record_construct_conforms`, and `v2.std.compilers.target_model`
`target_value_expr_record_construct_gate`. The construct shape holds only before resolve and in
hand-built fixtures.

## The model

The tag edge stops being a nickname and becomes a reference.

- **Construct** = `Conj { <construct_tag_marker>: <reference>, field edges.. }`.
  - `construct_tag_marker` is a STRUCTURAL CORE MARKER in the typed `EdgeLabel` vocabulary of
    #12473 (quiet-koi-814, ruling A; review condition from neat-boar-16). It names the edge's role
    (constructor tag); the target is an ordinary authored reference. It is counted once in the
    #12473 census. Landing order, agreed with quiet-koi-814: this lane's PR2 lands first, because #12473 is the
    model document only and its cut has not started. PR2 therefore declares the marker beside
    `declaration_reference_marker` in `v2.std.node`, in the same form. It also records the marker once, in
    the Core-marker list of `docs/plans/edge-label-coproduct/README.md` (item 1 under "Who owns the
    closed set") if #12473 has landed by then, or in #12701 otherwise. The #12473 cut converts it to
    the Core arm `ConstructTag` together with the other markers. It is distinct from
    `declaration_reference_marker`. If they were the same, a nullary `Rec {}`
    would read as a declaration reference under `declaration_reference_body_marked`.
  - The label is never the constructor's spelling. That removes the label-as-second-name (§3).
- **Reference form: one form, a qualified_name spine.** Before resolve, `<reference>` is the
  authored `qualified_name` spine built by `v2.std.qualified_name` `qualified_name_spine_of_atoms`
  from the authored segment atoms, so each segment keeps its token occurrence. An unqualified `R` is
  the one-segment case of that spine: `fold_list_node` over one atom is the two-edge head/tail Conj
  that `qualified_name_spine_shape_present` accepts. Readers do not branch on atom-or-spine.
  - The ambiguity of a bare spine (an integer literal is also a cons list) is discharged by the
    marker label, not by the spine's shape.
- **After resolve,** `<reference>` is `declaration_reference_node(path)` produced by
  `resolved_reference_node`, the same carrier every other resolved reference uses. The path is the
  VARIANT's own declaration path, never its owning coproduct (quiet-hawk-702's v1 trap). No new tag
  type; the shared identity stays the declaration path (`std.decl_ref DeclarationRef` in v1).
- **Lowering never resolves or trims.** The construct producer receives the whole authored path.
  Nothing takes the last segment anywhere on the route.
- **Resolution is one route.** A tag spine resolves in both walks through a single function that
  reads the path with `qualified_name_from_node` and dispatches by length:
  - two or more segments go to the existing `try_resolve_qualified_name_node`;
  - one segment goes to the existing bare door, `resolve_atom` on its only atom.

  Both end in `resolved_reference_node`. The one-segment case is needed because
  `try_resolve_qualified_name_node` answers Absent for length 1.
- **Nullary qualified values** (`v2.std.diagnostic.None`) are not constructs. They are dotted
  references, which already lower to the spine and resolve through `try_resolve_qualified_name_node`.
  PR2 adds a control proving they bind to the variant declaration. It changes that route only if
  the control goes red.
- **Patterns** use the same producer and the same reference form for the constructor head.
  `resolve_pattern_binders` keeps reading only the field edges.

## Reader and producer census, with each migration

Established by greps over `src/` and `dag/` (`*.dag` and `*.rs`) for `construct_node`,
`construct_tag`, `construct_field_edges`, `RecordConstructBody`, `record_construct`,
`TargetRecordConstructShape`, and the label-equals-identity comparison. No Rust code reads the v2
construct shape. `src/v1/stage0` `coproduct_reflection` emits the unrelated v1
`record_construction_spelling` edge.

| Module | Symbol | Role | Migration |
|---|---|---|---|
| v2.std.node_query | `construct_tag_edge`, `construct_node` | producer | Take the reference node (a spine) in place of `tag: Symbol`, and emit `Named{construct_tag_marker}` targeting it. |
| v2.std.node_query | `construct_tag_optional` | reader | Key on first-edge label == marker, and answer the reference node (a pre-resolve spine or a post-resolve marked reference). It no longer answers a Symbol. |
| v2.std.node_query | `construct_field_edges` | reader | Unchanged apart from the gate. |
| v2.std.node | `arrow_body_record_construct_conforms` | well_formed gate | First edge is the marker, targeting a spine or a marked reference, and every other edge is Named. Fixes resolved bodies being rejected today. |
| v2.std.node | `classify_arrow_body_form`, `declaration_reference_body_marked` | classifier | Classify a construct by the marker explicitly, not as "any unmarked Conj". |
| v2.compiler.body_lowering_fold | `body_lower_try_record_literal`, `body_lower_record_literal_tag_optional` | producer | Read the head as a qualified-name spine (bare or dotted) and pass it to `construct_node`. The brace suffix leaves `PostfixChainSuffixCarried`; the declined arm and its advisory for this suffix are deleted. |
| v2.compiler.body_lowering_fold | constructor-pattern lowering (the `construct_node` call in the pattern arm) | producer | Same: the spine from the pattern head. |
| v2.compiler.body_lowering_fold | `body_lower_is_core_substrate` | reader | Through the migrated `construct_tag_optional`. Check the order against the `qualified_name_spine_shape_present` test. |
| v2.compiler.resolve | `resolve_pattern_node_walk`, `resolve_node_walk` Conj arm | rewriter | Resolve the marker edge's spine through the one length-dispatching function above. Field edges walk as today. Check the `resolve_ctx_snoc_position` side effect now that the label is the marker. |
| v2.compiler.resolve | `resolve_pattern_binders` | reader | Unchanged apart from the gate. |
| v2.compiler.infer | Conj gather arms (`infer_gather_*`, `infer_product_facts_from_entries`) | reader | The marker edge must not be read as a product field. Skip it by its TYPED edge role, never by the label's spelling, and type the construct by the referenced declaration where a row exists. Today the tag is `infer_gather_fold_not_derived`. |
| v2.compiler.eval | `eval_callee_body_refusal_reason` | reader | Refusal only; no change. |
| v2.std.compilers.target_model | `target_value_expr_record_construct_gate`, `target_project_record_construct` | reader | Gate on the marker. `type_name` comes from the resolved declaration's path, spelled through the target's binding rows, not from an atom identity. This admits resolved constructs the gate refuses today; that is a behaviour change, and it is named. |
| v2.std.compilers.target_model | `target_value_expr_arrow_has_record_construct_body`, the arm body projection and match-arm slot readers | reader | Follow the gate. |
| v2.std.compilers.target_model | `TargetRecordConstructShape` decode; the `record_construct_form` rows in extdeps.languages rust, typescript, c and dag | token rows | No change. |
| v2.extdeps.languages.dag | `dag_rec_construct_body_node`, `dag_variant_holds_construct_node` and their users | fixture | Rebuild through `construct_node` with a spine. |
| v2.lens.machine_shape | `constructed_tag` | reader | Compare the resolved declaration path of `MachineShape`, not a leaf Symbol. |
| tests: `variant_field_lowering_test`, `wildcard_pattern_form_test`, `arrow_body_form_witness_test` and `arrow_body_form_semantic_helpers`, `record_construct_emit_test`, `rust_record_construct_emit_test`, `rust_record_construct_emit_host_equals_eval_test`, `reference_conservation_test` (occurrence counts), `value_position_whole_read_test`, `declaration_graft_assemble_test` | | test | Rebuild each fixture with the marker, and recount occurrences (segments keep their token occurrences). The unmarked reference-shaped negative must still go red. |
| recurring_failure_mode and rung_drop prose (`postfix_suffix_step_declined_to_the_pre_existing_arms`, `call_expression_erased_at_v2_body_lowering`), `docs/plans/lowering-occurrence-projection-design.md` | | prose | Update the prose. Retire the brace-suffix population of the rung drop by its trigger. |

Not readers of this shape: `v2.std.data_initializer_identity`, `v2.std.decl_facts_skeleton` and the
v1 `record_construction_census` fixtures all read the v1 reflection edge. The orchestration
`construct_tag:` fields are an unrelated name clash.

## One decision for a dotted path (ruled by gentle-koi-724, 2026-09-29)

Whether a dotted path is a qualified name or a field projection on a local is decided in ONE place:
`v2.compiler.resolve` `try_resolve_qualified_name_node`, by `qualified_head_bound_on_chain`, which is
the scope of the first segment. eager-newt-412's lane owns the projection arm there. This lane only
consumes that door. A construct tag whose resolved answer is not a constructor declaration (or a
kernel canonical atom) refuses `resolve_reason_construct_tag_not_a_constructor`. The door's projection arm is gunbc#12506's
`resolve_bound_head_projection`. The LOCAL wins: a bound first segment shadows a whole-path
declaration, so the bound-head check precedes any declaration lookup, and that change is made inside
#12506. Whichever of #12506 and this lane's PR2 lands second adds the control. It is one module with
`artifact.tree` (a match binder), `v2.std.diagnostic.Rejected { .. }` (qualified), and a local
shadowing a module segment.

## The live defect on main

Filed as `gunbc.recurring_failure_mode`
`construct_tag_read_by_label_equality_after_resolve_rewrote_the_target`. Confirmed by execution: a
resolved `PcrA { n: true }` is `Conj { PcrA -> <declaration reference> v2.test.pc_provider.PcrA, n -> true }`,
and `construct_tag_optional` answers Absent. The well_formed and target_model gate arms are inferred
from their definitions, and the row labels them that way.

## PR2 (the cut) and its evidence

All readers migrate in one motion, and no `Symbol`-tag path remains. Controls, fed by production:

1. `a.b.R { f: e }`, `a.b.C` (nullary) and `a.b.R { f: x } =>` each lower and resolve to the same
   node as their unqualified form under an import of the same declaration. This is a content
   equality check.
2. Two variants with the same leaf in different modules, constructed qualified, bind to different
   declaration paths.
3. Mutations go red: dropping the qualifier binds the other module's variant or refuses as
   ambiguous, and dropping the record body changes the node.
4. The variant is bound, not its coproduct.
5. A qualified tag whose last segment names a TYPE rather than a variant, or a variant of a
   different type, refuses at a located site and never resolves by spelling (neat-boar-16).
6. The target_model gate is shown by a discriminating pair: a resolved construct is admitted, and
   an unresolved or wrong-variant construct still refuses at that gate.
7. The seven's actual form, `v2.std.diagnostic.Rejected { diagnostics: .. }`, is the positive
   control on the production route.
8. The live defect below has a control that is RED on main (a resolved user construct read by
   `construct_tag_optional`), turns green in PR2, and stays enrolled as the regression control.
9. `01_tokenize.dag` is admitted on the native route. A base-vs-head census (the #12550 recipe)
   shows the newly admitted files and no unexplained new refusal. quiet-hawk-702 runs the native
   seven against the head.
