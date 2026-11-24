## Node
`v2-core-emitter-lowers-conjunction-in-value-position` (lane: v2 self-host and native emission). Dispatched unbound ad-hoc; acceptance recording frozen, so the full case lives here. **Draft** — the done condition `corpus_probes_emits_holds` is still red (prefix `!` render), so a ready flip would invite approval on a red batch.

## What this PR does (scope)
Widens the single body-expression lowering so a type-shaped **Conj/Disj** and a **nested primitive-use operand** reaching value position lower to a value-expression instead of being coerced to a binding ref and refused with the generic `target_value_expr_reason_transform_shape_invalid`.

src/v2 footprint is **two files** (authorised as Option A, four functions, from neat-badger via warm-fox):
- `src/v2/std/compilers/target_model.dag` —
  1. `target_arrow_scope_positional_operand_value`: the `else` branch now splits on `target_value_expr_local_reference_leaf`. **Present** (a bare binding leaf) keeps the untouched scope-bound binding branch; **Absent** (a primitive-use operand) recurses through the injected `handle_transform` callback. Numeral atom branch untouched.
  2. `target_project_binary_primitive_apply_in_arrow_scope` and 3. `target_project_transform_primitive_apply_in_arrow_scope`: gain a `handle_transform` parameter and forward it (and to the 3 operand-value calls).
  Plus the earlier render-half widening: `target_value_expr_primitive_apply_operand_bindings` reifies operand edges as value-expressions and `target_operator_shape_emit_primitive_apply_tokens` renders each through the value-expression token path with precedence grouping (one precedence authority; infix rows `||`=1 `&&`=2 cmp=3 `+/-`=4 `*/`=5, prefix=6, coerce=7, atomic=99), and the 2-positional-child prefix primitive-use lowers to a one-operand apply.
- `src/v2/compiler/target_serialize.dag` — the 4 `transform_primitive_apply` call sites pass the existing recursive closure `fn(a,b,t){translate_project_transform_in_arrow_scope(a,b,t)}` downward. The callback stays the single route into the transform lowering; no dispatcher, no fifth function.

The `op_not` `PrefixToken` catalog row at `src/v2/extdeps/languages/rust.dag:1369` **pre-existed on base `6c964572de9`** — it is not part of this diff. No seed, no v1, no hand-written seed Rust.

## Caller pass-throughs (hit counts, repo-wide `grep -rn *.dag`)
`target_arrow_scope_positional_operand_value` 3 sites (binary branch: idx1, idx2, prefix idx1) all pass `handle_transform`; `target_project_binary_primitive_apply_in_arrow_scope` 4 sites (transform branch) all pass; `target_project_transform_primitive_apply_in_arrow_scope` 4 sites in `target_serialize.dag` (770/796/806/831) all pass the existing closure. 11 pass-throughs + 1 new recursive call in the `else`. None needed reshaping → no fifth function.

## Gate item 1 — census (EXECUTED)
Instrument (cached) `gunbc` sha256 `3df4eeda06ee92d3673ec546071394e7754a2f502820990b925b3048cfb16975` (before == after). `COMPILE_RC=0`, tree `59fedef65b8` clean. **268 emitted artifacts from 246 sources; lost = 0 (hit count 0); added = 0 (hit count 0); the only two differing artifacts are `v2_std_compilers_target_model.rs` and `v2_compiler_target_serialize.rs`, the emitted forms of the two files under edit; everything else byte-identical to base `6c964572de9`.** The previously-emitting set did not move: a flat binding operand is a `local_reference_leaf`, hits the untouched `Present` branch, and produces the same single `binding_ref` token as before — the census tests that claim rather than defining it.

## Emitted diff of `v2_compiler_target_serialize.rs` (base `6c964572de9` → head `59fedef65b8`)
It is the existing `handle_transform` callback being passed down at the 3 `transform_primitive_apply` call sites, and **nothing else behavioural**. One non-behavioural addition is disclosed: because the callback makes `translate_project_transform_in_arrow_scope` self-referential, the emitter auto-inserts its standard `stacker::maybe_grow(512*1024, 2*1024*1024, …)` stack-depth guard around it. That is a stack guard, not a logic change.

```diff
@@ -549,11 +549,11 @@
-    Outcome::Accepted { value: canonical_op, .. } => crate::v2_std_compilers_target_model::target_project_transform_primitive_apply_in_arrow_scope(arrow.clone(), body.clone(), canonical_op.clone(), target.clone()),
+    Outcome::Accepted { value: canonical_op, .. } => crate::v2_std_compilers_target_model::target_project_transform_primitive_apply_in_arrow_scope(arrow.clone(), body.clone(), canonical_op.clone(), target.clone(), |a, b, t| translate_project_transform_in_arrow_scope(a, b, t)),
-    Connective::Conj => crate::v2_std_compilers_target_model::target_project_transform_primitive_apply_in_arrow_scope(arrow.clone(), body.clone(), crate::v2_std_compilers_target_model::canonical_operation_op_and(), target.clone()),
-    Connective::Disj => crate::v2_std_compilers_target_model::target_project_transform_primitive_apply_in_arrow_scope(arrow.clone(), body.clone(), crate::v2_std_compilers_target_model::canonical_operation_op_or(), target.clone()),
+    Connective::Conj => crate::v2_std_compilers_target_model::target_project_transform_primitive_apply_in_arrow_scope(arrow.clone(), body.clone(), crate::v2_std_compilers_target_model::canonical_operation_op_and(), target.clone(), |a, b, t| translate_project_transform_in_arrow_scope(a, b, t)),
+    Connective::Disj => crate::v2_std_compilers_target_model::target_project_transform_primitive_apply_in_arrow_scope(arrow.clone(), body.clone(), crate::v2_std_compilers_target_model::canonical_operation_op_or(), target.clone(), |a, b, t| translate_project_transform_in_arrow_scope(a, b, t)),
@@ -562,10 +562,12 @@
 pub fn translate_project_transform_in_arrow_scope(arrow: Rc<Node>, body: Rc<Node>, target: Rc<TargetModel>) -> Rc<Outcome<Rc<TargetValueExpression>>> {
+    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
+        crate::v2_std_diagnostic::bind_outcome(/* ... */ |operator_child| match ... {
-    Outcome::Accepted { value: canonical_op, .. } => crate::v2_std_compilers_target_model::target_project_transform_primitive_apply_in_arrow_scope(arrow.clone(), body.clone(), canonical_op.clone(), target.clone()),
+    Outcome::Accepted { value: canonical_op, .. } => crate::v2_std_compilers_target_model::target_project_transform_primitive_apply_in_arrow_scope(arrow.clone(), body.clone(), canonical_op.clone(), target.clone(), |a, b, t| translate_project_transform_in_arrow_scope(a, b, t)),
     Outcome::Rejected { diagnostics: _, .. } => translate_project_transform_surface_or_callable_in_arrow_scope(arrow.clone(), body.clone(), operator_child.clone(), target.clone()),
 })
+    })
```

## Claim batch at head `59fedef65b8` (EXECUTED — 13 PASS / 6 FAIL, `CLAIM_BATCH_RC=1`)
Entry `src/v2/test/claim/self_host/stage0_conjunction_emission_claims.dag`. Provenance: HOST=srv1 BOOT=c095c0cd (== srv1), claim_batch sha256 `9cb21df4c20325fc53ba4187cf4c55bd08037862a45d3900c5cc5519e360a177`, heavy-slot `swift-hawk-617 acquired slot 4/12 at 2026-11-24T04:42:47Z`, run 04:42:47→04:56:32Z. Typecheck: target_model 70s, target_serialize 4s. **Movement from the base state is 9 PASS → 13 PASS.**

```
PASS conj_binary_emits_holds
PASS disj_binary_emits_holds
PASS literal_compare_emits_holds
PASS int_literal_value_emits_holds
PASS corpus_reason_is_not_transform_shape_holds
PASS negate_refuses_not_emits_holds
PASS negate_reason_is_named_not_transform_shape_holds
PASS negate_reason_is_not_body_scope_holds
PASS negate_reason_names_operator_operand_or_arity_holds
PASS prec_or_grouped_and_flat_bodies_differ_holds
PASS prec_or_grouped_has_grouping_holds
PASS prec_or_flat_has_no_grouping_holds
PASS flat_binding_operand_emits_without_redundant_parens_holds
FAIL prefix_not_emits_holds
FAIL corpus_probes_emits_holds
FAIL corpus_no_longer_refused_at_emission_holds
FAIL inner_refusal_surfaces_inner_reason_holds
FAIL prec_not_grouped_and_flat_bodies_differ_holds
FAIL prec_not_grouped_wraps_operand_holds
```

The 13 PASS are the core of the first slice: outer Conj/Disj in value position now **emit**, nested binary applies lower, `n > 0` and the literal case emit, the OR-precedence grouping controls (including "flat binding emits without redundant parens") pass, and the unspellable negate keeps a named (non-generic) refusal. The 6 FAIL are all prefix-`!`-centric: `prefix_not`, the two `prec_not` cases, the two corpus claims (the subject carries a `!closed_alias` prefix operand), and the nested-inner refusal.

## Reason dump at head `e948da35ce0` (EXECUTED — entry `src/v2/test/claim/self_host/stage0_prefix_inner_reason_probe.dag`, hit count 31, `CLAIM_BATCH_RC=1`)
Provenance: HOST=srv1 BOOT=c095c0cd (== srv1), tree clean, claim_batch sha256 `9cb21df4c20325fc53ba4187cf4c55bd08037862a45d3900c5cc5519e360a177`.
For each subject the claim whose refusal lexeme the tool actually raised PASSES (executed, not predicted):
- `op_negate` (`-x`, no catalog row) refuses at **emission** with the **named** reason `target_value_expression_emit_primitive_apply_arity_invalid` — satisfies "an operand form the catalog cannot spell refuses with its own named reason," not the generic bucket.
- `op_not` prefix (`!closed_alias`) and the corpus subject refuse at **emission** with a lexeme **outside** the primitive-apply/arity/catalog/shape/transform-shape candidate set: the one-operand apply gets past the catalog gate (row found) and past arity, then fails at emission on an unenumerated lexeme (leading suspect by code read: the primitive-apply wire-schema lexeme `target_value_expr_operand_wire_malformed`, `target_model.dag:5673/5680/5703` — not asserted).
- The wrapped negate `y + (-x)` refuses **pre-emission** and its reason ≠ the standalone negate's, so the inner arity reason does **not** surface through the operand recursion.

## red_control
Row-measured RED at `red_at 4a63370f4a677ae8541cdbf2655e0f4ba131226b`, claim_batch sha256 `9cb21df4…360a177`, released runner sha256 `ad25b8ee…c09ee45`; the four stage-pin verdicts (FAIL emits / PASS refuses_at_emission / FAIL refuses_at_inference / PASS assembles), reason `target_value_expr_reason_transform_shape_invalid`. `green_at` not yet claimed: the row is done when `corpus_probes_emits_holds` passes on the same subject — currently blocked on prefix-`!` render, the row's own "prefix-operator support is unproven" area.

## Status / budget
Draft. The prefix-`!` render + `!`-precedence is the row-flagged unproven area and is a neat-badger decision; I have not touched the four authorised functions on a hypothesis. Acquisition budget is at the mandatory pair only (fixed-head claim_batch + fixed-head census); no spare.
