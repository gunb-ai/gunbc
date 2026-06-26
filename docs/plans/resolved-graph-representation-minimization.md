# Compiler memory minimization — the "8 GiB small compiler" root cause

Status: landed (root fix) + future work, 2026-06-26. Owner: calm-carp-204.
Evidence: phase-peak instrumentation of `claim_batch` (the per-shard witness
runner) + on-disk resolved-graph cache artifacts.

## What the 8 GiB actually was (measured, corrected)

The operator's instinct was right: a small compiler should not use 8 GiB. The
dominant cost was **not** the per-witness resolve or the cache — it was one
unconditional step. Phase-peak RSS of `claim_batch` resolving a single tiny
witness (`ci_floor_test_threads`):

| phase | peak RSS |
|---|---|
| after `build_multi_entry_index` | 11 MB |
| after `precompute_whole_tree_published_mock_keys` | **1,529 MB** |
| after the witness resolve | 1,535 MB |

The entire jump is `precompute_whole_tree_published_mock_keys` (`cli_run.rs`): it
Strict-resolved the **entire 608-module dsl tree** into one `ResolvedGraph` just
to extract **58 published-mock operation keys**, then discarded the graph. Only
**13 of 608** modules declare `PublishedMockCase`. This is §2 *irrelevant work*:
600+ modules resolved to read from 13.

`claim_batch` is the per-shard runner whose `[measurement] per-shard-peak-rss`
the floor's `per_shard` budget (the 2.1→2.5 GiB that drove the OOM saga) is
calibrated from (`roadmap_authority.dag` node `1-sched-resource-aware`). So this
one step *is* the floor's per-shard memory.

## The fix (landed)

Scope the precompute to the declarers' import closures instead of the whole tree
(`precompute_whole_tree_published_mock_keys`): select modules whose source
`.contains("PublishedMockCase")` (safe over-inclusive prefilter — `.dag` has no
comment syntax, and the downstream `type_annotation_names(.., "PublishedMockCase")`
check is exact, so a false-positive only widens the closure, never fabricates a
key), take their transitive import closures via `resolve_transitively`, resolve
only those.

Result — same 58 keys, witnesses still pass:

| | before | after |
|---|---|---|
| precompute step | 1,529 MB | **37 MB** |
| per-shard peak (non-mock witness) | ~1,570 MB | **~180–270 MB** |
| per-shard peak (mock-consuming witness) | ~1,500 MB | **112 MB** |

**~8.7× reduction at the root**, with no cap/per_shard/hardware change. Verified:
non-mock witness (`ci_floor_test_threads`) and a published-mock consumer
(`git_mock_totality`) both PASS.

## Remaining representation work (future — much smaller fish now)

The per-witness resolve still carries the §2 redundancy below; it's now a few
hundred MB, not the 8 GiB headline, so it's a quality/disk concern, not an OOM.

`ResolvedGraph` retains, per module, a fully-merged copy of every binding /
source-index / inductive-field-list / function it transitively sees. Measured on
one 59-module witness artifact: `type_env` 54% + `func_env` 19% = **73% of the
233 MB is duplicated closure**; bindings 8,015 stored / 434 distinct by content
(7% retained); `func_env` copied once per module; `source_indices` up to 59×. In
RAM these are `Rc`-shared, but `serde`'s `rc` feature does **not** preserve
identity, so a cache round-trip (211 MB JSON) shatters the sharing on
deserialize.

Two layers, both deferred:

- **Cache-boundary intern (B).** Content-addressed pool of unique
  bindings/indices/funcs, modules reference by index; rebuild one `Rc` per pool
  entry on load (sharing restored). Shrinks the artifact ~211 → ~75 MB. **Attempted
  and reverted in this window**: a working interned encoder/decoder shrank the
  artifact to 124 MB and round-tripped value-faithfully, but the on-disk pool
  order is sensitive to in-RAM HashMap iteration order; even sorting pools by
  content hash, the DESIGN §5 `warm==cold` byte-identity purity oracle
  (`cache_purity_oracle_test`) is not yet satisfied because the per-value hash is
  taken over `serde_json::to_vec` of values that themselves contain HashMaps. A
  clean landing needs a canonical (key-sorted) value serialization for the pool
  hash. Deferred to its own change with the purity oracle as the guard. It buys
  disk size, not the floor RSS (that was the precompute), so it is not urgent.
- **Source-side de-merge (A).** Stop materializing the closure in
  `build_type_env` (`04_infer.dag`): env = local bindings + references to imports'
  envs (scope chain); `lookup_*` walk the chain. Removes duplication on every
  path and subsumes B. Touches a **load-bearing pipeline file** → model-first,
  escalate if the brief predates the relevant model PR.

## sccache reliability connection

`resolved_graph_cache.rs` is the `.dag`-cited content-addressed cache the operator
wants to lean on instead of flaky sccache. Layer B (when landed with a canonical
pool hash) makes its artifacts ~3× smaller and its reads/writes lighter, directly
reducing the cache-server memory pressure that triggers the sccache flakiness.
