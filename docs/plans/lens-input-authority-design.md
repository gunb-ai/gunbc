# Lens input authority — resolve once, share by reference (the lens-runner binding)

**Status:** SKETCH for merry-moth-539 review against the materialization ladder cells (#6375). Lane split (operator-agreed 2026-07-09): merry-moth owns the law + projection (`std.materialization_ladder`, `extdeps.cache.materialization.provider_from_catalog`); this lane (loyal-wren-398) owns the **lens-runner binding + lens-contract signature** — the consumer side. Companions: `resolver-graph-major-design.md` (the resolve producer), #6422 (the provider-row template on the v2 parse/staging carriers).

## 0. The instance, in the ladder's terms

Every v2 lens is a pure reader over the resolved graph (`InferredTree` → `dependency_lens` → `DependencyView`, then O(edges) reachability). N lenses demanding one resolve-identity with no `Share` between them = the ladder's rule-1 **`AuthoredDuplication`** at a `SharedStateFrame` (one process). Affected-set is the flagship: it needs the whole-tree resolve and OOMs at corpus scale (24 GiB slot, `exit 137`). The lens code is ~0 cost; 100% of the cost is the resolve it consumes.

Fix per the ladder: **rewire, not cache.** One materialization per process; every consumer holds a *reference*. The plural readonly reads are then legal as `ReferenceTier @ SharedStateFrame` (keyless by shape — the ladder's value-share cell). A keyed store for the same-process case would be the exact pathology the ladder exists to refuse.

## 1. Grounded current state (audit, 2026-07-09)

Most of the rewire already landed; the gap is that it is **undeclared**.

- **Lens contract already clean.** All 8 v2 lenses (`affected_set`, `edit_locus`, `effect`, `idempotency`, `ownership`, `parallelism`, `structural_resolution`, `unused_parameters`) take `tree: InferredTree` / `dependencies: List<DependencyView>` as **input parameters**; none has a resolve door. The "change the lens contract" end-state is already true at the model layer.
- **Host resolves once per (thread, source_roots).** Union-resolve S1 (#6234) landed `process_shared_index` → one shared `MultiEntryIndex` (parse_cache + typed_module_cache); floor-runner / entry-selection / discovery rows all `resolve_entry_with_index(&index, …)` against it (`cli_run.rs`). The Rust reverse-reachability fork `NodeFrontierSeeds` is **already deleted** (`cli_run.rs:6767`).
- **So the same-process share EXISTS but is UNDECLARED** — merry-moth §10a: *"M1 within-walk resolve memo … correct Share@plan … **declare provider row (C2)**."* That declaration is this lane's core deliverable.

Two residuals, both OUT of this lane:
- **Per-shard W× (same process, cross-thread).** `Rc<ResolvedGraph>` is `!Send`, so each shard holds its own index and the shared prefix is re-resolved per shard. Still rule-1 (rewireable in principle — same process), blocked on the `Rc`→`Arc` realization migration = graph-major S3. Model-wise a `SharedStateFrame`; the `Rc` is a realization constraint (design §5b), not a model fact.
- **Cross-process / cross-run (compile subprocess vs `claim_executor`).** An `IsolatedChildrenFrame` LCA → dischargeable only by a **store tier**, and only a **content-keyed** one (id/existence-keyed is refused by shape, #6352). That provider is `resolved_graph_cache` (`dag/extdeps/realization/resolved_graph.dag`, on main: `PerHostFilesystem`, `ContentAddressed`, `SizeBounded` 10 GiB) = graph-major S2b. **No interim persistent cache** (merry-moth Q3d).

## 2. The two providers of one resolve-identity

| | frame / LCA | tier | carrier | owner |
|---|---|---|---|---|
| **P1 in-process share** | `SharedStateFrame` (process) | `ReferenceTier` (keyless value share) | `process_shared_index` (exists, undeclared) | **this lane declares** |
| **P2 cross-process store** | `IsolatedChildrenFrame` | `CasTier`/`ArtifactTier` (content-keyed) | `resolved_graph_cache` (cited, on main) | graph-major S2b |

Recorded relation (the fnv1a64 dual-surface discipline): P1 and P2 are two providers of **one** computation-identity (the resolved graph), never two unrelated stores.

## 3. What this lane delivers

1. **Declare P1** — the in-process resolve-share as the sanctioned lens-input authority, grounded on the ladder's value-share cell (`ReferenceTier @ SharedStateFrame`). Carrier-marked per the #6422 template.
2. **Enrolled witness** — the share's green-by-execution proof + RED control: a lens reading the shared reference == a lens reading a private cold resolve, byte-identical (purity oracle). Enroll the existing S1 oracles (`memo_warm_cold_results_are_identical` confirmed in `claim_executor.rs`; the union-resolve equivalence oracle from `resolver-graph-major` §6 to be located/confirmed). RED: a re-introduced private resolve, or an existence-keyed projection, flips it.
3. **The contract wall** — the runner is the sole resolve site; a lens/consumer that resolves privately is the undeclared-provider shape (merry-moth §10b wall 2). Already true for the `.dag` lenses; this makes it a checked invariant, not a convention.

## 4. Open questions for merry-moth (ladder-cell review)

- **Q-A (key):** P1 is `ReferenceTier @ SharedStateFrame` — keyless value share, NOT a `provider_from_catalog` store projection (that projects catalog rows into store tiers). You referenced "the ladder's `value_materialization`" as covering this cell, but I don't see that symbol in `materialization_ladder.dag` — **what is the exact carrier to declare a ReferenceTier value-share?** (`value_materialization`? a `ReferenceTier` provider row? the `ValueSharedByReference` verdict on the resolve identity directly?)
- **Q-B:** Confirm P2 = `resolved_graph_cache` discharges the isolation-LCA obligation, graph-major S2b owns it, and this lane only **records the P1↔P2 one-identity relation** (no P2 build here).
- **Q-C:** Witness home + which S1 oracle(s) to enroll + the exact #6422 carrier-mark to copy.

## 5. Branch dependency + sequence

The ladder + `provider_from_catalog` + `materialize` are **only on #6375** (not main); `resolved_graph.dag` is on main. So this lane's `.dag` imports #6375's carriers → it **lands after / behind #6375**. I develop against #6375's branch state. First code step (post Q-A): the P1 value-share declaration + witness; the host is largely unchanged (S1 already resolves-once), so the delta is **declaration + the contract-wall check**, not a resolver rewrite.

## 6. Implementation status (2026-07-09)

merry-moth's rulings received (Q-A/B/C confirmed) and the `.dag` **is authored** against the real ladder signatures, incorporating merry-moth's grain correction:

- **Identity = `union-index-build(dag,src/v2)`, NOT `union-resolve`.** S1 shares the *index build* (parse + typecheck materialized once per (thread, roots)); it does **not** collapse per-entry *closure* resolves, which stay ≈K and are `ci_materialization`'s `resolves_total` receipt (the M2/graph-major target). Three distinct quantities kept apart: index-build = 1 / K readers (this lane's P1), entry-closure-resolves = K (merry-moth's receipt), readers = K (this lane's `reader_count`). Declaring the share moves nothing at runtime, so this lane does **not** touch `ci_floor_declared_resolve_count`.
- **P1** = `provider_row(id: "process_shared_index", scope: [claim_executor-process SharedStateFrame], coverage: [union-index-build], tier: ReferenceTier, eviction: scope_exit)` — directly, **not** `provider_from_catalog` (that births store tiers; ReferenceTier is keyless by shape). Witness folds the **live** consumer roster: `project_ci_floor_gates` + `project_ci_floor_witness_entries` (from `commit_gate_roster`) + the two prelude entries; asserts `ValueSharedByReference`, RED control `providers=[]` → `ValueRefusedNoCarrierProvider`. Backing oracles cited (run under `rust_tests`): `memo_deduplicates_resolve_count`, `memo_warm_cold_results_are_identical` (`claim_executor.rs`), `union_view_result_equals_private_resolve_in_every_order`, `union_resolve_typechecks_each_node_once` (`src/v1/tests/src/union_resolve_receipts_test.rs`).

**Held off the branch, by construction.** The `.dag` (`dag/gunbc/floor_materialization.dag` + `dag/test/claim/floor_materialization_witness_test.dag`) imports `std.materialization_ladder`, which is on #6375, not main — so committing it to a main-based branch breaks the compile-clean gate (unresolvable import → CI red, the #6426 failure). The authored files are staged in-session and land the moment #6375's ladder is on main: merge main, drop the two files in, `claim_batch` them green-by-execution (+ RED control), enroll the `CommitWitnessClaim` row, flip #6426 ready. Until then #6426 carries only this design doc (compiles clean).
