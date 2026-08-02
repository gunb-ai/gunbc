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
symbol-index construction, pool fill, symbol-index merge, closure variant-base construction,
per-root symbol-index composition, per-root variant-base construction, environment
installation, diagnostics, registry merge, services expansion, three separate rewire
passes, emit-info construction, and derived `other`. The accounting refuses
`OverAttributed`, `NestedSpanAttribution`, and `NoSpans`; it never clamps a negative
remainder.

## Grain and downstream displacement

| Row | Grain | Native selected-witness bundle | Module-grain materialization |
|---|---|---|---|
| `schedule`, `graph`, `diagnostics`, `registry`, `emit_info`, `other` | per entry/closure view | disappears when the native bundle supplies the selected assembled view | remains unless the assembled view itself is reused |
| `probe`, `environment` | per module membership | disappears with a complete native bundle | lookup/install remains; module compute does not |
| `symbol_index`, `symbol_index_merge`, `variant_base` | per entry with misses | disappears with a complete native bundle | disappears only if the reusable module-grain artifact also carries the required environment/index facts |
| `root_symbol_index`, `root_variant_base` | per source-tree root first used by an entry | disappears with a complete native bundle | disappears only when reuse carries the root-composed index and variant-owner facts |
| `pool_fill` | per index (later calls are memo hits) | amortized or absent when bundle loading bypasses the seed resolver | amortized once per shared index |
| `services`, three `rewire_*` passes | per module/item in the assembled closure | disappears with a fully assembled native bundle | remains if cached modules must still be rewired into a fresh entry view |

These names are prices, not mechanism endorsements. In particular, a large per-entry
symbol-index merge does not by itself choose either bundle construction or module reuse.

## Representative 50-entry receipt

The roster is the exact 50 entries from the post-merge representative slice-2 cell, plus
the existing `output_policy` machinery span opened by `claim_batch` (51 top-level spans in
the additive receipt). Both corrected runs used the same locally built binary, SHA-256
`5d5815ccc0cc6969f95c5b4d3a76f3ba4166555a304a050d895774e19608d340`.

| Exclusive assembly row | run 1 | run 2 | assembly share range |
|---|---:|---:|---:|
| symbol-index merge | 25,170.2 ms | 21,935.4 ms | 29.17–30.11% |
| per-root symbol-index composition | 14,974.5 ms | 13,661.6 ms | 17.92–18.16% |
| import-string rewiring | 15,045.2 ms | 13,103.0 ms | 17.42–18.00% |
| closure symbol index | 9,712.7 ms | 9,114.2 ms | 11.62–12.12% |
| per-root variant base | 6,440.7 ms | 5,890.2 ms | 7.71–7.83% |
| emit info | 3,327.2 ms | 3,083.1 ms | 3.98–4.10% |
| pool fill | 2,669.8 ms | 2,735.9 ms | 3.19–3.64% |
| services | 2,701.9 ms | 2,500.4 ms | 3.23–3.32% |
| type-env rewiring | 1,311.7 ms | 1,215.9 ms | 1.57–1.62% |
| closure variant base | 960.1 ms | 819.3 ms | 1.09–1.15% |
| environment installation | 528.6 ms | 469.6 ms | 0.62–0.63% |
| derived `other` | 267.9 ms | 243.3 ms | 0.32–0.32% |

The remaining exclusive rows are each below 0.5% in both runs. Total measured assembly is
83,585.9 ms and 75,210.3 ms respectively. True `typecheck_compute`, whose timer now starts
immediately before `typecheck_module`, is 21,593.3 ms and 20,747.6 ms. These are additive
span totals for this harness, not elapsed wall and not the fleet's 848-group whole-floor
total. Every share above appears
in both complete reproductions; no single-run share is a decision input.

Receipts:

- `receipts/per-entry-assembly-decomposition/representative-50-r1.txt`
- `receipts/per-entry-assembly-decomposition/representative-50-r2.txt`

## Retractions and decision boundary

The first finer split temporarily left symbol-index merge in `other` (~27.3 s). That
intermediate reading was retracted before decision use, the merge received its own row,
and both receipts were rerun.

Review 46763 then found that the per-module `typecheck_compute` timer began before lazy
per-root symbol-index composition and per-root variant-base construction. The earlier
receipts and their shares are therefore retracted: they classified root preparation as
typechecking and could not support the documented grain/displacement interpretation. The
timer now wraps only `typecheck_module`; root preparation has two exclusive rows; and both
50-entry receipts above were regenerated. The corrected final `other` is 0.32% in both
runs.

This lane measures the price surface only. It does not begin union construction, select a
bundle mechanism, or claim fleet recovery from additive harness shares.
