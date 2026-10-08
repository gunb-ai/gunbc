# Census ledger: `concept_index_enumeration` (7 reds, one root)

Parent lane: sunny-bat-82 (out-of-gate v2 silent reds). Pin at dispatch: `9bb74408bd`. Re-derived on current main.

Classification: **(a) producer encoding vs type-name reader**, not (b) drifted claims, not (c) gone subject.

| Row | Cause | Disposition | PR |
| --- | --- | --- | --- |
| `witness_fieldref_self_projection` | `member_edge_type_name` required an XL-2 binder; `concept_decl_facts` / `_live` marshal `name → Atom` | Read Atom identity when the binder reader is Absent; claims keep `String` | this PR |
| `witness_qualified_concept_self_projection` | same root (`qualified_name: String`) | same | this PR |
| `witness_local_record_enumerated` | same root (`alpha: String`) | same | this PR |
| `witness_parse_only_extract_reflects_canonical_field_types` | live and parse-only share `marshal_variant_arm_target` | same | this PR |
| `witness_parse_only_extract_reflects_perturbed_field_type` | same root; `beta: Int` was never the failing conjunct | same | this PR |
| `witness_coproduct_arms_live` | payload `{ alpha: String, beta: Bool }` is `conj_authored_payload_type_name` over the same function | same | this PR |
| `witness_coproduct_arms_parse_only` | same as live | same | this PR |

Not Instantiation/`FreeMonoid<Char>`: marshal always stamps an Atom from the authored type spelling (`marshal_type_expr_ref`). Discrimination (`String` vs `Bool` vs `Int` vs `ConceptStruct`, coproduct payload vs nullary) is unchanged. Marshal wrapping through `binder_edge` waits on stage0 HOLD (#13388). `member_edge_type_node` is not relaxed.
