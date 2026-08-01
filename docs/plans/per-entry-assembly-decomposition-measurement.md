# Per-entry assembly decomposition measurement

**Lane:** five-minute-ci-gate / #7597  
**Predecessor:** [entry-graph-union slice 2](entry-graph-union-slice2-typecheck-attribution.md)  
**Status:** measurement complete; no mechanism selected by this change

## Question and denominator

This is a new hypothesis, not an extension of the rejected shared-typecheck hypothesis.
It partitions the assembly work inside each top-level resolve span. The additive basis is
the sum of those spans; elapsed wall and the inclusive `rewire_total_observation` are
observations only and are never quoted as shares.

The exclusive rows are scheduling, cache probing, graph-view construction, closure
symbol-index construction, pool fill, symbol-index merge, variant-base construction,
environment installation, diagnostics, registry merge, services expansion, three separate
rewire passes, emit-info construction, and derived `other`. The accounting refuses
`OverAttributed`, `NestedSpanAttribution`, and `NoSpans`; it never clamps a negative
remainder.

## Grain and downstream displacement

| Row | Grain | Native selected-witness bundle | Module-grain materialization |
|---|---|---|---|
| `schedule`, `graph`, `diagnostics`, `registry`, `emit_info`, `other` | per entry/closure view | disappears when the native bundle supplies the selected assembled view | remains unless the assembled view itself is reused |
| `probe`, `environment` | per module membership | disappears with a complete native bundle | lookup/install remains; module compute does not |
| `symbol_index`, `symbol_index_merge`, `variant_base` | per entry with misses | disappears with a complete native bundle | disappears only if the reusable module-grain artifact also carries the required environment/index facts |
| `pool_fill` | per index (later calls are memo hits) | amortized or absent when bundle loading bypasses the seed resolver | amortized once per shared index |
| `services`, three `rewire_*` passes | per module/item in the assembled closure | disappears with a fully assembled native bundle | remains if cached modules must still be rewired into a fresh entry view |

These names are prices, not mechanism endorsements. In particular, a large per-entry
symbol-index merge does not by itself choose either bundle construction or module reuse.

## Representative 50-entry receipt

The roster is the exact 50 entries from the post-merge representative slice-2 cell, plus
the existing `output_policy` machinery span opened by `claim_batch` (51 top-level spans in
the additive receipt). Both runs used the same locally built binary, SHA-256
`fbd661fdd71b9ad13b6ccc50c87d86bf46172e42e1998e5a4e26d8e67aea3aa2`.

| Exclusive assembly row | run 1 | run 2 | assembly share range |
|---|---:|---:|---:|
| symbol-index merge | 27,649.9 ms | 23,385.7 ms | 38.91–40.00% |
| import-string rewiring | 17,978.4 ms | 13,956.1 ms | 23.87–25.30% |
| closure symbol index | 11,806.5 ms | 9,523.7 ms | 16.29–16.62% |
| emit info | 4,195.4 ms | 3,150.4 ms | 5.39–5.90% |
| pool fill | 2,843.4 ms | 2,677.6 ms | 4.00–4.58% |
| services | 2,523.8 ms | 2,422.4 ms | 3.55–4.14% |
| type-env rewiring | 1,890.2 ms | 1,335.6 ms | 2.28–2.66% |
| variant base | 826.6 ms | 789.4 ms | 1.16–1.35% |
| environment installation | 487.7 ms | 462.8 ms | 0.69–0.79% |
| derived `other` | 291.3 ms | 241.0 ms | 0.41–0.41% |

The remaining exclusive rows are each below 0.5% in both runs. Total measured assembly is
71,056.7 ms and 58,467.5 ms respectively. These are additive span totals for this harness,
not elapsed wall and not the fleet's 848-group whole-floor total. Every share above appears
in both complete reproductions; no single-run share is a decision input.

Receipts:

- `receipts/per-entry-assembly-decomposition/representative-50-r1.txt`
- `receipts/per-entry-assembly-decomposition/representative-50-r2.txt`

## Retractions and decision boundary

The first finer split temporarily left symbol-index merge in `other` (~27.3 s). That
intermediate reading was retracted before decision use, the merge received its own row,
and both final receipts were rerun. The final `other` is 0.41% in both runs.

This lane measures the price surface only. It does not begin union construction, select a
bundle mechanism, or claim fleet recovery from additive harness shares.
