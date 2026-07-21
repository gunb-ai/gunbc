# Emitter residual site map — crisp-fox-839, 2026-07-21
# Post-#6981 emitted deep-module probe histograms → emitter function sites (05_emit_rust.dag / 04_emit_info.dag)
# Measurement: repr_mismatch_emitted_e0308_probe.sh EMITTED_namespace @ 5e5f038abb (post-#6981)

## Class 1 — generic-T in fold closures (E0425 `cannot find type T`)
**Histogram:** 66 on 04_infer closure; 79 on 06_translate closure. E0308=0.

| layer | site | why |
|---|---|---|
| fold dispatch | `emit_rust_fold_method_call` (05_emit_rust.dag ~6256–6403) | Builds `acc_type_str` via `render_rust_type`, passes to `emit_typed_fold_lambda` as `lambda_acc_type_str`. Guard at ~6363 (`acc_has_unbound_type_var`) falls through when `T` is rendered via applied-type/atom path, not `TypeVariable` node. |
| fold lambda | `emit_typed_fold_lambda` (~6228–6253) | Emits `\|acc: Rc<FreeMonoid<T>>, e: Rc<Edge>\|` closure syntax from `lambda_param_type_strs` fallback types. |
| lambda typing | `lambda_param_type_strs` (~6177–6210) | Annotates closure params; uses `render_rust_type` on inferred param types without fn-generic scope extension into closure. |
| type render | `render_rust_type` / `render_rust_applied_type` (~159–175, ~535–572) | Emits literal `T` in `FreeMonoid<T>` when element type is unresolved type variable but `fn_generic_param_names` is empty on non-generic fns. |
| info carrier | `04_emit_info.dag` `EmitGraphInfo.fn_generic_param_names` | Threaded at enclosing-fn level via `emit_info_with_fn_type_context`; not propagated into fold-closure annotation scope. |

**Emitted exemplar (04_infer closure):**
```rust
// v2_std_node_query.rs:37 — source: v2.std.node_query coproduct_arm_key_list_from_node fold
|acc: Rc<FreeMonoid<T>>, e: Rc<Edge>| list_snoc_item(acc, named_edge_sort_key(e.clone()))
```
Enclosing fn has no `<T>` generic param.

**Blast radius (FreeMonoid<T> in closure annotations, 04_infer emitted closure):**
`v2_std_compilers_target_model.rs` (11), `v2_std_node_query.rs` (5), `v2_std_grammar.rs` (3), `std_algebra.rs` (2), `v2_std_collection.rs` (1), `v2_std_cardinality.rs` (1).

**Owner:** quiet-bee #6924 Phase-1 residual (closure sites not covered). MAP ONLY — do not fix here.

---

## Class 2 — Witness-dup E0255 (`use v1_rt::Witness` + `pub enum Witness`)
**Histogram:** 1 error, first_error on 04_infer closure post-#6981 (106 total rustc errors).

| layer | site | why |
|---|---|---|
| module prelude | `emit_prelude` (~3575–3600) | Emits `use crate::v1_rt::Witness` when `Witness` absent from `prelude_imported_names` (authored import names only). Does **not** check `local_type_names`. |
| module assembly | `emit_module_full` (~2079–2091) | Computes `local_type_names` from type defs/aliases in module, then calls `emit_prelude(imported_names: prelude_imported_names)` — local defs not passed to prelude. |
| contrast (unused) | `rust_import_name_already_resolved` (~3516) | Already checks `local_type_names`; prelude does not call it. |

**Emitted exemplar:**
```rust
// v2_std_witness.rs:8 + :18 — sole file in 04_infer closure with BOTH import and local enum
use crate::v1_rt::Witness;
pub enum Witness<C> { Holds { value: C }, Violates { diagnostic: Rc<Diagnostic> } }
```

**Owner:** new/unowned.

---

## Class 3 — value-position qualified emit (E0425 `cannot find value extdeps`)
**Histogram:** materialization_carriers sidecar only (316 total); not in 04_infer/06_translate emitted closures.

| layer | site | why |
|---|---|---|
| value render | `emit_var_ref` / `emit_typed_expr_base` ExprVar Absent (~5276–5363) | `expr_var_name_at` → full dotted spelling `extdeps.realization.parse_table_memo.parse_table_memo_id`; `emit_ident(name)` emits verbatim as Rust field chain on bare `extdeps`. |
| import synth (parallel, insufficient) | `collect_value_ref_names` (~1975–2009) + `reference_derived_use_lines` (~2026+) | Collects dotted value ref names for use-line synthesis; does not route **render** through module path (`::`) like types. |
| type analog (fixed) | `render_rust_alias_rhs_type` + `qualified_last_segment` (#6981) | TYPE-position dotted fix; no VALUE-position counterpart. |

**Emitted exemplar:**
```rust
// v2_compiler_materialization_carriers.rs:155
extdeps.realization.parse_table_memo.parse_table_memo_id.clone()
```
**Source:** `materialization_carriers.dag` `parse_table_memo_provider_id()` → `extdeps.realization.parse_table_memo.parse_table_memo_id`.

**Owner:** new/unowned sidecar (parallel to #6981 type lane).

---

## Floor-red hunt (STAND DOWN)
Sibling quick-carp pinned: batch-4 `extdeps_external_authority_gate_passes` — #6991 missing anchor rows, fix at bar PR #7016. Not duplicated here.
