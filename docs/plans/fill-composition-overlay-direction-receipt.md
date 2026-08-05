# Fill composition overlay direction — measured receipt

**Lane:** [floor-prep-tax-program](floor-prep-tax-program.md) §7 on-resume item 2 (assembly follow-on after the P1 retention REJECT)
**Predecessor (price surface, not mechanism):** [per-entry-assembly-decomposition-measurement](per-entry-assembly-decomposition-measurement.md)
**Status:** one mechanism, one cohort, one verdict — measured, reproduced twice per arm

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

## Cohort

50 entries, one `test fn` claim each, `dag/test/claim/*_test.dag` in sorted order (the first
50 files declaring a zero-arg `-> Bool` `test fn`). Roster and both arms' raw output are the
receipt's inputs; the cohort is fixed across arms and the two arms differ only in the seed.

```sh
claim_batch --source-root dag --source-root src/v2 \
  --entry <path> --functions <one claim fn>   # x50
```

Additive span totals for this harness, per the predecessor's decision boundary: **not**
elapsed fleet wall, and not quotable as an 842-group floor total.

## Result (ms, two complete reproductions per arm)

| Exclusive assembly row | base r1 | base r2 | after r1 | after r2 | delta |
|---|---:|---:|---:|---:|---:|
| symbol-index merge | 25,517.7 | 26,279.1 | 530.6 | 538.8 | **−97.9%** |
| per-root symbol-index composition | 16,734.2 | 17,063.9 | 531.7 | 491.4 | **−97.0%** |
| closure variant base | 716.3 | 741.4 | 377.4 | 392.2 | −47.2% |
| import-string rewiring | 9,317.1 | 9,695.9 | 10,372.1 | 9,411.5 | +4.1% |
| closure symbol index | 6,814.4 | 6,900.7 | 7,037.0 | 7,028.8 | +2.6% |
| per-root variant base | 6,813.9 | 6,778.9 | 6,717.8 | 6,664.4 | −1.5% |
| pool fill | 2,932.2 | 2,951.7 | 3,029.6 | 2,965.9 | +1.9% |
| emit info | 2,284.4 | 2,326.3 | 2,299.7 | 2,320.5 | +0.2% |
| services | 1,931.9 | 1,943.5 | 1,954.2 | 2,042.7 | +3.1% |
| type-env rewiring | 930.3 | 961.7 | 1,119.1 | 931.1 | +8.4% |

Remaining rows are each below 400 ms in every run.

| Aggregate | base | after | delta |
|---|---:|---:|---:|
| Σ exclusive assembly | 74,876.0 / 76,535.5 | 34,856.8 / 33,671.5 | **−54.7%** |
| additive resolve (50 resolves) | 129,779 / 132,623 | 92,332 / 90,940 | **−30.2%** |
| cohort elapsed wall | 218,792 / 221,732 | 179,477 / 178,409 | −18.8% |

Only the two composition rows move materially. Every other row is inside run-to-run spread,
which is the control that the change is confined to the composition step: `rewire_import_str`
and `symbol_index` are larger in one after-run and smaller in the other.

The closure variant-base row halves without being touched. That is **observed, not claimed** —
it is a 700 ms row, its two after-runs bracket the drop, and nothing in the change alters
`build_global_bare_variant_locals` or its input. It is not counted as part of the mechanism.

## Equivalence

* Both arms: 48 PASS / 2 FAIL, and the sorted result sets are byte-identical (`diff` clean,
  base r1↔after r1 and base r2↔after r2). The two failures are pre-existing hermetic-mode
  effect refusals (`artifact_fs_roundtrip_holds`, `build_artifact_corruption_probe_holds`),
  unrelated to resolution and identical in both arms.
* `regen_stage0 --verify` → `regen_divergence_count=0` with the seed rebuilt from the changed
  sources: the self-host fixed point is unchanged.
* `symbol_index_fill_overlay_direction_test` (`v1-compiler-tests`) is the discriminating
  control on the direction clause. Executed both ways: green as landed; with the merge
  direction flipped in the seed, both arms red on the winner assertion
  (`left: "probe.underlay" right: "probe.closure"`) while still producing the right key set —
  so the test measures the direction, not the union.

## Bound on the claim

This is one slice of the shared-entry-view hypothesis, not the whole of it. The composition
rows collapse; the rows that remain are different defects and are untouched here:

| Row | after (mean) | Why it survives |
|---|---:|---|
| import-string rewiring | 9,891.8 | per-module fold over inherited binding keys — not a merge direction |
| closure symbol index | 7,032.9 | the entry's own census build; needs module-grain reuse |
| per-root variant base | 6,691.1 | recomputed per (entry, root) from the composed `global_bare` |
| pool fill | 2,997.8 | per-index memo; already amortized |

The review bar for the shared-entry-view PR is assembly wall down ≥60% on this cohort. One
mechanism delivers **54.7%**. The remainder is in those four rows, and reaching the bar
requires the module-grain reuse this receipt does not attempt.

Fleet recovery is **not** claimed from these numbers. The additive-vs-elapsed denominator
rule from the predecessor holds: this receipt licenses "the composition tax is gone from the
assembly surface", not a floor wall figure.

## Host

One host, one locally built binary per arm, arms run back to back and interleaved with
nothing else. Elapsed wall is reported for completeness and is the noisiest column.
