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
| `v2.compiler.infer` formation (`infer_formation_facts_from_entries`) | A binder node has its own arm, `infer_binder_node_facts_from_entries`. Its derived type is a binder node over the type's derived type with **no default**, so defaults are not type identity, and derived types and source types share one binder shape. A binder that carries a default refuses as `^infer_binder_default_unjudged` until a default judgment exists (2c). |
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

## Out of scope (2c and later)
- **Defaults.** Lowering a default, judging that it inhabits its binder's type, choosing its resolution scope (the outer scope, never the binder's own parameters), and each subtree lens's count-or-skip decision all land with the first producer of a default.
