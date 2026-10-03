# Typed EdgeLabel coproduct — model proposal (step 1 of 2)

Status: model approved and landed (gunbc#12473). Step 2, the cut, is the follow-up PR from quiet-koi-814; its scope was ruled by gentle-koi-724 (see "Step 2 as landed" below).
Climb on record: `gunbc.recurring_failure_mode` `behavior_named_edge_label_validated_not_constructed` (the next-rung trigger names "edge labels ... a closed coproduct owned by `v2.std.node`"). No parallel RFM row is filed. The text-matching readers below are a second instance of that class, and the step-2 PR adds their evidence to that row.

## Defect

`v2.std.node` `EdgeLabel = Named { name: Symbol } | Positional`. Grammar-structural labels (for example `^dag_surface_module_header`, `^arrow_body_edge`, `<cast-target>`) and AUTHORED names (a field, a named actual, a declaration name) share one `Symbol` identity space. A reader that asks "is this the module-header edge?" can only compare text, and an authored name spelled the same way inhabits the answer. The invalid state is constructible, and today only `v2.std.type_binder` `type_binder_labels_conform` refuses it, for behavior nodes (rung 3), while four readers are not guarded at all:

| reader | module | reads |
|---|---|---|
| `dag_surface_module_header_metadata_edge` | `v2.extdeps.languages.dag` | `Symbol ==` two header symbols; its callers bind `Named { name }` first |
| `dag_node_is_module_root_conj` (fallback arm) | `v2.extdeps.languages.dag` | `sym == ^dag_surface_module_header` |
| `d1_edge_names_where_clause` | `v2.test.claim.parse.d1_declaration_grammar_parse_test` | `n == ^dag_surface_where_refinement_clause` |
| `site_is_import_syntax_mention` | `v2.lens.module_graph` | `contains(site.position, ^dag_surface_import_*)`; the path is a Symbol list that already mixes production names with authored names (`v2.compiler.reference_site_collector` `reference_sites_in_edge`) |

## Proposed shape

```
// v2.std.node
type EdgeLabel
  = Structural { label: StructuralEdgeLabel }   // constructed only by a grammar / lowering authority
  | Authored   { name: Symbol }                 // carries the source name
  | Positional
```

`Named { name: Symbol }` does not survive beside these arms (DESIGN §3 replacement migration). Step 2 deletes it first and fixes forward.

### Who owns the closed set (ruled: A)

Candidate structural labels come from two layers:

1. **Core substrate markers**, owned by `v2.std.node` and `v2.std.type_binder`: arrow body and signature order, loop bound and carrier, loop domain (`^loop_domain_edge`, #12548) and the MQ-5 Loop head-reference edge `^loop_realized_declaration_edge` (Core arm `LoopRealizedDeclaration`; at most one, never on for/while; deep-raven-602 S3), match arm pattern and body, cast target, type annotation, type params, type body, type alias, the `v2.std.node_query` projection markers, and `declaration_reference_marker`. This set is language-independent and closed today, so it can be a closed coproduct in `v2.std.node`: `CoreEdgeLabel = ArrowBody | ArrowSignatureOrder | LoopBound | ...`.
2. **Grammar production edges**, per language: `dag_surface_*`, `rust_*`, `ts_*`, `target_model_edge_*`, and the per-language `*_named_edge` helpers that build them. These belong to the language rows in `extdeps/languages/`. Enumerating them in `v2.std.node` would be a layer inversion (DESIGN §3: each upstream has its own authority, and a generic hub may not enumerate products). Adding a language would also widen a core enum.

Recommended (A):
```
type StructuralEdgeLabel
  = Core { marker: CoreEdgeLabel }                      // closed, v2.std.node
  | Production { grammar: GrammarRef, edge: Symbol }    // minted only by the grammar fold from a declared row
```
Here `Production` is closed per grammar by construction. Its only constructor is the grammar's own projection (`v2.std.grammar` / `v2.compiler.02_parse` `parse_tree_projection_edge`, already a closed roster, is the precedent). Readers compare `Production` values minted from the same row, not `^text`.
Alternative (B): one flat closed enum of every candidate in `v2.std.node`. Rejected for the layer inversion above.
**Ruled (neat-boar-16, 2026-09-28): A.** Core markers live in `v2.std.node`; `Production { grammar, edge }` can be minted only by the grammar fold. Rung 4 is claimed only with a discriminating RED enrolled on the real acceptance path. Otherwise the claim is rung 3.
Open for the cut: whether `Production.edge` can be a row reference rather than a Symbol. That depends on `v2.std.grammar` exposing a row identity type. If it cannot, `Production` is rung 3 per grammar (it is checked when the grammar is admitted), not rung 4.

### Other judgment calls flagged for review

- **Hand-built type-node literals** (`^magma_field_op` in `v2.std.algebra`, `effects`, `testgen`, `target_model`, and others) model the AUTHORED field names of modeled records but are spelled like markers. **Ruled: `Authored`.** If those nodes are meant to equal what ingest produces, their names are already wrong (`magma_field_op` ≠ `op`). That is a separate finding.
- **`ReferenceSite.position`** must carry `EdgeLabel` segments, or only Authored segments plus a separate structural context. This is a model change to `v2.compiler.reference_site_collector`, not a rename. **Ruled: it lands in the cut**, because `site_is_import_syntax_mention` is a consumer that needs it.
- **Pending question (adhoc-db43a2ff-e50):** whether the namespace-spine mark belongs in this coproduct as a `Core` marker. The rule is the same as for the other markers: it joins `CoreEdgeLabel` only if it is language-independent and minted by the substrate, not by a grammar row.
- **Consistency with the Loop edge roster (#12548 and MQ-5 S3, deep-raven-602):** the loop domain edge and the Loop head-reference edge are `Core` markers, added ONCE, so the Loop edge roster and this census agree. Whichever PR lands second adds the label to the other roster. If this model lands first, S3 constructs the label through the `Core` arm and never as an authored Symbol. `v2.std.node` `loop_edge_role` (#12548) is the one reader of Loop labels, so it is the single site the cut converts.
- **Symbol-keyed query APIs** (`v2.std.node_query` `find_named_child`, `named_child_lookup`, `named_edge_target_lookup`, and `v2.std.node` `name_occurrences`) split into a structural lookup that takes a `StructuralEdgeLabel` and an authored lookup that takes a `Symbol`. A single Symbol-keyed lookup must not survive.
- **named-args PR-B #12382** (vivid-ram-65): the Named edges for named actuals under Transform are `Authored`. `PositionalPlusOneNamedEdges` for Transform then counts `Structural { Core { CastTarget } }` separately from the authored actuals.

## Consumers of each new declaration (DESIGN §3c)

Every declaration below lands in the step-2 cut together with its consumers. None of them lands in this model PR.

| declaration | consumer, executing route |
|---|---|
| `EdgeLabel.Structural` / `Authored` (replacing `Named`) | every current `Edge.label` reader and constructor. The fail-closed deletion refuses each one until it is dispositioned. `v2.std.node` `content_hash` hashes both arms. |
| `StructuralEdgeLabel.Core` / `CoreEdgeLabel` | `v2.std.node` edge discipline (`kind_edge_discipline`, `arrow_signature_order_label`, the loop bound/carrier checks) and `v2.std.type_binder` `type_binder_labels_conform`, whose behavior-label arm dissolves |
| `StructuralEdgeLabel.Production` | the grammar fold (`v2.compiler.02_parse` `parse_tree_projection_edge`, `v2.extdeps.languages.dag` productions) mints it. `dag_surface_module_header_metadata_edge`, `dag_node_is_module_root_conj` and `d1_edge_names_where_clause` read it. |
| `ReferenceSite.position` segments carrying `EdgeLabel` | `v2.lens.module_graph` `site_is_import_syntax_mention` |
| split structural/authored lookups | callers of `v2.std.node_query` `find_named_child` and its siblings |

## Census

[census.md](census.md) keeps only the findings the ruling was made over: the four readers and the non-EdgeLabel `Named` types that are excluded (`std.algebra` `ContainerSource`, `extdeps.formats.spice`). No counts are transcribed (DESIGN §6). The authoritative consumer population is the step-2 deletion itself (DESIGN §3, "the deletion is the census"): removing `Named` makes every dependent refuse under self-host and neat-boar-16's srv1 per-file native census, and each refusal gets one disposition: Structural/Core, Structural/Production, Authored, or arm-only. The largest dependent populations are the per-language `*_named_edge` helpers and the Symbol-keyed lookups in `v2.std.node_query`. Rust `src/v1/stage0` has no hand-written EdgeLabel sites; the generated reflection regenerates.

## Identity impact (a deliberate change)

`v2.std.node` `canonical_hash_of_edge_label` gains distinct tags for Structural/Core, Structural/Production and Authored. `Positional` keeps its tag. `label_sort_key` (canonical order) follows. Churn:

- **Pinned digests:** in `node_hash_protocol_witness_test`, every vector with a named edge changes. The positional and no-edge vectors stay stable, which is the control that only the label arm moved.
- **SCM:** every `object_store` object id derived through `content_hash_of_children` changes. The JSON wire `encode_label` / `decode_named_label` splits its `"named"` tag. No committed fixture carries a `"named"` label.
- **Hermetic fixtures keyed by a content-hash locator** change. Their on-disk store is still to be located before step 2.
- **Invalidate only, nothing pinned:** the test-claim cache digest (`05_eval`), the parse memo grammar digest (`02_parse`), and the bootstrap closure hash.

## Step-2 controls (the cut)

1. For each of the four readers, an authored name spelled like its structural label (for example an authored `dag_surface_module_header`) reaches none of them.
2. The real structural labels still route (positive control over the real ingest path).
3. A mutation that re-keys a reader on text makes the control red.
4. Gating: base-vs-head over shape consumers, neat-boar-16's srv1 per-file native census, and self-host.

## Step 2 as landed

Arm names differ from the sketch above because `.dag` names are corpus-global and `Structural`, `Production` and `Core`-prefixed spellings already name other types: the arms are `Authored { name }`, `StructuralLabel { label: StructuralEdgeLabel }` and `Positional`, with `StructuralEdgeLabel = CoreMarker { marker: CoreEdgeLabel } | ProductionEdge { language, edge }`.

- **Root first.** `Named { name: Symbol }` was deleted and every site renamed to `Authored` in the same change; the compiler's exhaustiveness refusals were the census of every reader that had to decide what a structural edge means at its site.
- **Converted, every minter and reader:** the core markers (arrow body and signature order; loop bound, carrier and domain; match arm pattern and body; cast target, type annotation, type params, type body, type alias; declaration reference; field projection base and field) and the dag surface productions the four readers need (module header and its qualified name, where-refinement clause, import decl, import block, import decl block, import decl qualified name). `v2.std.node` `Path` steps and `ReferenceSite.position` carry whole labels.
- **Deviation, stated:** the positional-payload field `"0"` stays Authored. It is consumed as `DeclaredField.name` like every authored field, so making it a marker would fork the field-name authority.
- **Frontier:** grammar-emitted labels no converted reader matches stay on the Authored arm. The population (by minting declaration) and the capability trigger are recorded on `gunbc.recurring_failure_mode` `behavior_named_edge_label_validated_not_constructed`.
- **Identity:** an authored label keeps the tag every Named edge had, so only nodes holding a structural label change identity. The two pinned vectors in `test.claim.node_hash_protocol_witness` that carry converted labels were re-derived by an independent reference implementation that first reproduced their previous values.
- **Coordination:** the construct tag (#12714) landed first, so this cut converted it to `ConstructTagEdge`. `^loop_realized_declaration_edge` (#12550) is not on main; whichever lands second adds its arm.
