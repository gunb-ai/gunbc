# Plan — ground `cli_run.rs`'s resolve/reconcile/discovery engine in `.dag` (Lane 4b ∪ seed-shrink)

**Status:** DESIGN — scoping only, no implementation yet. Reported for operator/sunny-newt-884 sign-off before any cutover PR. Linked from `docs/plans/seed-shrink-census.md` Chunk F (`cli_run.rs`).

**Scope boundary (fence, DESIGN §3/§6):** does NOT touch `src/v1/03_resolve.dag` / `src/v1/04_infer.dag` or their GENERATED outputs (`v1_compiler_resolve.rs`, `v1_compiler_infer.rs`) — those are resolver-B's (#6155, open) territory and already `.dag`-authored. This lane only grounds the **orchestration layer around them** that currently lives hand-written in `cli_run.rs`.

## 0. What's actually in scope

Six functions named in the brief, ~450 LOC in `cli_run.rs` (lines ~478–1011, 1455–1476, 2192–2290ish):

| fn | does |
| --- | --- |
| `build_module_index` / `extract_module_path` / `extract_import_paths` | regex-scan every `.dag` file's `module`/`import` lines into a `module_path → SourceFile` index + raw import-path strings |
| `resolve_transitively` | BFS the import strings to the transitive source-file closure for one or more entries |
| `resolve_entry_graph`, `resolve_entry_with_parse_cache`, `resolved_graph_from_sources` | orchestrate parse → resolve → normalize → typecheck → ownership over a closure, with 3 independent hand-rolled caches |
| `reconcile_with_typed_cache` | per-module typecheck reuse across batch entries (my own #4867, resolve-cost lever PR1) — keyed by `mod_name` string |
| `whole_tree_resolved_ctx`, `discover_owned_data_decls` | whole-tree / multi-entry batch orchestration — groups entries into `DiscoveryResolveGroup`s by overlapping source closure to avoid re-resolving shared modules |

Not in scope: `run_walk` and anything else deep-hawk-756 is touching (coordination note below); the parse/resolve/infer/typecheck internals themselves (already `.dag`-authored, GENERATED).

## 1. The redundancy this dissolution should fix, not just relocate (§2/§3)

Three findings from reading the code, each a reason to *design* rather than transliterate:

1. **The import graph is a second, weaker parser.** `extract_module_path`/`extract_import_paths` regex-scan raw text for `module `/`import ` lines to build the dependency graph used for closure BFS — duplicating what the real parser already extracts structurally. `dsl.std.concept_index`/`decl_facts` (host builtin landed #5966, the same one `decl_facts_project.rs`'s dissolution — `adhoc-34b0f2cf-3a8`, in flight — is grounding) already produces structural declaration facts including import edges. **The .dag authority for the module graph should consume `decl_facts`, not re-scan text.** This directly reuses the child lane already running; sequence them together.
2. **The closure BFS is a generic graph fold that already exists.** `dsl/std/graph.dag` has `CallGraph`/adjacency/`dfs_finish_order` over generic `String` node names, built for the termination-proof SCC check. The module-dependency closure is the same shape (nodes = module paths, edges = imports) — `resolve_transitively` should be an *instance* of `std.graph`, not a bespoke `HashMap`+`Vec` BFS. (§2 horizontal: one graph-closure concept, two breadths — termination proofs and module resolution.)
3. **Three independent cache tiers, one undeclared authority.** `parse_cache` (keyed by file path), `typed_module_cache` (keyed by module *name*), and the cross-process `resolved_graph_cache` (keyed by content digest via `subject_digest_for_closure`) are three hand-rolled `HashMap`s with three different keying policies for variations of "have I already computed this from this content." `dsl/std/cache_interface.dag` is already modeled AND already has real `CacheInterfaceCatalogFacts` inhabitants — `extdeps/realization/resolved_graph.dag` (the disk-tier `resolved_graph_cache` itself) and `extdeps/realization/parse_table_memo.dag` (an existing `locality: InProcess` sibling row — the exact shape `parse_cache`/`typed_module_cache` should take, not a new pattern). So this is a de-fork, not a greenfield inhabitance: the in-process tiers need `InProcess`-locality sibling rows under the same authority, and the disk-tier realization (`resolved_graph_cache.rs`) should be checked for a model↔realization fork against its own `resolved_graph.dag` fact row. Keying `typed_module_cache` by name instead of content is a live §5 cache-impurity instance ("key on declared-input content") — the disk tier already keys correctly by content digest; two policies for one fact is the §3 violation to close.

## 2. Proposed phased authoring (do NOT attempt all of this as one PR)

**Phase 1 — module dependency graph (mechanical, low risk).** New `dsl/gunbc/compile_module_graph.dag` (sibling to `compile_source_model.dag` from #5894): `module_dependency_graph(decl_facts: List<DeclFact>) -> CallGraph` (repoint onto `std.graph.CallGraph`, reuse `build_call_graph_from_proof_edges`-shaped construction) + `transitive_closure(entry: String, graph: CallGraph) -> List<String>` via `std.graph` adjacency, no new graph algorithm authored. Depends on `decl_facts_project.rs` dissolution (`adhoc-34b0f2cf-3a8`) landing first so the import-edge source is the real `decl_facts`, not a second regex scan.

**Phase 2 — align the reconcile caches to the existing `cache_interface.dag` inhabitant, don't invent a parallel one.** Correction from sign-off review: `cache_interface.dag` is NOT uninhabited — `dsl/extdeps/realization/resolved_graph.dag` already grounds the disk-tier cache as a `CacheInterfaceCatalogFacts` row (`resolved_graph_cache_id`, `PersistenceLocality::PerHostFilesystem`, `ContentAddressed` key derivation, evidence citing `resolved_graph_cache.rs`'s temp-file+rename write path and `subject_digest_for_closure` keying). Phase 2 is therefore: (a) add `InProcess`-locality `CacheInterfaceCatalogFacts` sibling rows under the same `resolved_graph_cache` family for `parse_cache`/`typed_module_cache` — one interface, N `PersistenceLocality` realizations (the Realization pattern, §2), not a flattened single cache; (b) reconcile `resolved_graph_cache.rs` itself against `resolved_graph.dag` — the realization may already be a model↔realization fork worth closing in the same phase; (c) fix the `typed_module_cache` name→content re-key — keying by `mod_name` instead of content is the DESIGN §5 cache-impurity failure mode ("key on declared-input content") since the disk tier already keys correctly by content digest, two different keying policies for one fact is the §3 violation to close. Highest-value, highest-risk phase (resolve-cost-critical path, Lane 4b's goal) — needs its own **explicit pre-merge sign-off** (separate from this design sign-off) plus a before/after wall-clock receipt (≤ current cost) and a discriminating RED (cache-miss-when-should-hit AND hit-when-should-miss) before cutover.

**Phase 3 — whole-tree/discovery batch grouping.** `discover_owned_data_decls`'s `DiscoveryResolveGroup` splitting-by-shared-closure is conceptually the same problem `v2.lens.affected_set` and `ci_floor_plan.dag` batch scheduling already solve (group work by shared dependency surface to avoid duplicate resolution). De-fork against those, don't re-author a third batching algorithm. Lowest priority — latent/advisory path only, not on the floor's hot path; drop if the de-fork doesn't pay a measured cost.

## 3. Receipt (per brief)

Each phase: seed LOC moved HAND→GENERATED in `cli_run.rs` · `regen_stage0 --verify` byte-identical · Phase 2 additionally requires an explicit pre-merge sign-off separate from this design sign-off (load-bearing pipeline surface) plus a wall-clock before/after on the same reconcile workload (folds Lane 4b's perf goal) with a discriminating RED (cache-miss-when-should-hit AND hit-when-should-miss) and must preserve the in-process/cross-process `PersistenceLocality` distinction (one interface, N realizations — not a flattened cache).

## 4. Coordination

`cli_run.rs` is also touched by the CI-repair wave (`run_walk`, M1 memo — deep-hawk-756). This lane owns exactly the 6 functions in §0; no edits outside that set without an ordering note posted first.

## Dissolution trigger (DESIGN §6)

Delete this doc when all three phases land and `cli_run.rs`'s resolve/reconcile/discovery functions are GENERATED, not HAND_MAINTAINED.
