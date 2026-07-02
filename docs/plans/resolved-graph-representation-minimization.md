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
Strict-resolved the **entire 608-module dag tree** into one `ResolvedGraph` just
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

- **Cache-boundary intern (B). LANDED — PR #5834** (`tidy-wren-707`, off main).
  Content-addressed pool of unique bindings/indices/inductive-fields/intern-tables/
  func-sigs, modules reference by `u32` index; `decode()` rebuilds one `Rc` per pool
  entry so the cache-HIT structural sharing (which `serde`'s `rc` shatters) is
  restored — verified by `Rc::ptr_eq` holding across modules. Measured **147.1 → 63.7
  MB** (0.433 ratio, 57% cut) on the 28-module `cache_layer_planner_test` witness, by
  execution (`resolved_graph_intern_test`). The earlier revert (a value-faithful encoder
  that still failed the DESIGN §5 `warm==cold` oracle because its per-value hash was over
  `serde_json::to_vec` of HashMap-containing values) was unblocked by making the pool
  dedup **key** canonical: `serde_json::to_value` → sorted `BTreeMap` → bytes-identity
  hash. The oracle compares graph *values*, not on-disk bytes, so a value-faithful decode
  round-trips byte-identical and the disk format is free to change underneath — all 4
  `cache_purity_oracle_test` pass single-threaded. Mechanism: hand-written
  `src/v1/stage0/src/resolved_graph_intern.rs` (HAND_MAINTAINED, not emitted),
  `resolved_graph_cache.rs` `FORMAT_VERSION` 1→2. It buys disk size and lighter
  cache-HIT reads (helps sccache memory pressure), not the floor RSS (that was the
  precompute), so it was never the urgent part — but it is now done.
- **Source-side de-merge (A). DESIGNED, awaiting operator sign-off** (`tidy-wren-707`).
  Stop materializing the closure in `build_type_env` (`04_infer.dag` ~line 5388):
  replace the merged flat env with a scope-chain env
  `TypeEnv { local_bindings, parents: Vec<Rc<TypeEnv>>, .. }` where `parents` are the
  directly-imported modules' already-`Rc`-shared envs; `lookup_binding`/`lookup_func`/
  `lookup_source_index` walk local-first then DFS parents (first-match, import-order,
  memoizable per `(env, key)`). A binding then lives in exactly one module's env and
  every consumer reaches it by chain-walk, never a copy — so there is nothing left to
  intern and this **subsumes B**. Sign-off asks: (1) a by-execution equivalence witness
  asserting the chain-walk preserves current shadowing / first-match order exactly
  (resolve a multi-import witness both ways → identical typed graph); (2) the chain is
  the acyclic import DAG, guarded by the existing import-cycle check (DESIGN §4, no value
  cycles). Touches a **load-bearing pipeline file** → model-first; not started, needs
  operator/design sign-off before edit.

## sccache reliability connection

`resolved_graph_cache.rs` is the `.dag`-cited content-addressed cache the operator
wants to lean on instead of flaky sccache. Layer B (landed, #5834, canonical
key-sorted pool hash) makes its artifacts ~2.3× smaller and its reads/writes
lighter, directly reducing the cache-server memory pressure that triggers the
sccache flakiness.
