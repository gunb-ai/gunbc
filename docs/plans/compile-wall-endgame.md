# Compile-wall endgame — measured decomposition + the plan to end it

> **Status:** MEASUREMENT + PLAN FOR REVIEW, 2026-07-17, session clever-seal-476 ("investigate typecheck"). Operator ask: "take measurements, then put together a plan end to end to solve this once and for all, with concrete expectations on where we end up" and "be particular about hardware — there are a lot of unsubstantiated claims about fleet contention." Every number below is a receipt from an executed run (command + location cited) or is explicitly marked PENDING; every prior unsubstantiated claim this lane made is listed in §3.4 with its correction. Review the claims against the receipts — that is this PR's purpose.
>
> Parent lanes: [v1-run-stability-throughline](v1-run-stability-throughline.md) (memory axis; this doc is the compute axis), [cross-entry-typed-module-memo-sketch](cross-entry-typed-module-memo-sketch.md) (the A-lane store this plan's W3 realizes), [cross-worker-typecheck-share-design](cross-worker-typecheck-share-design.md), [duplicate-work-graph-lens-design](duplicate-work-graph-lens-design.md) (`ComputationIdentity` — the §2 authority all memo tiers dissolve into), [execution-spine-design](execution-spine-design.md) §2, [floor-shared-compute-memoization](floor-shared-compute-memoization.md) (M1/M2 — realized in part by #6783). Cross-axis complement: the landed `CostAccount.space` `basis: Derived` work (peak_space as the dual of `DescentEvidence`, active in the compiled compiler) is the *space*-axis half of "know the cost before running" — W2 (make the kernel cheap) and that lens (derive the cost statically) are two readings of one move; neither lane re-derives the other.
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
| **`infer_expr` (all expression inference, depth-0)** | **81.5% (283.5s of 347.7s)** | **the target.** Coincides with the item-grain `infer_items` ≈ 82% — expression inference IS the fold. Call-storm receipts inside it: **16.9M `authored_name_at` calls** (5.7M in `host_effect_realize` alone; name re-derivation via source-span lookup) and 936K `lookup_binding_by_name`. The probe-grain split *inside* this share landed via review and is verified in-source — §2.4; W2 step 0 re-confirms it on the then-current tree. |

(An earlier draft circulated `infer_expr` ≈ 41% — that figure divided by a double-counted denominator, the same awk artifact already registered in §3.4, and is corrected here; 81.5% is the receipt-consistent share. All battery rows are session-base measurements; §3.3's `d077` rows confirm the *concentration* shape carries to today's tree (top-10 = 49%, top-25 = 78%), and the full mechanism split re-runs on today's tree as W2 step 0.)

### 2.3 Concentration (why "the algorithm is inefficient" is fixable)

(Session base; re-established on today's `d077057d26`: top-10 = 49%, top-25 = 78% — §3.3.)

- median module = **4.4ms**; >1,000 modules are single-digit ms — the algorithm is fine on normal input.
- **top-25 modules = 77%** of the fold; all are CiSpec/HostEffect-tower importers.
- cleanest specimen: `gunbc.ci_spec` — 17.4s at **1.2% resolve**; ~2k record-field instances ⇒ **~8ms per field check** ≈ tens of millions of instructions per field. That is a cost-shape defect (§6), not inherent cost.
- `tools.ci_gates`: provenance walk 9.0s vs inference 4.5s — the *second* walk costs 2× the first on some modules.
- precedent that these fall to root fixes: `merge_envs` (reconcile 81%→6%, ~2× self-compile), M1a (env term → 0.1%), #6773 (variant-locals M×K).

### 2.4 The kernel defect at probe grain (post-#6750) — adopted from review, verified in-source

An operator-relayed probe-grain review located the mechanism inside §2.2's phase shares; its structural claim verifies directly against the source, and it supersedes the phase-grain framing:

- `lookup_resolved_sig` (`src/v1/04_sigs.dag:59`) misses on `env.local` and calls `lookup_in_parent_chain` (`:44–57`), which re-enters `lookup_resolved_sig` for **every** parent env with **no visited-state and no memo**. `ResolvedFuncEnv.parents` forms a DAG (shared ancestors via diamond imports), so a shared ancestor is re-traversed once per *path*, not per node — and a **negative** lookup (the common case: a name bound elsewhere) traverses the entire reachable DAG-as-tree before answering `Absent`. In-source contrast: `func_reaches_self` (same file, `:91`) carries an explicit `visited` map — the pattern exists in-file; this walk lacks it.
- Secondary, same family: `parents |> take(n: count − 1)` (`:52`) copies the list prefix on each recursion step — quadratic per lookup chain (§6 bare-minimum-cost class).
- Review's counts (profile-independent — their whole-tree timing run used a debug build, so wall times are not comparable, but counts are): identical **541 signature requests** expand **53.3M → 902.8M env probes (16.95×)** across #6750, while `authored_name_at` calls slightly *decrease* — the regression rides the sig-env walk, **not** name derivation. #6750 at this grain: the `host_effect_realize → … → bmc_token_federation → bash_orchestration_emit` closure grew by **54 modules**, multiplying parent-DAG path counts.
- Post-#6750 phase shares on slow modules (review measurement): **60.5% inference / 39.4% provenance** — §2.2's session-base single-PHASE dominance does **not** carry to the post-regression tree. The hypothesis that reconciles all measurements: both phases ride the same un-deduplicated walk (`output_provenance`/`variant_provenance` are fields *on* `ResolvedFuncSig`, `:15–22`), so **one mechanism surfaces as two phase shares**. W2 step 0 must confirm exactly this before §6's fold target is load-bearing (§6 note).

W2's fix protocol therefore prices in **probe counts, not just wall**: identical-input probe count is the oracle (902.8M-class → ~53.3M-class), with a planted diamond-import RED fixture asserting the per-path→per-node collapse.

**Resolved by execution (W2 landed).** The flat-closure fix (ResolvedFuncEnv held as a name-deduped, precedence-ordered flat closure; the recursive walk and its quadratic `take` deleted) took the whole-tree fold **2,461.7s → 65.4s (37.6×)** on the same box/tree-class, pinned, with `HISTOGRAM_TOTAL_HARD 0` on **both** runs — corpus-identical health, only the wall changed. That one local fix recovering ~97% of the fold confirms the single-mechanism hypothesis: the 60.5/39.4 phase split was two views of the one walk. The depth-64 diamond witness (2⁶⁴ paths pre-fix) pins the per-path→per-node collapse in-tree.

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

Receipt durability: the in-table log paths are live sources on the hosts; the load-bearing terminal lines (verbatim `[gantt]` rows for all four environments, protocol, openssl control, concurrency datapoint) are archived durably in [PR comment 5003478068](https://github.com/gunb-ai/gunbc/pull/6793#issuecomment-5003478068), fetched from the hosts 2026-07-17. What remains session-local by policy (§7): the per-module probe CSVs on the `[NEVER MERGE]` instrumentation branch.

### 3.3 The tree-growth finding (CONFIRMED in-container)

The 118-min run's CI log showed `host_effect_realize` at **213s** and `srv3_os_install_reconcile_receipt` at **180s** on runner `srv3-06`, vs 28.6s / 3.7s in my container — which read as a 6–48× environmental gap until the tree variable was controlled. Same container, same binary, tree moved one day (`af57cd65bf` → `d077057d26`, **+2,260 lines / 36 files**, concentrated in the HostEffect/live_deploy neighborhood — `readiness.dag` +283, `service_ready.dag` +100, REST transport axes):

| module | session base | today's main (in-container) | growth | CI runner (srv3-06, same-era tree) |
|---|---|---|---|---|
| `gunbc.srv3_os_install_reconcile_receipt` | 3.7s | **142.1s** | **38×** | 180s |
| `test.claim.srv3_os_install_reconcile` | 1.28s | **46.5s** | **36×** | — |
| `gunbc.host_effect_realize` | 28.6s | **171.7s** | **6.0×** | 213s |
| `gunbc.ci_spec` (control — closure untouched) | 17.4s | 17.8s | 1.02× | — |

**Conclusions.** (a) The environment gap is gone entirely (§3.2 parity) — CI's 47-min gate ≈ today's tree's 40-min fold × nothing. (b) "CI getting longer and longer" is **R1 × R2 compounding live**: one day's merges made importer modules 6–38× slower, because the kernel's cost is superlinear in the visible tower. The blown-up modules' own files did not change. Whole-fold: **346s → 2,413s in one day of merges** (top-10 modules = 49%, top-25 = 78% of the new fold; new #1 `host_identity_assimilation` 214.5s). (c) **Bisect COMPLETE — the regression is `b6fe67b565` (#6750)**, "Shell→dag Slice 4 tail: route bmc_token_federation / ci_workflow RunSteps through `orch_emit_step(intent, Bash)`": clean at its parent #6757 (`srv3_os_install_reconcile_receipt` 4.1s, `host_effect_realize` 29.6s), blown at #6750 (**144.9s / 174.2s** — 35× / 5.9×), inherited at #6770/#6759-clean cross-checks (four legs total; concurrent legs cost each other ≤3%, §3.1). Mechanism: routing orchestration `Do{Run}` steps through the bash-emit surface pulls the bash grammar/emit tower into the type-visible closure of the host-effect module family — the kernel then pays its superlinear cost in every importer. The *semantics* of #6750 (killing hand-concat run-bodies) are sound; the cost is the kernel's shape × the widened visibility, which is why W2a targets the visibility/cost shape, not a revert of the de-fork. (d) Until W2 lands, every further tower merge repeats this — the fleet/shell lanes should see this table. Independent corroboration, structural and by-execution: `host_effect_realize.dag` imports `shell_exec_via_bash` + the shell tower (lively-heron import-graph check — independent of any timing); the #6750 closure grew by 54 modules (probe-grain review, §2.4); targeted compiles at the #6757/#6750 pair reproduce 29.620s→219.974s (7.43×) in a separate session and build, with `ci_spec` 22.897s→23.479s as the unchanged-closure control; adjacent Actions runs 29540924059 / 29541485814 bracket the same boundary publicly.

### 3.4 Claims register (corrections owed)

| prior claim | status |
|---|---|
| "CI's 55min = ~7× fleet contention" (this lane, early) | **RETRACTED** same-day |
| "floating single-threads run clock-starved at 1.0–1.5GHz; pin/governor fix buys ~2×" (this lane, this session) | **REFUTED by own control** (openssl pinned==floating, §3.1). Kept as the worked example of writing the control before the claim ships. |
| "srv3 per-core ~2× slower than srv2" (from lively-heron's >13min unfinished single-thread run vs 453s here) | **REFUTED** — §3.2: srv3 == srv1 == srv2 == container within 3% on identical work. The seeding measurements compared different trees (current-heavy vs session-light). |
| "8 concurrent runs ⇒ ~3–4× stretch (memory contention)" | **UNSUBSTANTIATED as stated**; the 213s-vs-28.7s module gap is currently better explained by tree growth (§3.3). A controlled 1×-vs-8× pinned concurrency experiment on srv1 remains open in W5 before any contention number is asserted. |
| "runner slots are memory-capped/cgroup-throttled" | not measured this session; PENDING (slot cgroup caps readable on-host, W5) |
| "one mechanism dominates all top-25 (~80%+), so fold ≤60s after W2" (this doc, first draft) | **RESOLVED — mechanism-grain claim CONFIRMED by execution** (W2 landed): the single flat-closure fix took the fold 2,461.7s → 65.4s (37.6×), so one mechanism did dominate — at the walk grain, not the phase grain (the 60.5/39.4 split was two views of one walk, §2.4). Fold target measured 65.4s vs the ≤60s row (9% over — inside the >30% miss rule; no root-cause note owed). |

## 4. Root causes (named, each with its fix)

| # | root cause | status | fix home |
|---|---|---|---|
| R1 | **Kernel cost-shape defect: the un-visited sigs-env DAG walk** — `lookup_resolved_sig`/`lookup_in_parent_chain` traverse shared parent envs per-path with no visited-state, + quadratic `take` prefix copies (§2.4; phase-grain views of the same defect: `infer_expr` 81.5% session-base, 60.5/39.4 inference/provenance post-#6750) | mechanism named + verified in-source; step-0 confirm pending | root fix in `src/v1/04_sigs.dag` authoring (+ any `04_infer.dag` residue); §6 bare-minimum-cost; NOT a cache |
| R2 | **O(corpus) per run** — zero cross-run persistence; every PR re-typechecks ~1,090 modules, ~95%+ byte-identical to main's last green | design signed (A-lane) | W3 store |
| R3 | **Serial fold** — 1 core of 128 works; 127 idle | shard-A carrier landed (partition + compose proof); process-grain half open | W4 composes with `dag_compile_clean_shard_*` (§5) |
| R4 | environmental per-box gap (if any survives §3.2) | under test | W5 receipts; no fix proposed until substantiated |
| R5 | strip-tree hang = missing fixpoint guard in `peel_alias_once_for_field_access` | fix built + unit witness; sign-off gate (04_infer load-bearing) | §5 of this doc |
| R6 | in-process double/triple-pass | **FIXED** #6783 (witnesses 45–63min → 5.7min, run 29543941811) | landed |

## 5. Workstreams

- **W1 — current-main baseline truth (immediate; measurement, not code).** The §3.2 matrix on `d077057d26` establishes what today's tree actually costs per environment, and §3.3 decides how much of "CI got slower" is module growth. Output: this doc's tables filled, plus a per-module growth table (session-base vs today) for the fleet-lane modules. If growth confirmed, the fleet lane gets a heads-up that its modules carry a per-merge CI tax until W2 lands (§6 displaced cost made visible, not a block).
- **W2 — kernel root fix: LANDED.** The un-visited sigs-env DAG walk (§2.4) fixed by flat-closure-at-construction: `ResolvedFuncEnv` gains a `name` identity, `parents` holds the transitive closure flat (precedence-ordered, name-deduped — the dedup is load-bearing, `sigs_env_flat_parents_note`), the recursive walk and its quadratic `take` are deleted, and lookup is one ordered scan. Shadowing linearization proven witness-pinned (last-import-wins, closure-of-last beats earlier-direct, own-local beats all); depth-64 diamond RED pins the per-path→per-node collapse; regen byte-fixed-point. **Measured: fold 2,461.7s → 65.4s (37.6×), `HISTOGRAM_TOTAL_HARD 0` both sides.** The `ci_spec`-class field-check shape goal is absorbed (the walk was the shape defect).
- **W2a — re-priced by W2's result: likely moot.** The #6750 growth term was the kernel walk paying per-path on a 54-module-wider closure; with the walk dead, visibility widening costs per-module, not per-path. Re-check the #6750 shape only if residual importer cost shows post-merge; do not spend the shell lane on it preemptively.
- **W3 — cross-run typed-module store (the law-changer; ~1–2 weeks).** Realize the operator-signed A-lane sketch on the #6783 seam: persist typed modules keyed by `std.interface_summary.typed_module_key` (source ⊕ direct-import interfaces ⊕ compiler identity); a PR run loads by key and recomputes only its changed cone. Lineage honesty: the parent designs mark this **future S2b work** — this plan is its realization *schedule*, not a claim it exists today. §5 shape: key-miss ⇒ compute (never skip); byte-identical cached-vs-cold purity oracle; main-push + 4-hourly falsifier stay cold controls (same pattern as regen skip / witness selection). Target: typical .dag PR pays **minutes, proportional to the diff**; corpus growth stops taxing PRs.
- **W4 — shard the cold pass (parallel to W3).** Process-level sharding of the module fold across cores (sidesteps the Rc→Arc/!Send interpreter constraint; scheduler already batches by closure). **Prior art to compose with, not fork (§3): the landed shard-A carrier** — `dag/tools/dag_compile_clean_shard_*` with its partition boundary, one-shard run, and compose proof (`shard-green ∧ … ≡ whole-tree green`, planted-bad-module RED); shard B (floor-plan enrollment) is the open half. The "~≤5 min on 128 cores" figure is an **extrapolation pending a resource receipt** — W4 step 0 measures max-shard wall on a real partition via that carrier before the number is committed. Overlap cost per process is bounded by the shared prefix (std/extdeps ≈ 14s of today's fold).
- **W5 — fleet hygiene receipts (background).** The remaining §3.4 PENDING rows: slot cgroup caps, the controlled 1×-vs-8× pinned concurrency experiment (substantiate or retire "contention" for good), and runner-count right-sizing per box (9 vs 51 slots). Plus one witness-hygiene landmine on this plan's own critical path: `dag/test/claim/deploy_mutation_gate_witness_test.dag` blocks a hermetic whole-corpus run at cpu 0.0% on a wet live_deploy effect instead of refusing — hang-not-refusal, the §4-boundedness twin of the §5 silent widen; unlike its sibling deploy witnesses it is not excluded (reported by lively-heron; re-verify exclusion state at fix time). Anyone re-running whole-corpus attribution (W2 step 0) hits it. Interim: explicit exclusion; fix: typed refusal; the exclusion dies with the fix (named dissolution trigger).

Sequencing: W1 done (this doc). **W2 step 0 + the walk fix first** — it is small, local to one function family, and probe-priced; W2a is re-priced after it lands (the kernel fix may restore ~pre-growth walls on its own, demoting W2a to modeling hygiene). W3+W4 in parallel after (both consume the store seam; W4 must not fork the cache carrier — one `CacheProvider`, §3). R5 peel guard rides its own PR now.

## 6. End-state expectations (the concrete commitments)

Baselines: floor-proper today ~64 min post-#6783 (47 cold + 11.6 residue + 5.7 witnesses); 100–118 min on pre-#6783 bases; build job ~13 min; regen ~21 min on src/v1-touching PRs.

| milestone | typical .dag PR (CI total) | heavy PR (src/v1 touch) | main-push cold control | local whole-tree dev fold |
|---|---|---|---|---|
| today (post-#6783, today's tree) | ~75–85 min | ~100 min | ~85 min | **2,413s** (was 346s one day earlier — §3.3) |
| + W2a (repair #6750's cost shape: keep the de-fork semantics, narrow the bash-tower visibility reaching host-effect typecheck — owner: shell-sidecar lane with this doc's receipts; **re-priced after W2's walk fix lands**, may demote to modeling hygiene) | back to ~pre-growth (~65 min) | ~85 min | ~70 min | ~350s |
| + W2 (kernel root fix — the §2.4 walk; **LANDED**) | **~35–45 min** | ~60 min | ~45 min | **65.4s measured** (was 2,461.7s same-tree — 37.6×) |
| + W3 (store) | **~15–18 min** (build job dominates) | ~35 min | unchanged (cold by design) | warm dev loop: seconds |
| + W4 (shard) | ~15 min | **~30 min** | **≤20 min** | cold ≤60s, warm seconds |

(No W1 row: W1 is measurement; it produced §3.2–3.3 and split W2a out of W2. Any further environmental win must appear as a numbered claim + receipt in §3 before it may appear here. W2a = the bisected merge's specific cost-shape gets repaired — or the modeling restructured — as an immediate regression fix, independent of the general kernel work; the general fix subsumes it but should not gate it.)

Anything that misses its row's number by >30% triggers a written root-cause note in this doc before the next workstream proceeds (no silent target drift). **The W2 fold target is RESOLVED by execution**: measured 65.4s against the ≤60s row (9% over — inside the miss rule), via the single flat-closure fix (§2.4), which also confirmed single-MECHANISM dominance (2,461.7s → 65.4s from one local change). The CI-column projections for the W2 row are measured next at this PR's own CI run (the compile-clean gate rides the same kernel). The build job (~13 min) and regen (~21 min) become the dominant terms at the end state; both have known follow-ups (prebuilt-bin cache; regen input-closure scoping already landed #6732) explicitly out of this plan's scope.

## 7. Controls that hold the whole time (§5)

- Every selection/skip failure arm **refuses with a typed, located, counted diagnostic**. For a *gate*, the refusal's remedy is the loud whole-tree baseline (running is the safe side) — but the refusal state is never absorbed into the answer (⊤-as-ignorance ≠ ⊤-as-answer, DESIGN §5), and never silent-skip. "Refusal ⇒ compute" as shorthand means exactly this counted form, never an absorbing rerun.
- Cold controls survive every workstream: main-push unconditional regen; 4-hourly affected-set falsifier; W3 adds the cached-vs-cold byte-identity oracle.
- Attribution probes stay session-local (never merged); every claim in this doc cites its run receipt; the §3.4 register is updated — not deleted — as rows resolve.
