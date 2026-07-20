# Rc→Arc share spike — memory pricing report

> **Status:** measurement receipt, session smart-lark-630, 2026-07-20. **No migration landed** — harness + report only.
>
> Answers open decision **2** in [cross-worker-typecheck-share-design](cross-worker-typecheck-share-design.md) §9 (`Arc migration depth: store-path-only vs whole-seed`).
>
> Harness: `measure_rc_arc_share_spike` (`src/v1/stage0/src/bin/measure_rc_arc_share_spike.rs`), structural accounting in `index_memory_receipt.rs`. Lines are `[rc-arc-spike] kind=...` for grep/oracle consumption.

---

## 1. Executive summary

| Question | Answer |
|---|---|
| **Is cross-worker Arc share worth the memory trade on a 125 GiB box?** | **Yes at floor width ≥3** for witness-shaped closures — width-scaling shows ~linear private-index duplication (~620 MiB/worker per discovery entry); union-sample crossover at width **3** (optimistic) / **4** (pessimistic) on a 3-entry sample. |
| **store-path-only vs whole-seed Arc?** | **store-path-only** for increment C. Whole-seed is not required to land the typed store; `TypedModule.module: Rc<Node>` and `im_rc` collections are separate follow-ons (§4). |
| **im-rc blocker price?** | **123 / 154** stage0 `.rs` files import `im_rc` (aliased `HashMap`/`Vector`/`BTreeSet`). Collections stay `!Send` until `im-rc` → `im` swap — orthogonal to outer `Rc`→`Arc` on store payloads. |

**Operator note:** [execution-spine-design](execution-spine-design.md) removed the interpreter `Rc`→`Arc` gate (2026-07-10). This spike prices the **typecheck-store** lane only; escalate if these numbers gate increment C landing.

---

## 2. Methodology

### 2.1 Shareable vs per-worker residue (design §4.2)

| Bucket | `MultiEntryIndex` fields | Spike accounting |
|---|---|---|
| **Shareable** | `typed_module_cache` (+ transitive typed payloads) | Serde transport bytes (`SharedTypecheckCaches::encode_typed_snapshot`) + map shell. One module sampled, extrapolated × entry count (§2.2). |
| **Per-worker residue** | `parse_cache`, `resolved_graph_memo`, `intern_table`, `normalize_diag_cache`, `ownership_diag_cache` | Shallow map-shell + key bytes (residue **underestimates** parse/graph payload — noted below). |

Populate uses `resolve_entry_with_index_for_discovery_corpus` (floor's `DiscoveryCorpusAdvisory` gate), not strict whole-tree.

### 2.2 Measurement limits (honest)

1. **Serde extrapolation** — encoding every typed module OOM'd on this host; receipt uses **one** module's serde size × `typed_module_cache.len()`. Modules vary widely; extrapolated shareable bytes are an **order-of-magnitude proxy**, not a byte oracle. Peak RSS (VmHWM) is the reliable retention metric.
2. **Shallow residue** — parse trees and resolved graphs are not walked; reported residue is a **lower bound**. Real per-worker residue is larger.
3. **Full discovery union** — `populate_multi_entry_index_discovery_corpus(None)` OOM'd (exit 137, ~116 s). Default corpus is `representative` (9 large-closure entries) or `discovery:N` cap.
4. **union-sample N=9** — OOM'd during per-entry populate; N=3 completed.

---

## 3. Receipts (green-by-execution)

### 3.1 im-rc census

```
[rc-arc-spike] kind=im-rc-census im_rc_files=123 stage0_rs_files=154
```

Swapping `im-rc` → `im` (Arc-backed sibling) touches persistent maps/lists across the stage0 seed plus serde feature parity. **Not on increment C's critical path** if only `typed_module_cache` store payloads migrate to `Arc`.

### 3.2 Width-scaling baseline (`discovery:1`, parallel worker-index simulation)

| Width | Peak RSS (VmHWM) |
|---|---|
| 1 | 650 113 024 B (~620 MiB) |
| 2 | 1 295 810 560 B (~1.21 GiB) |
| 3 | 1 940 291 584 B (~1.81 GiB) |

**~linear ×W duplication** — consistent with per-worker `build_multi_entry_index` retaining a cold private cache (governor comment ~8091–8120 in `cli_run.rs`). At falsifier width **9**, naive extrapolation ≈ **5.6 GiB** duplicate index shells vs **~620 MiB** shared union for the same closure shape.

### 3.3 Index fields — single discovery entry (`discovery:1`)

```
[rc-arc-spike] kind=index-summary shareable_bytes=8144530640 residue_bytes=96096
  typed_module_cache_entries=37 peak_rss_bytes=653803520
```

37 typed modules; peak RSS **~624 MiB** (credible). Serde extrapolation **~7.6 GiB** (proxy only).

### 3.4 Index fields — representative union (`representative`, 9 entries)

```
[rc-arc-spike] kind=index-summary shareable_bytes=159913142162 residue_bytes=425132
  typed_module_cache_entries=318 peak_rss_bytes=746315776
```

318 typed modules across 9 large-closure entries; peak RSS **~712 MiB** for the combined index. Serde extrapolation **~149 GiB** is not credible (sample-module skew); **trust peak RSS** for retention pricing.

### 3.5 Union-sample net-win curve (3 witness entries, max width 9)

```
shareable_per_worker_avg=3349243766  residue_per_worker_avg=57030
union_shareable_max=8144530640       union_shareable_sum=10047731300
```

| Model | Crossover width (net_win_bytes > 0) |
|---|---|
| Optimistic (`union_shareable_max`) | **3** |
| Pessimistic (`union_shareable_sum`) | **4** |

On a **125 GiB** box at falsifier width **9**, shared retention is well inside envelope even before M2 strip; the **time** prize (52% `typecheck_compute`, design §0) dominates.

---

## 4. Decision 2 — store-path-only vs whole-seed

### 4.1 Recommendation: **store-path-only** for increment C

Aligns with [cross-worker-typecheck-share-design](cross-worker-typecheck-share-design.md) §9.2 default.

**In scope (store-path `Rc`→`Arc`):**

- `TypecheckModuleResult`, `TypedModule`, `ModuleInterface` inserted into `typed_module_cache`
- `parse_cache` entries on the typecheck critical path (design §4.2)
- `SharedTypecheckCaches` shell (`std::collections::HashMap` + `Arc` payloads — already modeled in #6561)

**Stays `Rc` (thread-local eval):**

- `ResolvedGraph` handed to `make_eval_context` per worker
- Ephemeral inference temporaries

### 4.2 Why whole-seed is not required now

1. **Eval path** — execution-spine ruling keeps interpreter on `Rc`; increment C does not need eval cross-thread.
2. **Peak RSS evidence** — duplicate cost is the per-worker **index shell + caches**, not every `Rc` site in the seed. Width-scaling shows the prize is killing **W× cold typed caches**, not a global `Rc` sweep.
3. **im_rc** — 123 files use `im_rc::HashMap`/`Vector`; outer `Arc` on store wrappers does not make these `Send`. A whole-seed sweep that stops at outer `Arc` leaves the im_rc blocker; a sweep that includes `im-rc`→`im` is a **separate migration** with much larger blast radius.

### 4.3 Residual risks (counted, not silent)

| Risk | Mitigation |
|---|---|
| `TypedModule.module: Rc<Node>` inside store payloads | Store-path migration must `Arc` nested carriers on **insert paths** only; eval keeps `Rc<Node>`. |
| Shared retention ↑ co-resident bytes | Governor dial + M2 strip (design §7); this spike shows headroom at width 9 on 125 GiB. |
| Serde transport size for cross-worker clone | Measure on falsifier host with batched per-module encode before S2b; not blocking C1 host proof. |

---

## 5. Harness invocation

```bash
cargo build --release -p v1-compiler --bin measure_rc_arc_share_spike

./target/release/measure_rc_arc_share_spike --mode im-rc-census
./target/release/measure_rc_arc_share_spike --mode index-fields --corpus representative
./target/release/measure_rc_arc_share_spike --mode width-scaling --corpus discovery:1 --max-width 3
./target/release/measure_rc_arc_share_spike --mode union-sample --sample-entries 3 --max-width 9
```

---

## 6. Dissolution

This doc dissolves when increment C lands (C1 host receipt + C2 ladder row) or when a successor measurement supersedes the harness receipts above.
