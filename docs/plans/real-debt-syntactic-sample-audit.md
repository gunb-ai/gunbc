# Syntactic audit triage — headline-number honesty

**Lane:** nimble-ibex-655  
**As-of:** 2026-06-30, HEAD `1f5d517f7b`  
**Audience:** loyal-bee-794 / operator scope sign-off

## Executive summary (current code)

**The path-heuristic `real-debt` / `open-domain` site classifiers are retired.** `complexity_linearity_audit_project.rs` no longer uses `is_real_debt_site` or `is_open_domain_site`. Syntactic triage is grounded on **closed-coproduct param resolution** at the AST walk:

1. Any `match` with a top-level `_ =>` arm is a syntactic finding.
2. `triage_wildcard(site, fn_name, has_closed_coproduct_wildcard)` gates on whether the scrutinee is a **bare fn param** whose declared type head is in the corpus closed-coproduct index (`fn_param_type_heads` + `non_fold_residue_closed_coproduct_type_names()`).
3. If not closed-coproduct → **`open-domain`** (legitimate wildcards over open/primitive domains — permanently out of the elimination wave).
4. If closed-coproduct + on migration roster → **`migration-debt`**; + on irreducible roster → **`kernel-permanent`**; + off roster → **`closed-coproduct-debt`** (must drain or enroll — §5 invariant).

**Resolved-half authority** (floor gate): `non_fold_residue_project::residue_sites` — conservative bare-param + closed-coproduct-type + top-level `_ =>`.

| Metric | Count | Authority |
|--------|------:|-----------|
| Full exception roster slots | **76** | `NON_FOLD_RESIDUE_ROSTER.len()` |
| Migration-debt roster slots | **24** | `NON_FOLD_MIGRATION_DEBT_ROSTER.len()` |
| Irreducible roster slots | **52** | **76 − 24** |
| Live closed-coproduct sites (resolved) | **76/76** rostered | `non_fold_residue_unrostered_count() == 0` |
| Syntactic wildcard findings (whole corpus) | **392** | Parse-only AST walk |
| On-roster syntactic wildcards | **76** | Same set as resolved half |
| Off-roster syntactic wildcards | **316** | All tagged **`open-domain`** |
| **`closed-coproduct-debt` (unrostered)** | **0** | §5 invariant — nothing hidden from floor |

**Operator headline (locked):** **N=24** migration-debt roster slots to drain (closed-coproduct fail-opens on `NON_FOLD_MIGRATION_DEBT_ROSTER`); **316** open-domain stay permanently out. Prior **N=29** count superseded after grammar-ladder Wave 1 (−5 roster) and roster reconciliation on HEAD. **N=318** (`real-debt=294` + path heuristics) is **retired**.

---

## Discriminator comparison (landed)

### A) `open-domain` (316 sites) — closed-coproduct filter

Wildcard `match` where scrutinee is **not** a closed-coproduct fn param (open `Node`/`String`, field access, nested match, extdeps witness parsers, etc.). **Not** elimination-wave scope.

### B) Resolved-half roster (76 sites) — construction-grade

`non_fold_residue` census: bare-param + declared closed-coproduct type + top-level `_ =>`. Partition: **24 migration-debt** (drain) + **52 irreducible** (permanent sign-off catalogue).

### C) Syntactic triage tags (emit-only TSV)

| Tag | Live count (2026-06-30) | Meaning |
|-----|------------------------:|---------|
| `open-domain` | 316 | Not closed-coproduct-param wildcards |
| `migration-debt` | 24 | On `NON_FOLD_MIGRATION_DEBT_ROSTER` |
| `kernel-permanent` | 52 | On irreducible roster |
| `closed-coproduct-debt` | 0 | Unrostered closed-coproduct — must stay at 0 |
| `eval-interpreter-debt` / `grammar-ladder-debt` | 0 | Substring backstops; roster partition is authoritative |

### D) Superseded (do not cite)

- `is_real_debt_site` / `is_open_domain_site` path-prefix heuristics — **removed**.
- `real-debt=294` headline — **retired** (pre closed-coproduct filter).
- “Pending closed-coproduct filter” — **landed** in `triage_wildcard` (`complexity_linearity_audit_project.rs:267-304`).

---

## Sample spot-check (closed-coproduct-grounded)

| # | Site | Closed-coproduct? | Roster bucket | Syntactic tag |
|---|------|-------------------|---------------|---------------|
| 1 | `src/v2/lens/testgen.dag::testgen_emit_language_behavior_equivalence_claim` | Yes (`ManualAnchorKey`) | migration-debt | migration-debt |
| 2 | `src/v2/compiler/01_tokenize.dag::lex_try_rules_prefer_longer` | Yes | migration-debt | migration-debt |
| 3 | `dsl/extdeps/cron/schedule_model.dag::render_cron_field` | No | — | open-domain |
| 4 | `src/v2/lens/cost.dag::asymptotic_class_dominates` | Yes | irreducible | kernel-permanent |
| 5 | `dsl/tools/generated_artifact_gate.dag::exit_ok` | Yes | irreducible | kernel-permanent |

Witness: `syntactic_audit_witness_test.dag` executes interpreter builtins over witness roots; site-pinned rows are intentional dissolution ratchets.

---

## Drain lane (unchanged)

Grammar-ladder Wave 1 (-5 roster, exhaustive `ConcreteSyntaxToken` arms) and eval_bind PROPOSE are separate escalate-review PRs. This doc tracks **census honesty** only; roster edits land in the same PR as the fold.
