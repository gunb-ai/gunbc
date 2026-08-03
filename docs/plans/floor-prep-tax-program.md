# Floor prep-tax program — stop paying ~2s per entry to re-arrange work

**Status:** executing program (session `still-moth-459`, 2026-08-03). Measurement-first. **P1 gates width expansion and broad native enrollment only** — it does **not** defer five-minute step 3’s already-landed bounded production native cutover (#7599 → #7671’s live 3-member cohort), which remains NEXT under `gunbc.roadmap_authority` / [five-minute-ci-gate-design](five-minute-ci-gate-design.md).

**Parent authorities (do not fork):** [five-minute-ci-gate-design](five-minute-ci-gate-design.md) · [v1-run-stability-throughline](v1-run-stability-throughline.md) · [m2-floor-retention-measurement-receipt](m2-floor-retention-measurement-receipt.md) · [per-entry-assembly-decomposition-measurement](per-entry-assembly-decomposition-measurement.md) · [cross-worker-typecheck-share-design](cross-worker-typecheck-share-design.md) · [witness-realization-plan](witness-realization-plan.md) · `dag/gunbc/executor_schedule_retention.dag` · `dag/gunbc/plans/ci_selection_vs_scheduling.dag`

**DESIGN refs:** §1 (time is the value), §2 (one shared preparation, N consumers), §3 (single dispatch authority — five-minute sequence owns bounded native cutover), §5 (refuse / count degradation — never present a full floor as a successful “affected” run), §6 (displaced cost; no purity trap on parallelize-first).

---

## 0. Bottom line (the displaced cost)

A full floor spends roughly **three quarters of wall time arranging work whose evaluation takes about 84 seconds**.

| Population | Estimated repeated entry-setup tax |
| ---: | ---: |
| 100 entry groups | ~3m 12s |
| 602 (typical “affected”) | **~19m 28s** |
| 842 (whole corpus) | **~27m 15s** |

Instrument basis (entry-to-entry gaps on a live floor excerpt): mean wall gap **2.116 s**, mean witness body **171.9 ms**, unaccounted setup **1.944 s** (~92% of the interval). Whole-corpus receipt (#7581 / Dispatch J): 842 groups / 5,818 witnesses, **33m17s** end-to-end, **25m12s resolving**, **1m24s evaluating**, width 1; schedule construction ~**94.7%** of resolver-plus-evaluation time.

Native execution and affected-set selection are real and useful, but **neither cures this lifecycle**. #7671’s selected native batch spent ~77s with ~79s aggregate resolving and ~149 ms evaluating. Perfect selection of 100 entries would still burn >3 minutes on repeated setup.

The floor currently behaves less like “run 5,818 witnesses” and more like **“construct or recover 842 execution worlds, evaluate briefly inside each, then discard enough state to pay again.”**

---

## 1. Two separate defects (do not fuse)

| Defect | Symptom | Honest fix axis |
|---|---|---|
| **D1 — per-entry prep tax** | ~1.9s between entry groups after `[floor-drain] … evicted_graph=1` | Shared preparation + retention under a memory envelope; drain only under measured pressure |
| **D2 — selection effectiveness / honesty** | “Affected” runs that are `SelectionSuperset`, selection-unavailable, or 602/842 (~71.5%) without saying so | Publish `selection_state`, selected/total, ratio, `fallback_reason`; treat large supersets as loud performance degradation, not quiet success |

Saving (842−602)×1.944s ≈ **7m47s** is real but secondary. D1 is the fundamental defect: even an exact 100-entry selection remains setup-dominated.

Related in-flight (do not duplicate): PR **#7720** (sleek-dove) — affected-set skip retained a zero-result entry + schedule-retention **log accounting** (release ≠ eviction). Absorb after merge; this program owns the *time* axis and the selection *receipt surface*.

---

## 2. What already landed (and what it does not buy)

| Layer | Status | What it does **not** do |
|---|---|---|
| M2 schedule-derived retention (#7129) | Live; width-1 peak ~6.3–10.7 GiB | Still drops the completed entry’s **resolved-graph pin** every entry; still pays per-entry **assembly** (symbol-index merge / rewiring — see #7597) |
| Entry-graph union (shared typecheck hypothesis) | **CLOSED** no-go (#7533) | Typed cache already once-per-content-key; union construction is not the prize |
| Per-entry assembly decomposition (#7597) | Measured | Prices the surface (~1.5–1.7s/entry class on the 50-entry harness); does not select a mechanism |
| Native selected-batch (#7599) + production kind cutover (#7671) | Live; **3-member** cohort | Optimizes the ~150 ms evaluation slice; leaves the ~77s resolve intact. **Five-minute step 3 stays NEXT** — this program does not park it |
| Five-minute program “retention PARKED” row | Stated for **memory pressure** absence at width 1 | **Reopened here on the time axis** — lack of memory distress is not license to destroy reusable prep every entry |

`resolved_graph_memo` is keyed by **entry subject**. Retaining graph A does not serve entry B. So “stop dropping graphs” alone is insufficient unless paired with **shared module/index universe + cheap per-entry assembly from retained facts** (materialization / native bundle / assembly memo — mechanism chosen by P1 receipt, not by taste).

### Do not misuse #7597 as a Mode-B prior (retraction)

An earlier draft of this note treated the #7597 representative-50 `claim_batch` resolve times as a “retain-all lower bound” that *tilted* P1 toward assembly/materialization. **Retracted.** Three independent reasons (closeout `sleek-fox-808`, 2026-08-03):

1. **Denominator:** the floor’s ~1.944 s figure is an entry-to-entry **elapsed wall** gap; #7597’s assembly totals are **additive span totals** for a harness — the receipt itself forbids quoting them as fleet wall (`per-entry-assembly-decomposition-measurement.md`). Same two-denominator class that forced slice-1 share retractions.
2. **Arming ≠ Mode B:** “schedule-retention not armed” is not `GUNBC_SCHEDULE_RETENTION_EVICT=0` on the floor path. An unarmed harness may never have offered cross-entry reuse; it does not forecast what Mode B does when retention is armed and counting.
3. **Source forbids mechanism selection:** #7597’s decision boundary — price surface only; does not select a bundle/reuse mechanism or claim fleet recovery from additive shares. Its row names are prices, not endorsements.

The independent argument that `resolved_graph_memo` is entry-subject-keyed (retention of graph A does not serve entry B) **stands without** that measurement. **P1’s value is discrimination** — Mode A vs Mode B on the armed floor path — not confirmation of a handed prior. The floor A/B receipt is the only evidence that can redirect (or not).

---

## 3. Parallelism on a 16 GiB runner

**Width 2 is not the first move — and is not reachable profitably today.** Process peak ~6.27 GiB (uncapped host) → 2× ≈ 12.5 GiB before OS, rustc, allocator transients, duplicated indexes. Native selected-batch cgroup peak ~11.1 GiB. Width 3 excluded at present footprint.

**Measured latch (production discovery pool):** `cli_run.rs` samples `target_width` **once** at adaptive-pool entry; when `≤1` it takes the inline-drain branch and never reaches the governor AIMD path that would admit plural workers. Un-latching without shared indexes is a **measured loss** on the same 621 entry-groups: serial **11.75 min GREEN** (CI 29707161743, `max_width_reached=1`, peak 6.97 GB) vs un-latched **47 min+ unfinished** (CI 29714863168) vs un-latched OOM-killed at **101.6 GB in 11 min** (CI 29710324768). Reason named in-code: Amdahl — each worker’s front cost is its **own** whole-tree index (~10.7 GB, minutes) against ~12 minutes of corpus work. dissolve-on cited there: **Rc→Arc / shared index** ([cross-worker-typecheck-share-design](cross-worker-typecheck-share-design.md) open decision 2), which also **increases** co-resident retention — the win is a width crossover, not a given.

Productive parallelism shape (ordered):

1. Build / load the shared module/index universe **once** (Rc→Arc / index share is the mechanism that retires the width latch)
2. Reuse resolved / assembled facts across entry groups under a memory envelope
3. Partition **already-prepared** work
4. Admit workers under the global memory governor
5. Drain only under measured pressure (never unconditionally after each entry)

---

## 4. Priority order (binding)

### P1 — Discriminating retention vs drain (this program’s first receipt)

Fixed cohort, two modes:

* **A — current:** schedule-retention eviction on (production default)
* **B — retain-all pole:** `GUNBC_SCHEDULE_RETENTION_EVICT=0` (existing measurement control; drops nothing)

Then, only if A→B shows the ~2s tax collapsing for entries 2..N sharing a module universe, implement **bounded retention**: drain when a memory threshold is crossed, not on every entry completion. If A→B does **not** collapse the tax, the prize is **assembly reuse / materialization / shared index view** (route to `module-grain-materialization` / assembly mechanism), not softer eviction. **Do not enter P1 with a prior on which arm wins** — the A/B receipt is the discriminator (see “Do not misuse #7597” above).

Per entry (required receipt fields):

```text
resolve_ms
eval_ms
schedule_cache_hit
resolved_graph_hit
modules_evicted
graphs_evicted
process_rss
cgroup_memory_current
cgroup_memory_peak
```

**Acceptance:** second and subsequent entries sharing the same module universe do **not** each repay ≈2s. “Faster” alone is not enough — the receipt must show the setup tax moving.

### P2 — Selection degradation explicit

Every floor receipt publishes:

```text
selection_state=SelectionApplied|SelectionSuperset|SelectionUnavailable|…
selected_entry_groups=N
total_entry_groups=M
selection_ratio=…
fallback_reason=…
```

An “affected” run that selects all 842 must **say so**. Large supersets remain sound selection but are a **counted performance degradation**, not a green affordance story. May proceed in parallel with P1 (receipt surface only). Do not duplicate #7720.

### P3 — Width-2 trial (conjunction gate)

**Gate = P1 banked AND index sharing landed** (the Rc→Arc / shared-index dissolve-on that retires the width≤1 latch). P1 alone is **not** sufficient — a perfect retention result does not make width-2 profitable while each worker still builds its own whole-tree index (measured refutation: CI 29707161743 / 29714863168 / 29710324768 above).

When both hold, on a real 16 GiB runner: hard-cap aggregate worker memory; prevent simultaneous native compile peaks; start with two **interpreted** shards; refuse the second worker when the envelope cannot admit it. Compare **CPU time and wall** so duplicated cold work cannot masquerade as a win. Expect a retention/width crossover, not a free win ([cross-worker-typecheck-share-design](cross-worker-typecheck-share-design.md) §7).

### P4 — Expand native enrollment (broad), after prep amortized

**Not** five-minute step 3 (bounded 3-member cohort — already live / stays NEXT under that authority). This leaf is **class-by-class enlargement** of the native cohort after D1 setup is amortized. Until then, broad native migration optimizes the wrong term for floor wall.

---

## 5. Relationship to the five-minute sequence

Does **not** replace the operator sequence in `five-minute-ci-gate-design.md` and does **not** fork its dispatch authority (§3). Specifically:

* **Five-minute step 3** (bounded production native cutover) remains **NEXT** / live — this program must not be read as parking it behind P1.
* This program inserts a binding precondition in front of **width>1** and **broad** native enrollment (P3/P4 here).
* Steps that claim **floor wall** recovery without amortizing prep are mispriced until P1’s mechanism lands or is honestly routed to materialization/assembly.

Update the five-minute “retention PARKED” disposition when P1’s receipt is banked (time-axis reopen + mechanism or honest redirect).

---

## 6. Dissolution

Delete or fold this note when:

1. Floor receipts show amortized prep (entries 2..N in a shared universe ≪ 2s setup), **and**
2. Selection receipts expose state/ratio/fallback on every affected run, **and**
3. Width>1 / broad native decisions cite those receipts (and the index-share conjunction) rather than this bridge doc —

or when the operator recuts the program into the five-minute / materialization authorities directly.

Session closeout: `sleek-fox-808` (waits on parent Complete). Live leaves: P1 `valiant-deer-205`, P2 `warm-wolf-777` (PR #7722 OPEN — tracking: five fields must reach every floor receipt emission path before ready-for-review; carrier-only is not the bar), P3 `merry-ibex-227` (HOLD — conjunction), P4 `sharp-carp-537` (HOLD — broad enrollment only).
