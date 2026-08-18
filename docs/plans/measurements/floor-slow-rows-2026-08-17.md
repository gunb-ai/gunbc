# Floor slow-row measurement, 2026-08-17

Raw per-row timings from four `witnesses` workflow runs, one row per
`[floor-witness-slow]` line, carrying the run it came from.

`floor-slow-rows-2026-08-17.tsv` columns:
`source_log`, `subject`, `planned`, `module`, `function`, `max_ms`.

## Read the `planned` column before using a row

The four runs are NOT the same subject, and merging them silently is an error
this table exists to prevent.

| log | subject | planned | shape |
|---|---|---|---|
| ci   | 109d8d966ce221be | 10114 | pre-decline |
| ci2  | 3c8646a744c007d8 | 10114 | pre-decline |
| ci4  | unrecorded (truncated) | unrecorded | partial |
| ci5  | a5d1c49d9a802c3a | 9363 | current shape |

`planned=10114` predates the `reads_live_tree` decline reaching these sites.
Rows from those runs include witnesses the current fold declines and never
executes, so their time is not fold time. Only `ci5` describes the fold as it
runs today.

## What the current-shape run says

846 slow rows, 1224.5s. Concentration by module:

```
526.1s  test.claim.resolution_divergence_silent_pick_gate_witness_test
 97.3s  test.claim.extdeps_scope_placement_gate_loudness_witness
 83.8s  test.claim.rust_test_fixtures_import_closure_witness
 64.1s  test.claim.decl_facts_initializer_projection_witness_test
 63.0s  test.claim.dissolution_census_witness_test
 37.8s  test.claim.decl_facts_reflection_witness_test
 29.4s  v2.test.manual.grounding_lens_whole_tree
```

One row, `resolution_divergence_silent_pick_gate_keystone_holds`, is 526.1s —
43% of all slow time, and 4.6x the next module. It builds a second strict
whole-corpus resolve inside itself at `ResolvedGraphMemoShare::Ephemeral`.

## Retracted from an earlier reading of this data

A merge of all four logs produced a "second cause" family
(`enforcement_inventory_witness`, `enforcement_live_witness`,
`grammar_coverage_witness`, `cost_coverage_witness`,
`lifecycle_survivor_corpus_census`) totalling 652.1s. Those five modules all
declare `ReadsLiveTree`, `scan_site` declines such sites, and they appear
**zero** times in `ci5`. The family was an artifact of merging pre-decline runs
into a current-shape denominator. The 652.1s is real measured time, but it is
future load if those obligations are ever executed, not present fold cost.
