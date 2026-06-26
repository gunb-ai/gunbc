# Resolved-graph representation minimization (the "8 GiB small compiler" root fix)

Status: design, 2026-06-26. Owner: calm-carp-204. Evidence: live on-disk artifact
`/tmp/gunbc-rg-cache-shared/da/da20714eb29c3945.bin` (the `ci_floor_test_threads` witness closure).

## The defect (measured, not inferred)

`ResolvedGraph` retains, **per module**, a fully-merged copy of every binding / source-index /
inductive-field / function it can transitively see. With 59 modules in the witness closure:

| category | bytes | % of payload | cross-module dup |
|---|---|---|---|
| `type_env`  | 125 MB | 54% | bindings 18.9×, source_indices up to 59×, inductive ~10× |
| `func_env`  | 45 MB | 19% | **59×** (same merged env copied into every module) |
| `items`     | 44 MB | 19% | none — each module's own AST (legitimate) |
| `module`    | 14 MB | 6%  | none (legitimate) |
| `item_registry` (per-module) | 5 MB | 2% | 1.0× (legitimate) |

**73% of the 233 MB is duplicated closure.** `type_env.bindings`: 8,015 stored entries,
**434 distinct by content-hash** → content-dedup retains **7%** (3.2 MB of 48.5 MB). 9 names
(`Int`,`Bool`,`Json`,`Float`,`Unit`,…) carry **2** distinct resolved contents across modules, so a
binding-id-keyed pool is **lossy** — dedup must key by **content hash** (§2 content-addressing).

This is a §2 horizontal-redundancy violation: one binding is one fact, stored once per module that sees it.

## Why it costs 8 GiB (two compounding leaks, both localized — not spaghetti)

The in-RAM types already declare single-authority sharing —
`TypeEnv.bindings: Rc<HashMap<i64, Rc<TypeBinding>>>`, `source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>`.
The authors *intended* sharing. It leaks in exactly two named places:

1. **Cache (de)serialization flattens it.** `serde`'s `rc` feature is enabled, and by its own
   contract it *does not preserve identity* — every `Rc<T>` deserializes to a **fresh** allocation.
   So on a cache **hit** (`resolved_graph_cache::read_cached_file`, the floor's common path via
   `cli_run::resolve_entry_with_parse_cache`), the 211 MB JSON inflates into a graph with all
   sharing destroyed: 8,015 distinct `TypeBinding` allocations instead of 434, 4,248 `NewlineIndex`
   copies instead of 72. That deserialized bloat × parallel test-shards is the floor's 8 GiB.
2. **`build_type_env` merges eagerly.** `src/v1/04_infer.dag:5427` folds every import's bindings
   into each module (`map_merge(acc, typed_parent.type_env.bindings)`), and `func_env` likewise.
   Even on the miss path this materializes the closure N times.

Steady-state idle RSS is a healthy ~111 MB; everything above is this representation.

## Fix — two layers, B then A

### Layer B — content-addressed intern at the cache boundary (contained, zero resolution-semantics risk)

`resolved_graph_cache.rs` only: change the serialized `CachePayload` from "inline everything per
module" to an interned projection (§2 one-grammar-both-directions):

- **Serialize:** build a content-hash-keyed pool of unique `TypeBinding`s (and `NewlineIndex`s,
  `func_env` entries, `inductive_fields`). Each module's env serializes as a list of *pool indices*,
  not inline values. Drop per-module `source_indices` entirely — the top-level `source_indices`
  already holds them once; modules reference by file key.
- **Deserialize:** materialize one `Rc` per pool entry, then each module's `HashMap` maps key →
  `pool_rc.clone()` (pointer copy). **Sharing restored** — the deserialized graph matches the
  fresh-resolve footprint.

In-RAM `TypeEnv`/`ResolvedGraph` types are **untouched**, so every v1/stage0 consumer keeps working.
Pure encoding change.

- Estimated on-disk: 211 MB → ~70–80 MB. In-RAM cache-hit footprint collapses ~3–5×; floor 8 GiB → ~2–3 GiB.
- **Discriminating witness (fail-closed):** resolved graph **byte-identical** after a
  serialize→deserialize round-trip (DESIGN already names "byte-identical cached-vs-cold" the purity
  oracle), plus a RSS-drop assertion on the witness closure. A perturbation that drops one pool
  entry must turn the round-trip RED.

### Layer A — single-authority environments at the source (the true minimal; careful, model-first)

Stop materializing the closure in `build_type_env` (`04_infer.dag`). A module's env = its **local**
bindings + **references** to its imports' envs (a scope/parent chain); `lookup_type*` walk the chain
local→imports. Removes the duplication at the source on **every** path (RAM included) and subsumes
Layer B's pool (the cache then serializes locals + import edges only).

Touches a **load-bearing pipeline file** (`04_infer.dag`) and the lookup consumers in
`v1_compiler_infer_env.rs` → do model-first, escalate if the brief predates the relevant model PR.

## Sequence

1. **B now** — banks most of the floor win, reversible, safe, with the round-trip oracle. Unblocks
   the floor OOM saga (supersedes per_shard/cap/malloc_trim symptom treatments).
2. **A next** — the real §3 single-authority representation; folds B's pool into the import-edge structure.
