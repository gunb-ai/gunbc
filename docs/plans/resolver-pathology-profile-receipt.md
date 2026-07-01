# Resolver pathology — cold-resolve profile receipt

**Status:** measurement doc only (no resolver refactor in this PR). **DESIGN.md + the carriers remain the authority** — this doc is a timestamped profiling receipt for ROADMAP §1 *profile the 518s resolve*, not a fact ledger.

**Owner:** vivid-carp-798 (`1-resolver-pathology-a`). Linked from `ROADMAP.md` §1 and `dsl/gunbc/roadmap_authority.dag` (`1-resolver-pathology`).

**Host:** container session, linux x86_64, 2026-07-01. `claim_batch` / `claim_executor` built `release` locally (`CTRL_BUILD_BYPASS_SHIMS=1 cargo build -p v1-compiler --release --bin claim_batch --bin claim_executor`). `GUNBC_RESOLVED_GRAPH_CACHE_DIR` **unset** (cross-process resolve cache off).

---

## 1. Reproducible profile commands

### A. Per-entry resolve over the discovery roster (primary receipt)

Emits one `[resolve] <entry>: <ms> (<modules> modules, <items> resolved items in closure)` line per unique witness entry. The `MultiEntryIndex` **parse_cache** and **typed_module_cache** warm across entries (matches `run_discovery_corpus` serial-sum semantics).

```bash
unset GUNBC_RESOLVED_GRAPH_CACHE_DIR
./scripts/profile-cold-resolve.sh          # full corpus log + top-15 summary
./scripts/profile-cold-resolve.sh --top 25 # wider tail
```

Equivalent one-liner:

```bash
unset GUNBC_RESOLVED_GRAPH_CACHE_DIR
./target/release/claim_batch \
  --source-root src/v2 --source-root dsl \
  --scan-dir src/v2/test/claim \
  --scan-dir dsl/test/claim \
  --roster-from-discovery \
  --claim-run --wet 2>&1 | tee /tmp/cold-resolve.log
```

Aggregate serial sum:

```bash
rg '\[resolve\]' /tmp/cold-resolve.log \
  | sed -E 's/.*: ([0-9]+)ms.*/\1/' | awk '{s+=$1} END {print s "ms"}'
```

### B. Pathological pair only (budget_roster vs structural twin)

```bash
./scripts/profile-cold-resolve.sh --pair
```

### C. Full floor (resolve + gates + eval) — optional cross-check

```bash
unset GUNBC_RESOLVED_GRAPH_CACHE_DIR
GUNBC_FLOOR_GANTT=1 ./target/release/claim_executor \
  --source-root src/v2 --source-root dsl \
  --plan-entry src/v2/workflow/ci_floor_plan.dag \
  --plan-function gunbc_ci_floor_batches \
  --notice-title resolver-pathology-profile 2>&1 | tee /tmp/floor.log
```

Batch 3 `[measurement] discovery corpus: … resolve …ms` line is the operator-facing serial-sum cross-check for the ~518s ROADMAP figure (corpus size and cache state must match the cited baseline).

---

## 2. Measured corpus resolve (this run)

**Partial corpus pass:** 293 / 447 unique entry groups resolved before host session moved on; serial sum **93,029 ms** (~93 s) with warm `typed_module_cache`. Linear extrapolation to 447 entries → **~142 s** serial resolve (still below the operator **~518 s** figure, which counts a larger witness roster on CI arm64 and includes gate-batch resolves that re-pay heavy closures cold).

| Metric | This run (warm index) | ROADMAP operator prior |
| --- | --- | --- |
| Unique entry files | 447 rostered (293 measured) | ~870 witnesses / ~N unique entries |
| Resolve serial sum | 93 s (293 entries); ~142 s projected | ~518 s (full floor, cold-dominated) |
| Avg per unique entry | ~317 ms (warm) | ~600 ms/witness amortized (includes non-resolve work in places) |

**Interpretation:** the gap vs 518 s is expected — (1) `typed_module_cache` reuse across entries with shared `v2.compiler.*` / lens imports, (2) smaller roster than the cited 870-witness baseline, (3) gate batches not included in the partial `claim_batch` sum.

---

## 3. Top resolve offenders (warm index, ms → items → modules → entry)

| Rank | Resolve ms | Items | Modules | Entry |
| --- | ---: | ---: | ---: | --- |
| 1 | 9,410 | 1,973 | 38 | `src/v2/compiler/manual/dag_import_block_lexeme_stamp_test.dag` |
| 2 | 6,164 | 2,594 | 40 | `src/v2/test/claim/emit_host_gate/target_coverage_completeness_test.dag` |
| 3 | 5,459 | 3,199 | 84 | `src/v2/lens/application/sg_claims_test.dag` |
| 4 | 4,757 | 2,789 | 60 | `src/v2/compiler/manual/typescript_import_pipeline_test.dag` |
| 5 | 3,999 | 1,252 | 148 | `dsl/test/claim/build_artifact_verification_witness_test.dag` |
| 6 | 2,869 | 2,772 | 52 | `src/v2/test/claim/bash_command_fold_test.dag` |
| 7 | 2,687 | 2,848 | 70 | `src/v2/lens/affected_set/edit_locus_resolver_test.dag` |
| 8 | 2,679 | 2,286 | 208 | `dsl/test/claim/generated_artifact_drift_test.dag` |
| 9 | 2,183 | 2,983 | 78 | `src/v2/lens/affected_set/sg_claims_test.dag` |
| 10 | 1,860 | 1,460 | 157 | `dsl/test/claim/ci_yaml_serializer_witness_test.dag` |
| 11 | 1,808 | 2,687 | 58 | `src/v2/lens/testgen/dag_input_surface_test.dag` |
| 12 | **1,556** | **2,412** | **57** | **`src/v2/test/claim/complexity_gate/budget_roster_completeness_test.dag`** |
| 13 | 1,684 | 2,109 | 182 | `src/v2/test/claim/ci_floor_plan_witness_test.dag` |
| 14 | 1,484 | 1,442 | 157 | `dsl/test/claim/ci_deploy_witness_test.dag` |
| 15 | 1,043 | 1,175 | 87 | `dsl/test/claim/live_deploy/emit_test.dag` |

**Takeaway:** a handful of compiler-manual, emit-host, and lens-application entries dominate warm resolve wall. `budget_roster` is top-15 but **not** rank-1 on entry-level resolve ms.

---

## 4. Smallest pathological pair — `budget_roster` vs structural twin

### Pair (named in ROADMAP)

| Role | Entry / module | Cold resolve (fresh process) | Modules | Items in closure |
| --- | --- | ---: | ---: | ---: |
| **Pathological (`budget_roster`)** | `budget_roster_completeness_test.dag` | **5,513 ms** | 57 | **2,412** |
| **Roster data authority** | `subject_complexity_budget_roster.dag` | 5,140 ms | 55 | 2,405 |
| **Structural twin** | `fold_list_generic_instantiation.dag` | **159 ms** | 3 | **226** |
| Single-row eval twin | `source_bridged_add_budget_test.dag` | 5,565 ms | 57 | 2,409 |

**Cold resolve wall ratio (pathological / twin):** 5513 / 159 = **34.7×** (not the operator-cited ~450× at entry wall-clock).

**Closure item ratio:** 2412 / 226 = **10.7×**.

**Why this twin:** `fold_list_generic_instantiation.dag` is the smallest in-tree cert that exercises the same `fold_list` algebra over `List<_>` without importing the complexity-gate roster (`List<LensSubjectComplexityBudgetRow>` with embedded `fn() -> Outcome<Node>` producer thunks wired through four `source_bridged_*_subject_producer` modules and the full `v2.compiler.{tokenize,parse,normalize,resolve}` pipeline).

**Cache amplification (same process, warm `typed_module_cache`):** resolving `source_bridged_add_budget_test.dag` then `budget_roster_completeness_test.dag` drops roster resolve **6373 ms → 490 ms** (13×) — shared `v2.compiler.*` / lens modules typecheck once.

**Eval skew (same entry resolve, witness body):**

| Witness | Eval ms |
| --- | ---: |
| `complexity_budget_roster_family_gate_holds` | 10,248 |
| `source_bridged_add_budget_claim_holds` (single row) | 1,360 |
| Ratio | **7.5×** |

The roster gate folds four producer rows through `lens_subject_arrow_body_for_budget_row` (each row re-materializes a source-bridged compiler pipeline at eval). That is witness eval cost, not the entry-resolve timer — but it shares the same fn-typed roster surface.

### Reconciling the operator ~450× figure

No measured **entry-resolve** pair in-tree reaches ~450× wall-clock. The cited ratio plausibly targets **registry item work** inside `reconcile_with_typed_cache` (2412 items vs O(1) row-shaped twin ≈ 2400×) or **per-module `typecheck_module` cost** for `subject_complexity_budget_roster` (fn-typed list literal + four bridged producers) — **phase instrumentation not yet landed** (see §5). This receipt pins the reproducible command and the smallest honest structural twin; follow-on work should add per-phase timers inside `resolve_entry_with_parse_cache`.

---

## 5. Hypothesis — resolver phase responsible

Resolve pipeline (`cli_run.rs` `resolve_entry_with_parse_cache`):

1. parse (+ `parse_cache`)
2. `resolve_modules` (import graph wiring)
3. `normalize_graph`
4. **`reconcile_with_typed_cache` → `typecheck_module` per module in dep order** ← **hypothesis**
5. `extract_ownership_proofs` / `expand_transitive_services`

**Hypothesis:** the pathology is **`typecheck_module` inside `reconcile_with_typed_cache`**, not `resolve_modules` graph wiring. Evidence:

- `budget_roster` and its single-row twin share **the same 57-module / ~2410-item closure** at cold entry resolve (~5.5 s vs ~5.6 s) — import wiring is not the skew.
- The **structural twin** `fold_list_generic_instantiation` drops to **3 modules / 226 items / 159 ms** — the blow-up tracks **typed registry materialization** for fn-typed roster rows + source-bridged compiler imports, not `fold_list` itself.
- Warm `typed_module_cache` collapses subsequent roster resolves to **~0.5 s** — cache hits land in the **typecheck** map keyed by module name (`cli_run.rs` `reconcile_with_typed_cache`).
- Top offenders (§3) cluster in **compiler-manual / lens-application** entries with large `item_registry` fan-out — consistent with per-module typecheck + registry expansion cost.

**Not the primary hypothesis:** cross-process `GUNBC_RESOLVED_GRAPH_CACHE_DIR` (off in this receipt); `resolve_modules` duplicate import wiring; witness eval (separate timer, though roster eval is also hot).

---

## 6. Follow-on (out of scope for this receipt)

- Land per-phase resolve timers (`parse` / `resolve_modules` / `normalize` / `typecheck_module` / `ownership`) behind `GUNBC_RESOLVE_PROFILE=1`.
- Fix fn-typed roster row instantiation path (construction or sharing) once phase receipt confirms `typecheck_module` dominance.
- Extend affected-set pruning to skip unchanged entry resolves (ROADMAP `1-resolve-not-pruned`).

## Dissolution trigger

Delete this doc when resolve-phase timings are emitted as structured substrate rows (realization_measurement_loop / CI observability carrier) and the ROADMAP `1-resolver-pathology` item is marked done with a fix PR — not when this receipt merges.
