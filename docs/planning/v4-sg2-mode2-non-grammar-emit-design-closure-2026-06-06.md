# SG-2 mode-2 emit — sequencing plan (non-grammar-matched nodes)

Status: cert harness landed in `src/v4/test/claim/manual/sg2_mode2_non_grammar_emit.dag` (PR #4465, draft). **`06_translate.dag` edits blocked until A (#4462) merges.**

## Sequencing

1. **A (#4462) first** — grammar-first short-circuit in `translate_type_expression_tree` + v2_interpreter `Value '==' List<->FreeMonoid` arm. **Off-limits to SG-2** (crisp-seal owns).
2. **SG-2 prep (done)** — design + cert harness + `rust_sg2_lex_only_atom` GAP-2 fixture in `rust.dag` (no mvp1 row/inhabitant changes).
3. **SG-2 implementation (after A merge)** — integrate A, then edit `06_translate.dag` mode-2 serialize/projection region only until harness greens.

## Own cert (execution gate)

**Consumer node:** `InferredTree { root: rust_sg2_instantiation_source_type_node(), … }` on `rust_sg2_type_expr_target_model()` — empty `translation_rules`, so no grammar row → must traverse mode-2 (project + `TypeExprKindAuthority` serialize).

**Gate claims:** `claim_sg2_mode2_emit_accepts` + `claim_sg2_mode2_emit_instantiation_text` (`"Rc<FooBar<X, Y>>"`). Dissolve targets: `claim_sg2_mode2_gap1_*`, `claim_sg2_mode2_gap2_*`.

Authoritative model marks live in the `.dag` files; this doc is sequencing only.

## Post-A edit plan (`06_translate.dag` only — no A touch)

| Gap | Site | Edit |
|-----|------|------|
| GAP-2 | `serialize_type_expr_emitted_wire_bounded` / `TargetTypeExprAtom` | Spell via `projection.atom_form.ident_token` lex authority; fail-closed; `binding_spellings` fallback only where dual-sourced. Cert: `claim_sg2_mode2_gap2_lex_only_atom_serialize`. |
| GAP-1 | `serialize_type_expr_boundary_*` + `project_type_expression_field_slots_go` ownership at struct-field boundaries | Ensure serialize-time boundary consult matches projection-time `translate_apply_use_site_ownership_to_projected_boundary` discipline for param/return/field. Cert: `claim_sg2_mode2_gap1_outcome_return_rc_serialize`. |
| Per-node | `serialize_type_expr_emitted_wire_bounded` arms | Complete any remaining `TargetTypeExprKind` serialize paths; certs: arrow/conj/sum claims in harness. |

#4465 stays draft until `claim_sg2_mode2_emit_accepts` greens by execution.
