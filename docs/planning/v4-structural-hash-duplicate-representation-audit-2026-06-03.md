# Structural hash / duplicate representation audit (v4)

**Author:** CI Manager (silent-crane-669) · **Date:** 2026-06-03  
**Context:** Review of [#4336](https://github.com/gunb-ai/gunbc/pull/4336) (F.11c `CiSelectionReceipt` persist/lookup) surfaced a
second hashing style inside `src/v4/workflow/ci.dag`. This note records the pattern, where it appears,
and why it is a modeling debt — not a one-off PR mistake.

**Related PRs:** [#4352](https://github.com/gunb-ai/gunbc/pull/4352) (disposition-only follow-up; safe to merge
independently). [#4336](https://github.com/gunb-ai/gunbc/pull/4336) (adds ~1k lines of hand-rolled digests; needs
architectural decision before merge). [#4345](https://github.com/gunb-ai/gunbc/pull/4345) (closed wrong-premise).

---

## Executive summary

The language has a **canonical hash for `Node`** (`v4.std.node.content_hash`) and a **CI-local convention**
of `*_projection_node` → `content_hash` for workflow carriers. It does **not** have a generic
**`hash : T -> Hash`** (or codegen from type definitions) for arbitrary typed values.

When a new feature needs a stable key over a nested record (cache key, receipt storage key, claim hash),
authors either:

1. **Culture A (preferred in `ci.dag` on `main`):** project the value to a `Node` tree, then `content_hash`.
2. **Culture B (dominant in `v4.compiler.eval`):** write a tower of `fn foo_digest(...)` functions using
   `combine_hash`, per-coproduct `Symbol` tags, and ad hoc list folds.

**#4336 imports Culture B into `ci.dag`**, duplicating digest logic for std types (`Diagnostic`, `Locus`,
`Extent`, `AffectedSet`, …) that **already** have digest towers in `v4.compiler.eval`. That is duplicate
representation — the same semantic object hashed two different ways in two modules.

This is a **language / substrate gap**, not “CI importing the whole language.” The file grows because
each module becomes a private hash compiler until a shared primitive or generated projections land.

---

## Measured surface (snapshot: `main` @ 2026-06-03)

| Location | `*_digest` fn defs | `_projection_node` uses | `combine_hash` uses | `content_hash` uses |
|----------|-------------------:|------------------------:|--------------------:|--------------------:|
| `src/v4/workflow/ci.dag` | 11 | 148 | 5 | 33 |
| `src/v4/compiler/05_eval.dag` | 34 | — | 47 | 16 |
| `src/v4/std/node.dag` | 10 | 9 | 51 | 14 (defines `content_hash`) |
| `src/v4/workflow/bootstrap.dag` | 0 | 14 | — | 4 |

**#4336 branch (`session/zesty-lark-828`) delta vs `main`:**

| Metric | `main` | #4336 branch |
|--------|-------:|-------------:|
| `ci.dag` lines | 5,309 | 6,292 (+983) |
| `fn ci_*_digest` | 11 | **72** (+61) |
| `ci_*_tag: Symbol` (coproduct tags for digests) | ~53 | **~167** (+114) |
| `fn ci_*_cache_digest` (projection-backed) | 15 | 15 (unchanged) |

So #4336 does **not** extend the established CI cache pattern; it adds a parallel ladder.

---

## Culture A — projection + `content_hash` (CI workflow on `main`)

**Pattern:**

```dag
fn ci_command_projection_node(c: CiCommand) -> Node { … }
fn ci_command_cache_digest(c: CiCommand) -> Hash {
  content_hash(n: ci_command_projection_node(c: c))
}
```

**Where in `ci.dag` (representative):**

| Digest wrapper | Backs onto projection |
|----------------|----------------------|
| `ci_pipeline_cache_digest` | `ci_pipeline_projection_node` |
| `ci_job_cache_digest` | `ci_job_projection_node` |
| `ci_gate_cache_digest` | `ci_gate_projection_node` |
| `ci_command_cache_digest` | `ci_command_projection_node` |
| `ci_upsert_cache_digest` | `ci_upsert_projection_node` |
| `ci_upsert_step_cache_digest` | `ci_upsert_step_projection_node` |
| `ci_lens_id_cache_digest` | `ci_lens_id_projection_node` |
| `ci_gate_run_policy_cache_digest` | `ci_gate_run_policy_projection_node` |

**Properties:**

- Single authority: the `Node` tree is the canonical serial form; `content_hash` applies B1 merkle rules
  (`canonicalize_node_for_content_hash` in `v4.std.node`).
- Coproduct discipline is structural (edges, connectives), not a parallel tag table per module.
- **Cost:** large amount of `ci_*_projection_node` boilerplate (~30 definitions, ~148 call sites in one file).

---

## Culture B — hand-rolled `combine_hash` ladders

**Pattern:**

```dag
fn diagnostic_cache_digest(d: Diagnostic) -> Hash {
  combine_hash(
    a: locus_cache_digest(at: d.at),
    b: combine_hash(
      a: test_claim_symbol_digest(sym: d.reason),
      b: correction_verdict_digest(c: d.correction)
    )
  )
}
```

**Primary home:** `src/v4/compiler/05_eval.dag` (34 `*_digest` functions), including:

| Function | Carrier / purpose |
|----------|-------------------|
| `test_claim_claim_hash_digest` | Whole `TestClaim` (IRT-4 claim hash) |
| `extent_cache_digest` | `Extent` |
| `locus_cache_digest` | `Locus` |
| `diagnostic_cache_digest` | `Diagnostic` |
| `diagnostic_tail_bag_cache_digest` | `List<Diagnostic>` (bag via `bag_hash_digest`) |
| `non_empty_diagnostics_cache_digest` | `NonEmptyDiagnostics` |
| `inferred_tree_digest` | `InferredTree` |
| `interpretation_*_digest` | Interpreter algebra cache slots |
| … | |

**#4336 adds the same style under `ci_*_` prefixes**, e.g. `ci_diagnostic_digest`, `ci_locus_digest`,
`ci_affected_set_digest`, `ci_selection_receipt_storage_key` (nested `combine_hash` over those).

**Failure modes observed in review (why this pattern is fragile):**

- Missing coproduct variant tags → collisions between arms.
- List fold where bag/multiset semantics require order-independence.
- Reading fields that do not exist on the declared substrate type (compile gaps hidden by string-only smoke tests).
- **Duplicate authority:** same std type hashed differently in `eval` vs `ci`.

---

## Duplicate representation (concrete example)

| Type | Authority in `eval` | Introduced in #4336 under `ci` |
|------|---------------------|--------------------------------|
| `Extent` | `extent_cache_digest` | `ci_extent_digest` |
| `Locus` | `locus_cache_digest` | `ci_locus_digest` |
| `Diagnostic` | `diagnostic_cache_digest` | `ci_diagnostic_digest` |
| `Correction` | `correction_verdict_digest` | `ci_correction_digest` |

A receipt storage key built from `ci_diagnostic_digest` is **not guaranteed** to agree with any key path
that used `diagnostic_cache_digest` on the same value. Consumers cannot treat “hash of Diagnostic” as a
global concept.

---

## What is *not* the problem

- **Importing `v4.extdeps.languages/*` into `ci.dag`.** Language files appear as upsert path strings only.
- **`ci.dag` being large for workflow reasons.** Shadow selection, upsert tables, bankruptcy wiring, and
  fixture imports add real domain surface area.
- **#4352.** Two comment-only 🟡 disposition lines; no new digest or projection surface.

---

## What blocks the “proper” end state

| Debt / gate | Blocks |
|-------------|--------|
| No typed `hash<T>(t: T)` or codegen from records/coproducts | Forces per-module digest towers |
| `node://adhoc-331899f9-19a` | Live `ci_selection_receipt_shadow_from_git_diff` (git diff → `AffectedSet` on live `Dag`) |
| `node://adhoc-87c3a213-099` (F.11c tag) | Runtime TestClaimRun / cache owning persisted receipts |
| INVARIANTS reflection (not closed theorem) | Auto-`project : T -> Node` for all carriers |
| Review bar | Line-item P2/P3 on diff, not “one hash path per carrier” |

**Interim F.11c persist/lookup in `.dag` is not blocked** from using Culture A today:
`ci_selection_receipt_projection_node` + `content_hash`, reusing or sharing projections for nested std types.

---

## Recommendations (for operator / modeling pass)

1. **Merge policy**
   - **#4352:** OK — disposition only, no new hash surface.
   - **#4336:** Hold until choose Culture A refactor **or** explicit accept of Culture B with shared std digests (no `ci_*` duplicates of `eval` types).

2. **Hash authority rule (proposed)**
   - For any carrier already in `v4.std.*` or hashed in `v4.compiler.eval`, **workflow modules must not**
     introduce parallel `ci_*_digest` ladders.
   - Extend `*_projection_node` + `content_hash`, or add a **single** shared module (e.g. `v4.std.canonical_hash`)
     with dissolve-on tags.

3. **Engineering direction**
   - Short term: split F.11c into `v4.workflow.ci_selection_receipt` (or similar) — stop growing the 5k-line monolith.
   - Medium term: land **structural hash primitive** or derive projections from type defs (compiler task).
   - Tests: require compile/typecheck witnesses for hash paths, not only `CI_DAG.contains(...)`.

4. **Re-audit command** (re-run after large `ci.dag` / `eval` changes)

   ```bash
   rg 'fn [a-z0-9_]+_digest\(' src/v4 --glob '*.dag' -c | sort -t: -k2 -nr
   rg '_projection_node\(' src/v4/workflow/ci.dag -c
   rg 'content_hash\(n: ci_' src/v4/workflow/ci.dag -c
   git diff --stat origin/main...<branch> -- src/v4/workflow/ci.dag
   ```

---

## Appendix: `ci.dag` digest inventory on `main` (11 functions)

| Function | Style |
|----------|--------|
| `ci_symbol_digest` | Primitive (`content_hash` of atom) |
| `ci_gate_run_policy_cache_digest` | **Culture A** |
| `ci_char_cache_digest` | Special case (byte-offset eligible / ineligible) |
| `ci_string_cache_digest` | `fold_list` + `combine_hash` (Peano string path) |
| `ci_lens_id_cache_digest` | **Culture A** |
| `ci_command_cache_digest` | **Culture A** |
| `ci_job_cache_digest` | **Culture A** |
| `ci_gate_cache_digest` | **Culture A** |
| `ci_pipeline_cache_digest` | **Culture A** |
| `ci_upsert_step_symbol_cache_digest` | Delegates to Culture A |
| `ci_test_claim_evaluator_digest` | `combine_hash` over eval-related inputs |

On `main`, **Culture A dominates** CI-owned cache keys. #4336 would invert that for receipt persistence unless refactored.
