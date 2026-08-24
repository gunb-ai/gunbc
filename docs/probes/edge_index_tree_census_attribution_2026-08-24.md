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

## The two closures DIVERGE — the duplicate-build hypothesis is refuted

A candidate raised while this probe was running (smart-ram-730, from a
`GUNBC_EDGE_INDEX_CENSUS_TRACE=1` run that names the two misses as `root=src/v2` and
`root=dag` on ONE index address): the census keys on root, but its content is a symbol
index over the ADJACENCY closure of the files under that root, and adjacency follows
imports with no root restriction. If the two reached sets coincided, we would be building
one symbol index twice and the root key would be buying nothing.

**They do not coincide, and set cardinality alone settles it** — no digest is needed,
because sets of different size are different sets. The `modules=` figures in the table
above ARE the reached-set sizes: `build_symbol_index_census_nodes` receives exactly the
`pool.nodes_by_file` list filtered to `reached`.

| root | files under the root | modules reached | pulled in from the other tree |
|---|---|---|---|
| `dag` | 2610 | 2818 | 208 |
| `src/v2` | 1265 | 1893 | 628 |
| corpus | 3875 | | |

So each root's closure does reach across into the other tree, which is what made the
hypothesis worth raising — but neither closure is anywhere near the whole corpus and they
differ by 925 modules.

**The overlap is real and is bounded, not measured here.** Two subsets of a 3875-module
corpus with sizes 2818 and 1893 intersect in at least 836 and at most 1893 modules, so
somewhere between 836 and 1893 module-censuses are computed twice. That is a genuine
duplication and it is the residue this probe leaves open; it is NOT the
build-one-index-twice shape, and closing it would have to preserve the root-scoping,
which exists so that a bare name resolves under the census exactly as it resolves under
the root's whole-tree gate compile.

**Boundary on the counts.** These are for source roots `dag` + `src/v2`, with the censuses
forced directly rather than reached through an entry. The reached set is a function of the
roots and the pool, not of the entry, so it carries across entries — but `misses=2` does
not: a run given a different root set has a different denominator, and the trace that
produced `tree_census_calls=699 / misses=2` was one witness entry.

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
- **The per-file build cost, priced net of the parse.** The census is ~1.48ms per module
  censused (6.96s over 2818 + 1893). A figure of ~2.7ms/file circulated while this was being
  measured; it is the same work divided by the same denominator with the 14.24s parse still
  inside it, and it should not be quoted now that the parse has its own row.
- **A second-order duplication is visible and is NOT repaired here.** The entries fold and the
  global-bare fold upgrade the same `(module_path, binding)` pairs under two key surfaces
  (1.13s + 1.00s on the `dag` root). Whether they are the same `Rc` at every site was not
  established, and a memo keyed on identity is not expressible in the `.dag` authority these
  functions are generated from (`src/v1/04_infer.dag`), so naming it is all this probe does.

## The repair in this change — attribution only

`pool_parse` gets its own row (nanos, builds, modules), metered once inside `pool_parse` itself
so it is counted wherever it is forced from, and every window that can contain it is recorded
net of it.

**There are TWO forcing paths and they land in different top-level rows.** This is the half the
first version of the repair got wrong, and review 55349 caught it:

| path | window that first forces the parse |
|---|---|
| `load` -> bare-reference closure -> edge index | the per-root census (`edge_index_tree_census`), and the loop's cross-tree fallback `pool_bare_census` |
| reconcile | **`assembly_pool_fill`** via `pool_qualified_fill`, then `assembly_root_symbol_index` via the per-root census |

`assembly_pool_fill` runs before `assembly_root_symbol_index`, so on the reconcile path it is
the *real* payer, not a hypothetical one. Netted windows are therefore: `load`,
`assembly_pool_fill`, `assembly_root_symbol_index`, `load_bare_edge_index`,
`edge_index_bare_half`, `edge_index_bare_resolve_loop`, `edge_index_tree_census`, and
`edge_index_tree_census_miss_nanos`.

**The row is an EXCLUSIVE PEER, not an inclusive row under a parent.** It shipped as
`InclusiveCostRow { contained_in: "load" }`, which is false whenever reconcile forces it
first — `assembly_pool_fill` is a top-level peer of `load`, not a descendant. No single parent
is true, because which row forces the parse depends on the path; that is the original defect
restated one level up, and naming any one parent would have been the same lie in a smaller
font. Once the parse is carved out of every window that can contain it, it *is* a disjoint
window inside the parent span, which is what an exclusive row is.

That also keeps `sum_exclusive` invariant across the repair. Before it, the parse was inside
whichever row forced it and was counted once; after it, the parse is its own row and is still
counted once. The inclusive form would have quietly dropped `sum_exclusive` by 14.24s and
migrated the parse into `remainder_nanos` — a second, smaller accounting move that nothing in
the first version declared.

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

**That residue is not theoretical, and the evidence is this change's own first version.** It
netted five windows and missed two — `assembly_pool_fill` and `edge_index_bare_resolve_loop` —
so the reconcile path kept the exact mis-attribution the repair exists to remove. A helper each
caller must remember was forgotten by the author who wrote the helper, in the same diff
(review 55349). Anyone weighing the rung above should weight that receipt over the reasoning.
