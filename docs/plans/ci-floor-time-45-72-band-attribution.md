# CI floor time audit — redundant-work ledger + lever ranking

**Status:** measurement receipt, 2026-07-23 (session vivid-fox-471). **DESIGN.md + carriers remain
authority** — prose + TSV receipts only; **no floor behavior changes** in this PR. Dissolves when
`realization_measurement_loop` Phase-0 lands a durable `.dag`-native Gantt carrier.

**Product (operator mandate):** phase attribution is the **map**; the **product** is a per-stage
**redundant-work ledger** (what each stage recomputes that an earlier stage already computed on
the same input content) plus a **ranked lever table** priced in displaced minutes.

**Carriers (this PR):**

- [`docs/probes/ci_floor_phase_attribution_2026-07-23.tsv`](../probes/ci_floor_phase_attribution_2026-07-23.tsv) — per-run per-phase walls
- [`docs/probes/ci_floor_redundancy_ledger_skeleton_2026-07-23.tsv`](../probes/ci_floor_redundancy_ledger_skeleton_2026-07-23.tsv) — stage × recomputes × duplicate-of
- [`docs/probes/ci_floor_lever_ranking_2026-07-23.tsv`](../probes/ci_floor_lever_ranking_2026-07-23.tsv) — ranked levers

---

## 1. Band census (map only)

| Grain | median | 45–72 min band | notes |
|---|---:|---|---|
| Workflow (build+ci+deploy) | 55m | 56% of main runs | operator-facing "~1 hour" |
| **ci job** | **44m** | 40% | **use this for floor attribution** |
| Floor step (`gunbc ci` claim_executor) | ~35–48m | — | regen excluded (~3m) |
| **Extreme tail (action ceiling)** | **270m+** | 1 receipt | PR #7110 run `29986954853` — see §1.1 |

The **~72 min** figure in `gunbc_ci_witness_corpus_only_batches_note` is whole-tree **emit**
infeasible for pre-push — not typical ci job wall. Scoped PRs skip compile-clean emit entirely
(`compile-clean scope: skipped`).

### 1.1 Extreme tail — run `29986954853` (PR #7110, plan-only)

**Receipt:** PR #7110 @ `ad44c819c`, run `29986954853`, **killed at the ci-step
`timeout-minutes: 270` ceiling** (workflow wall ~281 min including setup). Diff is **one file**
(`dag/gunbc/v1_deletion_plan.dag` only) — the most trivial affected-set path; compile-clean
should be skipped.

**Reading:** NOT diff-size-driven. Strong evidence the tail is **corpus-denominated +
infra-bound** (serial `width=1` + memory thrash on a bad-luck runner). When width latches at 1
there is **no recovery arm** — the run rides silently to the action ceiling (DESIGN §5
absorbing-fallback shape: corpus-denominated cost breaks the budget later, not a typed refusal
mid-flight). Phase breakdown unavailable: log rotated on re-queue at cancel; floor never emitted
final batch receipts.

**Lever sharpen:** extends ranked lever 5 (width=1 latch) — add **fail-fast at action ceiling**
as a separate scheduling finding (270 min silent ride vs early typed refusal).

---

## 2. Receipt anchor — run `29976989996` (re-derived)

Branch `session/gentle-raven-495`, green, srv1-01, ci job **54.9 min**, floor step **~48.4 min**.
7-batch schedule (post-#7088 cheap-gate early batch). **Whole-tree compile-clean** because diff
had no shard intersection.

| phase | wall (min) | % of floor |
|---|---:|---:|
| preamble (plan resolve + hygiene) | 1.9 | 4% |
| compile-clean receipt (whole-tree emit) | 3.6 | 7% |
| batch 1 cheap gates (3 nodes, 1 resolve-group) | **10.1** | **21%** |
| batch 2 compile gate consume | 0.5 | 1% |
| batch 3 discovery (663 entry-groups, 2206 rows) | **12.8** | **26%** |
| batch 4 wet corpora | 1.3 | 3% |
| batch 5 emit_host | 0.1 | 0% |
| batch 6 source_root_ingest (ONE node) | **12.1** | **25%** |
| batch 7 reads_real_bytes | 3.3 | 7% |

**Top-3 = 35.0 of 48.4 min (72%):** discovery 12.8 + source_root_ingest 12.1 + cheap gates 10.1.

Governor receipt: `budget=16GiB` (cgroup memory.high), `max_width_reached=1`,
`measured worker share=3.36GB`, `peak_current=10.1GiB`, `cross_worker_store withheld`.
Declared cold resolves: **4** (matches `ci_floor_declared_resolve_count`).

---

## 3. Redundancy ledger (product)

Each row: what the stage computes, what earlier stage already computed on the **same content**,
and redundancy class per DESIGN §2 (duplicated / unnecessary / irrelevant).

| stage | recomputes | duplicate of | class | receipt |
|---|---|---|---|---|
| **compile-clean receipt** | whole-tree load + resolve + typecheck + emit | — (first whole-tree touch) | **necessary** | 3.6min; builds `process_shared_index` |
| **cheap gates (batch 1)** | re-resolve witness entry + scan imports/extdeps/drift | compile-clean receipt on **same** `witness_layer_roots` | **duplicated** | 10.1min **after** 3.6min compile; 3 gates parallel, same resolve-group |
| **compile gate consume** | reads receipt artifact | compile-clean receipt | **necessary** | 27s verify only |
| **discovery** | per-entry `extend_sources_to_both_closure_fixpoint` + eval | compile-clean typed cache **in principle**; **not** per-entry walk | **duplicated per-entry** | resolve serial **643s**; `reusing process_shared_index` but #6848 walk dominates |
| **source_root_ingest** | `discover_source_root_ingest` bin full tree scan | compile-clean + discovery on same roots | **duplicated** | **12.1min** one node; separate binary path |
| **reads_real_bytes** | heavy whole-tree resolve + filesystem read | prior heavy gates | **duplicated heavy resolve** | 3.3min; serial after ingest |
| **width=1 governor** | serializes all witness work | — | **irrelevant** (scheduling) | NOT proposing cap raise; index shrink / M2 lane |
| **materialization unkeyed** | 2.19M unkeyed pure calls | keyed memo path | **duplicated (identity unknown)** | unkeyed=47% of demand; ComputationIdentity lane |

**Key finding vs "4–5× whole-tree re-ingest" hypothesis:** declared **cold resolve count = 4**
per run — NOT four independent whole-tree cold graphs. The band is **not** four full re-ingests;
it is **one** whole-tree compile + **many per-entry walks** inside the shared index (discovery
643s serial resolve on 663 groups ≈ **970ms/group**), plus **two 12-min single-node gates** that
re-touch the tree through different code paths (ingest bin, cheap-gate scans).

Full skeleton: [`ci_floor_redundancy_ledger_skeleton_2026-07-23.tsv`](../probes/ci_floor_redundancy_ledger_skeleton_2026-07-23.tsv).

### 3.1 Batch-1 internal (cheap gates)

All three gates (`layering_imports`, `extdeps_external_authority`, `generated_artifact_drift`)
PASS at the **same timestamp** — one resolve-group, wall = **max** of parallel gate evals, not sum.
Dominant cost is the **shared resolve + host-effect scan** of the gate witness closure (~10min),
not one gate beating the others in serial. Per-gate split requires `GUNBC_FLOOR_GANTT=1` on a
replay (follow-up, not this audit PR).

### 3.2 Batch-6 / source_root_ingest — why 12 min for one node?

Evidence from run `29976989996` log: batch 6 invokes `discover_source_root_ingest` repeatedly
(shell `test -x` preamble then long-running ingest). This is a **separate release binary**, not
the compile-clean receipt path. It re-derives source-root ingest facts from the live tree —
work **not** consumed from the typed store the compile-clean receipt populated. Same pattern on
scoped main runs: batch 5 **10.9min** (`29970583893`) even when compile-clean is **skipped**.

---

## 4. Quadratic hunt (partial — historical arm)

Fit: discovery `resolve_serial_s` vs `entry_groups` (logged per run).

| run | class | entry_groups | resolve_serial_s | ms/group |
|---|---|---:|---:|---:|
| `29763408563` | PRE-6848 | ~500† | 99 | ~200 |
| `29819122813` | POST-6848 | ~500† | 403 | ~800 |
| `29976989996` | deep-diff | **663** | **644** | **971** |
| `29970583893` | trivial-diff | **663** | **620** | **935** |

†PRE runs lack `adaptive pool over N entry-groups` log line; groups estimated from witness count.

**Reading:** ms/group grew **~5×** PRE→POST (#6848 bare-reference fixpoint) while group count
grew ~30% (2087→2206 witnesses). The premium is **superlinear in per-group walk cost**, not
merely corpus size growth. **Local ptrace** on the two ~12min single-node gates is **not yet
run** (this audit PR is measurement-only); candidates: `rc_map_insert`, typecheck-env inductive
duplication, s1_closure re-walk (named in mandate).

---

## 5. Mandate questions — answers

| # | question | answer |
|---|---|---|
| 1 | What dominates each duration class? | **Trivial-diff (~48m ci):** discovery (~12m) + source_root_ingest (~11m) + effectful (~7m). **Deep-diff (+6m):** adds whole-tree compile-clean (+3.6m) + cheap gates (+10m when pre-compile ordering). No 127–159m green runs in last 500 workflow samples — operator class may be falsifier/cold-control or older fleet. |
| 2 | Why 12min for source_root_ingest? | Separate `discover_source_root_ingest` binary re-scans tree; does not consume compile-clean receipt. Batch-1 gates: parallel group, ~10min shared resolve — per-gate split needs GANTT replay. |
| 3 | How many whole-tree index rebuilds? | **1** explicit whole-tree compile emit + **4** declared cold resolves — but **663 per-entry walks** inside discovery on shared index. `fe_begin` RSS climbs 9.5→15.2 GiB across discovery despite index reuse. |
| 4 | Width=1 fleet-wide on 16GiB? | **Yes on measured runs:** `max_width_reached=1`, `cross_worker_store withheld`. Worker share ~3.4GB leaves headroom on paper but governor does not grow width (width_growths=0). Recovery = per-worker index shrink / M2, **not** cap raise. |
| 5 | #6848 / #6999 claims? | **Verified:** resolve_serial 99→644s (+545s) PRE→seed; #6999 **~0%** batch-wall recovery on comparable hosts (29855080611 vs 29819122813). Discovery loads each entry once per worker at width=1 — memo hits near zero on that path. |

---

## 6. Ranked levers

See [`ci_floor_lever_ranking_2026-07-23.tsv`](../probes/ci_floor_lever_ranking_2026-07-23.tsv). Top
three by displaced minutes:

1. **Per-entry bare-reference fixpoint** — 8–12 min (namespace §PR-5b)
2. **source_root_ingest re-walk** — 10–12 min (module-identity lane)
3. **Cheap-gate scan after whole-tree compile** — 5–10 min (#7088 ordering may shift; sleek-crane owns)

Config-grade follow-ups (named, not landed here): `GUNBC_FLOOR_GANTT=1` on fleet for per-gate
split; ptrace on ingest + discovery for quadratic stacks.

---

## 7. Reproduction

```bash
gh run view RUN_ID --log | rg 'claim_executor: batch|PASS \[batch|compile-clean scope|adaptive pool|discovery corpus:|\[governor\] receipt|floor materialization|floor resolve count'
```

---

## 8. Provenance

- vivid-fox-471, 2026-07-23, log-diff by execution on runs in TSV.
- Parent mandate: sharp-bee-290 msg_eae17a34 (redundancy ledger + quadratic hunt).
- Related: [floor-time-namespace-walk-regression-diagnosis.md](floor-time-namespace-walk-regression-diagnosis.md),
  [floor-shared-compute-memoization.md](floor-shared-compute-memoization.md),
  [v1-run-stability-throughline.md](v1-run-stability-throughline.md).

---

## 9. Correction appendix (2026-07-23, post-merge): the cold-child class this audit never named

Corroborated-and-corrected by the Pi/srv1 probe session (log re-derivation by execution on the
same runs, plus counterfactuals on srv1 with PR-head binaries; Pi exaggeration bench). The phase
walls in §2 and the governor story stand exactly. Two mechanism attributions and one lever were
wrong:

**The class: cold-index-per-process — the floor shells out to itself.** Every wet gate routes
through `run_gunbc_claims` (`dag/tools/host_prelude.dag`), whose fold spawns ONE cold
`gunbc run --claim-run` child per `ClaimRun` — serial, no short-circuit (the fold's
`acc && result.success` always evaluates the child), each rebuilding the module index and closure
resolve from scratch. ~26 children/run ≈ 23 of the 48 floor minutes (48%). `resolves_total` is
blind to them BY DESIGN (executor-only), which is why §2 could not see the class. Third and
largest instance of the one root: #7030 fixed cold-index-per-THREAD, double-resolve-rewire fixes
cold-resolve-per-ENTRY, this is cold-index-per-PROCESS.

**Corrected mechanism rows** (same log, re-derived):
- batch 1 (cheap gates, 10.1m): NOT parallel-gate max — 12 serial children Σ=480s (layering
  7/290s, extdeps 5/190s, drift 0) + ~2m executor resolve; the wall is the SUM.
- batch 6 (ingest, 12.1m): the `discover_source_root_ingest` bin costs 0.008s (two fixture
  reads; sub-second even on the Pi) — the wall was 12 children at 48–73s across 3 sub-gates.
  §2's "separate bin re-walks tree" is retracted as mechanism (the minutes were real; the story
  was not).
- batch 7 (3.3m): 2 children = 100% of the wall.

**Counterfactual receipts (srv1, PR-head binaries):** single cold child 47.2s (CPU-bound);
all 12 batch-1 claims pooled in ONE process 93.4s vs CI's 480s (warm marginal resolve ≈ 0ms per
claim); 3 ingest claims pooled 106.5s vs 190.5s. Per-gate pooling displaces ~13–17 min/run.
The child tax is identical on trivial diffs (verified vs run 29970583893). #7122's post-fix
residuals corroborate: cheap gates 4.65m vs pooled counterfactual ~1.6m, ingest 5.24m vs ~1.8m —
the gap IS the remaining child tax.

**Lever 1 (§4 rank 1) is STALE — do not dispatch against it as priced.** Both fixpoint
dissolution PRs (#7030, #7056) are ancestors of the measured run's commit, yet discovery still
costs ~971ms/group (5x pre-regression, and the ms/group denominator in §4 was off: the affected
set skips resolve for 1738/2206 rows). The 643s discovery serial needs re-diagnosis before any
further spend.

**Fix shape, with the memory coupling stated:** per-gate pooled child NOW (one child, N claims —
a change in the one `run_gunbc_claims` fn every wet gate inherits; separate process, dies and
frees). Executor-warm index sharing LATER only with the eviction lane: the executor is already
pinned at the 16GiB cap, and adding 26 claim closures to its retention is the crawl class of
2026-07-23 (run 29976854620: 16.3G + 34.4G swap, 37M high_events).

**Pi exaggeration receipts:** a single cold 2-entry resolve >25 minutes on the Pi (~2s warm on
srv) — the 5x-on-srv class becomes 30–60x, which is how the batch-6 misattribution fell apart on
contact. Remaining Pi probes (pooled variants, discovery n-scaling, whole-tree resolve) append
here as they land.

**Process lesson (workflow lane, ts-wf-node-schema):** rank-1 was priced without checking that
its fix had already merged — receipt freshness applies at the DIAGNOSIS grain, not only dispatch.

### 9.1 Pi-bench completion (2026-07-23, later): the pooling-footprint trade + four new rows

The full Pi-vs-srvN paired decomposition landed after the section-9 correction and both
qualifies it and extends it:

- **The pooled-child win has a memory cliff.** On the Pi, pooled 73m ≈ spawn-sum 66m —
  the win vanishes because the pooled process's UNIONED closure (~2.7GB) thrashes where
  12 small sequential children (~700MB each) do not. The same cliff exists inside 16GiB
  slot cgroups under fleet pressure: #7122's own post-rework acceptance corroborates
  (pooled cheap gates 3.08m and ingest 6.90m vs idle-srv1 counterfactuals of ~1.6/~1.8 —
  the gap is slot contention). §9's "pooled child now, low risk" is QUALIFIED: pooling
  pays fully only alongside per-worker index shrink; under a capped slot, size pools to
  the slot (K sub-pools) rather than one union.
- **Teardown tail (~2.5–3.1 min, twice-confirmed):** the floor process spends minutes
  freeing its own ~16GB retained store at exit (swap grows during Drop on the Pi;
  #7122's R4 row independently shows "+3.1m post-b7 executor teardown inside ci_job").
  Classic fix shape: end-of-floor fast-exit after receipts flush (no full Drop of a
  store the process is about to abandon) — priced ~2.5–3 min, near-zero risk.
- **Selection-control step: 4m51s with no audit row** — a real post-floor lever this
  ledger never carried (its 15m cap was treated as envelope; the measured cost was not).
- **Whole-tree baselines:** strict resolve 23m48s / 31.4GB on srv (~7x compile-clean's
  3.5m / 5.9GB); compile-clean is 86% reconcile. Infeasible outright on the Pi.
- **A standing red flag, not a lever (§5 class — two enforcement surfaces disagree):**
  bare `gunbc compile` on the CI-green tree exits with 2,652 unlisted-import errors —
  the floor receipt path tolerates hygiene the CLI enforces. Same class as the
  fleet_converge_emit standalone-closure failures and the import-strip Class-B
  pool-coincidence finding; needs an owner and a single hygiene authority, not a
  per-surface tolerance.
- **Bench heuristic worth keeping:** Pi/srv stretch ~8x = CPU-shaped; 25–78x =
  memory-shaped. This single signal killed the ingest-bin myth (sub-second on the Pi)
  and exposed the pooling/footprint trade.

### 9.2 Independent reproduction (2026-07-23, third leg) — count fix, the plumbing-PR cost profile, and the green-run pegging receipt

A from-scratch verification (code paths read, logs re-derived to the second, counterfactual
rebuilt on an uncontended VM: 12 cold children 243.2s vs one pooled process 53.9s, 4.5x —
same shape as srv1's 5.1x) confirms every §9 claim it could test, and adds:

- **Count fix:** the precise claim-child census is **25** (batch 1: layering 7 + extdeps 5
  + drift 0; batch 6: 3+3+6 across the ingest sub-gates; batch 7: 1); the 26th process is
  batch-7's `regen_stage0` sibling, not a claim child. §9's "~26" stands as written but
  the split is now exact. Also exact: the serial-children walls sum to 479.6s, and the
  no-short-circuit property is a LANGUAGE fact — the v1 interpreter evaluates both
  operands of every binary op (v1_interpreter.rs ~1918), so `acc && child` can never
  skip; every claim always runs. Fail-closed by construction, priced accordingly.
- **The plumbing-PR cost profile (reads on #7122's 52-min wet spike):** that spike was
  the affected set working CORRECTLY on a PR touching host_prelude/cli_run — plumbing in
  nearly every bin-witness closure, so 46 of 55 heavyweight self-host rows ran instead
  of 8. Not a pooling regression, and not fully the enrolled-witness story either: ANY
  plumbing PR legitimately pays the whole self-host roster. Consequence for reading
  receipts: on plumbing PRs, only per-batch residuals are the honest post-fix metric —
  headline walls are affected-set artifacts. Consequence for the ledger: plumbing PRs
  have a structurally different cost profile; a future budget wall must denominate
  per-batch, not per-run, or every touch of cli_run reds spuriously.
- **The green-run pegging receipt (strengthens §9.1's cliff):** even the green anchor
  run pegs its 16GiB cgroup from MID-DISCOVERY onward (swap 3.2GB by discovery end,
  9.4GB by regen, 4,785 throttle events on a green run). Children spawn inside the
  already-pegged cgroup — plausibly why CI children run 35–73s where an uncontended VM
  takes 19–26s. The crawl was always one straw away, on every green run.
- **The remaining gap, decomposed:** #7122 pools per-CALL, not per-gate — batch 1 still
  spawns 6 pooled children (Σ=250s) and ingest 4. Batch-1's calls share identical
  source roots, so per-gate/per-batch pooling is feasible there (the one-process run of
  all 12 claims is the existence proof); the ingest gate's mktemp overlay roots are the
  GENUINE constraint keeping its 4 calls separate — named, not hand-waved. The durable
  fix past that remains the cross-process content-keyed store (W3), already the
  declared dissolve-on.

### 9.3 Endgame landing + post-merge prediction record (2026-07-23)

The endgame PR lands every priced lever from §9–§9.2 plus the cost wall
([resolve-regression journey](resolve-regression-journey.md) §5 item 1). What landed, each with its receipt anchor:

- **Per-batch pooling (D1):** batch-0's claim-backed gates now share ONE pooled
  `claim_batch` child (`tools.cheap_gate_pool` — union list derived from the two concern
  authorities, K sub-pools sized to the slot per §9.1's cliff, `CheapClaimPoolGate` runs
  it; the layering/extdeps gates became pure enrollment walls). Ingest's 4 mktemp overlay
  children stay separate — the genuine constraint, stated in-row
  (`ingest_pool_separation_note`).
- **Teardown fast-exit (D2):** the floor process exits via `floor_terminal_fast_exit`
  after receipts flush — the 2.5–3.1 min Drop walk of the retained store (§9.1,
  twice-confirmed) is gone from the ci job tail. Truncated/unwritable receipts still red
  (fold into `any_failed` before the exit; RED control
  `unwritable_receipt_base_reds_not_vanishes`).
- **Selection-control (D3):** the step's 4m51s (vs 80s local) decomposed to THREE cold
  whole-pool index builds inside `floor_skip_discovery_witness` (two bare
  `build_multi_entry_index` warmup cases + the first corpus case's shared build); the
  warmup case now rides `resolve_entry_graph_shared`, so the suite pays ONE shared build
  plus the deliberately-cold control build. The step stays per-PR (the widen-detector).
  Ledger row added.
- **Discovery re-diagnosis instrumentation (D4):** the pump's phases land as typed rows
  in the floor resolve receipt (`discovery_pump_wall_ms`, `discovery_roster_walk_ms`,
  `discovery_diff_observe_ms`, `discovery_frontier_attribution_ms`,
  `discovery_shared_index_build_ms`, `discovery_preresolve_calibration_ms`,
  `discovery_runner_resolve_ms`, plus the existing resolve/eval serial sums). Lever-1
  stays RETIRED as priced; the fresh per-phase profile from this PR's own runs names the
  next mechanism, and if it names the identity authority or the retention lane, the
  receipt + frontier row is the deliverable — no downhill patching.
- **The cost wall (D5):** per-batch wall budgets as data
  (`gunbc.ci_spec.gunbc_ci_floor_batch_wall_budget_seconds`, denominated per-batch per
  §9.2's plumbing-PR profile), enforced by `claim_executor` as typed located refusals
  (`FLOOR-BATCH-OVER-BUDGET`), recorded as typed receipt rows
  (`target/floor-batch-wall-receipt.txt`), RED-controllable both directions via the
  tighten-only injection. Ruling reconciliation in the carrier note.
- **Wet-batch discipline (D6):** `bin_witness_wet_per_row_wall_budget_seconds` (60s);
  the two rows over it re-homed to the falsifier wet cadence as typed frontier rows
  (`falsifier_rehomed_bin_wet_rows`: floor_skip keystone 289s — also a per-PR duplicate
  of the selection-control step — and cross_shard_seam live-tree 183s), 52 rows remain
  as the per-PR smoke subset.

**Prediction (stated before merge, recorded after):** first post-merge ordinary-diff main
floor ≤ ~30 min. Basis: run 30009199696's floor step 49.5 min minus teardown ~3.1 min,
batch-0 pooling ~1 min, wet re-homes ~7.9 min (plumbing profile; ordinary diffs saw
~1.3 min wet so their recovery is smaller but their baseline was already lower), and the
ordinary-diff wet/discovery profile per §2/§9.2.

**Post-merge actual (append here — a miss is a receipt, not a silent pass):**
- [ ] first ordinary-diff main floor run id + floor-step wall: _pending merge_

Floor cap 55 → 40 is proposed as a data change AFTER the post-merge receipts exist
(operator signs; deliberately not in the endgame PR).

**Process lesson (workflow lane, endgame rework push):** every endgame piece was proven
by execution before the PR — executor battery, budget RED both directions, pooled child,
plan witnesses, whole-tree compile — EXCEPT one full floor walk, and that is exactly the
lane that redded (twice: the merge-tree ROADMAP drift, then #6866's admission invariant
on the two re-homed D6 rows). Receipt freshness applies at the INTEGRATION grain too: the
proof set must include one run of the composed thing, not only its parts.

**On-call pre-positioning (operator, 2026-07-23):** batches 3 and 6 sit at 1.29×/1.45×
headroom against their budget rows — inside the 1.3–1.5× slot-noise band — so expect the
first ORGANIC `FLOOR-BATCH-OVER-BUDGET` on an innocent PR within days. That is the wall
working, not a defect: D4's phase rows in the floor resolve receipt make the batch
diagnosable, and the remedy is **diagnose-or-signed-raise** (the operator-signed raise
discipline in `gunbc_ci_floor_batch_wall_budget_note`), never a rerun and never
pre-widening the budgets to avoid the first firing.
