# Fill composition overlay direction — measured receipt

**Lane:** [floor-prep-tax-program](floor-prep-tax-program.md) §7 on-resume item 2 (assembly follow-on after the P1 retention REJECT)
**Predecessor (price surface, not mechanism):** [per-entry-assembly-decomposition-measurement](per-entry-assembly-decomposition-measurement.md)
**Retained evidence:** `receipts/fill-composition-overlay-direction/`
**Status:** one mechanism, one cohort, one verdict — measured, two reproductions per arm

## The defect

`symbol_index_with_qualified_fill` and `symbol_index_with_bare_fill` (`v1.compiler.infer`,
authority `src/v1/04_infer.dag`) compose an entry's closure census with a shared underlay:
the whole-pool qualified fill, and the whole-tree bare census for each source root the entry
reaches. Both denote one map — union of the two key sets, closure value wins on the
intersection.

Both walked the **underlay's** key list (`sorted_map_keys`), probed the accumulator once per
key, and path-copied an insert for every miss. The underlay is a per-`MultiEntryIndex` memo,
so it is constant while the closure varies: every entry paid O(|whole pool|) work and
retained O(|whole pool|) fresh HAMT nodes to build an index that differs only by its own
closure. Overlaying the closure onto the shared memo instead is the same denotation
(closure still wins) at O(|closure|), starting from an O(1) persistent clone.

This is DESIGN §6 bare-minimum-cost: a copied accumulator is fixed regardless of the
realized n. It is a cost fix and not a semantic change **because** the two directions are
identical — that identity is what the witness pins.

## Subject and arms

Both arms are built from **one tree with one variable**. The after arm is the commit named in
`subject.tsv`; the base arm is that same tree with `base-arm-revert.patch` applied, which
restores the two fill functions and nothing else. Binary digests for both are retained, so a
reader can rebuild either arm and check the hash rather than trust a table cell.

The invocation is **one process holding one `MultiEntryIndex`** — a single `claim_batch`
carrying 50 `--entry` groups, which is why `[resolve-summary]` reports 50 resolves and the
assembly totals are process-wide. It is *not* 50 invocations.

The roster (`cohort.tsv`) is the first 50 `dag/test/claim/*_test.dag` files in sorted order
that declare a zero-arg `-> Bool` `test fn`, one claim function each; identical in both arms.

Runs are sequential and interleaved (`base-r1, after-r1, base-r2, after-r2`) so host drift
lands on both arms. Peak memory is the kernel's `VmHWM`, not a sampled maximum.

Additive span totals for this harness, per the predecessor's decision boundary: **not**
elapsed fleet wall, and not quotable as an 842-group floor total.

## Result

Every figure below is derived by `derive_summary.py` from the four retained arm files alone
(`summary.json`); none is transcribed.

| Aggregate | base r1 / r2 | after r1 / r2 | delta |
|---|---:|---:|---:|
| Σ exclusive assembly (ms) | 100,773.3 / 83,980.4 | 41,011.9 / 36,017.0 | **−58.3%** |
| additive resolve, 50 resolves (ms) | 155,680 / 146,484 | 105,012 / 94,624 | **−33.9%** |
| elapsed wall (ms) | 268,369 / 247,963 | 200,932 / 183,286 | **−25.6%** |
| peak RSS (kB) | 8,160,560 / 8,108,424 | 5,804,768 / 5,803,012 | **−28.7%** |

| Exclusive assembly row | base r1 / r2 | after r1 / r2 | delta |
|---|---:|---:|---:|
| symbol-index merge | 34,654.4 / 28,420.1 | 624.6 / 585.4 | **−98.1%** |
| per-root symbol-index composition | 27,095.2 / 18,642.2 | 719.4 / 618.5 | **−97.1%** |
| import-string rewiring | 10,631.5 / 10,820.2 | 13,812.5 / 10,223.1 | see note |
| per-root variant base | 8,610.8 / 7,514.7 | 7,594.8 / 7,209.6 | −8.2% |
| closure symbol index | 7,832.0 / 7,676.4 | 7,693.2 / 7,491.8 | −2.1% |
| pool fill | 3,193.6 / 3,335.0 | 3,231.0 / 3,000.7 | −4.5% |
| emit info | 2,746.7 / 2,576.5 | 2,590.2 / 2,421.0 | −5.9% |
| services | 2,030.5 / 2,127.6 | 2,113.1 / 2,061.2 | +0.4% |

Remaining rows are each below 1.4 s in every run.

Two honesty notes on that table. **Import-string rewiring's +12% mean is one outlier, not a
regression**: its after runs are 13,812.5 and 10,223.1 ms, a 35% spread within one arm, while
its base runs agree at 10,631.5/10,820.2 — after-r2 sits inside the base range. The host was
loaded during this session, which also shows in the base arm's own assembly spread
(100.8 s vs 84.0 s). The robust statement is not any single row's mean but that **every
aggregate is lower in both after runs than in both base runs**, with no overlap.

**Peak RSS falls 28.7% on completed runs.** That is the retained-bytes half of the same
mechanism: the old shape path-copied one insert per underlay key per entry, and those nodes
stayed reachable from the composed index.

## Equivalence

* All four arms: 48 PASS / 2 FAIL with identical result sets. The two failures are
  pre-existing hermetic-mode effect refusals (`artifact_fs_roundtrip_holds`,
  `build_artifact_corruption_probe_holds`), unrelated to resolution and present in both arms.
* `regen_stage0 --verify` → `regen_divergence_count=0` with the seed rebuilt from the changed
  sources: the self-host fixed point is unchanged.
* `symbol_index_fill_overlay_direction_test` (`v1-compiler-tests`) is the discriminating
  control on the direction clause. Executed both ways: green as landed; with the merge
  direction flipped in the seed, both arms red on the winner assertion
  (`left: "probe.underlay" right: "probe.closure"`) while still producing the right key set —
  so the test measures the direction, not the union.

## Unpaired observation: the discovery path

The ordinary discovery/entry-preparation path was also run on both arms, and is reported
here only as an observation because **both runs were OOM-killed by the kernel** (exit 137):
`claim_batch --roster-from-discovery` does not restrict discovery to the supplied `--entry`
rows — it scans the default roster and appends them — so this is a whole-corpus run, the
known claim_batch OOM class, not a bounded 50-entry cohort.

| | wall | peak RSS at kill | resolves reached | witnesses reached |
|---|---:|---:|---:|---:|
| base | 24m28s | 14.4 GB | 82 | 661 |
| after | 25m00s | 15.7 GB | 155 | 1,262 |

The after arm reached 1.85× the resolves in the same wall before hitting the same ceiling.
The peak column is **where each run was killed**, not a completed workload's peak, so it does
not support a memory claim in either direction; the completed-cohort figure above does.
A bounded selection-driven measurement needs `claim_executor` with the floor plan, not
`claim_batch`.

## Bound on the claim

This is one slice of the shared-entry-view hypothesis. The composition rows collapse; the
rows that remain are different defects and are untouched here:

| Row | after (mean) | Why it survives |
|---|---:|---|
| import-string rewiring | ~11,000 | per-module fold over inherited binding keys — not a merge direction |
| closure symbol index | ~7,600 | the entry's own census build; needs module-grain reuse |
| per-root variant base | ~7,400 | recomputed per (entry, root) from the composed `global_bare` |

Fleet recovery is **not** claimed from these numbers. The additive-vs-elapsed denominator
rule from the predecessor holds: this receipt licenses "the composition tax is gone from the
assembly surface, and the cohort's wall and peak memory fall with it", not a required-path
floor figure. That figure needs a paired affected-floor run on the real runner.
