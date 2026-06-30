# Syntactic `real-debt` sample audit — headline-number honesty (N≈318)

**Lane:** nimble-ibex-655  
**As-of:** 2026-06-30  
**Audience:** loyal-bee-794 / operator scope sign-off

## Executive summary (honest)

**`real-debt=294` is NOT closed-coproduct audited today.** It is a **path-prefix heuristic** (`is_real_debt_site` in `complexity_linearity_audit_project.rs`) applied after excluding kernel/migration/eval/grammar/open-domain tags. It does **not** inspect whether the wildcard `match` scrutinee is a parameter of a **closed coproduct type**.

The **authoritative closed-coproduct discriminator** is `non_fold_residue_project::residue_sites` (resolved-half):

1. Scrutinee must be a **bare identifier** matching a **function parameter**.
2. Parameter's declared type head must appear in the corpus `closed_coproduct_names` index (`type X = A | B | …` with `|`).
3. The `match` body must have a **top-level** `_ =>` arm.

That census finds **75 live sites** (all rostered; `unrostered=0`). Partition: **23 migration-debt** (drain) + **52 irreducible** (permanent sign-off catalogue).

**Off-roster syntactic wildcards:** 322 total. Triage: `real-debt=294`, `open-domain=25`, plus 3 kernel/migration off-roster.

**Cross-check:** `real_debt ∩ non_fold_roster = ∅` (zero overlap). Every closed-coproduct wildcard is roster-tagged kernel/migration/eval/grammar — none are `real-debt`.

**Automated re-scan of 294 `real-debt` sites** (param-type head vs closed-coproduct index): **~6** closed-coproduct scrutinees, **~287** open/unknown (e.g. `Node`, `String`, field access, unbound scrutinee). **The 294 count is inflated for operator headline purposes.**

### Recommended operator headline (honest)

| Metric | Count | Authority |
|--------|------:|-----------|
| Closed-coproduct wildcard sites (resolved) | **75** | `non_fold_residue` census |
| Migration-debt to eliminate | **23** | Roster partition |
| Irreducible permanent (CONFIDENT) | **47** | Catalogue sign-off |
| Syntactic wildcard (any `_ =>` in fn body) | **397** | Parse-only AST walk |
| Path-heuristic `real-debt` (do **not** sign as fail-open scope) | **294** | Emit-only triage — **demote pending closed-coproduct filter** |

**N=318** ≈ `real-debt (294) + open-domain (25)` off-roster syntactic wildcards. Acceptable **only after** closed-coproduct reclassification demotes mis-tagged sites.

---

## Discriminator comparison

### A) `open-domain` (25 sites) — path heuristic

`is_open_domain_site`: `dsl/extdeps/**`, `dsl/ctrl/**`, `dsl/gunbc/plans/**`, `dsl/test/**`, witness/parse fn name patterns. **Legitimate** open integration surfaces.

### B) `real-debt` (294 sites) — path heuristic (NOT type-grounded)

`is_real_debt_site`: `src/v2/compiler/**`, `src/v2/std/**`, `src/v2/lens/**`, `dsl/std/**`, `dsl/gunbc/**` (non-plans), `dsl/tools/**`, etc.

**Does not** require closed coproduct. A `match node { … _ => … }` on `Node` (open) lands here.

### C) Closed coproduct (75 sites) — construction-grade

`non_fold_residue_project::residue_sites` — see §Executive summary. This is the fail-open eliminator census aligned with §5/§6.

### D) Gate-promotion requirement (not yet implemented for syntactic triage)

`complexity_linearity_audit_project.rs` comment at `walk_expr` notes WallAfterGrounding dissolution: syntactic triage must eventually use the same closed-coproduct resolution as (C), not path prefixes.

---

## Sample spot-check: 10 `real-debt` sites

| # | Site | Scrutinee | Declared type | Closed? | Variant set (if closed) | Verdict |
|---|------|-----------|---------------|---------|-------------------------|---------|
| 1 | `dsl/gunbc/commit_workflow.dag::project_local_tidy_fmt_rows` | `surface` | (unresolved param / record) | **No** | — | **DEMOTE → open-domain or triage-pending** |
| 2 | `dsl/gunbc/compile_source_model.dag::dependency_pool_index_from_flag` | `flag` | `String` | **No** | — | **DEMOTE** — stringly match |
| 3 | `dsl/std/effects.dag::check_modifier_vs_derivation` | (nested) | — | **No** | — | **DEMOTE** — not bare coproduct param |
| 4 | `src/v2/std/compilers/target_model.dag::target_type_expr_emitted_validate_positional_wire` | `kind` | `TargetTypeExprKind` | **Yes** | Atom, Instantiation, Cardinality, Record, Sum, Arrow (6) | **GENUINE** — migrate to total match |
| 5 | `src/v2/std/compilers/target_model.dag::target_type_expr_emitted_validate_labeled_wire` | `kind` | `TargetTypeExprKind` | **Yes** | (same 6) | **GENUINE** |
| 6 | `src/v2/compiler/05_eval.dag::eval_node_is_effect_io_roundtrip` | `effect_io` | `EffectIoEvalContext` | **Yes** | Absent, Present (2) | **GENUINE** — not on migration roster today |
| 7 | `src/v2/lens/testgen.dag::testgen_emit_language_behavior_equivalence_claim` | `anchor` | `ManualAnchorKey` | **Yes** | (closed enum) | **GENUINE** — lens testgen debt |
| 8 | `dsl/std/cache_interface.dag::cache_layer_beats_recompute_path` | (call/field) | — | **No** | — | **DEMOTE** |
| 9 | `dsl/gunbc/host_standup.dag::host_standup_step_is_effective_gap` | `w` | (record field) | **No** | — | **DEMOTE** |
| 10 | `src/v2/program.dag::program_runtime_bool_true` | `value` | `RuntimeValue` | **Yes** | (large runtime coproduct) | **Already irreducible roster** — mis-tagged `real-debt` if off-roster path hits; on roster → kernel |

**Spot-check result:** majority of `real-debt=294` are **not** closed-coproduct fail-opens. Honest eliminator scope remains **migration-debt 23** (+ Wave drains), not 294.

---

## CANDIDATE demotions (parallel work)

Apply closed-coproduct filter to syntactic triage before operator signs N=318:

1. Re-tag sites where scrutinee is not a closed-coproduct param → `open-domain` or `syntactic-wildcard-unclassified`.
2. Re-tag rostered sites → existing kernel/migration/eval/grammar buckets (never `real-debt`).
3. Re-count; expect `real-debt` to drop from **294** to **O(10–30)** genuinely unrostered closed-coproduct debt.

Drain lane continues unaffected (grammar Wave 0, eval_bind PROPOSE).
