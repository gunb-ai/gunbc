# SG-2 mode-2 emit — sequencing plan (non-grammar-matched nodes)

Status: cert harness in `src/v4/test/claim/manual/sg2_mode2_non_grammar_emit.dag` (PR #4465). **`06_translate.dag` edits blocked until A (#4462) merges.**

## Sequencing

1. **A (#4462) first** — grammar-first short-circuit + v2_interpreter arm. **Off-limits to SG-2.**
2. **SG-2 prep (this PR)** — cert harness + `rust_sg2_*` fixtures (no `rust_mvp1_*` changes).
3. **SG-2 implementation (follow-up after A)** — `06_translate.dag` mode-2 region until gate claims green.

## Gate (execution)

| Item | Value |
|------|-------|
| Consumer | `InferredTree { root: rust_sg2_instantiation_source_type_node() }` on `rust_sg2_type_expr_target_model()` |
| Why mode-2 | empty `translation_rules` → no grammar row |
| Golden text | `"Rc<FooBar<X, Y>>"` |
| Perturbed twin | `rust_sg2_foobar_xy_no_rc_source_type_node()` → `"FooBar<X, Y>"` (no outer Rc) |
| Gate claims | `claim_sg2_mode2_emit_accepts`, `claim_sg2_mode2_emit_instantiation_text`, `claim_sg2_mode2_emit_discriminates_perturbed_twin` |

Marks authoritative in `.dag`; this doc is sequencing + post-A checklist only.

## Claim tiers

| Tier | Claims | When green |
|------|--------|------------|
| Prep (now) | `claim_sg2_mode2_translation_rules_empty`, `claim_sg2_mode2_emitted_non_grammar_matched`, `claim_sg2_mode2_wire_kind_authority`, `claim_sg2_mode2_gap2_lex_only_prep`, `claim_sg2_mode2_gap2_lex_only_rejects_binding_spellings` | This PR |
| Post-A gate | `claim_sg2_mode2_emit_accepts`, `claim_sg2_mode2_emit_instantiation_text`, `claim_sg2_mode2_emit_discriminates_perturbed_twin` | After mode-2 serialize complete |
| Post-A dissolve | `claim_sg2_mode2_gap1_*`, `claim_sg2_mode2_gap2_lex_only_atom_serialize`, arrow/conj/sum text claims | After GAP-1/GAP-2/per-node edits |

## Post-A edit checklist (`06_translate.dag` — mode-2 region only)

### GAP-2 (fail-closed — no binding_spellings fallback for type atoms)

**Replace** `serialize_type_expr_emitted_wire_bounded` `TargetTypeExprAtom` arm:

- Today: `type_expr_spelling_for_atom` → `map_get(binding_spellings)` only.
- Target: new `type_expr_spelling_for_atom_via_ident_token(projection, target, node)`:
  1. Decode atom binding from emitted wire.
  2. Resolve spelling **only** through `projection.atom_form.ident_token` lex class + ident spelling authority for that binding.
  3. **Absent / not in lex → `Rejected`** (`translate_binding_spelling_not_found` or dedicated ident-miss diagnostic).
  4. **Do NOT** consult `binding_spellings` for `TargetTypeExprAtom` — that map is for value/bind-site spellings, not type-atom lex authority.

Cert: `claim_sg2_mode2_gap2_lex_only_atom_serialize` greens; `claim_sg2_mode2_gap2_lex_only_rejects_binding_spellings` stays green (no silent map fallback).

### GAP-1

- Verify `serialize_type_expr_boundary_atom_bounded` + `project_type_expression_field_slots_go` / `translate_apply_use_site_ownership_to_projected_boundary` agree at param/return/field.
- Cert: `claim_sg2_mode2_gap1_outcome_return_rc_serialize` → `"Rc<Outcome<Node>>"`.

### Per-node serialize

- Confirm all `TargetTypeExprKind` arms in `serialize_type_expr_emitted_wire_bounded` round-trip; certs: arrow/conj/sum text claims.
