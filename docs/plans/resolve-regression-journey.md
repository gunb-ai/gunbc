# The resolve-regression journey — why the fixes keep not holding

Operator question (2026-07-23): *"what fundamentally keeps regressing here? we have fixed
resolve several times over the past weeks, and it always seems to come back worse."*

This document is the answer, assembled by a five-agent sweep over origin/main's history
(65 dated fix events since 2026-06-25), the in-tree receipt docs, and an adversarial
verification pass. Every claim carries its receipt. The one-line answer up front:

> **No single fix has failed. The floor pays interest on one unbuilt model —
> computation identity ("this work was already done") — across a corpus whose demand
> keeps being re-grown by correctness lanes that face no cost wall at merge time, on
> executors whose memory retention converts residual CPU wins to zero.**

Every fix worked at its grain. What regresses is the system around them.

---

## 1. The measured trajectory (all receipts in-tree)

| era | floor / job wall | what happened | receipts |
| --- | --- | --- | --- |
| pre-2026-07-09 | floor ~8m (job ~18m) | opt-in witness roster (8 entries / 67 fns) | `gunbc_ci_floor_step_timeout_discovery_flip_note` regression ledger; TSV PRE-6848 row (17.9m job, ~9m witness batches) |
| 2026-07-09/10 | step cap climbs 30→60 | **the discovery flip** (#6438): tree-wide enrollment; eval demand ×12 (keyed 2.31M→27.9M); cost denominated in the corpus, not the diff | ci_workflow.dag:406; ci-performance-tanking-evidence.md |
| 2026-07-10–12 | cap bounces 60→90→120→180→270 | runs time out mid-batch with **0 witness FAILs**; the 60m fail-fast attempt regresses merge CI and reverts; the cap is raised to *fit* the cost each time | timeout-note bounce history (runs 29141663541 … 29197126623) |
| 2026-07-20 | job 22.2→46.6m; RSS 6.5→20GB | **#6848** (namespace wave 1, 968 files): buys real correctness (bare-name census closure — import-stripped files have no import edges to follow) at an unpriced cost: +5m discovery, +12.8m effectful gates, and a ~1GB/process whole-pool full-body parse baseline that pins 16GB runners at memory.high | floor-time-namespace-walk-regression-diagnosis.md §1; floor-memory-pool-parse-regression-diagnosis.md |
| 2026-07-21–22 | 40–50m, cap-pinned | the follow-up fixes land, each mechanism-correct, each recovering less than priced: #6998 (call-order + advisory flood), #6999 M1 memo (**~0% recovery** — caches lived per-worker), #7030 (cold-index-per-THREAD: the memo was `thread_local!` and batch-4 gates spawned cold threads), #6956/#6972 (heads-only parse, partial memory fix), #7056 (`BothClosureEdgeIndex` once-per-pool: cold-per-ENTRY dissolved) | DESIGN.md M1 thread; diagnosis doc §5–6 |
| 2026-07-23 | 4h crawl → cap 55m | retention finally exceeds the slot cap: 16.3G + 34.4G swap, 37M high_events, PSI 34, `oom_kill=0` — the memory.high throttle-crawl; 9 main runs wedge; #7120 sets the honest ceiling | run 29976854620 log; timeout note |
| 2026-07-23 | 13m recovered; 23m named | #7122 (one-tree-one-resolve) collects cheap-gates/consume/ingest; the §9 correction discovers **cold-index-per-PROCESS** — 26 `gunbc run --claim-run` children ≈ 23 of 48 floor minutes, present all along, invisible to `resolves_total` by design | attribution doc §2/§9; lever TSV |

Three different mechanisms wear one trend line: 8→18 was *added demand* (flip + #6848),
18→45 was *retention accreting to the cap*, 45→4h was *the cap lost*. "Resolve keeps
regressing" is the visible sum of the three.

## 2. The five grains of one duplication (inventory, verified)

The same computation — build the module index, resolve the closure — is re-derived at
five independent grains. Each was discovered **serially** and fixed **independently**;
no authority owns "already computed" across them:

| grain | state | fix |
| --- | --- | --- |
| per-THREAD | fixed 2026-07-21 | #7030 (the memo was `thread_local!`; gates spawned cold threads) |
| per-ENTRY | fixed 2026-07-22 | #7056 (`BothClosureEdgeIndex` once per pool) |
| per-RUN (within-walk) | fixed 2026-06-30 | M1 #6008/#6999 (recovered ~0% while the other grains dominated) |
| per-PROCESS (claim children) | **open** — the largest | attribution §9; pooled-child counterfactual 93.4s vs 480s |
| per-PR (cold start) | open by design | the durable-store lanes (W3 typed-module store, resolve cache — reverted to opt-in 2026-06-26 after the always-on default OOMed the floor: the memory coupling, already, in June) |

This is the §5 lesson applied to performance: **patching found instances is validation;
the fix that ends the class is construction** — a shared materialization authority all
five grains consume. That authority (ComputationIdentity, duplicate-work lane) is
designed and unbuilt; the materialization receipt that would feed it is explicitly
disclosure-only, and 2.19M calls/run (47% of demand) are unkeyed — invisible to any
memo by construction.

## 3. Why recovered minutes don't stay recovered (verified H1/H4)

There is **no floor-time regression gate**: the commit-gate roster has nine gates, none
time-related, and an explicit ruling forbids wall-clock terms in verdicts. The enforced
5-second eval law is real but scoped to witness EVAL — the regression mass lives in the
explicitly exempted infra carve-out (resolve, index builds, child processes). Since
2026-07-01 at least five merged changes added corpus-denominated work (tree-wide
discovery, always-on receipts, whole-corpus scans) and **none priced its floor cost at
merge time** — each cost surfaced post-hoc as an incident receipt, and the step
timeout's own bounce history (30→…→270) is the record of the budget being raised to
fit. So the ratchet: every fix's recovered minutes are re-spent by the next lane's
enrollment, silently, because nothing reds a merge that adds minutes.

## 4. The memory coupling (why "mechanism-correct" fixes recovered ~0%)

Time and memory are one budget and were treated as two. #6848's parse baseline plus
typecheck-env retention pinned executors at the 16GiB cap; at the cap, the wall is
reclaim throttle, not compute — so CPU fixes recover nothing (M1's "~0% on capped
hosts" receipt) and, past the tipping point, demand converts to unbounded wall time
(the 4h crawl, `oom_kill=0`). Any future fix priced in CPU minutes must state its
retention delta, and vice versa — the June InternTable fix (14.2→5.5GB) and the
reverted always-on resolve cache are both early receipts of the same coupling.

## 5. What ends it (construction, in order)

1. **The cost wall** — per-stage/per-row time budgets with typed refusal, plus a
   floor-time regression gate (delta vs baseline reds the PR that adds minutes). This
   requires revisiting the "no wall-clock in verdicts" ruling: a budget refusal is not
   a wall-clock term in a *witness verdict* — it is admission/scheduling, the same
   split as the 5s law. Until this exists, every other fix leaks.
   **LANDED 2026-07-23 (CI floor endgame):** per-batch wall budgets as data
   (`gunbc.ci_spec.gunbc_ci_floor_batch_wall_budget_seconds`) with typed
   `FLOOR-BATCH-OVER-BUDGET` refusals, typed per-batch receipt rows, the raise-requires-
   receipt-note discipline, and the ruling reconciliation in the carrier note (the
   operator dispatching the endgame brief signed that reading). Per-ROW budgets exist for
   the bin-wet lane (`bin_witness_wet_per_row_wall_budget_seconds`). This document's
   dissolution still waits on item 2 (ComputationIdentity) — the wall stops the ratchet;
   the identity authority ends the class.
2. **The identity authority** — ComputationIdentity over resolve/index demand (the
   designed duplicate-work lane): one authority, five grains become derivations.
   Interim instances that are already priced: pooled-per-gate child (13–17 min
   measured idle; QUALIFIED by the Pi bench — the win has a memory cliff under
   capped slots, so it pays fully only alongside per-worker index shrink), the
   #7122 residuals.
3. **The retention lane** — M2 eviction / W3 resolved-graph store, so the cap stops
   eating wins; and the honest cap conversation happens per-run (refuse at the cap,
   never crawl — the governor's terminal arm).
4. **Cost pricing at merge** — a lane enrolling corpus-denominated work states its
   floor delta in the PR, and the regression gate holds it to the statement. This is
   the workflow lane's receipt-freshness discipline applied to cost.

## 6. Provenance

Five-agent workflow sweep 2026-07-23 (fix-ledger: 65 events; receipt-trail; #6848
deep-dive; structural H1–H4 verification; adversarial reconciliation — including the
8m-vs-17.9m scope reconciliation: 8m denominates the witness-batch floor, 17.9m the ci
job, both pre-#6848). Source docs: floor-time-namespace-walk-regression-diagnosis.md,
floor-memory-pool-parse-regression-diagnosis.md, ci-floor-time-45-72-band-attribution.md
(§9), floor-shared-compute-memoization.md, v1-run-stability-throughline.md,
ci-performance-tanking-evidence.md, `gunbc_ci_floor_step_timeout_discovery_flip_note`.
