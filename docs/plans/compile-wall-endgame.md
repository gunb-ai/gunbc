# Compile-wall endgame — measured decomposition + the plan to end it

> **Status:** MEASUREMENT + PLAN FOR REVIEW, 2026-07-17, session clever-seal-476 ("investigate typecheck"). Operator ask: "take measurements, then put together a plan end to end to solve this once and for all, with concrete expectations on where we end up" and "be particular about hardware — there are a lot of unsubstantiated claims about fleet contention." Every number below is a receipt from an executed run (command + location cited) or is explicitly marked PENDING; every prior unsubstantiated claim this lane made is listed in §3.4 with its correction. Review the claims against the receipts — that is this PR's purpose.
>
> Parent lanes: [v1-run-stability-throughline](v1-run-stability-throughline.md) (memory axis; this doc is the compute axis), [cross-entry-typed-module-memo-sketch](cross-entry-typed-module-memo-sketch.md) (the A-lane store this plan's W3 realizes), [cross-worker-typecheck-share-design](cross-worker-typecheck-share-design.md), [duplicate-work-graph-lens-design](duplicate-work-graph-lens-design.md) (`ComputationIdentity` — the §2 authority all memo tiers dissolve into), [execution-spine-design](execution-spine-design.md) §2, [floor-shared-compute-memoization](floor-shared-compute-memoization.md) (M1/M2 — realized in part by #6783).
>
> DESIGN refs: §1 (time is the value), §2 (minimize redundancy — the wall is redundancy, three ways at once), §5 (fail-closed selection; cold controls; no absorbing fallback), §6 (bare-minimum cost — a proven cost-shape defect is always fixed; denominate in displaced cost).

## 1. The problem, denominated

A PR's CI floor pays

**wall ≈ corpus × per-module-kernel ÷ per-core-throughput × passes-per-run + fixed steps**

and every factor was (until this week) moving the wrong way: the corpus grows each merge; the newest modules are the most expensive class ever added (CiSpec/HostEffect importers); per-core throughput on the runners is *variable and was unexplained* (§3); and the floor paid the whole-tree typecheck 2–3× per run. Observed end state: 100–118-minute floors (runs 29509404911, 29543941811).

This doc (a) fixes the ground truth with receipts, (b) names the root causes, (c) lays out workstreams with owners and expected wins, (d) commits to end-state targets.

## 2. Measured ground truth — the kernel

All §2 runs: whole-tree compile-clean (`compile_clean_diagnostic_histogram`), single-threaded, instrumented seed (probes on branch `session/clever-seal-476`, never merged), executed in the session container (which runs **on srv2** — kernel btime match, §3.1), **tree = the session base `af57cd65bf` (#6758)** — one day behind `d077057d26` at time of measurement. Which §2 conclusions were re-established on today's `d077057d26` tree and which await the W2 step-0 re-run is stated per-row below; §3.2–3.3 (environment parity, growth) were measured on `d077057d26` directly.

### 2.1 Phase decomposition (346–353s total fold, 1,090 modules, session base)

| phase | share | receipt |
|---|---|---|
| frontend (parse+normalize) | ~4s | `[gantt]` rows |
| **per-module typecheck fold** | **~96%** | `[gate-typecheck]` per-module sum |
| emit leg | ~17s | emit gantt |

Within `typecheck_module`: `infer_items` ≈ 82%, `populate_output_provenance` ≈ 17%, `build_type_env` ≈ 0.3s total (M1a landed — dead as a term).

### 2.2 Mechanism elimination table (the week's measurement arc)

| suspect | measured share of fold | verdict, receipt |
|---|---|---|
| `build_type_env` / import-DAG env construction | 0.1% (0.3s) | **dead** (M1a #6528; tc-split probes) |
| alias-chain expansion `expand_type_for_field_access` | **0.01%** (0.044s; 8,123 calls, chains avg 1.00 step) | **dead on main** (probe run 2026-07-17); on the strip tree it was an **infinite loop**, not a cost — §4 peel guard |
| `resolve_node_bounded` (all resolve work through it) | **12.5%** (43.1s, 733k calls) | real but bounded; ~half of it in ONE module (`host_effect_realize`, 70% rnb) — Option B's honest main-tree ceiling |
| `Node` deep-equality (`PartialEq`, hand-instrumented at the impl) | **0.0006%** (2ms, 15,071 calls) | **dead** |
| `substitute_generics` | **0.005%** (16ms) | **dead** |
| `resolve_scrutinee_type_node_seen` (a second resolve recursion that **bypasses** rnb counters) | ~0% (52,139 calls, sub-ms depth-0 time) | **dead** (battery run) |
| `compute_variant_provenance` | 3.0% (10.4s) | minor (battery run; the item-grain `populate_output_provenance` 17% remains the second walk's honest number) |
| **`infer_expr` (all expression inference, depth-0)** | **81.5% (283.5s of 347.7s)** | **the target.** Coincides with the item-grain `infer_items` ≈ 82% — expression inference IS the fold. Call-storm receipts inside it: **16.9M `authored_name_at` calls** (5.7M in `host_effect_realize` alone; name re-derivation via source-span lookup) and 936K `lookup_binding_by_name`. The next split goes *inside* `infer_expr` (record-literal checking / method-arg folds / the name-derivation storm), on today's tree — W2 step 0. |

(An earlier draft circulated `infer_expr` ≈ 41% — that figure divided by a double-counted denominator, the same awk artifact already registered in §3.4, and is corrected here; 81.5% is the receipt-consistent share. All battery rows are session-base measurements; §3.3's `d077` rows confirm the *concentration* shape carries to today's tree (top-10 = 49%, top-25 = 78%), and the full mechanism split re-runs on today's tree as W2 step 0.)

### 2.3 Concentration (why "the algorithm is inefficient" is fixable)

(Session base; re-established on today's `d077057d26`: top-10 = 49%, top-25 = 78% — §3.3.)

- median module = **4.4ms**; >1,000 modules are single-digit ms — the algorithm is fine on normal input.
- **top-25 modules = 77%** of the fold; all are CiSpec/HostEffect-tower importers.
- cleanest specimen: `gunbc.ci_spec` — 17.4s at **1.2% resolve**; ~2k record-field instances ⇒ **~8ms per field check** ≈ tens of millions of instructions per field. That is a cost-shape defect (§6), not inherent cost.
- `tools.ci_gates`: provenance walk 9.0s vs inference 4.5s — the *second* walk costs 2× the first on some modules.
- precedent that these fall to root fixes: `merge_envs` (reconcile 81%→6%, ~2× self-compile), M1a (env term → 0.1%), #6773 (variant-locals M×K).

## 3. Measured ground truth — hardware (the substantiation section)

Fleet inventory, measured directly this session (ssh receipts; commands in the session transcript):

| box | CPU | max clock | RAM | runner slots | notes |
|---|---|---|---|---|---|
| srv1 | Neoverse-N1 ×128 | 3.0GHz | — | 51 dirs, ~5 listeners live | ondemand governor |
| srv2 | Neoverse-N1 ×128 | 3.0GHz | 128GB | 51 dirs | **hosts this session's container** (kernel btime identical) — all "local" numbers in §2 are srv2-container numbers |
| srv3 | Neoverse-N1 ×128 | 3.0GHz | 125GB | **9** dirs | password-auth box; hosts the `srv3-06` runner from the 118-min run |

### 3.1 The frequency observation, and its refutation (worked example in being particular)

All three boxes run governor `ondemand` (`ignore_nice_load=0`). Two observations that *looked* like a clock story, then the microbenchmark that killed it:

- sysfs `scaling_cur_freq` sampled on the core hosting my free-floating single-threaded fold read **1.0–1.5GHz** on all three boxes (the process migrated cores between samples), while srv1's CI compile processes read **3.0GHz**, and `taskset`-pinned runs read 3.0GHz immediately.
- **Refutation:** `openssl speed sha256` on srv1 — floating vs pinned-core-120 vs pinned-core-5 — is **identical to <2%** (~1.86GB/s at 16K blocks). Delivered per-core compute does not depend on pinning or core choice; the low sysfs readings on a migrating thread are a sampling artifact (by the time the core is read, the thread has moved and the core has down-clocked). **"Clock starvation" is NOT a real term in CI's wall**, and no governor/pinning change is proposed. (An earlier draft of this section proposed exactly that — kept here, crossed out in spirit, as the §3.4 discipline: the claim was written down before the control experiment and did not survive it.)
- Also eliminated by direct measurement: NUMA (both boxes are single-node monolithic — `numactl --hardware`), paging (108GB available, bench VmSwap=0, maj_flt=0 during runs), IRQ concentration on pinned cores (openssl unaffected), and one self-inflicted artifact (a doubled awk aggregation — the pattern `gate-typecheck` also matched `gate-typecheck2` rows — that briefly made pinned container runs look 2× slower AND deflated an early `infer_expr` share to 41%; both corrected in §2.2, per-module rows were always consistent).
- Positive contention datapoint: bisect legs 2–3 ran as two concurrent whole-tree folds on this box and produced session-base-level per-module times (29.4s/29.6s vs 28.6s single) — two co-running folds cost each other ≤3%.

### 3.2 Cross-environment benchmark (identical binary, identical tree `d077057d26`, single thread, pinned) — RESULT: PARITY

| environment | whole fold (1,112 modules) | `host_effect_realize` | `srv3_os_install_reconcile_receipt` | `ci_spec` | receipt |
|---|---|---|---|---|---|
| srv2 container, pinned | **2,413s** | 171.7s | 142.1s | 17.8s | bench-container-d077.log |
| srv1 host, pinned | **2,355s** | 168.1s | 139.9s | 17.4s | /tmp/bench-srv1-pinned.log |
| srv2 host, pinned | **2,335s** | 167.1s | 136.9s | 17.1s | /tmp/bench-srv2-pinned.log |
| srv3 host, pinned | **2,341s** | 167.1s | 137.3s | 17.1s | /tmp/bench-srv3-pinned.log |
| (context) srv2 container, session-branch tree (**one day** older base `af57cd65bf`) | 346–353s | 28.6s | 3.7s | 17.4s | eq_probe/rnb_probe/battery logs |

**Total cross-environment spread: ~3%.** Container == host; srv1 == srv2 == srv3. There is no slower box, no container tax, no measurable contention term at current load. The entire 7× difference between the two rows-groups is the TREE — one day of merges (§3.3). This also resolves the lively-heron >13-min-vs-453s discrepancy that seeded the "srv3 2× slower" claim: the two measurements ran different trees (current-heavy vs session-light), not different hardware speeds.

### 3.3 The tree-growth finding (CONFIRMED in-container)

The 118-min run's CI log showed `host_effect_realize` at **213s** and `srv3_os_install_reconcile_receipt` at **180s** on runner `srv3-06`, vs 28.6s / 3.7s in my container — which read as a 6–48× environmental gap until the tree variable was controlled. Same container, same binary, tree moved one day (`af57cd65bf` → `d077057d26`, **+2,260 lines / 36 files**, concentrated in the HostEffect/live_deploy neighborhood — `readiness.dag` +283, `service_ready.dag` +100, REST transport axes):

| module | session base | today's main (in-container) | growth | CI runner (srv3-06, same-era tree) |
|---|---|---|---|---|
| `gunbc.srv3_os_install_reconcile_receipt` | 3.7s | **142.1s** | **38×** | 180s |
| `test.claim.srv3_os_install_reconcile` | 1.28s | **46.5s** | **36×** | — |
| `gunbc.host_effect_realize` | 28.6s | **171.7s** | **6.0×** | 213s |
| `gunbc.ci_spec` (control — closure untouched) | 17.4s | 17.8s | 1.02× | — |

**Conclusions.** (a) The environment gap is gone entirely (§3.2 parity) — CI's 47-min gate ≈ today's tree's 40-min fold × nothing. (b) "CI getting longer and longer" is **R1 × R2 compounding live**: one day's merges made importer modules 6–38× slower, because the kernel's cost is superlinear in the visible tower. The blown-up modules' own files did not change. Whole-fold: **346s → 2,413s in one day of merges** (top-10 modules = 49%, top-25 = 78% of the new fold; new #1 `host_identity_assimilation` 214.5s). (c) **Bisect COMPLETE — the regression is `b6fe67b565` (#6750)**, "Shell→dag Slice 4 tail: route bmc_token_federation / ci_workflow RunSteps through `orch_emit_step(intent, Bash)`": clean at its parent #6757 (`srv3_os_install_reconcile_receipt` 4.1s, `host_effect_realize` 29.6s), blown at #6750 (**144.9s / 174.2s** — 35× / 5.9×), inherited at #6770/#6759-clean cross-checks (four legs total; concurrent legs cost each other ≤3%, §3.1). Mechanism: routing orchestration `Do{Run}` steps through the bash-emit surface pulls the bash grammar/emit tower into the type-visible closure of the host-effect module family — the kernel then pays its superlinear cost in every importer. The *semantics* of #6750 (killing hand-concat run-bodies) are sound; the cost is the kernel's shape × the widened visibility, which is why W2a targets the visibility/cost shape, not a revert of the de-fork. (d) Until W2 lands, every further tower merge repeats this — the fleet/shell lanes should see this table.

### 3.4 Claims register (corrections owed)

| prior claim | status |
|---|---|
| "CI's 55min = ~7× fleet contention" (this lane, early) | **RETRACTED** same-day |
| "floating single-threads run clock-starved at 1.0–1.5GHz; pin/governor fix buys ~2×" (this lane, this session) | **REFUTED by own control** (openssl pinned==floating, §3.1). Kept as the worked example of writing the control before the claim ships. |
| "srv3 per-core ~2× slower than srv2" (from lively-heron's >13min unfinished single-thread run vs 453s here) | **REFUTED** — §3.2: srv3 == srv1 == srv2 == container within 3% on identical work. The seeding measurements compared different trees (current-heavy vs session-light). |
| "8 concurrent runs ⇒ ~3–4× stretch (memory contention)" | **UNSUBSTANTIATED as stated**; the 213s-vs-28.7s module gap is currently better explained by tree growth (§3.3). A controlled 1×-vs-8× pinned concurrency experiment on srv1 remains open in W5 before any contention number is asserted. |
| "runner slots are memory-capped/cgroup-throttled" | not measured this session; PENDING (slot cgroup caps readable on-host, W5) |

## 4. Root causes (named, each with its fix)

| # | root cause | status | fix home |
|---|---|---|---|
| R1 | **Kernel cost-shape defect(s)** in expression inference (§2.2: `infer_expr` 81.5% of the fold; name-derivation/lookup storm receipted; mechanism-inside-`infer_expr` split = W2 step 0) | narrowed, splitting | root fix in `src/v1/04_infer.dag` (§6 bare-minimum-cost; NOT a cache) |
| R2 | **O(corpus) per run** — zero cross-run persistence; every PR re-typechecks ~1,090 modules, ~95%+ byte-identical to main's last green | design signed (A-lane) | W3 store |
| R3 | **Serial fold** — 1 core of 128 works; 127 idle | unowned until now | W4 shard |
| R4 | environmental per-box gap (if any survives §3.2) | under test | W5 receipts; no fix proposed until substantiated |
| R5 | strip-tree hang = missing fixpoint guard in `peel_alias_once_for_field_access` | fix built + unit witness; sign-off gate (04_infer load-bearing) | §5 of this doc |
| R6 | in-process double/triple-pass | **FIXED** #6783 (witnesses 45–63min → 5.7min, run 29543941811) | landed |

## 5. Workstreams

- **W1 — current-main baseline truth (immediate; measurement, not code).** The §3.2 matrix on `d077057d26` establishes what today's tree actually costs per environment, and §3.3 decides how much of "CI got slower" is module growth. Output: this doc's tables filled, plus a per-module growth table (session-base vs today) for the fleet-lane modules. If growth confirmed, the fleet lane gets a heads-up that its modules carry a per-merge CI tax until W2 lands (§6 displaced cost made visible, not a block).
- **W2 — kernel root fix (days).** Battery output (§2.2): **`infer_expr` = 81.5% of the fold** (all expression inference; coincides with item-grain `infer_items` 82%), `populate_output_provenance` 17% (the second walk), everything else ≤3%. Step 0 = re-run the split on today's tree and go one level *inside* `infer_expr` (record-literal checking, method-arg folds, and the receipted name-derivation storm — 16.9M `authored_name_at` / 936K `lookup_binding_by_name` calls). Fix lands in `04_infer.dag` authoring with regen, priced by before/after attribution on the same tree. Target: fold **≤60s** container-measured on today's main (top-25 modules to near-linear cost; `ci_spec`-class field checks from ~8ms → µs-class). Stretch: ≤20s if one mechanism dominates all top-25.
- **W3 — cross-run typed-module store (the law-changer; ~1–2 weeks).** Realize the operator-signed A-lane sketch on the #6783 seam: persist typed modules keyed by `std.interface_summary.typed_module_key` (source ⊕ direct-import interfaces ⊕ compiler identity); a PR run loads by key and recomputes only its changed cone. §5 shape: key-miss ⇒ compute (never skip); byte-identical cached-vs-cold purity oracle; main-push + 4-hourly falsifier stay cold controls (same pattern as regen skip / witness selection). Target: typical .dag PR pays **minutes, proportional to the diff**; corpus growth stops taxing PRs.
- **W4 — shard the cold pass (parallel to W3).** Process-level sharding of the module fold across cores (sidesteps the Rc→Arc/!Send interpreter constraint; scheduler already batches by closure). Cold whole-tree (main-push, falsifier, store-miss storms) → **≈ max-shard wall, ~≤5 min** on 128 cores even pre-W2. Overlap cost per process is bounded by the shared prefix (std/extdeps ≈ 14s of today's fold).
- **W5 — fleet hygiene receipts (background).** The remaining §3.4 PENDING rows: slot cgroup caps, the controlled 1×-vs-8× pinned concurrency experiment (substantiate or retire "contention" for good), and runner-count right-sizing per box (9 vs 51 slots).

Sequencing: W1 immediately (independent), W2 on battery results (independent), W3+W4 in parallel after (both consume the store seam; W4 must not fork the cache carrier — one `CacheProvider`, §3). R5 peel guard rides its own PR now.

## 6. End-state expectations (the concrete commitments)

Baselines: floor-proper today ~64 min post-#6783 (47 cold + 11.6 residue + 5.7 witnesses); 100–118 min on pre-#6783 bases; build job ~13 min; regen ~21 min on src/v1-touching PRs.

| milestone | typical .dag PR (CI total) | heavy PR (src/v1 touch) | main-push cold control | local whole-tree dev fold |
|---|---|---|---|---|
| today (post-#6783, today's tree) | ~75–85 min | ~100 min | ~85 min | **2,413s** (was 346s one day earlier — §3.3) |
| + W2a (repair #6750's cost shape: keep the de-fork semantics, narrow the bash-tower visibility reaching host-effect typecheck — owner: shell-sidecar lane with this doc's receipts) | back to ~pre-growth (~65 min) | ~85 min | ~70 min | ~350s |
| + W2 (kernel root fix) | **~35–45 min** | ~60 min | ~45 min | **≤60s** |
| + W3 (store) | **~15–18 min** (build job dominates) | ~35 min | unchanged (cold by design) | warm dev loop: seconds |
| + W4 (shard) | ~15 min | **~30 min** | **≤20 min** | cold ≤60s, warm seconds |

(No W1 row: W1 is measurement; it produced §3.2–3.3 and split W2a out of W2. Any further environmental win must appear as a numbered claim + receipt in §3 before it may appear here. W2a = the bisected merge's specific cost-shape gets repaired — or the modeling restructured — as an immediate regression fix, independent of the general kernel work; the general fix subsumes it but should not gate it.)

Anything that misses its row's number by >30% triggers a written root-cause note in this doc before the next workstream proceeds (no silent target drift). The build job (~13 min) and regen (~21 min) become the dominant terms at the end state; both have known follow-ups (prebuilt-bin cache; regen input-closure scoping already landed #6732) explicitly out of this plan's scope.

## 7. Controls that hold the whole time (§5)

- Every selection/skip stays fail-closed: refusal ⇒ compute, never widen, never silent-skip.
- Cold controls survive every workstream: main-push unconditional regen; 4-hourly affected-set falsifier; W3 adds the cached-vs-cold byte-identity oracle.
- Attribution probes stay session-local (never merged); every claim in this doc cites its run receipt; the §3.4 register is updated — not deleted — as rows resolve.
