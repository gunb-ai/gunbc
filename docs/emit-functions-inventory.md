> Part of: [phase1-lane3-consolidation-build-plan.md](./phase1-lane3-consolidation-build-plan.md) · [single-emitter-design.md](./single-emitter-design.md)

# Emitter function inventory (Stage 1d)

**Scope:** Every `fn render_*` and `fn emit_*` in the three v3 emitters  
`src/v3/compiler/src/emit_rust.rs`, `emit_go.rs`, `emit_python.rs`.

**Verification (line counts must match this doc):**

```bash
rg -c 'fn (render_|emit_)' \
  src/v3/compiler/src/emit_rust.rs \
  src/v3/compiler/src/emit_go.rs \
  src/v3/compiler/src/emit_python.rs
# expect: 40 + 25 + 27 = 92
```

**Classifications (from the build plan):**

| Bucket | Meaning | Typical destination after consolidation |
|--------|---------|----------------------------------------|
| **spec-driven** | Emission driven by `LanguageSpec` / `RealizationIndexes` templates and structural substrate facts | Generic walker + per-target spec data |
| **lens** | Computes derived facts consumed by emission (ownership, copy, etc.) | Dedicated `lens_*.dag` + facts (not named `render_*` / `emit_*` in today’s code — see §Related) |
| **substrate walk** | Shared structural walk over Conj / algebra (reusable primitive) | `std/substrate_walks.dag` (today’s helpers are mostly *not* `render_*` / `emit_*`) |
| **per-target integration** | Target entrypoint, mode flags, formatting policy wired outside the recursive template core | Stays target-specific (thin driver) |

---

## Summary

Counts are the **same grep surface** as §Verification (every `fn render_*` / `fn emit_*`). **Production** rows = spec-driven + per-target (consolidation-relevant). **Test-only** rows satisfy the acceptance `grep` but are **not** part of driver-vs-walker or escalation percentages.

| Emitter file | Grep total | spec-driven | per-target integration | test-only | lens | substrate-walk |
|--------------|-----------:|------------:|------------------------:|----------:|-----:|---------------:|
| `emit_rust.rs` | 40 | 34 | 3 | 3 | 0 | 0 |
| `emit_go.rs` | 25 | 22 | 3 | 0 | 0 | 0 |
| `emit_python.rs` | 27 | 24 | 3 | 0 | 0 | 0 |
| **Total** | **92** | **80** | **9** | **3** | **0** | **0** |

**Checksum (must close):** `34+3+3 = 40`, `22+3+0 = 25`, `24+3+0 = 27`; **`80+9+3 = 92`**.

**Production-only (89 functions):** `80` spec-driven + `9` per-target integration — this is the set the consolidation split applies to. **Test-only (3):** all in `emit_rust.rs` §tables below.

### Interpretation

- **No row in the 92 is classified `lens` or `substrate-walk`.** Lens builders (`InputUseFacts::build`, copy/ownership analysis, etc.) and algebra walks (`algebra_field_for_operator`, `walk_to_algebra_conj`, optional/list disj walks) use other names — they live beside these functions and are called from them. Stage 1e still extracts them per the build plan; they are simply outside the `render_*` / `emit_*` grep surface.
- **Per-target integration (9, production only):** the three public/module drivers per emitter (`emit_*`, `emit_*_module`, `emit_*_with_mode`). **`emit_rust_with_mode`** is the Rust driver (indexes, `InputUseFacts`, `EmitRustMode`, main shell). Everything else in production is **spec-driven** relative to the “walker + spec” split: recursive rendering from `RealizationIndexes` and `CleanEmissionContract`, with Rust-only presentation details (e.g. `pub` prefix) localized inside `render_*` bodies until spec fields absorb them.
- **Test-only (3 rows in `emit_rust.rs`):** counted by acceptance `grep`; regression harnesses only — **excluded** from the **80 / 9** production split and from §Escalation driver share (see below).

---

## `emit_rust.rs` (40)

| Function | Classification | Destination |
|----------|----------------|-------------|
| `render_named_template` | spec-driven | Shared template helper (walker utility or tiny shared module) |
| `emit_rust_with_mode` | per-target integration | Target driver: builds indexes/facts, applies `EmitRustMode`, assembles program shell |
| `emit_rust` | per-target integration | Thin wrapper → `emit_rust_with_mode(Program)` |
| `emit_rust_module` | per-target integration | Thin wrapper → `emit_rust_with_mode(Module)` |
| `Ctx::render_borrowed_expr` | spec-driven | Walker + `rust_rendering` / read strategy |
| `Ctx::render_collection_receiver` | spec-driven | Walker + collection + input-use |
| `Ctx::render_binding` | spec-driven | Walker + let/binding templates |
| `Ctx::render_port` | spec-driven | Core port recursion + bind index |
| `Ctx::render_input_use` | spec-driven | Walker consumes **lens** output (`InputUseFacts`) — lens file separate |
| `Ctx::render_copy_input_use` | spec-driven | Same |
| `Ctx::render_top_level_value` | spec-driven | Walker + top-level bind semantics |
| `Ctx::render_transform` | spec-driven | Walker + `TransformTarget` |
| `Ctx::render_field_project` | spec-driven | Walker + field access templates |
| `Ctx::render_operator` | spec-driven | Walker + `OperatorRealization` / binary op template |
| `Ctx::render_branch` | spec-driven | Walker + branch/match templates |
| `Ctx::render_realized_pattern_branch` | spec-driven | Walker + pattern realization |
| `Ctx::render_vector_list_pattern_branch` | spec-driven | Walker + list pattern strategy |
| `Ctx::render_path_body` | spec-driven | Walker + path |
| `Ctx::render_branch_pattern` | spec-driven | Walker + pattern syntax |
| `Ctx::render_payload_binding_name` | spec-driven | Walker + `rust_clean_emission` pattern rules |
| `Ctx::render_bool_pattern` | spec-driven | Walker + bool patterns |
| `Ctx::render_callable_transform` | spec-driven | Callable dispatch |
| `Ctx::render_substrate_accessor` | spec-driven | Walker + `SubstrateAccessorBinding` (DB-14); template from spec |
| `Ctx::render_realized_callable` | spec-driven | Walker + `CallableRealization` strategies |
| `Ctx::render_general_callable` | spec-driven | Fallback callable emission |
| `Ctx::render_record_constructor` | spec-driven | Walker + value construction |
| `Ctx::render_variant_constructor` | spec-driven | Walker + variant syntax |
| `Ctx::render_callable_body` | spec-driven | Walker + callable body |
| `Ctx::render_closure` | spec-driven | Walker + function value |
| `Ctx::render_loop` | spec-driven | Walker + `Behavior::Loop` (or unsupported — policy in spec) |
| `Ctx::render_function_declaration` | spec-driven | Walker + `FunctionSyntax` |
| `Ctx::render_type_declaration` | spec-driven | Walker + type defs (**see bridge:** hardcoded `#[derive]` today) |
| `Ctx::render_struct_field` | spec-driven | Walker + struct field template |
| `Ctx::render_enum_variant` | spec-driven | Walker + enum variant templates |
| `Ctx::render_instantiated_type` | spec-driven | Walker + type application syntax |
| `Ctx::render_list_item_construct_expr` | spec-driven | Walker + list item construction |
| `render_value` | spec-driven | Literals via `LiteralSyntax` |
| `tests::render_field_project_reads_borrowed_nodes_without_cloning` | *(test only)* | N/A — keeps regression coverage |
| `tests::render_field_project_constructs_owned_list_from_borrowed_nodes` | *(test only)* | N/A |
| `tests::render_fold_iterates_named_list_input_by_borrow` | *(test only)* | N/A |

---

## `emit_go.rs` (25)

| Function | Classification | Destination |
|----------|----------------|-------------|
| `emit_go` | per-target integration | Thin wrapper |
| `emit_go_module` | per-target integration | Thin wrapper |
| `emit_go_with_mode` | per-target integration | Target driver (indexes, decl filtering, sections) |
| `Ctx::render_port` | spec-driven | Core recursion |
| `Ctx::render_top_level_value` | spec-driven | Top-level values |
| `Ctx::render_transform` | spec-driven | Transforms |
| `Ctx::render_field_project` | spec-driven | Field project |
| `Ctx::render_operator` | spec-driven | Operators |
| `Ctx::render_branch` | spec-driven | Dispatches bool / optional / sum / pattern |
| `Ctx::render_bool_branch` | spec-driven | Bool `if` |
| `Ctx::render_optional_branch` | spec-driven | Optional unwrap-style branches |
| `Ctx::render_sum_branch` | spec-driven | Sum-type `switch` |
| `Ctx::render_realized_pattern_branch` | spec-driven | Pattern branches |
| `Ctx::render_vector_list_pattern_branch` | spec-driven | List patterns |
| `Ctx::render_path_body` | spec-driven | Path bodies |
| `Ctx::render_realized_callable` | spec-driven | Realized callables |
| `Ctx::render_general_callable` | spec-driven | General callables |
| `Ctx::render_record_constructor` | spec-driven | Struct literals |
| `Ctx::render_variant_constructor` | spec-driven | Variant constructors |
| `Ctx::render_callable_body` | spec-driven | Callable bodies |
| `Ctx::render_function_declaration` | spec-driven | Func decls |
| `Ctx::render_type_declaration` | spec-driven | Type decls |
| `Ctx::render_struct_field` | spec-driven | Struct fields |
| `render_named_template` | spec-driven | Template helper |
| `render_value` | spec-driven | Literals |

---

## `emit_python.rs` (27)

| Function | Classification | Destination |
|----------|----------------|-------------|
| `emit_python` | per-target integration | Thin wrapper |
| `emit_python_module` | per-target integration | Thin wrapper |
| `emit_python_with_mode` | per-target integration | Target driver |
| `Ctx::render_top_level_value` | spec-driven | Top-level values |
| `Ctx::render_port` | spec-driven | Core recursion |
| `Ctx::render_transform` | spec-driven | Transforms |
| `Ctx::render_operator` | spec-driven | Operators |
| `Ctx::render_branch` | spec-driven | Bool / list / general match |
| `Ctx::render_list_branch` | spec-driven | List `Empty`/`Cons` lowering (**bridge:** label checks) |
| `Ctx::render_general_match` | spec-driven | General `match` lowering |
| `Ctx::render_branch_condition` | spec-driven | Branch conditions (**bridge:** `None` name) |
| `Ctx::render_branch_body_expr` | spec-driven | Arm bodies + pattern_bindings |
| `Ctx::render_match_binding` | spec-driven | Payload extraction (**bridge:** `_0` label) |
| `Ctx::render_path_body` | spec-driven | Path bodies |
| `Ctx::render_loop` | spec-driven | Today: fail-closed unsupported (`Behavior::Loop`) |
| `Ctx::render_callable_transform` | spec-driven | Callable transforms |
| `Ctx::render_realized_callable` | spec-driven | Realized callables |
| `Ctx::render_general_callable` | spec-driven | General callables |
| `Ctx::render_record_constructor` | spec-driven | Record ctor |
| `Ctx::render_variant_constructor` | spec-driven | Variant ctor |
| `Ctx::render_callable_body` | spec-driven | Callable bodies |
| `Ctx::render_closure` | spec-driven | Closures |
| `Ctx::render_function_declaration` | spec-driven | Func decls |
| `Ctx::render_type_declaration` | spec-driven | Type decls |
| `Ctx::render_enum_variant` | spec-driven | Enum variants |
| `render_value` | spec-driven | Literals |
| `render_named_template` | spec-driven | Template helper |

---

## Related helpers (not `render_*` / `emit_*`)

These are **not** counted in the 92 but are load-bearing for consolidation:

| Item | Location | Classification |
|------|----------|----------------|
| `InputUseFacts::build` | `emit_rust.rs` | **lens** (ownership / last-use facts) |
| `decl_is_copy` / `decl_is_copy_rec` | `emit_rust.rs` | **lens** (copy-type; structural walk) |
| `algebra_field_for_operator` / `walk_to_algebra_conj` / `canonical_operator_field` | `emit_{rust,go,python}.rs` | **substrate walk** + **bridge** (OrderedRing fallback — see `emit-bridges.md`) |
| `RealizationIndexes::build` | `emit_rust.rs` | Spec index build (feeds walker) |
| `is_bootstrap_file` | each emitter | **per-target integration** policy (stdlib path filter) |

---

## Escalation check (build plan)

- **Per-target integration share (production only):** 9 / 89 ≈ **10.1%** — above the “~5%” illustrative split but **far below** the 30% escalation threshold; the share is concentrated in the intentional **driver** layer (`emit_*_with_mode`), not scattered semantic branches. *(Denominator **89** = production `fn`; the **3** test-only rows are excluded.)*
- **Lens / substrate-walk** work is **not** missing — it is **named outside** the `render_*` / `emit_*` inventory, as documented above.
