# Non-fold irreducible residue catalogue — operator permanent sign-off artifact

**Lane:** nimble-ibex-655 (complexity/linearity audit-first)  
**Roster authority:** `src/v1/stage0/src/non_fold_residue_project.rs` (`NON_FOLD_RESIDUE_ROSTER` ∖ `NON_FOLD_MIGRATION_DEBT_ROSTER`)  
**As-of:** 2026-06-30, HEAD `1f5d517f7b` (grammar-ladder Wave 1 drained −5 migration slots; ManualAnchorKey +6 enrolled → **24/24** migration-debt live)

This document is the **per-site catalogue** for operator permanent-residue sign-off on the **52 irreducible-kernel** roster slots. Each row is one `file::fn` site where a `match` on a **closed coproduct parameter** carries a top-level `_ =>` wildcard arm that is **correct semantics**, not unmigrated modeling debt.

**Sign-off scope:** 47 **CONFIDENT** rows below. **5 CANDIDATE → route-to-migration** rows are explicitly excluded from permanent sign-off pending review (§8).

---

## 1. Arithmetic reconciliation (exact)

| Quantity | Count | Meaning |
|----------|------:|---------|
| Full exception roster slots | **76** | `NON_FOLD_RESIDUE_ROSTER.len()` |
| Migration-debt roster slots | **24** | Must drain; floor RED on 24 if migration roster fiction dropped |
| **Irreducible roster slots** | **52** | **76 − 24**; subject of this catalogue |
| Live irreducible on census | **52/52** | `non_fold_residue_irreducible_live_count()` |
| Syntactic wildcard finding (all 52) | **52/52** | `complexity_linearity_audit` `syntactic_match_wildcard_arm` on every irreducible site |
| Syntactic triage `kernel-permanent` tag (subset) | **27/52** | `triage_wildcard` substring table (`*_eq`, `*dominates*`, `*lattice_join*`, …) |
| Syntactic triage **not** `kernel-permanent` (irreducible only) | **25/52** | Roster partition is authoritative; triage heuristics are emit-only and incomplete (🟡 until on-carrier, #5966) |
| Corpus-wide `kernel-permanent` triage tags | **54** | Includes all wildcard findings corpus-wide that match the substring table (not limited to this 52-list) |
| **7-class table sum** | **52** | See §2 |
| **CONFIDENT irreducible** | **47** | Excludes 5 CANDIDATE rows (§8) |
| **CANDIDATE → migration** | **5** | Do **not** sign permanent until reviewed |

### Resolved-half vs syntactic-half

- **Resolved-half (`non_fold_residue`):** `match <bare-fn-param> { … top-level `_ =>` … }` where the param's declared type head is a **closed coproduct** (`type X = A | B | …`). This is the floor gate census; all 52 sites are live here.
- **Syntactic-half (`complexity_linearity_audit`):** any `Match` node with a `Wildcard` arm anywhere in the fn `Node.body` AST (strictly broader — catches nested wildcards, e.g. `diagnostic_interface_kind_eq`, `multiply_classes`).
- **Roster-but-not-syntactic-triage-tagged:** all 52 have syntactic **findings**; **25** lack the interim `kernel-permanent` **triage tag** because their fn names do not match the hand-Rust substring table (`constant_bound_value`, `symbolic_product`, `qn_fold_step`, …). They are irreducible by **roster partition**, not by triage heuristics.

### Recognition rules (shared)

| Layer | Rule | If disabled |
|-------|------|-------------|
| Resolved gate | `non_fold_residue_site_is_rostered(site)` must be true for every live closed-coproduct wildcard site | `live_tree_no_unrostered_non_fold_residue` goes RED |
| Roster ratchet | Roster entry with no live residue is stale | `live_tree_residue_roster_has_no_stale_entries` goes RED |
| Syntactic witness | `syntactic_match_wildcard_arm` finding on fn body | `syntactic_audit_witness_test.dag` count witnesses fire |
| Triage tag (emit-only) | `triage_wildcard` closed-coproduct param resolution + roster bucket | Informational TSV column only — **not** a wall |

---

## 2. Class summary

| Class | Count | Role |
|-------|------:|------|
| **OffDiagonalEq** | 18 | Equality on closed coproduct: diagonal arms compare payloads; `_ => false` is off-diagonal inequality |
| **LatticeJoinMeet** | 10 | Semilattice join/meet/step with absorbing top/bottom or default combiner |
| **BooleanCollapse** | 10 | Predicate true only on named variants; wildcard is false/absent |
| **LatticeDomination** | 3 | Partial-order dominance; wildcard is non-domination or reflexive default |
| **FoldStep** | 5 | Fold/witness step with error sink or reflexive hold on non-excepted shapes |
| **AtomCoercion** | 4 | Typed atom projection with `optional_absent()` on mismatch |
| **ExitPredicate** | 2 | `exit_ok`: success only on `ExitSuccess` |
| **Total** | **52** | |

---

## 3. OffDiagonalEq (18)

| Site | Line | Scrutinee type | Wildcard arm | Why irreducible | Recognition rule |
|------|-----:|----------------|--------------|-----------------|------------------|
| `dag/gunbc/generated_artifact.dag::artifact_eq` | 82 | `GeneratedArtifact` | `_ => false` | Same-constructor arms compare fields; cross-constructor pairs are unequal by definition. | Roster + `_eq`; `match a: GeneratedArtifact` top-level wildcard. |
| `dag/gunbc/commit_workflow.dag::commit_workflow_surface_eq` | 114 | `CommitWorkflowSurface` | `_ => false` | Surface variants equal only on identical constructors. | Roster + `_eq`. |
| `dag/gunbc/commit_workflow.dag::gate_eq` | 133 | `Gate` | `_ => false` | Gate identity is diagonal on matching variants. | Roster + `_eq`. |
| `dag/gunbc/commit_workflow.dag::local_tidy_check_eq` | 169 | `LocalTidyCheck` | `_ => false` | Workflow tidy rows equal only on same constructor. | Roster + `_eq`. |
| `dag/std/effects.dag::key_source_eq` | 45 | `KeySource` | `_ => false` | Key provenance equal only on matching source constructors. | Roster + `_eq`. |
| `src/v2/std/effects.dag::key_source_eq` | 117 | `KeySource` | `_ => false` | (duplicate authority row — v2 std re-export) | Roster + `_eq`. |
| `dag/std/induction.dag::recursion_shape_eq` | 38 | `RecursionShape` | `_ => false` | Recursion-shape equality is diagonal-only. | Roster + `_eq`. |
| `dag/std/induction.dag::shrink_factor_eq` | 95 | `ShrinkFactor` | `_ => false` | Shrink-factor equality compares same-constructor payloads. | Roster + `_eq`. |
| `dag/std/induction.dag::sub_value_structural_eq` | 114 | `SubValueRelation` | `_ => false` | Structural sub-value equality names diagonal constructors; cross-variant unequal. | Roster + `_eq`. |
| `src/v2/lens/idempotency.dag::idempotency_verdict_eq` | 126 | `IdempotencyVerdict` | `_ => false` | Verdict equality is diagonal on identical constructors. | Roster + `_eq`. |
| `src/v2/lens/ownership.dag::ownership_mode_eq` | 102 | `OwnershipMode` | `_ => false` | Ownership modes equal only on same mode. | Roster + `_mode_eq` heuristic. |
| `src/v2/lens/parallelism.dag::parallelism_relation_eq` | 198 | `ParallelismRelation` | `_ => false` | Parallelism relation equality is diagonal. | Roster + `_relation_eq`. |
| `src/v2/lens/registry.dag::lens_id_v0_eq` | 89 | `LensIdV0` | `_ => false` | Lens id v0 equal only on matching ids. | Roster + `_eq`. |
| `src/v2/lens/unused_parameters.dag::use_relation_eq` | 138 | `UseRelation` | `_ => false` | Use-relation equality is diagonal on identical use shapes. | Roster + `_relation_eq`. |
| `src/v2/std/determinism.dag::determinism_class_eq` | 47 | `DeterminismClass` | `_ => false` | Determinism class equality is diagonal. | Roster + `_eq`. |
| `src/v2/std/determinism.dag::non_det_source_eq` | 26 | `NonDetSource` | `_ => false` | Non-det source tags equal only on same constructor. | Roster + `_eq`. |
| `src/v2/std/node_minimal.dag::node_superset_field_eq` | 254 | `NodeSupersetField` | `_ => false` | Superset field tags equal only on same field. | Roster + `_eq`. |
| `src/v2/std/probe_selector.dag::diagnostic_interface_kind_eq` | 100 | `DiagnosticInterfaceKind` | `_ => false` (nested per left arm) | Per-kind nested equality: right must match left kind; off-diagonal false. | Roster + `_eq`; nested `match right` wildcards. |

---

## 4. LatticeDomination (3)

| Site | Line | Scrutinee type | Wildcard arm | Why irreducible | Recognition rule |
|------|-----:|----------------|--------------|-----------------|------------------|
| `src/v2/lens/complexity.dag::complexity_bound_dominates` | 299 | `ComplexityBound` | `_ => false` | Named comparable complexity arms decide; incomparable/unknown bounds do not dominate. | Roster + `dominates` in fn_name. |
| `src/v2/lens/cost.dag::asymptotic_class_dominates` | 427 | `AsymptoticClass` | `_ => true` | Reflexive/comparable dominance; wildcard is the positive default where the partial order says "at least as asymptotic". | Roster + `dominates`. |
| `src/v2/lens/cost.dag::symbolic_cost_dominates` | 488 | `SymbolicCost` | `_ => false` | Dominance false unless a named comparison arm proves it. | Roster + `dominates`. |

---

## 5. LatticeJoinMeet (10)

| Site | Line | Scrutinee type | Wildcard arm | Why irreducible | Recognition rule |
|------|-----:|----------------|--------------|-----------------|------------------|
| `dag/std/encoding.dag::encoding_lattice_join` | 72 | `Encoding` | `_ => Unknown` | Encoding join top is Unknown for incompatible pairs. | Roster + `lattice_join`. |
| `dag/std/encoding.dag::encoding_lattice_meet` | 40 | `Encoding` | `_ => ASCII` | Meet bottoms at ASCII for unlisted pairs. | Roster + `lattice_meet`. |
| `dag/std/reducible.dag::reduce_verdict_combine` | 11 | `ReduceVerdict` | `_ => match b { … }` | Unknown poisons; non-Unknown `a` defers to `b` — semigroup combine wildcard. | Roster; `match a: ReduceVerdict`. |
| `dag/std/termination.dag::descent_evidence_lattice_join` | 34 | `DescentEvidence` | `_ => NonIncreasing` / absorb | Join on {Strict, NonIncreasing, DescentUnknown} with Strict top. | Roster + `lattice_join`. |
| `dag/std/termination.dag::descent_evidence_lattice_meet` | 18 | `DescentEvidence` | `_ => DescentUnknown` | Meet bottoms at DescentUnknown. | Roster + `lattice_meet`. |
| `dag/std/termination.dag::promote_to_strict` | 52 | `DescentEvidence` | `_ => DescentUnknown` | Promotion preserves Strict/NonIncreasing; unknown stays unknown. | Roster; `match evidence`. |
| `src/v2/lens/cost.dag::multiply_classes` | 507 | `AsymptoticClass` | `_ => ClassExponential` (nested on `b`) | Sparse class multiplication table; exponential absorbs product on unmatched `b`. | Roster; nested `match b: AsymptoticClass`. |
| `src/v2/lens/cost.dag::symbolic_max` | 254 | `SymbolicCost` | `_ => false` | Max false when neither dominates — incomparable off-diagonal. | Roster; join on SymbolicCost. |
| `src/v2/lens/cost.dag::symbolic_product` | 239 | `SymbolicCost` | `_ => ProductCost { lhs: a, rhs: b }` | Default binary product when shapes are not already decomposed. | Roster; `match a: SymbolicCost`. |
| `src/v2/lens/cost.dag::symbolic_sequential` | 226 | `SymbolicCost` | `_ => SumCost { lhs: a, rhs: b }` | Default sequential sum when operands are not decomposed. | Roster; `match a: SymbolicCost`. |

---

## 6. ExitPredicate (2)

| Site | Line | Scrutinee type | Wildcard arm | Why irreducible | Recognition rule |
|------|-----:|----------------|--------------|-----------------|------------------|
| `dag/tools/ci_gates.dag::exit_ok` | 19 | `ProcessExit` | `_ => false` | Only `ExitSuccess` is ok; all failure exits are not ok. | Roster + `fn_name == exit_ok`. |
| `dag/tools/generated_artifact_gate.dag::exit_ok` | 18 | `ProcessExit` | `_ => false` | (same kernel — generated gate re-export) | Roster + `exit_ok`. |

---

## 7. BooleanCollapse (10)

| Site | Line | Scrutinee type | Wildcard arm | Why irreducible | Recognition rule |
|------|-----:|----------------|--------------|-----------------|------------------|
| `dag/std/effects.dag::create_double_init_collapsible` | 73 | `EffectShape` | `_ => false` | Only `CreateDoubleInit` is collapsible. | Roster; `match a: EffectShape`. |
| `dag/std/effects.dag::create_effect_is_dedupable` | 62 | `EffectShape` | `_ => false` | Dedupability is a Create-only property. | Roster; `match shape: EffectShape`. |
| `dag/std/filesystem.dag::is_text_encoding` | 35 | `Encoding` | `_ => false` | Text-safe encoding predicate true only on listed encodings. | Roster; `match e: Encoding`. |
| `src/v2/lens/fact_density.dag::connective_is_kernel_ambient_atom` | 64 | `Connective` | `_ => false` | True only on kernel-ambient Atom connectives. | Roster; `match c: Connective`. |
| `src/v2/program.dag::program_runtime_bool_false` | 261 | `RuntimeValue` | `_ => false` | False only from `Bool` false literal. | Roster; runtime bool collapse. |
| `src/v2/program.dag::program_runtime_bool_true` | 253 | `RuntimeValue` | `_ => false` | True only from `Bool` true literal. | Roster; runtime bool collapse. |
| `src/v2/std/float.dag::float_body_is_nan` | 70 | `FloatBody` | `_ => false` | NaN only on the NaN body variant. | Roster; `match x: FloatBody`. |
| `dag/std/computation.dag::constant_bound_value` | 130 | `SizeBound` | `_ => none` | **CANDIDATE** — see §8. | Roster; partial SizeBound arms. |
| `dag/std/computation.dag::is_constant_bound` | 119 | `SizeBound` | `_ => false` | **CANDIDATE** — see §8. | Roster; partial SizeBound arms. |
| `dag/std/induction.dag::is_strict_style_structural` | 141 | `SubValueRelation` | `_ => false` | **CANDIDATE** — see §8. | Roster; `match r: SubValueRelation`. |

---

## 8. AtomCoercion (4)

| Site | Line | Scrutinee type | Wildcard arm | Why irreducible | Recognition rule |
|------|-----:|----------------|--------------|-----------------|------------------|
| `src/v2/std/compilers/target_model.dag::source_atom_value_as_symbol` | 3163 | `SourceAtomValue` | `_ => optional_absent()` | Symbol projection absent on non-symbol atoms. | Roster; atom coercion family. |
| `src/v2/std/compilers/target_model.dag::source_atom_value_as_bool` | 3170 | `SourceAtomValue` | `_ => optional_absent()` | Bool projection absent on non-bool atoms. | Roster; atom coercion family. |
| `src/v2/std/compilers/target_model.dag::source_atom_value_as_char` | 3177 | `SourceAtomValue` | `_ => optional_absent()` | Char projection absent on non-char atoms. | Roster; atom coercion family. |
| `src/v2/std/compilers/target_model.dag::source_atom_value_as_string` | 3184 | `SourceAtomValue` | `_ => optional_absent()` | String projection absent on non-string atoms. | Roster; atom coercion family. |

---

## 9. FoldStep (5)

| Site | Line | Scrutinee type | Wildcard arm | Why irreducible | Recognition rule |
|------|-----:|----------------|--------------|-----------------|------------------|
| `src/v2/lens/complexity.dag::complexity_bound_from_class` | 213 | `AsymptoticClass` | `_ => match variables { … }` | Sized classes delegate to witness-held variables; wildcard bucket is witness-gated unknown handling. | Roster; top-level `_ =>` on `class`. |
| `src/v2/lens/cost.dag::symbolic_cost_witness` | 147 | `SymbolicCost` | `_ => Holds { value: c }` | Only `UnknownCost` violates; constructed costs witness reflexively. | Roster; `match c: SymbolicCost`. |
| `src/v2/std/qualified_name.dag::qn_fold_step` | 83 | `QnFoldStatus` | `_ => QnFoldError { … }` | Fold propagates first error; wildcard is invalid-structure sink. | Roster; `match acc: QnFoldStatus`. |
| `dag/std/induction.dag::compose_sub_value` | 226 | `SubValueRelation` | `_ => StrictSubValue { … }` | **CANDIDATE** — see §8. | Roster; default strict step on non-unknown base. |
| `dag/std/induction.dag::compose_sub_value_relations` | 234 | `SubValueRelation` | `_ => NonIncreasingValue` (nested) | **CANDIDATE** — see §8. | Roster; nested relation composition wildcards. |

---

## 10. CANDIDATE → route-to-migration (5) — **exclude from permanent sign-off**

These sites are on the irreducible roster today but **fail the operator failure mode test** (migration debt masquerading as permanent residue). Route to migration-debt drain; do **not** include in permanent sign-off until reviewed.

| Site | Concern |
|------|---------|
| `dag/std/computation.dag::constant_bound_value` | `SizeBound` has 8 variants; only 3 named — wildcard `none` masks 5 structural bounds (CollectionSize, TreeSize, …). Likely intentional filter, not off-diagonal semantics. |
| `dag/std/computation.dag::is_constant_bound` | Same partial coverage as `constant_bound_value`. |
| `dag/std/induction.dag::compose_sub_value` | Catch-all `_ => StrictSubValue` default for every non-unknown base — composition table should be explicit. |
| `dag/std/induction.dag::compose_sub_value_relations` | Nested wildcards route unmatched arg relations to `NonIncreasingValue` — relation algebra may need explicit per-pair arms. |
| `dag/std/induction.dag::is_strict_style_structural` | Predicate false on all non-listed `SubValueRelation` shapes — verify each variant before permanent residue. |

**Action:** reclassify to `NON_FOLD_MIGRATION_DEBT_ROSTER` when drain lane picks them up, or prove irreducibility and move back to CONFIDENT with operator re-sign.

---

## 11. Dissolution notes

- Per-entry kernel-vs-migration tagging is **hand-maintained in this doc and the roster** until gunbc#5364 grounds a pure `.dag` Node-tree reader (DESIGN §6 — mark on carrier is authority).
- Syntactic `triage_wildcard` substring table is emit-only (🟡 `decl_facts` #5966 dissolution).
- Exhaustiveness-by-default in the typechecker would collapse OffDiagonalEq wildcards to named off-diagonal arms — roster entries then become documentation of the algebraic law, not a fail-open escape.
