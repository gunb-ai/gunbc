# The binder node (XL-2 PR2b-3)

Parent design: [service-interface-and-uses-carriers.md](service-interface-and-uses-carriers.md).

Rulings (side chat, 2026-10-01):
- A default belongs to the BINDER, never to its type.
- One binder representation (a required type, an optional default, closed labels, one reader and one builder) is shared by fn parameters, service io fields and record fields.
- For the representation, option A: a `Conj` told apart in the way `v2.std.type_binder` tells type-parameter binders apart. That needs no new connective, so no operator sign-off.

## The representation

A binder is still the Named edge `name -> target` inside an Arrow domain, io payload or record payload. Its **target** is now a **binder node**:

```
Conj {
  <binder-type>    -> T          (exactly one)
  <binder-default> -> v          (at most one)
}
```

- **Self-identifying labels.** Both labels are interned from lexemes no source can spell, as `type_params_marker` is. A binder node therefore identifies itself from its own labels, with no parent context and no guessing. No authored record field can collide with it: a field spelled `binder_type` interns a different symbol.
- **One reader.** `v2.std.node_query`: `binder_node_parts`, `declared_field_from_edge` (which now carries `default`), `member_edge_type_node` (now `Optional`), `find_binder_type`.
- **One builder.** `binder_edge`, `binder_edge_with_default`, `binder_relabelled`. 2b-1 (#12899) and 2b-2 (#12904) routed every reader and every producer through this pair, so the representation changed in these functions alone.
- **Fail-closed.**
  - A Named member whose target is not a binder node is **not** a binder. `declared_field_from_edge` answers Absent, `member_edge_type_node` answers Absent, and `find_binder_type` refuses with `^named_child_not_a_binder`.
  - The **binder wall**, `v2.std.type_binder` `arrow_domain_members_are_binders`, run by `type_binder_node_conforms` on every Arrow, refuses a hand-built domain member at the Arrow.
  - A record payload the wall does not reach still fails closed at its first reader. `type_decl_view` cannot tell a record type from a module body structurally, so the wall cannot be stated there without guessing.

## Walkers that would have misread a binder node, and what each does now

The inventory covered every Arrow, domain and payload reader outside `src/v2/test`. **None needed parent context** (the ruling's stop condition):

| Walker | Now |
| --- | --- |
| `v2.compiler.infer` formation (`infer_formation_facts_from_entries`) | A binder node has its own arm, `infer_binder_node_facts_from_entries`. Its derived type is a binder node over the type's derived type with **no default**, so defaults are not type identity, and derived types and source types share one binder shape. A binder's default is judged at the binder's declared type (`infer_binder_default_check`, position `PositionBinderDefault`; XL-2 PR2c-ii). |
| `v2.compiler.symbol_index_fill` `symbol_index_fill_containment_edge` | Indexes a binder at its **type** (`symbol_index_binder_type_or_target`), so a parameter or field path resolves to its declared type. `<binder-type>`/`<binder-default>` never become path segments. |
| `v2.compiler.translate` `translate_algebra` | A binder node translates **as its type** (`TypeExprTranslateBinder`, `translate_binder_node_step`). The default's fold result is discarded. A non-binder record member refuses (`translate_binder_member_type`). Arrow type expressions read parameter types through `find_binder_type`. |
| `v2.compiler.resolve` declaring path | Does not extend across `binder_node_label`s. |
| `v2.std.bounded_lattice_completeness` | Reads a binder field's shape from its type (`bounded_lattice_field_subject`). |
| Lenses (unit modeling, lifecycle, residency) | Through `member_edge_type_satisfies`. A non-binder answers no. |
| Type-name indexes (`node_query`, `concept_index`) | Through `member_edge_type_name`. A non-binder renders `<not-a-binder>`, never a fabricated name. |
| `target_model` parameter rendering | Through the reader. A non-binder refuses. |
| `lens/application` paths | A path that addressed a field's type now addresses its binder node. An edit replacing it with a bare type produces a hand-built binder, which the readers and the wall refuse. Loud, not silent. |
| Subtree-fold lenses (effect reach, determinism, cost, fn index, mandatory tag, decl-facts skeleton) | Will see a default as code once defaults exist. No default exists before 2c, which decides count-or-skip per lens. |
| Structural equality / anti-unification | Compare binder nodes structurally. Derived types carry no default, so two Arrows differing only in a default compare equal once defaults are judged in 2c. Nothing to change before then. |

## Expected content-hash churn (one-time, intended)

Every binder's target changes from `T` to `Conj { <binder-type> -> T }`. So the content hash (`v2.std.node` `content_hash`) of **every Arrow with a parameter, every record type with a field, every variant arm with a payload field, and every node containing one** changes once in this PR. The ruling anticipated this ("a claim-free Arrow keeps its identity except for an intentional one-time binder migration").

What moves with it:
- `v2.lens.interface_summary` `signature_fingerprint_of_node`. Every declaration with a parameter or field gets a new fingerprint. No API change is implied.
- `v2.lens.affected_set` `canonical_boundary_hash_receipt`.
- Any committed fixture or receipt that pins such a hash.

A nullary Arrow, an Atom type, and a payload-free variant keep their hashes.

## Defaults (XL-2 PR2c-ii)

- **Lowering.** Every default lowers onto its binder through `v2.compiler.body_lowering_fold` `body_lower_binder_edge_read` (`binder_edge_with_default`):
  - a fn or pattern parameter's from its own tail (`body_lower_typed_param_optional`);
  - a record field's and a service io field's from the field tail (`body_lower_io_field_default_optional`).

  The value lowers through the one value reader (`body_lower_value_read`) and refuses at the value under that reader's cause. `body_lowering_reason_default_value_unmodeled` has no producer and is deleted. A field's **wire key** is a realization fact and still refuses (records) or is set aside (io blocks) until the realization binding (XL-2 PR3).
- **Judgment.** `v2.compiler.infer` `infer_binder_default_check` judges the default at the binder's declared type through `infer_judge_declared_position`, at a fifth position, `v2.std.inhabitance` `PositionBinderDefault`. That is the same relation an argument meets at its formal. A non-inhabiting default refuses as `^infer_reason_default_does_not_inhabit_binder_type`, located at the default. `^infer_binder_default_unjudged` is retired.
- **Scope.** A default sits under the Arrow's domain, which resolve walks in the type-parameter frame over the OUTER scope, never the Arrow's own value parameters. So a default cannot read a sibling parameter.
- **Emission.** A target with no parameter or field defaults refuses a defaulted binder, located at the default (`^produced_decl_render_param_default_unrealized`, `^target_semantic_decl_field_default_unrealized`), rather than dropping it.
- **Subtree lenses.** A default is code that runs when its argument is omitted, so the subtree-fold lenses (effect reach, determinism, cost, fn index, mandatory tag, decl-facts skeleton) COUNT it as code of the declaration it sits in. No lens skips it.

## Declared frontier: omission at a call or a construction

A caller may not yet OMIT a defaulted argument, and a construction may not yet omit a defaulted field. Both bind through one binding plan, which still requires every formal, so an omitting call or construction refuses at the application or construct, loudly:
- calls: `v2.std.arrow_signature` `application_binding_plan`, used by infer and eval;
- record construction: `application_binding_plan_over`, used by `v2.compiler.infer` `infer_record_field_binding`.

**Trigger:** one shared binding plan that reads each formal's binder metadata (via `declared_field_from_edge`) and admits a defaulted formal as optional, together with default materialization, so that eval, and emission where the target admits it, supply the default for the missing slot or field. Until then a default is lowered, judged and carried, and an omitting site refuses.

## Scope and generics (side-chat review on #12948)

- A default resolves in the ENCLOSING value scope, with the declaration's type parameters only as its type scope (`v2.compiler.resolve` `resolve_arrow_domain_walk`). An outer value is visible; a sibling parameter is not; a type parameter's spelling does not resolve as a default value.
- A default under a generic declaration (an Arrow or type declaration with type parameters) refuses at the default as `^infer_binder_default_generic_unmodeled` (`v2.compiler.infer` `infer_generic_default_refusal`), until a default is judged with its declaring type parameters in scope.
