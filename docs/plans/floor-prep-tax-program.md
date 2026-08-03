# Floor prep-tax program — stop paying ~2s per entry to re-arrange work

**Status:** executing program (session `still-moth-459`, 2026-08-03). Measurement-first; no width or native expansion until the retention experiment decides the mechanism.

**Parent authorities (do not fork):** [five-minute-ci-gate-design](five-minute-ci-gate-design.md) · [v1-run-stability-throughline](v1-run-stability-throughline.md) · [m2-floor-retention-measurement-receipt](m2-floor-retention-measurement-receipt.md) · [per-entry-assembly-decomposition-measurement](per-entry-assembly-decomposition-measurement.md) · [witness-realization-plan](witness-realization-plan.md) · `dag/gunbc/executor_schedule_retention.dag` · `dag/gunbc/plans/ci_selection_vs_scheduling.dag`

**DESIGN refs:** §1 (time is the value), §2 (one shared preparation, N consumers), §5 (refuse / count degradation — never present a full floor as a successful “affected” run), §6 (displaced cost; no purity trap on parallelize-first).

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
| Native selected-batch (#7599) + production kind cutover (#7671) | Live; **3-member** cohort | Optimizes the ~150 ms evaluation slice; leaves the ~77s resolve intact |
| Five-minute program “retention PARKED” row | Stated for **memory pressure** absence at width 1 | **Reopened here on the time axis** — lack of memory distress is not license to destroy reusable prep every entry |

`resolved_graph_memo` is keyed by **entry subject**. Retaining graph A does not serve entry B. So “stop dropping graphs” alone is insufficient unless paired with **shared module/index universe + cheap per-entry assembly from retained facts** (materialization / native bundle / assembly memo — mechanism chosen by P1 receipt, not by taste).

---

## 3. Parallelism on a 16 GiB runner

**Width 2 is not the first move.** Process peak ~6.27 GiB (uncapped host) → 2× ≈ 12.5 GiB before OS, rustc, allocator transients, duplicated indexes. Native selected-batch cgroup peak ~11.1 GiB. Width 3 excluded at present footprint. Width 2 only as a **hard-governed** experiment **after** retention is bounded and shared preparation exists; otherwise workers duplicate the cold prep this program exists to remove.

Productive parallelism shape (ordered):

1. Build / load the shared module/index universe **once**
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

Then, only if A→B shows the ~2s tax collapsing for entries 2..N sharing a module universe, implement **bounded retention**: drain when a memory threshold is crossed, not on every entry completion. If A→B does **not** collapse the tax, the prize is **assembly reuse / materialization / shared index view** (route to `module-grain-materialization` / assembly mechanism), not softer eviction.

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

An “affected” run that selects all 842 must **say so**. Large supersets remain sound selection but are a **counted performance degradation**, not a green affordance story.

### P3 — Width-2 trial only after P1’s bounded retention

On a real 16 GiB runner: hard-cap aggregate worker memory; prevent simultaneous native compile peaks; share immutable indexes; start with two **interpreted** shards; refuse the second worker when the envelope cannot admit it. Compare **CPU time and wall** so duplicated cold work cannot masquerade as a win.

### P4 — Expand native enrollment afterward

Class-by-class after setup is amortized. Until then, native migration optimizes the wrong term.

---

## 5. Relationship to the five-minute sequence

Does **not** replace the operator sequence in `five-minute-ci-gate-design.md`. It **inserts a binding precondition** in front of “width” and “broad native”:

* Steps that enlarge **evaluation** efficiency (native cutover) stay valuable for self-host and large bodies.
* Steps that claim **floor wall** recovery without amortizing prep are mispriced until P1’s mechanism lands or is honestly routed to materialization/assembly.

Update the five-minute “retention PARKED” disposition when P1’s receipt is banked (time-axis reopen + mechanism or honest redirect).

---

## 6. Dissolution

Delete or fold this note when:

1. Floor receipts show amortized prep (entries 2..N in a shared universe ≪ 2s setup), **and**
2. Selection receipts expose state/ratio/fallback on every affected run, **and**
3. Width>1 / native expansion decisions cite those receipts rather than this bridge doc —

or when the operator recuts the program into the five-minute / materialization authorities directly.

Session closeout work item: `floor prep tax e2e closeout`. Child leaves: P1–P4 as created under `still-moth-459`.
