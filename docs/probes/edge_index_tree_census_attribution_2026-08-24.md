# The two `tree_census_misses` are 62% whole-corpus parse (2026-08-24)

**Subject:** the observation this lane was opened on —

> Two `tree_census_misses` cost 23.0243s of the edge index's 31.3164s — a cost-SHAPE root
> neither relocating nor warming the build touches (`bare_eligible=699`, `misses=2`).

**The attributed subject is partly EXONERATED, and the exoneration is the finding.** Of the
23.02s the counter reported, ~14.25s is `pool_parse` — the whole-corpus tokenize+parse behind
every pool-derived term — which `tree_bare_census_for_root` calls *inside* its own miss timer.
The census's own work is ~6.96s across the two roots. The brief's framing is a consequence of
the mis-attribution rather than an error of reading it: a term named `tree_census` that is
mostly a corpus parse invites exactly the two remedies the title rules out, because neither
relocating nor warming a *census* touches a *parse*.

## Why the counter can be wrong without anyone mis-measuring

`pool_parse` is a **shared, lazily-forced** term. Three consumers need it — the per-root bare
census (`tree_bare_census_for_root`), the qualified fill (`pool_qualified_fill`), and the pool
bare census (`pool_bare_census`) — it is memoized on the `MultiEntryIndex`, and **none of them
owns it**. Whichever arrives first pays for all of them. `ResolveStageNanos` had no row for it
at all, so its cost had nowhere to land except on the arriving consumer's row.

In `build_both_closure_edge_index` the ref half runs first and touches no pool term, so the
first arrival is the first bare-eligible file's census — and `tree_bare_census_for_root` starts
`miss_started` before its `pool_parse(index)?` call. Nothing here is a mistake in the timer; the
timer is correct about the wall it spans. The row is wrong about **whose wall it is**.

## Measured

Remote amd64 runner, release build, roots `dag` + `src/v2`, `pool_parse` forced first so each
census is measured cold and alone. Instrumentation was scratch (`eprintln!`) and is not in the
diff; the repair below is.

| term | value |
|---|---|
| `pool_parse` (3875 modules, once per index) | **14.24s** |
| `tree_bare_census_for_root("dag")` — 2818 modules reached, 82483 entries, 66426 bare | 4.02s |
| `tree_bare_census_for_root("src/v2")` — 1893 modules reached, 57108 entries, 45958 bare | 2.94s |
| sum | **21.20s** |

against the 23.02s the two misses reported. The residue is the census BFS and the run-to-run
spread across separate dispatches (`pool_parse` alone measured 14.25s / 14.61s / 14.24s on three
runs); no attempt is made here to close 1.8s of a 23s figure measured on a different machine.

**Inside `pool_parse`:**

| | |
|---|---|
| `parse_with_table` | **7.15s** |
| `tokenize` | 4.16s |
| `build_newline_index` | 0.93s |
| `census_heads_module_node` (the strip) | 0.08s |
| remainder (`note_source_hash`, per-file intern-table and `single_si` setup) | ~1.9s |

**Inside one census** (`dag` root, the larger):

| | |
|---|---|
| `build_symbol_index_census_raw_nodes` | 1.52s |
| — corpus variant counts / item counts / insert fold / alias representatives | 0.05 / 0.20 / 0.74 / 0.51s |
| `census_with_resolved_fn_sigs` | 2.25s |
| — entries fold | 1.13s (`census_upgrade_binding` 0.81s, deep `Node` equality 0.08s) |
| — global-bare fold | 1.00s |
| — services fold | 0.01s |

## What this changes about the shape question

- **The largest single term is a parse whose bodies are discarded.** `parse_with_table` builds
  full function bodies; `census_heads_module_node` strips them 82ms later. Half the corpus parse
  is work the only consumer throws away. That is the cost SHAPE the brief was looking for, and it
  is in `pool_parse`, not in the census.
- **The census's own 6.96s carries no comparable shape defect.** The fn-sig pass is dominated by
  `census_upgrade_binding` doing real per-binding resolution (0.81s of a 1.13s fold), and 42% of
  entries genuinely change (`same=47495 diff=34988`), so the deep `Node` equality that guards the
  re-insert is 6.8% of the fold and is earning its place rather than wasting it. The candidate
  repair that looked obvious before measuring — short-circuit the equality — would have bought
  ~0.14s across both roots. Recorded because it will look attractive again.
- **A second-order duplication is visible and is NOT repaired here.** The entries fold and the
  global-bare fold upgrade the same `(module_path, binding)` pairs under two key surfaces
  (1.13s + 1.00s on the `dag` root). Whether they are the same `Rc` at every site was not
  established, and a memo keyed on identity is not expressible in the `.dag` authority these
  functions are generated from (`src/v1/04_infer.dag`), so naming it is all this probe does.

## The repair in this change — attribution only

`pool_parse` gets its own row (nanos, builds, modules), metered once inside `pool_parse` itself
so it is counted wherever it is forced from, and `nanos_net_of_pool_parse` subtracts it from
every enclosing timer that can contain it: `edge_index_tree_census`,
`edge_index_tree_census_miss_nanos`, `edge_index_bare_half`, `load_bare_edge_index`, and
reconcile's `assembly_root_symbol_index`. The row hangs off `load` — deliberately not off the
consumer that forced it, since naming one of three consumers as its parent is the same defect one
level up.

The sum is unchanged; what changes is that each row is about the work it is named for.

## What is NOT claimed

**This change makes nothing faster.** It is instrumentation, and calling it a cost repair would
be the rung inflation DESIGN §4b(1) names as worse than sitting low. The 14.24s corpus parse is
still paid on exactly the same schedule, by exactly the same first arrival.

**The heads-only parse is named, not scoped.** `parse_with_table` producing bodies for a
heads-only consumer is a defect with a measured 7.15s attached to it, but the repair is a parser
mode in `src/v1/03_parse.dag`, which is growth on a seed that is semantics-frozen
(DESIGN §3, `gunbc.v1_maintenance_standing`). It is the next-rung trigger for this class and it
belongs to whoever takes the parser, not to this diff.

**The 31.3164s denominator was not re-measured.** This probe measures the two censuses and the
parse; the rest of the edge index's wall is outside what it ran.

## Rung

The class here is *a shared lazily-forced term reported under its first consumer's name*. It was
below the ladder entirely — not mitigated, because nothing could observe it: the deficit's
frequency was zero by construction, since the parse always had a row to hide inside. It now sits
at **mechanically preventable**: two tests in `exclusive_cost_partition_law` hold the subtraction
and the row's placement, and both go red if either is dropped. It is not *structurally
guaranteed* — a new timer around a new pool consumer can still absorb the parse, because nothing
forces a caller to use `nanos_net_of_pool_parse`. The next-rung trigger is a span type that
carries its own exclusions rather than a helper each caller must remember.
