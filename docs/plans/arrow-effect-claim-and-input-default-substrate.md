# Arrow effect claims and input defaults: substrate model (XL-2 PR2a, DRAFT for ruling)

Parent design: [service-interface-and-uses-carriers.md](service-interface-and-uses-carriers.md) (#12851).
That document placed operation modifiers as "edges on the operation node" and io defaults "on the payload field". Neither is admitted today:
- `v2.std.node` `arrow_signature_edges_conform` counts every Arrow named edge other than the body and the declared order as the single type-binder edge;
- `v2.std.type_binder` `arrow_named_labels_conform` allowlists only the body, the order and the type-params marker;
- a payload record's fields are plain Named edges, so they have no slot for a default.

This draft is the substrate change that would admit them. The XL-2 manager leans toward option A (a modifier is an Arrow fact, a default is an input fact). The ruling is to be made on this diff. Nothing lowers onto these edges yet: lowering is PR2b, and it starts only after this ruling.

## The two edges

| Edge | Target shape | Kind | In type identity? | In content hash? |
| --- | --- | --- | --- | --- |
| `^arrow_effect_claim_edge` (≤1) | Conj of self-named childless Atoms, distinct (`readonly`, `idempotent`, `hermetic`; closed by the grammar `v2.extdeps.languages.dag` `dag_grammar_op_modifier_expr`) | LABELS (metadata, like the declared order) | **ruling point R3** (draft: no) | yes, sorted with the other named edges by `canonicalize_arrow_labeled` |
| `^arrow_input_default_edge` (≤1) | Conj of Named edges, distinct labels, each an input field of the Arrow; targets are value expressions | VALUES (resolved and inferred) | **no**: two arrows differing only in defaults have one type | yes, so `v2.lens.interface_summary` `signature_fingerprint_of_node` moves when a default changes, which is the intended API signal |

The conformance in this diff:
- `v2.std.node` `arrow_named_edge_is_non_binder`: the one predicate that names the non-binder edges, used by both counts in `arrow_signature_edges_conform`.
- `arrow_effect_claims_conform`.
- `arrow_input_defaults_conform`.
- `arrow_input_field_labels`.
- `v2.std.type_binder` `arrow_named_labels_conform`, extended.

Test: `v2.test.claim.arrow_effect_claim_and_input_default_conformance`. Positive controls are the bare operation arrow, claims, and defaults. Reds are:
- a repeated claim;
- two claim edges;
- a claim atom that is not its own label;
- a default for an undeclared field;
- a field defaulted twice;
- two default edges;
- a positional default.

## Ruling points

- **R1: what an "input field" is.** An operation takes ONE input of a record type (`v2.compiler.body_lowering_fold` `body_lower_operation_params`: one binder `^operation_input` targeting the payload). Its omittable fields are one level below the domain. The draft reader answers "the payload's fields when the domain is exactly `^operation_input`, else the domain binders", which puts the label `^operation_input` into `v2.std.node`. The alternative is to key the default edge on the domain binder and nest a per-binder Conj of field defaults. That keeps `v2.std.node` operation-agnostic, but the edge becomes two levels deep.
- **R2: the shared structural predicate.** The claim Conj and a type-binder Conj have the identical shape (`type_binder_conforms`: a self-named childless Atom), so the draft reuses `type_binder_conj_conforms`. Before this edge was exempted, a claim edge on a non-generic Arrow would have passed the wall *as type parameters*: the shapes really do coincide. Either rename the predicate role-neutrally (one structural fact, two roles, as `v2.std.type_binder` already describes binders), or keep the reuse.
- **R3: is an effect claim part of the Arrow's type?** In the draft it is metadata: it rides unchanged and does not enter `v2.compiler.infer` `infer_product_child_evidence_edges`, so a `readonly` and a plain operation of the same shape have one type. Making claims part of the type is the effect-typing direction (an arrow `A -> B` declared readonly would no longer inhabit a position expecting an unconstrained arrow, or vice versa). That is a larger decision than XL-2 needs, and the draft does not take it.

## Consumers that see the edges (audit of every Arrow reader outside `src/v2/test`)

Changes owed by PR2b (lowering) or by this PR once ruled. "Metadata" means: treat it exactly as the declared-order edge is treated.

| Reader | Today with an unknown named edge | Effect claim | Input default |
| --- | --- | --- | --- |
| `v2.std.node` `arrow_signature_edges_conform` | counted as the binder (refuses or is mis-read) | **this diff** | **this diff** |
| `v2.std.type_binder` `arrow_named_labels_conform` | refuses | **this diff** | **this diff** |
| `v2.std.type_binder` `node_inferred_subtree_nodes` | walked as inferable | skip (metadata) | keep walking |
| `v2.compiler.resolve` `resolve_arrow_node_in` | resolved under the body scope | carry unwalked (metadata arm) | resolve under the *outer* scope (a default may not read the arrow's own params), against the field's declared type |
| `v2.compiler.infer` `infer_gather_fold_step` / `infer_arrow_signature_order_edge` | inferred as a value child | add to the skip predicate | keep inferring; add a default-inhabits-field judgment beside `infer_arrow_declared_return_check` |
| `v2.compiler.infer` `infer_product_child_evidence_edges` (Arrow **type identity**) | its type enters the Arrow's derived type | metadata (rides) or drop (per R3) | **drop**, so defaults are not identity |
| `v2.std.arrow_signature` `application_binding_plan` | refuses when a label is unbound | none | a defaulted field may be absent from the actuals |
| `v2.compiler.eval` `eval_bind_arrow_params` | an unbound label refuses | none | evaluate the default for a missing field |
| `v2.compiler.translate` `translate_grounding_derived_gate_subtree` | requires grounding for every subtree node | exempt (metadata) | fine once inferred |
| `v2.std.compilers.target_model` `produced_decl_ordered_params` | via the order reader | via `v2.std.node` | render the default where the target admits one, else refuse located |
| `v2.compiler.body_lowering_fold` `body_lower_rename_fresh_type_variables` | walks non-body named edges | harmless | harmless |
| `v2.std.node` `content_hash` and canonicalization (`canonicalize_arrow_labeled`) | hashes every edge, named edges sorted by label | in the hash | in the hash |
| `v2.std.exact_structural_equality_zip_fold_predicate` (via `infer_type_equal_ignoring_provenance`) | compares every child | follows infer's evidence (above) | follows infer's evidence (above) |

Readers that ignore a named edge safely and need no change: `v2.compiler.resolve` `add_arrow_domain_named_params`; the infer readers of positionals and the body (`infer_arrow_declared_return_type`, `infer_arrow_domain_type_for_binding`); `v2.std.node_query`; `v2.std.decl_index` `export_signature_declared_arrow`; `v2.compiler.translate` `translate_arrow_carries_body`; the `v2.std.compilers.target_model` body readers; `v2.compiler.body_producer` `attach_arrow_body`; `v2.compiler.symbol_index_fill`; `v2.compiler.emit_produced`; `v2.compiler.compile`; `v2.std.node` `cost_edge_role`; `v2.lens.cost.copied_port_derivation`.

## Sizing (grep, not an oracle)

321 modifier lines and about 51 input defaults across the service-declaring modules. Output-block defaults do not occur. A `from "key"` wire key is realization (PR3) and is not carried by either edge.
