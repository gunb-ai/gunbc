# Rc→Arc share spike — memory pricing report

> **Status:** measurement receipt, session smart-lark-630, 2026-07-20. **No migration landed** — harness + report only.
>
> Answers open decision **2** in [cross-worker-typecheck-share-design](cross-worker-typecheck-share-design.md) §9 (`Arc migration depth: store-path-only vs whole-seed`).
>
> Harness: `measure_rc_arc_share_spike` (`src/v1/stage0/src/bin/measure_rc_arc_share_spike.rs`). Lines are `[rc-arc-spike] kind=...`.

---

## 1. Executive summary (what is actually established)

| Question | Answer |
|---|---|
| **What object was measured?** | The **floor worker index**: `build_multi_entry_index` over both roots (`dag`, `src/v2`), then discovery-corpus warm (`populate_worker_discovery_index` — all 73 unique discovery entries, 299 rows). This matches the adaptive worker spawn site (`cli_run.rs` ~11191–11195), **not** a single-entry import closure. |
| **Peak RSS on worker object (this box)?** | **~1.51 GiB** VmHWM after discovery warm (73 entries). Shell-only build: **~50 MiB**, **~7.5 s**. |
| **Width duplication (RSS)?** | **~linear**: W=1 → 1.58 GiB; W=2 → 3.16 GiB on the same box. |
| **RSS crossover for sharing?** | At W=2, private ≈3.16 GiB vs one union ≈1.58 GiB — memory favors sharing from width 2 **on RSS alone**. This box has a **31.25 GiB cgroup cap**; even W=9 private ≈14 GiB stays under cap — **bytes are not the binding constraint here**. |
| **Is width worth it for wall-clock?** | **No — decode dominates typecheck.** Module-grain ratio (§3.6): **0.01** typecheck/decode per module (~3.8 ms typecheck vs ~318 ms decode on 40 serde-eligible modules). Sharing typed snapshots does **not** remove the ~55 s warm; it would **add** serde decode cost. PR #6905 Amdahl bound still holds. |
| **store-path-only vs whole-seed?** | **store-path-only** for increment C (design §9.2 default). `im_rc` collections stay `!Send` without `im-rc`→`im`. |
| **im-rc price?** | **123 / 154** stage0 `.rs` files reference `im_rc` (dynamic census). `lib.rs` aliases `HashMap`/`Vector`/`BTreeSet` to it. |

**What is NOT established:** serde-based byte crossover (discarded — see §2.2). Whole-tree strict resolve to the ~10.7 GiB nimble-owl/srv1 receipt (not re-run here). `build-vs-attach` was **not retried** (OOM by construction on 31 GiB — two whole-tree indexes); module-grain (§3.6) answers the wall-clock question instead.

---

## 2. Methodology

### 2.1 Measurement host (always record with numbers)

| Field | Value |
|---|---|
| **hostname** | `b96c6d293441` |
| **MemAvailable** | ~107–117 GiB (`/proc/meminfo`) |
| **cgroup memory.max** | **33 578 549 248 B (~31.25 GiB)** — processes OOM here before host RAM is exhausted |
| **source roots** | `dag`, `src/v2` |

### 2.2 Honest limits

1. **Peak RSS (VmHWM) is the only byte metric used for conclusions.** Per-field `index-field` lines are shallow map-shell + key counts; they **under-count** parse trees, resolved graphs, and typed payloads. They are diagnostic, not summed for crossover.
2. **Serde transport bytes are not reported** — per-module encode OOM'd or produced non-credible extrapolations; using them for crossover was an error in the first draft (corrected per operator review).
3. **Residue is under-estimated** — `parse_cache`, `resolved_graph_memo`, and `intern_table` payloads stay per-worker after Arc migration; shallow accounting excludes their heap bodies.
4. **`build-vs-attach` not run** — holds two whole-tree indexes in one process; OOM on 31 GiB cgroup by construction. Module-grain (§3.6) measures the same decision at module residency.
5. **Module-grain sample bias** — serde snapshots capped at 50 MiB to stay inside cgroup; only **40 / 513** cold-miss modules fit (473 skipped). Ratio is over the serde-eligible tail, not a uniform corpus draw.

### 2.3 Shareable vs per-worker residue (design §4.2)

| Bucket | Fields | Spike treatment |
|---|---|---|
| **Shareable (typed store)** | `typed_module_cache` | Entry count + map shell bytes; **peak RSS** for retention |
| **Per-worker residue** | `parse_cache`, `resolved_graph_memo`, `intern_table`, normalize/ownership diag caches | Shallow shell (lower bound) — real residue larger |

---

## 3. Receipts (green-by-execution, pasted output)

### 3.1 Host metadata

```
[rc-arc-spike] kind=host-metadata hostname=b96c6d293441 mem_available_kb=107078536 cgroup_max_bytes=33578549248
```

### 3.2 im-rc census

```
[rc-arc-spike] kind=im-rc-census im_rc_files=123 stage0_rs_files=154 note=im-rc (Rc-backed HAMT) is aliased as HashMap/Vector/BTreeSet in lib.rs; the Arc-backed sibling crate is `im`. Collections remain !Send even if outer Rc→Arc lands — swapping im-rc→im touches every persistent map/list in stage0 .rs files plus serde feature parity.
```

### 3.3 Worker shell build (`--mode worker-shell-build`)

```
[rc-arc-spike] kind=timing label=worker-shell-build elapsed_ms=7452 peak_rss_bytes=52498432
```

~7.5 s, ~50 MiB — this is what every worker pays at thread start **before** any entry-group resolve.

### 3.4 Worker discovery warm (`--mode worker-discovery-warm`)

Full discovery roster: **73 entries**, **299 rows**.

```
[rc-arc-spike] kind=timing label=worker-shell-only elapsed_ms=6996 peak_rss_bytes=52486144
[rc-arc-spike] kind=timing label=worker-discovery-warm elapsed_ms=55011 peak_rss_bytes=1626210304
[rc-arc-spike] kind=worker-object discovery_entries=73 discovery_rows=299 peak_rss_bytes=1626210304
```

~55 s warm, **~1.51 GiB** peak — the retention shape after resolving all discovery entries on one worker index.

Per-field shallow accounting (shell + keys only — **not** payload bodies; peak RSS is authoritative):

```
[rc-arc-spike] kind=index-field field=typed_module_cache bucket=shareable bytes=49400 entries=517
[rc-arc-spike] kind=index-field field=parse_cache bucket=per_worker_residue bytes=66730 entries=517
[rc-arc-spike] kind=index-field field=resolved_graph_memo bucket=per_worker_residue bytes=5864 entries=73
[rc-arc-spike] kind=index-field field=intern_table bucket=per_worker_residue bytes=507672 entries=21153
[rc-arc-spike] kind=index-field field=normalize_diag_cache bucket=per_worker_residue bytes=59562 entries=517
[rc-arc-spike] kind=index-field field=ownership_diag_cache bucket=per_worker_residue bytes=59562 entries=517
[rc-arc-spike] kind=index-summary shareable_bytes=49400 residue_bytes=699390 total_accounted_bytes=748790 typed_module_cache_entries=517 serde_transport_bytes=0 peak_rss_bytes=1617641472
```

**Shareable vs residue (shallow shells):** 49 400 B shareable map shell vs 699 390 B residue shells — **misleading for crossover** because shallow residue excludes parse/typed payloads while peak RSS (~1.61 GiB) captures them. **517 typed modules** cached after full discovery warm.

### 3.5 Width-scaling worker object (`--mode width-scaling-worker --max-width 2`)

```
[rc-arc-spike] kind=width-scaling width=1 peak_rss_bytes=1582301184
[rc-arc-spike] kind=width-scaling width=2 peak_rss_bytes=3162173440
```

| Width | Peak RSS |
|---|---|
| 1 | 1 582 301 184 B (~1.47 GiB) |
| 2 | 3 162 173 440 B (~2.95 GiB) |

**~linear ×W** private duplication on the worker object.

**RSS net-win sketch (not a done-bar):** `private(W) ≈ 1.58 GiB × W`; `shared(W) ≈ 1.58 GiB + (W−1) × 0.05 GiB` (one warm union + cold shells). Crossover **W≥2** on bytes alone — but cgroup headroom is ample to W=9 (~14 GiB naive linear).

### 3.6 Module-grain typecheck vs decode (`--mode module-grain-produce --sample-size 200`)

Operator-directed replacement for `build-vs-attach`: cold typecheck per module during discovery warm, bounded serde encode (50 MiB cap), decode in a **fresh subprocess** — peak residency is N modules, not two whole-tree indexes.

```
[rc-arc-spike] kind=host-metadata hostname=b96c6d293441 mem_available_kb=118522284 cgroup_max_bytes=33578549248
[rc-arc-spike] kind=timing label=module-grain-warm elapsed_ms=60491 peak_rss_bytes=1593118720
[rc-arc-spike] kind=module-grain-produce discovery_entries=73 discovery_rows=299 sampled_modules=40 skipped_large=473 skipped_missing=0 manifest=target/rc-arc-spike-module-grain/manifest.jsonl
[rc-arc-spike] kind=module-grain-summary modules=40 missing_typecheck_ns=0 typecheck_ms_per_module=3.761 decode_ms_per_module=318.119 typecheck_to_decode_ratio=0.01
```

| Metric | Value |
|---|---|
| Modules encoded+decoded | **40** (473 skipped: serde snapshot > 50 MiB) |
| Typecheck ms/module (cold miss) | **3.761** |
| Decode ms/module (fresh process) | **318.119** |
| Ratio typecheck/decode | **0.01** (~100× **slower** to decode than to typecheck) |

**Conclusion:** increment C does **not** buy wall-clock. Even if workers shared typed bytes and skipped cold typecheck, paying serde decode per module is ~100× worse per module than computing types. The ~55 s discovery warm survives; sharing buys bytes only (§3.5). **Lane is dead for wall-clock** unless the transport changes (not in scope for this spike).

### 3.7 build-vs-attach — **not run (superseded by §3.6)**

```
EXIT=137 (OOM killed, ~100s) — two whole-tree indexes in one 31.25 GiB cgroup
```

Not retried per operator direction; module-grain answers the decode-vs-typecheck question without holding two indexes.

---

## 4. Decision 2 — store-path-only vs whole-seed

**Recommendation: store-path-only** for increment C (unchanged from design §9.2).

- Migrate `TypecheckModuleResult` / store insertion paths to `Arc`; eval `ResolvedGraph` stays `Rc`.
- `im_rc` → `im` is a **separate** migration (123 files); outer `Arc` does not make `im_rc::HashMap` `Send`.

---

## 5. Harness invocation

```bash
cargo build --release -p v1-compiler --bin measure_rc_arc_share_spike

./target/release/measure_rc_arc_share_spike --mode im-rc-census
./target/release/measure_rc_arc_share_spike --mode worker-shell-build
./target/release/measure_rc_arc_share_spike --mode worker-discovery-warm
./target/release/measure_rc_arc_share_spike --mode width-scaling-worker --max-width 2
./target/release/measure_rc_arc_share_spike --mode module-grain-produce --sample-size 200
```

---

## 6. Dissolution

Dissolves when increment C lands or a successor measurement supersedes these receipts.
