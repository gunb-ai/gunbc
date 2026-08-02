# Receipt — entry-graph-union slice 2: repeated-typecheck attribution

**Status:** operator verdict taken 2026-08-01 (§F; PR #7533). **Bounded to N≤50** disjoint production windows + 7-entry explicit fixture; `>50` unmeasured. **DESIGN.md + slice 1 remain the authority** — this is a dated receipt scaffold, not a floor fact ledger.

**Disposition (operator adjudication 2026-08-01):** Close `entry-graph-union-construction` as a **no-go** for its measured shared-typecheck hypothesis. Do not begin union construction. The priced benefit of a union/shared typed-computation graph is **absent** at N≤50 — repeated typecheck compute is zero because the existing typed cache already enforces once-per-content-key. A future assembly-specific investigation is a **new hypothesis** with a new denominator; it does not continue this lane.

**Dissolution (measurement orchestration):** intrinsic to this lane — **not** tied to #7534 (exact-tree materialization consumer, unrelated). Sequence: candidate receipts accepted → merge → reproduce one representative 50-entry cell + reorder control on merged `main` → archive provenance receipt → **DELETE** one-shot measurement orchestration (final commit on this lane or named tiny follow-up immediately after merged-SHA receipt; never smuggled into another PR's rebase).

**Post-verdict retention boundary (§G):** retain the **law**, not the instrument — see below.

**ROADMAP:** `gunbc.roadmap_authority` id `entry-graph-union-construction` (lane `ci-cost`, slice 2 of the same row as slice 1).

**Lane:** `ci-cost` · **Subject:** `entry-graph-union-slice2-typecheck-attribution` · **Deliverable:** measurement only. No union implementation.

**Predecessor:** [entry-graph-union slice 1](entry-graph-union-slice1-measurement.md) (#7483).

---

## A — what slice 1 left open

Slice 1 established that production-selected closures repeat **module membership** at 35.9–38.4× by count (43–49× byte-weighted), while the dominant single-entry `load` row is `build_both_closure_edge_index` — already paid once per `MultiEntryIndex` and flat in entry count (1.08× over N=2→7). It explicitly **withdrew** quoting membership duplication as a typecheck-time multiplier.

The operator question slice 2 answers: **how often does a repeated module membership cause a real `typecheck_compute` miss, and how much time do those repeated misses consume?**

---

## B — the retired instrument

**Historical probe (deleted after the merged-SHA receipt):**
`measure_repeated_typecheck_attribution`
(`src/v1/stage0/src/bin/measure_repeated_typecheck_attribution.rs`)

**Mechanism used for the archived measurement:** production-selected entries (same
machinery as slice 1) resolved sequentially against **one** `build_multi_entry_index`
shell. Attribution hooks in `reconcile_with_typed_cache` and
`try_reconcile_all_cache_hits` were **flag-gated**
(`arm_repeated_typecheck_attribution_probe`); default resolve flow was unchanged.

### Per (entry, typed module content key)

| field | meaning |
|---|---|
| `entry` | witness entry path |
| `module_key` | `typed_module_content_key` (never bare module name) |
| `module_path` | authored name (readability) |
| `in_closure` | always `true` when observed from reconcile |
| `cache_disposition` | `Hit` \| `Miss` \| `Refused` (three states, never two) |
| `typecheck_compute_ns` | wall on `Miss` only; `0` on `Hit` |
| `first_computing_entry` | entry that first computed this key in this run |
| `later_requester_count` | prior entries that already observed this key |

### Per entry

`entry_timings[]`: `reconcile_assembly_ns`, `typecheck_compute_ns`, `resolve_nanos` from `ResolveStageNanos`.

### Aggregates

- selected entry count, `sum_closure_memberships`, `union_modules`, membership duplication factor (slice-1 comparison)
- `total_cache_hits` / `total_cache_misses` / `total_cache_refusals`
- `repeated_typecheck_misses` — misses with `later_requester_count > 0`
- `first_computation_typecheck_ns` / `repeated_typecheck_compute_ns`
- `fanout_by_module` (from slice-1 overlap probe)
- `memory.process_vm_hwm_bytes` — process VmHWM at measurement end (includes selector/index setup, not an attribution-only peak)
- `memory.rss_before_measurement_bytes` / `memory.rss_after_measurement_bytes` — scoped VmRSS bracketing the resolve loop
- `memory.cgroup_memory_events_high` — leaf cgroup `memory.events` high counter when readable

### The decision quantity

```
decision_ratio = repeated_typecheck_compute_ns / total_typecheck_compute_ns
```

**NOT** closure duplication. This ratio was the slice-2 **hypothesis test** only (now adjudicated at N≤50). Low → cache already eliminates repeated work. High → repeated membership becomes repeated typecheck. Verdict: low at every measured cell — union construction **not justified** by this hypothesis.

Diagnostic only: `cache_hit_ratio = hits / (hits + misses)`.

---

## C — durable controls (proven by execution)

| control | witness |
|---|---|
| One content key computes once; later requesters hit | `union_resolve_typechecks_each_node_once` (union_resolve_receipts_test) |
| Reordering entries preserves the distinct content-key set | `shared_typecheck_distinct_compute_count_is_order_invariant` (union_resolve_receipts_test) |

The one-shot decision instrument also executed controls for shared-prefix hits, zero-cost
hits, refused rows, ratio arithmetic, and reordered miss sets. Their outcomes remain in
the archived receipts, but their witnesses were deleted with the instrument after the
merged-SHA integration receipt. They are historical controls, not enrolled regressions.

---

## D — host constraints (carried from slice 1)

- Whole-corpus discovery OOMs on dev hosts (exit 137 at 1,046/1,513 entries). Use `--max-entries N` explicit entries against one shared index.
- Do not rebuild the binary mid-probe (resolve cache content-addresses executable).
- M2 retention (#7129) on main — re-verify OOM binding before floor-scale runs.

---

## E — reproduction boundary

The production ratio instrument was deliberately deleted after its merged-SHA receipt,
so the historical N<=50 matrix is not a permanently reproducible CLI surface. The
retained structural law remains directly executable:

```sh
cargo test -p v1-compiler-tests union_resolve_typechecks_each_node_once
cargo test -p v1-compiler-tests shared_typecheck_distinct_compute_count_is_order_invariant
```

Section F and the receipt directory preserve the historical matrix output. Recreating its
ratio would require restoring retired orchestration and is intentionally unsupported:
retain the cache law, not the one-shot instrument.

---

## F — execution matrix (2026-08-01, `711341bb`, binary `target/slice2-measurement-bin/`)

### Host envelope / governor / retention

| field | value |
|---|---|
| cgroup `memory.max` | 33,578,549,248 bytes (~31.3 GiB) |
| `GUNBC_MEMORY_BUDGET_BYTES` | unset |
| probe scheduling | serial, one shared `MultiEntryIndex`; adaptive `MemoryGovernor` **not engaged** |
| M2 schedule-derived retention (#7129) | **not armed** on this instrument path — `index_arm_schedule_retention` runs on the `claim_executor` discovery/floor path only; the probe retains typed state for the process lifetime (pre-M2 pole) |
| `cgroup_memory_events_high` | 0 on every run |

**Retraction (kept):** uncapped narrow (`N=286`, `GUNBC_CI_DIFF_BASE=e30621111f37`) was **OOM-killed** at the cgroup ceiling (~31 min, exit `Killed`, no receipt). Because M2 retention is **not** active here, this is an **instrument-path retention artifact**, not a quotable floor fact — it is nevertheless a retention data point: serial single-index resolve without roster-aware eviction exceeds ~32 GiB before `N=286` completes on this host.

**Retraction (kept):** the first `narrow-production` / `typical-production` / `broad-capped` cells (shared first-50 prefix) measured **one subject three times** — withdrawn as breadth evidence; superseded by disjoint windows below.

Binary snapshot: `sha256:5b7c4673f4a26a0b168b48dfbb946c27e2b30eae6149b120dcd4262840abeb3f`.

### Disjoint production handback (50-entry windows, distinct subjects)

| run | diff base | offset | N | union modules | hits | misses | refusals | repeated misses | total TC wall | repeated TC wall | decision_ratio | cache_hit_ratio | Σ assembly | VmHWM |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| narrow-disjoint | `e30621111f37` | 0 | 50 | 619 | 4717 | 622 | 0 | **0** | 36.5s | 0 | **0** | 0.884 | 37.6s | 8.88 GiB |
| typical-disjoint | `b01cdf4d8914` | 117 | 50 | 708 | 4510 | 710 | 0 | **0** | 40.8s | 0 | **0** | 0.864 | 36.4s | 9.12 GiB |
| broad-disjoint | `0d6ffc4db975` | 235 | 50 | 555 | 2470 | 557 | 0 | **0** | 33.8s | 0 | **0** | 0.816 | 35.1s | 7.73 GiB |

Explicit reorder (7-entry slice-1 amortization set): `decision_ratio=0`, identical distinct-compute set (398) both orders — see `receipt-explicit-order-{a,b}.json`.

### Verdict (operator adjudication, 2026-08-01)

**Measured scale (explicit bound):** **N≤50** — three disjoint 50-entry production windows + 7-entry explicit reorder fixture. **`>50`-entry regime unmeasured.** No larger-host confirmation run (a bigger host would only extend the pre-M2 retain-everything pole; it measures retention capacity, not repeated typechecking).

At every measured cell at N≤50: **repeated typecheck compute is zero**, order-stable, and the typed cache is already strong (`cache_hit_ratio` 0.82–0.88 on production windows despite 8–14× membership duplication within each window).

**Disposition:** Close `entry-graph-union-construction` as a **no-go for union construction** on the shared-typecheck hypothesis. The existing typed cache already enforces the structural law (once per content key; later requesters hit). **Do not begin union construction.** Redirect to a separate **per-entry assembly decomposition** lane (new hypothesis, new denominator) — Σ assembly ≈ 35–38s vs total typecheck ≈ 34–41s per 50-entry cell is the surviving cost signal, not a presupposed union-graph fix.

Full per-row receipts: `docs/plans/receipts/entry-graph-union-slice2/receipt-*.json`.

Historical exact invocations (archived for provenance; intentionally not runnable after
the one-shot bin was deleted):

```sh
GUNBC_CI_DIFF_BASE=e30621111f37 measure_repeated_typecheck_attribution \
  --source-root dag --source-root src/v2 \
  --scan-dir dag/test/claim --scan-dir src/v2/test/claim/manual \
  --scan-dir src/v2/test/claim/emit \
  --entry-offset 0 --max-entries 50 \
  --receipt-out docs/plans/receipts/entry-graph-union-slice2/receipt-narrow-disjoint.json

GUNBC_CI_DIFF_BASE=b01cdf4d8914 measure_repeated_typecheck_attribution \
  --source-root dag --source-root src/v2 \
  --scan-dir dag/test/claim --scan-dir src/v2/test/claim/manual \
  --scan-dir src/v2/test/claim/emit \
  --entry-offset 117 --max-entries 50 \
  --receipt-out docs/plans/receipts/entry-graph-union-slice2/receipt-typical-disjoint.json

GUNBC_CI_DIFF_BASE=0d6ffc4db975 measure_repeated_typecheck_attribution \
  --source-root dag --source-root src/v2 \
  --scan-dir dag/test/claim --scan-dir src/v2/test/claim/manual \
  --scan-dir src/v2/test/claim/emit \
  --entry-offset 235 --max-entries 50 \
  --receipt-out docs/plans/receipts/entry-graph-union-slice2/receipt-broad-disjoint.json
```

---

## G — post-verdict retention boundary

**Retained (structural law — enrolled regression controls):**

Within one shared typed-cache authority:

1. One typed module **content key** computes at most once per process.
2. Later requesters observe **cache hits**, not recomputation.
3. Entry resolve **order does not change** the distinct-computation set.

Witnesses: `union_resolve_typechecks_each_node_once` (first compute only; later requester
adds fewer computes and re-resolves add zero) and
`shared_typecheck_distinct_compute_count_is_order_invariant` (`v1-compiler-tests`).

**Deleted after merged-SHA provenance receipt:**

- `measure_repeated_typecheck_attribution` bin and orchestration (`measure_*`, production-selection roster path)
- JSON receipt renderers (`render_repeated_typecheck_attribution_*`)
- `RepeatedTypecheckAttributionMeasurement` carrier and flag-gated probe plumbing (`arm_*` / `observe_*` / `note_*` in reconcile)
- `repeated_typecheck_attribution_arithmetic` (ratio aggregation — historical instrument math, not the structural law)

`decision_ratio=0` was the slice-2 hypothesis outcome, not a permanent structural invariant to defend via production CLI forever.

### Post-merge integration receipt (provenance, not a second decision gate)

The representative 50-entry cell was rerun from merged main with binary SHA-256
`1e8aba324353aca22bce3a9586e368633263757bb4d87584c15e5345301b9b45`:
4,302 hits, 680 first misses, zero refusals, and zero repeated misses. The seven-entry
reorder control produced 225 hits and 400 first misses in each direction; the sorted
distinct miss-key sets are identical.

Receipts: `receipt-post-merge-representative-50.json`,
`receipt-post-merge-reorder-a.json`, and `receipt-post-merge-reorder-b.json` under the
slice-2 receipt directory. The reverse receipt names dashboard commit `a3b2c6dcb9` while
the other two name `46c5c8fe71`: the dashboard committed only the representative JSON
between the frozen-binary runs. The compiler tree and binary were identical, so this is a
provenance wrinkle, not a source change or a second gate.

The successor measurement is
[per-entry assembly decomposition](per-entry-assembly-decomposition-measurement.md).
