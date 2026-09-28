# EdgeLabel `Named` census — scoping findings, not an instrument

This file keeps only the FINDINGS that need a ruling. The one-off lexical scan that located them is not committed, and neither are its counts: the repo admits no `*.py`, and DESIGN §6 forbids transcribing an instrument's output. The step-2 census is the fail-closed one DESIGN §3 names. Deleting `Named { name: Symbol }` first makes every real dependent refuse loudly, under `gunbc test //gunbc/instruments:self-host` and neat-boar-16's srv1 per-file native census. The dependents that refuse are the population the cut is scoped over. Symbols are cited by module and name.

## Premise correction: other types with a `Named` arm

Only three types in the corpus declare a `Named` variant:
- `v2.std.node` `EdgeLabel` (the subject)
- `std.algebra` `ContainerSource = SameAsReceiver | Named { name: String }`: string-method rows in `dag/std/algebra.dag`. Excluded.
- `extdeps.formats.spice` `Named { node: NamedNode }`: spice.dag and its fixtures and tests. Excluded.

`target_model.dag`, `testgen.dag`, `effects.dag`, `algebra.dag` (src/v2/std), and `body_lowering_fold.dag` have NO other `Named` type. All of their `Named {` hits are EdgeLabel (they import `Named` from `v2.std.node`).

## The four named readers

| reader (actual symbol) | file | how it reads | disposition |
|---|---|---|---|
| `dag_surface_module_header_metadata_edge(name: Symbol)` (asked-for "header_metadata_edge") | src/v2/extdeps/languages/dag.dag | Symbol equality against `^dag_surface_module_header`, `^dag_surface_module_header_qualified_name`; callers in symbol_index_fill, namespace_graft, 03_name_resolve, reference_site_collector `reference_sites_in_edge`, 03_resolve and zero_metadata_test — every caller first binds `Named { name: sym }` | becomes Structural arm match: take `StructuralEdgeLabel`, match the two arms; callers match `Structural { label }` instead of binding a Symbol |
| `dag_node_is_module_root_conj` | src/v2/extdeps/languages/dag.dag | fallback arm folds children: `Named { name: sym } => sym == ^dag_surface_module_header` | becomes Structural arm `dag_surface_module_header` (Authored arm → false) |
| `d1_edge_names_where_clause` (asked-for "edge_names_where_clause"; it lives in **src/v2/test/claim/parse/d1_declaration_grammar_parse_test.dag**, not module_graph.dag) | test | `Named { name: n } => n == ^dag_surface_where_refinement_clause` | becomes Structural arm `dag_surface_where_refinement_clause`; production minter is `v2.compiler.body_lowering_fold` (`label: Named { name: ^dag_surface_where_refinement_clause }`) |
| `site_is_import_syntax_mention(site: ReferenceSite)` | src/v2/lens/module_graph.dag | `contains(site.position, ^dag_surface_import_decl / ^dag_surface_import_block / ^dag_surface_import_decl_qualified_name)` — no Named match at all; `ReferenceSite.position` is a Symbol path mixing production and authored segments (`v2.compiler.reference_site_collector` `reference_sites_in_edge`) | requires `ReferenceSite.position` to carry label arms; then becomes a Structural-arm membership test. Not a mechanical rename. |

Related closed roster already present: `v2.compiler.02_parse` `parse_tree_projection_edge`. It is the precedent for `StructuralEdgeLabel` being a closed coproduct.

