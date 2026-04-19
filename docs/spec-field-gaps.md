> Part of: [phase1-lane3-consolidation-build-plan.md](./phase1-lane3-consolidation-build-plan.md) · [emit-functions-inventory.md](./emit-functions-inventory.md)

# Target spec field gaps (Stage 1d)

**Purpose:** For **spec-driven** emission paths (see inventory), list what each target’s `.dag` spec already declares vs what the emitter still encodes in Rust. Drives **P2 pre-walker** spec extensions: add fields **before** the generic walker replaces the hand-written `render_*` bodies.

**Priority tags**

| Tag | Meaning |
|-----|---------|
| **P0 — blocks consolidation** | Walker cannot be target-agnostic without this |
| **P1 — blocks parity** | Needed for Go/Python/Rust to share one walker shape |
| **P2 — nice-to-have** | Dissolves a bridge or improves ergonomics; emission can ship with a temporary Rust shim |

Specs referenced: `src/v3/spec/rust.dag`, `go.dag`, `python.dag` (+ shared `v3_l1.dag`, `v3/std/*` types they import). The Go target's hand-written body now lives under `src/v3/compiler/src/emit.rs`; references below to the former `emit_go.rs` body apply to that moved implementation.

---

## 1. Template + syntax carriers (broadly covered)

**Functions:** `render_named_template`, `render_value`, most `render_*` bodies that only substitute `RealizationIndexes.syntax.*` and `behavior` templates.

| Topic | Declared today | Gap |
|-------|----------------|-----|
| Statement / expression / type / pattern templates | `StatementSyntax`, `ExpressionSyntax`, `TypeDefinitionSyntax`, `PatternMatchSyntax`, `LiteralSyntax`, `CollectionOps`, `ValueConstructionSyntax`, `FunctionSyntax`, `ModuleSyntax` per target | **P2:** Centralize placeholder vocabulary (`{quote}`, `{p0}`…) as named slots in spec if tokenizer still lacks escapes (see `rust.dag` header). |
| Behavior templates | `BehaviorRealization` rows keyed by marker `DeclarationId` | **None** for markers that already have realizations. |
| Operator symbols | `OperatorRealization` rows | **None** once operand type resolves to field decl (see §4). |
| Clean emission | `CleanEmissionContract` (`rust_clean_emission`, `go_clean_emission`, `python_clean_emission`) | **P2:** Any remaining Rust-only validation messages could move to spec-backed policy enums. |

---

## 2. User-defined types: struct / enum emission

**Functions:** `render_type_declaration`, `render_struct_field`, `render_enum_variant` (Rust); `render_type_declaration`, `render_struct_field` (Go); `render_type_declaration`, `render_enum_variant` (Python).

| Target | Declared | Needed by function | Gap |
|--------|----------|-------------------|-----|
| Rust | `type_definitions.struct_def`, `enum_def`, `struct_field`, `enum_unit_variant`, `enum_data_variant` | Stable Rust type syntax for Conj/Disj | **P0:** **`#[derive(Clone, Debug)]` is hardcoded** above struct/enum templates — not in spec. Need e.g. `type_attributes: list Template` or `record_derive: String` on `TypeDefinitionSyntax` / per-emission hook. |
| Go | Go `type` syntax templates | Package-level emission | **P1:** Module/package preamble and exported symbol rules live partly in driver — ensure spec carries all presentation (see `emit_go_with_mode`). |
| Python | `class` / union templates | Dataclass / typing | **P2:** Qualification rules for variant classes partially encoded in helper logic — extend spec if new targets need different import/qualification. |

---

## 3. Callable / transform / callable-body emission

**Functions:** `render_callable_transform`, `render_substrate_accessor`, `render_realized_callable`, `render_general_callable`, `render_callable_body`, `render_record_constructor`, `render_variant_constructor`, `render_closure`, `render_function_declaration`.

| Topic | Declared | Gap |
|-------|----------|-----|
| Callable strategies | `CallableRealization` + `RustCallableStrategyBinding` / Go / Python equivalents | **P2:** Strategy matrix is rich; new callable shapes may need new enum variants — track per substrate addition. |
| Substrate accessors | `SubstrateAccessorBinding` + per-target realization rows | **P1:** Error strings in `render_substrate_accessor` hardcode `rust_language` / file paths — should become spec-driven diagnostic template or single “active language” display name field. |
| DB-14 positional templates `{p0}`… | `carrier` on accessor realization | **None** for supported accessors. |

---

## 4. Operators (`render_operator`) and algebra resolution

**Companion (not `render_*`):** `algebra_field_for_operator`, `canonical_operator_field`, `walk_to_algebra_conj`.

| Declared | Needed | Gap |
|----------|--------|-----|
| `OperatorRealization` entries for `(operand type, algebra field decl)` | Resolved `DeclarationId` for field | **P0 bridge B14:** Fallback **`canonical_operator_field` uses `declaration_by_name("OrderedRing")`** when algebra Conj walk fails — semantics are correct but the **name** is a bridge. Spec (or substrate) should expose **canonical algebra conj** for fallback without a string name, or declare explicitly that OrderedRing is the only fallback algebra. |
| `OperatorKind::algebra_field_name()` | Field lookup inside Conj | **P2:** Field labels are substrate names — acceptable if single-sourced; document as invariant. |

---

## 5. Branching and pattern match

**Functions:** `render_branch`, `render_path_body`, `render_realized_pattern_branch`, `render_vector_list_pattern_branch`, `render_branch_pattern`, `render_bool_pattern`, `render_payload_binding_name` (Rust); `render_bool_branch`, `render_optional_branch`, `render_sum_branch`, … (Go); `render_branch`, `render_list_branch`, `render_general_match`, `render_branch_condition`, `render_branch_body_expr`, `render_match_binding` (Python).

| Area | Declared | Gap |
|------|----------|-----|
| Rust `match` / if | `PatternMatchSyntax`, `match_arm`, etc. | **P1:** **List / vector patterns** and specialized branches may still assume std shapes — verify every path has a template or a declared fail. |
| Rust positional payload `_0` | `variant_pattern_positional` vs `variant_pattern` + `field_overrides` | **P1:** **`render_branch_pattern`** and **`render_path_body`** compare conj field labels to **`_0`** (`==` / `!=`) — same **positional-payload bridge** as Python (`emit-bridges.md` bucket **E**, B13). Spec/substrate should declare **positional vs named single-field payload** without magic `_0` strings. |
| Go `switch` / optional | Go control-flow templates | **P1:** **`None`/`Some`/`Empty`/`Cons` variant labels** compared by string in `render_optional_branch` / list branches — **bridge B13**; need **`MatchStrategy` / variant tagging in spec** or substrate metadata so the walker does not compare magic strings. |
| Python `lambda __match` | `python_clean_emission.pattern_bindings` | **P1:** **`render_list_branch`** assumes **`Empty` / `Cons` labels**; **`render_branch_condition`** special-cases **`None`**; **`render_match_binding`** special-cases **payload field `_0`**. These need spec-declared **variant identity** (or std list hook) instead of literals. |
| Bool splitting | Structural disj for `Classical` | **None** — uses substrate positions, not names (Rust `split_bool_paths`). |

---

## 6. Ports, bindings, ownership (`render_port`, `render_binding`, `render_input_use`, `render_copy_input_use`, `render_top_level_value`, `render_field_project`, `render_transform`)

| Declared | Gap |
|----------|-----|
| `ParameterDisposition`, `rust_rendering` / Go / Python rendering models | **P0:** **Move vs clone vs borrow** decisions come from **`InputUseFacts`** (lens), not from a spec field — by design today. Consolidation expects a **lens DAG** to own this; spec declares **how** to render each disposition, not the fact. Ensure `RenderingModel` covers every disposition the lens emits. |
| `FieldAccess` / `FieldBinding` in type realizations | **P2:** Edge cases (list lowering, `PassByValue` unsupported messages) — align error policy with spec enums. |

---

## 7. Loops (`render_loop`)

| Declared | Gap |
|----------|-----|
| Behavior realizations for `Loop` (if any) | **P0:** **`emit_go` / `emit_python` fail-closed** on `Behavior::Loop` (per Half B). **Rust** may differ. Need **target-aligned `BehaviorRealization` for Loop** + shared semantics **before** re-enabling ignored tests listed in the build plan §5.D. |

---

## 8. Literals (`render_value`)

| Declared | Gap |
|----------|-----|
| `LiteralSyntax` per target (string delimiter, numeric forms) | **P2:** Any literal kind not yet in `LiteralSyntax` carriers — extend spec when new `LiteralBits` variants get emission. |

---

## 9. Driver / module shape (not individual `render_*`, but blocks “one walker”)

**Functions:** `emit_*_with_mode`.

| Topic | Declared | Gap |
|-------|----------|-----|
| `Main` behavior template | `rust.dag` behavior for main marker | **P1:** **Program vs module** mode, **top-level bind** rules, **`pub` injection** — partially in Rust. Extend `ModuleSyntax` / execution model or add explicit **`RustEmitMode` spec hooks** so the walker does not encode policy. |
| Bootstrap filtering | — | **P2:** `is_bootstrap_file` path prefixes are **hardcoded** in each emitter — could become **`stdlib_path_prefixes: list String`** on `LanguageSpec` or `TargetExecutionModel`. |

---

## 10. Consolidation ordering (P2 execution hint)

1. **P0:** Loop behavior templates (§7); OrderedRing fallback / canonical algebra pointer (§4); Rust derive attributes (§2).  
2. **P1:** Match variant bridges — Go/Python list & optional (§5); substrate accessor error templating (§3); module/main `pub` & mode (§9).  
3. **P2:** Bootstrap path lists, placeholder vocabulary, remaining diagnostic polish.

This ordering matches **“spec extensions before deleting Rust”** so P2-L1 does not stall mid-walker on a missing field.

---

## 11. Lane 2 Stage 2f / DB-3 dimensions (not target-spec)

**Surface:** `src/v3/std/dimensions.dag`, `workflows.dag` — no additional fields required on `rust.dag` / `go.dag` / `python.dag` for the dimension report types. **Gap:** authoring `data … : Dimension<Carrier> = { … }` values still waits on class-5 `data` bodies (`DOWNSTREAM_REQUIREMENTS.md`); no separate compiler registry ships ahead of a consumer that needs it.
