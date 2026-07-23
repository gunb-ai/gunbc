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

The **~72 min** figure in `gunbc_ci_witness_corpus_only_batches_note` is whole-tree **emit**
infeasible for pre-push — not typical ci job wall. Scoped PRs skip compile-clean emit entirely
(`compile-clean scope: skipped`).

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
