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
| **Is width worth it for wall-clock?** | **Not answered by §3.6** — that section prices the **interim serde byte transport** (`SharedTypecheckCaches`), not in-memory `Arc` sharing (which would hold handles with **no decode**). PR #6905 Amdahl bound still the wall-clock context. Serde transport receipt: ~318 ms/module decode on a 7.8% biased sample (§3.6). |
| **store-path-only vs whole-seed?** | **store-path-only** for increment C (design §9.2 default). `im_rc` collections stay `!Send` without `im-rc`→`im`. |
| **im-rc price?** | **123 / 154** stage0 `.rs` files reference `im_rc` (dynamic census). `lib.rs` aliases `HashMap`/`Vector`/`BTreeSet` to it. |

**What is NOT established:** in-memory `Arc<TypecheckModuleResult>` share wall-clock (not measured — Arc share has no serde decode). Serde-based byte crossover (discarded — see §2.2). Whole-tree strict resolve to the ~10.7 GiB nimble-owl/srv1 receipt (not re-run here). `build-vs-attach` was **not retried** (OOM by construction on 31 GiB — two whole-tree indexes); module-grain (§3.6) prices serde transport at module residency instead.

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
4. **`build-vs-attach` not run** — holds two whole-tree indexes in one process; OOM on 31 GiB cgroup by construction. Module-grain (§3.6) prices serde transport at module residency instead.
5. **Module-grain measures serde transport, not Arc share** — `SharedTypecheckCaches` is the interim serde-byte scaffold (`shared_typecheck_store.rs` header; dissolve-on: store-path `Rc`→`Arc`). Real Arc sharing would pass in-memory handles — **no decode**. §3.6 prices the workaround that exists because `Rc` is `!Send`.
6. **Module-grain sample bias** — serde snapshots capped at 50 MiB; only **40 / 513** cold-miss modules fit (**7.8%**; 473 skipped are the large ones). Snapshots near 50 MiB imply `encode_typed_snapshot` serializes each module with its transitive closure — shared context re-encoded per module (transport redundancy, not a property of sharing).
7. **Module-grain timer mismatch (open)** — same run: `module-grain-warm` **60 491 ms** over 513 cold misses ≈ **118 ms/module** warm cost, but `typecheck_ms_per_module=3.761` from the per-miss hook. **31× discrepancy within one run** — the hook timer does not capture full produce cost; **do not treat `typecheck_to_decode_ratio=0.01` as a decisive wall-clock ratio** (also mixes inner-loop timing vs fresh-subprocess decode with startup/load overhead).

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

### 3.6 Module-grain **serde transport** cost (`--mode module-grain-produce --sample-size 200`)

**What this measures:** the interim **serde byte transport** in `SharedTypecheckCaches` — the scaffold that exists because `Rc` is `!Send` (`shared_typecheck_store.rs`: "interim serde byte transport for cross-worker share only"; dissolve-on: store-path `Rc`→`Arc`). **Not** in-memory `Arc<TypecheckModuleResult>` sharing (which would hold handles with no decode).

Operator-directed replacement for `build-vs-attach`: cold typecheck per module during discovery warm, bounded serde encode (50 MiB cap), decode in a **fresh subprocess** — peak residency is N modules, not two whole-tree indexes.

```
[rc-arc-spike] kind=host-metadata hostname=b96c6d293441 mem_available_kb=118522284 cgroup_max_bytes=33578549248
[rc-arc-spike] kind=timing label=module-grain-warm elapsed_ms=60491 peak_rss_bytes=1593118720
[rc-arc-spike] kind=module-grain-produce discovery_entries=73 discovery_rows=299 sampled_modules=40 skipped_large=473 skipped_missing=0 manifest=target/rc-arc-spike-module-grain/manifest.jsonl
[rc-arc-spike] kind=module-grain-summary modules=40 missing_typecheck_ns=0 typecheck_ms_per_module=3.761 decode_ms_per_module=318.119 typecheck_to_decode_ratio=0.01
```

| Metric | Value |
|---|---|
| Modules encoded+decoded | **40** (**7.8%** of 513 cold misses; 473 skipped: serde snapshot > 50 MiB) |
| Warm ms/module (receipt) | **~118** (60 491 ms ÷ 513 cold misses) |
| Hook `typecheck_ms_per_module` | **3.761** (inner per-miss timer — **31× below** warm receipt; discrepancy **unresolved**) |
| Decode ms/module (fresh subprocess) | **318.119** (includes process start + snapshot load) |
| Harness `typecheck_to_decode_ratio` | **0.01** (3.761 ÷ 318.119 — **not comparable bases**; see §2.2 item 7) |

**What is established:** the serde byte transport is expensive (~318 ms/module decode on this sample) and structurally redundant (per-module snapshots re-encode transitive closure). This is evidence **against** shipping increment C on the serde transport, and **for** real `Arc` handles (the migration this scaffold exists to enable).

**What is not established:** whether in-memory Arc sharing removes the ~55 s discovery warm — that path was not measured here. A naive same-run sketch (warm ~118 ms/module vs decode ~318 ms/module) is ~2.7×, but the two timers are on **incomparable bases** (§2.2 item 7); it is not reported as a done-bar ratio.

### 3.7 build-vs-attach — **not run (superseded by §3.6)**

```
EXIT=137 (OOM killed, ~100s) — two whole-tree indexes in one 31.25 GiB cgroup
```

Not retried per operator direction; §3.6 prices serde transport at module residency without holding two indexes.

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
