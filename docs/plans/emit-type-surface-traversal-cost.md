# Emit type-surface traversal: cost specimen

Measured 2026-08-29 on this repository at `dda42363639` (main, immediately after
gunbc#9666), single host, arm64, `--release`. **Instrumentation was a throwaway patch to
the emitted mirror; nothing here was landed, and no instrument in the tree re-derives
these figures.** They are reproducible only by re-instrumenting, by the method below.
This document is the specimen record for the `identity_absent_graph_traversal` row in
`gunbc.recurring_failure_mode`; that row carries the class, this file carries the numbers.

## Reproduction

```
cargo build --release -p v1-compiler --bin gunbc
./target/release/gunbc compile --source-root src/v1 --source-root dag \
    --entry src/v1/compile.dag --target rust --output-dir <out>
```

108 files, 2180 diagnostics. Stage walls come from the existing `trace_mark` bracket:
frontend 16s, normalize 0.5s, reconcile 67s, analyses 0.7s, **emit 9 minutes** before
gunbc#9666 and **4 minutes** after it. Everything below was obtained by compiling
`std::time::Instant` marks and atomic counters into `v1_compiler_emit_rust.rs` and
`v1_compiler_infer_emit_info.rs` and reverting them.

## Where emission goes

Post-gunbc#9666, inside the module loop:

```
reference_derived_use_line_plan     88% of emission
  field_surface_names               (was 203s; ~19s after #9666 shared the walk)
  type_surface_names                184s
  candidates filter                 (was 73s; ~0 after #9666 made it a lookup)
  rows                              0.1s   -- the export-proof DFS is not the cost
emit_imports                        38.5s
items_str (all emit_typed_item)     24.1s
```

## The amplification

`v1.compiler.infer_emit_info` `collect_type_node_import_surface_names` recurses on both
`children` and the `inferred` resolved node, so a shared type DAG is traversed as a tree.

```
entries                     7,458,424
distinct nodes (Rc identity)   23,647     amplification 315x
  of the entries: OccurrenceSynthetic  3,777,962   (50.7%)
                  OccurrenceMinted     3,680,462
                  OccurrenceProjected          0
distinct minted ids            11,003
top-level entries             137,562 over 14,542 distinct nodes
```

`normalize_access_type_node` accounts for 0.4s and `authored_name_at` for 3.9s of that,
so the cost is the re-traversal itself, not the work at each node.

## The four local escapes, each closed by execution

| escape | result |
|---|---|
| delete the producer | loses 738 unique names; candidates 7990 -> 6169 |
| replace with the cheap sub-producers beside it | of 1821 uniquely-contributed candidates, 1402 come only from the inferred-type walk and 1 only from the cheap arms |
| read `TypeSummary.field_import_surface_names` | circular (produced by this walk in `build_field_type_map`); of 137,562 calls, 51% of inferred type nodes have no row at all, and rows that exist agree with the walk 46 times in 66,960 |
| hoist it earlier | nothing to hoist: each item belongs to one module and each module is emitted once, so the producer already runs once per item |

## Two falsified hypotheses

Recorded because both are the first guesses, and neither is cheap to re-falsify.

- **List-copy lowering.** `flat_map` lowers to `__result.extend((*x).iter().cloned())` and
  `rc_list_concat` uses `extend`, so every tree level copies its subtree's list with a
  `String` clone per element -- apparently O(names x depth). Rewritten at **215** sites
  plus `rc_list_concat` to `im::Vector::append` (O(log n), structural sharing, no element
  clones): **byte-identical output, no measurable change** -- emission stayed at 4 minutes.
- **Match-scrutinee deep clone.** `match (*n.expr_data.clone()).clone()`, twice per node.
  Cumulative over the whole run: **0.08s**.

## What is not claimed

One seam, one corpus, one date. 315x is not a base rate for any other traversal. The
figures are not enrolled and nothing re-derives them; treat a stale number here as stale
rather than as a measurement of the current tree, and re-run the method above.
