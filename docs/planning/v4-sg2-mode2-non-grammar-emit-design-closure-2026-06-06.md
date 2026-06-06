# SG-2 mode-2 emit — non-grammar-matched nodes (design closure)

Status: **DESIGN + prep fixtures + cert harness landed** (PR #4465); `06_translate.dag` GAP-1/GAP-2 implementation edits are **blocked on PR #4462** (`r2-green-grammar-first-reroute`) landing first (collision-safe).

## Routing (mode-1 vs mode-2)

After #4462, `translate_type_expression_tree` becomes grammar-first:

| Mode | Trigger | Translate output | Serialize path |
|------|---------|------------------|----------------|
| **mode-1** | `grammar_relation_row_for_emitted` accepts (emitted node equals a row's `emitted` field) | Raw structure-preserving node (no SG-2 wire wrap) | `TypeExprKindNotEmitted` → grammar-inverse lex walk over row tokens |
| **mode-2** | No grammar row match | `project_type_expression_node` → `TargetTypeExpression` wire (`TypeExprKindAuthority`) | `serialize_type_expr_*` driven by `target_model_edge_type_expression_projection` row |

`rust_sg2_type_expr_target_model` is the intentional **mode-2-only** probe host: `target_model_edge_translation_rules` is an empty `Conj`, so every type-expression subtree is non-grammar-matched by construction.

Own cert (session SG-2): **`emit` greens a non-grammar-matched type-expression subtree** — translate projects to SG-2 wire, serialize completes without grammar-inverse row lookup. Harness: `src/v4/test/claim/manual/sg2_mode2_non_grammar_emit.dag`.

## GAP-1 — grounding-ownership at serialize boundary

**Debt:** fn-boundary atoms at Arrow param/return sites must consult `target_model_edge_use_site_ownership_realizations` (and SG-1b signature realizations when present) *during mode-2 serialize*, not only during projection.

**Authority today:** `serialize_type_expr_boundary_atom_bounded` in `06_translate.dag` (SG-RC + SG-1b path partially wired).

**Cert pin:** `claim_sg2_mode2_gap1_outcome_return_rc_serialize` — `Outcome<Node>` return type serializes as `Rc<Outcome<Node>>` when the use-site ownership row matches (reuses `rust_sg_rc_outcome_return` catalog entry on `rust_sg2_type_expr_target_model`).

**Dissolve-on:** claim greens under `target_serialize_source_from_model` without spelling-only bypass.

## GAP-2 — type-atom lex-token (`ident_token`)

**Debt:** `TargetAtomTypeShape.ident_token` is modeled on every per-language SG-2 row (`rust_type_expression_projection().atom_form.ident_token == rust_token_ident`) but `serialize_type_expr_emitted_wire_bounded` still spells atoms via `type_expr_spelling_for_atom` → `binding_spellings` map lookup only.

**Required fix (post-#4462, in `06_translate.dag`):** atom serialize must resolve spelling through the projection row's `ident_token` lex class (fail-closed when absent), with `binding_spellings` as fallback only where explicitly dual-sourced — not as the sole authority.

**Cert pin:** `claim_sg2_mode2_gap2_lex_only_atom_serialize` — binding `rust_binding_sg2_lex_only_atom` is **intentionally absent** from `rust_sg2_binding_spellings()`; emitted atom must still serialize to `LexOnlyProbe` via ident-token authority.

**Dissolve-on:** claim greens; delete any atom-only `binding_spellings` entries that duplicate ident-token resolution.

## Per-node serialize completion (mode-2 wire kinds)

Checklist for `TargetTypeExprKind` arms — each must round-trip under `target_serialize_source_from_model` on `rust_sg2_type_expr_target_model`:

| Kind | Fixture | Cert claim (harness) |
|------|---------|----------------------|
| Atom | `rust_sg2_rc_foobar_xy_emitted` (nested atoms) | via instantiation claim |
| Instantiation | `rust_sg2_rc_foobar_xy_emitted` | `claim_sg2_mode2_emit_instantiation_text` |
| Arrow | `rust_sg2_arrow_xy_emitted` | `claim_sg2_mode2_emit_arrow_text` |
| Record (Conj) | `rust_sg2_conj_xy_emitted` | `claim_sg2_mode2_emit_conj_text` |
| Sum (Disj) | `rust_sg2_sum_xy_emitted` | `claim_sg2_mode2_emit_sum_text` |
| Cardinality | *(fixture TBD when cardinality probe lands in rust.dag)* | deferred — not blocking own cert |

Malformed-wire rejection receipts remain in `sg2_type_expression_projection.dag` (fail-closed floor).

## Sequencing

1. **NOW (this session):** design closure (this doc) + prep fixtures + cert harness `.dag` + smoke surface pin.
2. **After #4462 merges:** land `translate_grammar_relation_row_match` grammar-first short-circuit in `06_translate.dag` (no parallel edit before merge).
3. **Then:** close GAP-1/GAP-2 + per-node gaps in `06_translate.dag` until harness claims green.

## Consumer

Primary consumer: `cargo test -p v3-compiler v4_std_target_realization_dag_smoke_test` (surface pin) + v4 `.dag` compile of manual claim module. Executable `TestClaimRun` verdict is downstream of T-22 eval host transport.
