# Floor memory 3× regression — diagnosis (bisected to #6848)

**Status:** diagnosis complete. Resolver read-only throughout — this document prices a
known-shape interim cost (namespace-resolution-design.md §PR-5b: "the reference producer
over-collects; the closure is a second authority"); it does **not** land a fix. The fix is
owned by the namespace lane (see *Fix axes*).

**One-line verdict:** #6848 made every compile pay a **whole-pool, full-body parse**
(`pool_parse` over all ~2255 `.dag` modules) *unconditionally*, adding a fixed
~800 MiB–1 GB per-process baseline that is independent of the entry's closure. The floor
runs each witness as a cold process, so peak ≈ `width × ~1 GB`. It is **not** closure
growth and **not** reconcile-env superlinearity, and it does **not** scale with the import
strip — it scales with **total pool size** (a repo-size wall).

---

## 1. Mandate and what this answers

- **Q1 — mechanism, with a receipt.** What holds the extra ~13 GB on the floor? → §4–§5.
- **Q2 — does the import strip make it worse?** (#6937 = 464 files src/v2/test+lens;
  #6938 = 368 files dag/extdeps+tools). Time-boxed by those PRs. → §6.
- **Q3 — retention split:** is the ~1 GB parse baseline disjoint from the typed cache? → §8.

## 2. Environment (record box + budget beside every number)

- Measurement box: **128 cores, cgroup `memory.max` = 33.5 GB (~31 GiB)**.
- CI receipt that opened the investigation (heterogeneous runners, 16.1–124 GB, ~8×):
  - run 29763408563 `ec22e8fab` (last green): budget 123.7 GB, **peak floor RSS 6.5 GB**, wall 22.2 min
  - run 29763713657 `c87d1b0d3` (#6848): budget 120.0 GB, **peak floor RSS 20.0 GB**, wall 32.5 min
  - sustained since: 17.6 / 16.2 / 21.6 / 16.1 / 18.1 GB. On a 16.1 GB runner (run 29770120178)
    it pins at exactly memory.high and thrashes (40.2 min) — reclaim, not compute.
- Pre-#6848 footprint was ~7 GB **independent of a 16.1–123.7 GB budget range**, so the
  regression is real demand, not a box artifact.

## 3. Commit identities (the arms)

| arm | commit | note |
|---|---|---|
| parent | `ec22e8fabb` | **direct parent** of #6848 (`git rev-parse c87d1b0d33^`) |
| main | `be9631e7c8` | #6848 + #6929 (im-rc→im) + follow-ups |
| #6937 | `7cbeaf1195` | calm-deer-217, "strip src/v2/test+lens"; **CI-RED** (patterns.dag:47 resolution defect — a confound, recorded) |
| #6938 | `6d773b9d74` | bright-koi-545, "strip dag/extdeps+tools" |

## 4. Mechanism (the code path)

Every compile runs `reconcile_with_typed_cache` (`src/v1/stage0/src/cli_run.rs`). At the top
of that function, **before** any cache-hit shortcut:

- `build_symbol_index_for_reconcile(index, …)` is called at **cli_run.rs:6023**
  (function at :5993), **ahead of** the `try_reconcile_all_cache_hits(…)` check at
  **cli_run.rs:6042**. So even an all-cache-hits entry pays it. *(Line numbers on the
  measured binary `be9631e7c8`; nimble-owl-658 cites ~6055/~6074 on a later checkout — line
  numbers drift, the **call order** is the invariant.)*
- It calls `pool_qualified_fill` → **`pool_parse`** (cli_run.rs:5848), which parses **every
  module in the pool** — all keys of `index.source_files`, ~2255 `.dag` files — into
  **full-body `Rc<Node>` ASTs**, retained in `pool.nodes_by_file` on the index for the
  process lifetime.

The memo (`index.pool_parse` `RefCell`) **works** — it is built once per process — but it is
(a) **whole-pool-scoped**, not closure-scoped, and (b) **per-process**, so it cannot span
floor workers. This is an *unconditional whole-pool call*, not a missing/ineffective memo.

**Heads-only cause (attributed to nimble-owl-658):** the sole consumer,
`build_symbol_index_qualified_fill`, folds over `module_items` / `local_binding_for_item` —
**declaration heads only**, never recursing into bodies. So every function body of every pool
module is parsed and retained to extract names and signatures. Materializing the maximum to
use the minimum. The ~1 GB is that.

Pre-#6848 the loader followed **declared import edges** (`resolve_transitively`) and reconcile
indexed only the entry's import closure; there was no whole-pool parse.

## 5. Receipt 1 — controlled per-entry table

Per-entry cold run (`gunbc run --claim-run --entry <e> --function main --source-root src/v2
--source-root dag`); the run prints `resolved N sources` (closure size) and
`[gantt] compile.reconcile.done rss_mib`; peak RSS via `wait4`/`ru_maxrss`. Same entry each
row, so the closure content is held fixed and only the pool/commit differs.

| entry | sources P/M/#6937/#6938 | **peak MiB** P / M / #6937 / #6938 |
|---|---|---|
| complexity_accumulator_copy_test | 44 / 46 / 47 / 46 | 143 / 1079 / 1070 / 1080 |
| dependency_fidelity_test | 51 / 55 / 55 / 55 | 170 / 1081 / 1072 / 1077 |
| doc_reachability_test | 13 / 13 / 13 / 13 | **52 / 1079 / 1070 / 1077** |
| inert_carrier_test | 19 / 19 / 17 / 19 | 51 / 1080 / 1070 / 1078 |
| visibility_test | 22 / 17 / 17 / 17 | 50 / 1080 / 1070 / 1078 |
| live_read_classification_test | 97 / 97 / 42 / 100 | 261 / 1079 / 1071 / 1079 |
| affected_set_universe_test | 260 / 260 / 260 / 262 | 369 / 1079 / 1071 / 1079 |
| bash_program_fold_test | 64 / 64 / 64 / 76 | 217 / 1078 / 1072 / 1078 |
| auth_declared_but_unwired_witness_test | 34 / 34 / 34 / 39 | 53 / 1078 / 1071 / 1078 |
| body_lowering_well_formed_wall_test | 8 / 8 / 8 / 8 | 50 / 1079 / 1070 / 1078 |

**Isolation receipt:** `live_read_classification_test` — **identical 97-source closure** at
parent and main — went **258 → 1077 MiB (4.2×)** with the closure unchanged.
`doc_reachability_test` (13 sources) went **52 → 1079 MiB (20×)**: the pool-parse floor
completely swamps a tiny closure. On the **parent**, peak RSS tracks closure size
(50 → 369 MiB); on **main** it is pinned at the ~1 GB pool floor regardless of closure.

## 6. Q2 — the strip is memory-neutral (with exercised-path proof)

**Verdict:** #6937 and #6938 do **not** materially move per-entry peak RSS. main→#6938 is
flat (<1%); main→#6937 is flat-to-slightly-lower.

**Why the null is predicted, not merely observed:** peak = *pool-parse floor* (a function of
**total pool size** — every file is parsed regardless of whether its imports are stripped)
**+** closure cost, and the pool floor (~1 GB) ≫ closure cost. Stripping imports removes
**no** pool files, so the floor is unchanged.

**Exercised-path proof** (pre-empts "your entries never pulled the stripped files"): the
strip demonstrably *did* change closures in the measured set —
- #6937: `live_read_classification` **97 → 42** sources (files *dropped* — this is
  calm-deer's resolution-defect direction, not a memory win)
- #6938: `live_read` **97 → 100**, `bash_program_fold` **64 → 76**, `auth_declared` **34 → 39**
  (providers pulled *wider* — directionally confirms the #6938-widens-more hypothesis)

…yet peak RSS stayed pinned at ~1078 MiB across all arms. **Closure moved up to 2.3×; RSS did
not move.** The mechanism was exercised and the null still holds.

## 7. The slope — a repo-size scaling wall

Fixed entry (`doc_reachability_test`, closure pinned at 13), varying the pool via source-root
scope:

| pool roots | pool files | peak MiB |
|---|---|---|
| src/v2 | 1185 | 675 |
| src/v2 + dag/std | 1281 | 719 |
| src/v2 + dag/std + dag/extdeps | 1584 | 803 |
| src/v2 + dag (full) | 2255 | 1080 |

**RSS ≈ 225 + 0.38 MiB × (pool modules), linear.** Every `.dag` file added costs ~0.38 MiB
on **every entry in every process, forever** — strip or no strip. Today it is 3×; it grows
with ordinary development even if nobody strips another file. This turns the incident into a
**roadmap item**.

## 8. Q3 — parse baseline vs typed cache are near-disjoint

Independent read (the manager disclosed a COI here; this is the evidence, not the preference):

- **Structural:** `pool_parse` parses via `parse_module_node_from_index_source`, which
  reads/writes `index.parse_cache` keyed by path (cli_run.rs:5791/5807). The closure resolve
  reads the **same** `parse_cache` (cli_run.rs:5264), so closure modules **share** pool_parse's
  `Rc<Node>` rather than reparsing. `TypedModule` then holds `module: resolved.module.clone()`
  — an **Rc alias**, not a copy (`v1_compiler_infer.rs:16388`, bound by cool-wren-467).
- **Empirical corroboration:** on main, typechecking a **260-module** closure
  (`affected_set_universe`) adds only **+1 MiB** over the pool floor (`frontend.begin`
  rss_mib=1081 → `reconcile.done` 1082), whereas the *same* closure costs ~317 MiB on the
  parent (no pool). The closure typecheck adds ~0 on main **because those ASTs are already
  resident in the pool** — the RSS-side face of the same sharing.

**Conclusion:** the overlap between pool-parse memory and typed-cache memory is *exactly the
entry closure* (aliased); the **non-closure bulk is held solely by `pool.nodes_by_file`**.
From the slope, non-closure ≈ `(2255 − N) × 0.38 MiB` dominates — ~99% for a small closure,
still ~88% at N = 260. The two costs are **near-disjoint**, so a heads-only parse shrink and a
cross-worker typed-cache share target largely independent memory.

## 9. Fix axes (named, not owned — namespace lane territory)

The primary cost is the whole-pool full-body parse. Two complementary axes:

- **(2a) heads-only parse.** `pool_parse`'s consumer needs declaration heads, not bodies;
  parse/retain heads for the pool and full bodies only for the entry closure. Shrinks the
  ~1 GB at the source. **Primary fix** — the floor runs cold processes where every entry pays
  the full baseline.
- **(2b) laziness / call-order.** Build the qualified fill only on a genuine resolution miss,
  and after (not before) `try_reconcile_all_cache_hits`. This helps the **warm single-process**
  case (an all-hits entry currently still pays the full 1 GB). It does **not** help the floor
  (cold process ⇒ everything is a miss), so it is complementary to (2a), not a substitute.
  *(Surfaced by pinning the call order at §4 — the fill is built before the cache-hit check.)*
- **Dedup framing (superseded by 2a, kept for the record):** `pool_parse` sorts paths
  deterministically and parses an identical file set in every process ⇒ byte-identical ASTs
  across all W floor workers ⇒ `(W−1) × ~1 GB` reclaimable by sharing. nimble-owl's (2a)
  shrinks the thing at the source rather than sharing it — strictly better.

This is the interim-bridge cost tracked by namespace-resolution-design.md §PR-5b; it dissolves
when name resolution stops needing a corpus-wide fill (namespace-only resolution, step 5).

## 10. Reproduction

Build the `gunbc` bin per arm in its own worktree (`cargo build --release --bin gunbc`), then
run the cold single-entry harness below from each worktree so `--source-root` resolves against
that arm's corpus. It captures `resolved N sources` (closure size),
`[gantt] compile.reconcile.done rss_mib`, and per-child peak RSS via `os.wait4`. The slope (§7)
uses the same launch varying the `--source-root` set.

```python
import subprocess, sys, re, os
# usage: measure.py <workdir> <binary> <entry1.dag> [entry2.dag ...]
workdir, binary, entries = sys.argv[1], sys.argv[2], sys.argv[3:]
print("entry,sources,reconcile_rss_mib,max_gantt_rss_mib,peak_rss_mib,ok")
for e in entries:
    p = subprocess.Popen([binary, "run", "--claim-run", "--entry", e, "--function", "main",
                          "--source-root", "src/v2", "--source-root", "dag"],
                         cwd=workdir, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    out = p.stdout.read()
    _, _, ru = os.wait4(p.pid, 0)          # per-child high-water, KB on Linux
    srcs = (re.search(r"resolved (\d+) sources", out) or [None, "NA"])[1]
    rec  = re.findall(r"compile\.reconcile\.done .*rss_mib=(\d+)", out)
    gantt = [int(x) for x in re.findall(r"\[gantt\].*rss_mib=(\d+)", out)]
    print(f"{os.path.basename(e)},{srcs},{rec[-1] if rec else 'NA'},"
          f"{max(gantt) if gantt else 'NA'},{ru.ru_maxrss // 1024},{'1' if srcs!='NA' else '0'}")
    sys.stdout.flush()
```

The function-arg error (`main` takes no such witness fn) is harmless — it occurs *after*
resolve + reconcile, so the instrument lines are already emitted.

## 11. Provenance

- Mechanism, per-entry receipt, Q2 exercised-path proof, slope, unconditional-call location,
  and the parse↔typed-cache disjointness read: this session (eager-pike-178), by execution.
- Heads-only cause and the fix-axis framing: nimble-owl-658.
- TypedModule Rc-alias bound: cool-wren-467.
- Coordination and CI receipt: sunny-wolf-225.

Related: [v1 run-stability throughline](v1-run-stability-throughline.md) ·
[floor shared-compute memoization](floor-shared-compute-memoization.md) ·
[v2 memory-control audit](v2-memory-control-audit.md) · namespace-resolution-design.md §PR-5b.
