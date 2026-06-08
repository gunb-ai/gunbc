# Q1 ~55-arm projection dissolution — PRE-STAGE inventory

> **Status:** PRE-STAGE (design/inventory only). **No substrate or compiler edits** until (1) T3 fold lands (`snappy-owl-682`) and (2) Mgr-SPINE emit-ladder equivalence gate closes (`stern-lynx-374`, ctrl#1489).
> **Work item:** `node://adhoc-68842efe-c48` · **Session:** `merry-fox-418`
> **Template lane:** `snappy-owl-682` (T3 / SG-2 `TargetTypeExpressionProjection`)

## 1. North star — anti-fold dissolution

**Coercion = emission** (`docs/design-emission-model.md`, THESIS §Tier 1): the compiler does not grow a parallel per-target match ladder in `06_translate.dag` / `05_emit.dag`. It reads declared substrate facts and runs a **structural fold**.

**Anti-fold dissolution** (this lane): retire hand-authored **per-target × per-connective projection arms** in favor of:

| Layer | Authority | Consumer |
|-------|-----------|----------|
| Shared fold | `project_type_expression_node` + `target_type_expr_*_emitted` helpers | `v4.compiler.translate` (`06_translate.dag`) |
| Per-target row | `TargetTypeExpressionProjection` on `TargetModel` bundle edge `target_model_edge_type_expression_projection` | extdeps `languages/*.dag` |
| Proof surface | `sg2_*` fixtures + `manual/sg2_type_expression_projection.dag` (+ per-language clones) | T-38 roster / claim runner |
| Downstream payoff | `coercion_fold` / `fold_program_to_target` | `v3-grounding-coercion-fold` (north-star emit consumer) |

**Why this is the biggest coercion-fold emit payoff:** once every Shape-A target carries a walkable projection row, integer inhabitance selection, type-expression serialize, and grammar-inverse translate all read the **same** `TargetModel` bundle — the fold stops re-deriving surface spellings from parallel string tables.

**~55-arm census (approximate, not a ratchet):** `TargetTypeExprKind` has **6** connective forms (`src/v4/std/target_model.dag`). Near-term ctrl#1489 Shape-A matrix targets **9** languages (MVP roster five: rust/python/go/ts/cpp; plus `.dag` RTADD keystone; java/kotlin/swift staged in extdeps). **6 × 9 ≈ 54 ≈ ~55** per-target connective projection arms to land or explicitly defer. Exact tracking lives in inline marks + claim rows, not this doc (P5 ledger rule).

## 2. Hard gates (do not implement past)

| Gate | Owner | Unblocks |
|------|-------|----------|
| **T3 fold lands** | `snappy-owl-682` | cpp `TargetTypeExpressionProjection` row + Phase-A roster greens without `06_translate.dag` delta |
| **Mgr-SPINE equivalence** | `stern-lynx-374` | T1 multi-target add + RTADD (ctrl#1489 D1) spine receipts before at-scale Q1 rollout |
| **Grammar-first A (#4462)** | Compiler spine | `06_translate.dag` mode-2 region edits (`docs/planning/v4-sg2-mode2-non-grammar-emit-design-closure-2026-06-06.md`) |
| **MVP1 projection attach** | Per-target follow-on | Dissolve `ProjectionAbsent` + `mvp1_translate_fallback` claims (`sg2_type_expression_projection.dag`) |

## 3. T3 template checklist (`snappy-owl-682`)

Copy **extdeps row + fixtures + roster**; do **not** copy load-bearing `06_translate.dag` edits in Phase A.

### 3.1 Per-language extdeps (authoritative pattern: `rust.dag`, `typescript.dag`, T3 `cpp.dag` on `session/snappy-owl-682-t3-sg2-phase-a`)

1. `fn <lang>_type_expression_projection() -> TargetTypeExpressionProjection` — six forms: `atom_form`, `conj_form`, `disj_form`, `arrow_form`, `cardinality_form`, `instantiation_form`.
2. Shape bundle helpers: `*_type_expr_atom_shape_node`, `*_type_expr_generic_apply_shape_node`, `*_type_expr_sum_shape_node`, `*_type_expr_arrow_shape_node`.
3. `fn <lang>_type_expression_projection_bundle_node() -> Node` — named edges `target_type_expr_field_*` → `target_model_edge_type_expression_projection`.
4. SG-2 lex supplement (`*_sg2_type_expr_lex*`) for delimiter token classes used by generic apply / arrow / record surfaces.
5. `fn <lang>_sg2_type_expr_target_model*()` — MVP target model **plus** projection bundle edge (separate from `*_mvp1_target_model` until attach gate).
6. Golden emitted nodes + source type nodes for falsification (`*_sg2_*_emitted`, malformed twins).

### 3.2 Claims + roster (Phase A vs B)

| Phase | Roster | Claims stay in module only |
|-------|--------|----------------------------|
| **A** (T3 current) | `witness_sg2_arrow_holds`, `witness_sg2_projection_falsification_holds` | instantiation / conj / sum structural + serialize (red until Phase B) |
| **B** | Add greens as connectives go green | Promote from `sg2_type_expression_projection.dag` |
| **MVP1 attach** | `mvp1_<lang>_emit_add_fn_accepts_holds` | Requires projection row on `*_mvp1_target_model` (cpp blocked today per `v4_roster_pilot.dag`) |

**Mgr ruling (533e14b492):** Phase A keeps **zero** `06_translate.dag` delta; roster carries only executable greens.

## 4. Inventory matrix (HEAD `merry-fox-418`)

### 4.1 `TargetTypeExpressionProjection` row landed?

| Target | Row fn + bundle edge | MVP1 model has edge | Notes |
|--------|----------------------|---------------------|-------|
| **rust** | ✅ `rust_type_expression_projection` | ❌ (`ProjectionAbsent` — `claim_sg2_mvp1_projection_absent`) | SG-2 claims green; MVP1 uses grammar-inverse fallback |
| **typescript** | ✅ `ts_type_expression_projection` | ❌ (separate sg2 TS claim module) | |
| **python** | ❌ | ❌ | T1 roster green via grammar-inverse only |
| **go** | ❌ | ❌ | `grounding_go/sg_claims.dag` — SG-2 **absence** receipt |
| **cpp** | 🟡 T3 branch (`snappy-owl-682`) | ❌ | `mvp1_cpp_add_translate.dag` defers roster to T3 |
| **dag** | ❌ | ❌ | RTADD keystone only (`mvp1_dag_add_round_trip.dag`) |
| java, kotlin, swift, … | ❌ | ❌ | extdeps present; outside ctrl#1489 MVP tranche |

### 4.2 Six connective arms — rust SG-2 claim coverage (`sg2_type_expression_projection.dag`)

| `TargetTypeExprKind` | `project_type_expression_node` | Serialize golden | Roster Phase A |
|----------------------|-------------------------------|------------------|----------------|
| Atom | via instantiation fixture | indirect | — |
| Instantiation | ✅ structural + falsification | ✅ `Rc<FooBar<X, Y>>` | deferred Phase B |
| Cardinality | (collection paths partial) | — | — |
| Record (Conj) | ✅ structural | ✅ `{ X, Y }` | deferred Phase B |
| Sum (Disj) | partial fixtures | — | deferred Phase B |
| Arrow | ✅ structural + arity reject | ✅ `(X) -> Y` | ✅ rostered |

### 4.3 Shared fold (load-bearing — touch only post-gate)

- **Fold entry:** `project_type_expression_node` / `project_type_expression_node_bounded` (`06_translate.dag` ~2299+).
- **Emitted wire builders:** `target_type_expr_*_emitted` (`target_model.dag` ~1769+).
- **Mode-2 gap:** GAP-1/GAP-2 serialize checklist in `v4-sg2-mode2-non-grammar-emit-design-closure-2026-06-06.md` — blocked on grammar-first A.

### 4.4 Coercion-fold linkage (v3 north star)

- `v3.std.emit_model::TargetIntegerTypeInhabitance` — rows consumed by `v3-grounding-coercion-fold::fold_program_to_target`.
- SG-2 type-expression projection is the **v4 translate** analogue: declared `TargetModel` facts → structural selection, no engine.
- Q1 at-scale rollout should keep v3/v4 projection rows **aligned per target** so coercion-fold and translate do not fork spellings.

## 5. Phased rollout (post-gate)

```text
[GATE] T3 Phase A merge (cpp row + arrow/falsification roster)
  → [GATE] Mgr-SPINE equivalence (T1 + RTADD)
    → Phase B: per-MVP-target row land (python, go, cpp mvp1 attach) using T3 template
      → MVP1 roster: enroll mvp1_*_emit_add_fn where projection row present
        → mode-2 translate (post #4462): GAP-1/GAP-2 + per-kind serialize
          → Extended targets (java/kotlin/swift/…) × 6 connectives
            → Coercion-fold consumer expansion / L6 cross-product row alignment
```

**Parallel lane (not Q1 code):** `sg2_mode2_non_grammar_emit.dag` cert harness — consumer is `translate()` on empty `translation_rules`, not grammar row.

## 6. Non-goals (this PRE-STAGE)

- No edits to `06_translate.dag`, `05_emit.dag`, `target_model.dag` fold helpers, or v3 coercion-fold crate.
- No new roster rows for red connectives (instantiation/conj/sum) until Phase B greens.
- No hollow alias or per-target string tables duplicating projection row token classes (P2 / T-30).
- Shape B format/framework emission (Branch D) — out of scope.

## 7. Escalation triggers

Escalate to Mgr-SPINE (`stern-lynx-374`) if:

- T3 Phase A requires `06_translate.dag` changes to green roster (violates 533e14b ruling).
- Equivalence gate needs a substrate extension not named in `target_model.dag` / `coercion.dag`.
- A target's six-form projection cannot be grounded without a new `TargetTypeExprKind` arm (substrate Mgr / INVARIANTS P1 procedure).

---

**Authority:** inline `.dag` marks on `target_model.dag`, extdeps language files, `sg2_*` claim modules, and `v4_roster_pilot.dag` supersede this planning slice when they diverge.
