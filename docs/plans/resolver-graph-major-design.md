# Resolver graph-major design — once-per-node module evaluation

**Status:** design draft, 2026-07-04, from the CI dig-out session (PR #6232). Operator direction: aggressive — the request-major resolver is the top time-waster in the system. This doc names the target architecture, the guard rails, and the change surface in both trees; §7 splits it into a ship-now stage and a gated tail so it is NOT a 10-PR arc.

DESIGN refs: §2 (minimize redundancy — one concept, every scale), §5 (correctness by construction: once-per-node by *schedule*, not by cache lookup), §6 (denominate in displaced cost). Supersedes-and-absorbs: [floor-shared-compute-memoization](floor-shared-compute-memoization.md) (M1 landed there is the entry-grained special case of this design; its own dissolution trigger — "dependency-batched resolve" — is this document).

---

## 0. The problem, measured (2026-07-04 receipts)

- Resolution is request-major: every entry gets a private universe. `resolve_entry_graph(entry)` walks the entry's import closure backward and re-parses + re-infers every module in it, sharing nothing with any other resolve — not across entries in one process, not across processes in one run, not across runs.
- Five-entry census (static import closures): `unicode_xid` 2 modules · canary 2 · `realization_schedule_witness` 11 · `commit_workflow_witness` 163 · `ci_floor_plan` 188. Pairwise overlap `commit_workflow ∩ ci_floor_plan` = **160 shared modules**. Sum if resolved separately = 366; distinct = **194**. At n=5 the redundancy is already 1.9×; at the corpus's ~543 witness entries the shared std/spec prefix is re-inferred hundreds of times per run.
- Cost is wildly non-uniform per module: 2-module closure = **7ms**; 11-module closure (generic-heavy `std.realization_schedule` prefix) = **killed at 5 minutes** (loaded box, ≤2.5× inflation). The expensive modules are exactly the widely-shared ones. (Non-uniformity itself is a separate lane: see resolver-pathology-profile-receipt.md — graph-major removes the *redundancy* factor, not a pathological per-module cost.)
- Fleet numbers: floor prelude (plan-closure resolve ×2 + interpreted scheduler evals) ran **35+ minutes silent** before the first output line; whole-tree compile-clean ≈ **48 min**. CI job timeout is now **10 minutes** (operator inversion, forcing function) — the floor cannot fit it until this design lands.
- Prior mechanisms and why they underdelivered — all the same lesson, *wrong unit / wrong key / wrong medium*:
  - **M1 `walk_memo`** (landed #6008): keyed by **entry**; deduplicates the same gate entry across batches (~105s/run). Zero cross-entry sharing — 543 distinct witness entries get 543 private universes regardless.
  - **`resolved_graph_cache.rs`** (#4867, dormant): keyed by **entry + compiler binary** (cold on every commit), serialized as **monolithic multi-GiB JSON** (unusable when warm).
  - **`SharedClaims`** (within-batch) and the resolver's `seen` set (within-one-closure): correct, but scoped below the redundancy.

## 1. The inversion: request-major → graph-major

Current shape: "run e" → walk backward from e, build private universe, discard. "run f" → repeat the shared prefix.

Target shape (operator's formulation, 2026-07-04):

1. **Edge graph first (cheap, text-level).** One pass over the tree extracts module→import edges — milliseconds; the whole graph (~5.5k nodes) trivially fits in memory. This artifact exists (`build_multi_entry_index` host-side; `module_graph`/`decl_index` model-side).
2. **Minimal set up front.** Given requested targets {e, f, …}: union of reverse-reachability over the edge graph → the induced subgraph → topological order. (Sets, not paths.)
3. **Forward, once per node.** Evaluate module nodes in dependency order; each node consumes its direct dependencies' **results**; requests are just leaf nodes. "Resolve a→d twice" becomes **unrepresentable** — a node appears once in the schedule. This is the §5 construction rung; a memoized backward resolver is the validation rung (the redundant recompute stays writable, one cache-key bug from returning).
4. **The kernel already exists one layer up.** `parallel_executor_plan_from_dependencies` + `RealizationPlan` schedule the CI floor's eleven gates exactly this way. Module inference is the same concept at a different scale (§2: one kernel, N workloads); this design points the existing scheduler at module nodes instead of inventing a resolver-private one.

## 2. The windowed frontier (memory model)

Holding "the whole graph" means holding the **edges** (cheap facts). Evaluating node `d` needs exactly: `d`'s source + the **results** of `d`'s direct dependencies. Not a→c's internals, not their inference traces. When `b`'s last dependent completes, `b`'s working state is dropped; what survives is its compact keyed result. Peak memory ≈ frontier width × result size, not closure size.

Two immutable artifacts per node (the rustc `rmeta`/rlib split):
- **ModuleInterface** — exported type/fn signatures and data heads. This is all *inference of dependents* may read (the §3 rename test: below-boundary representation is opaque). Small; participates in dependents' keys and in the evaluation window.
- **ModuleBody** — the evaluable form of the module's definitions. Needed only at *interpretation* time (the interpreter executes imported fns); loaded on demand, never part of the inference window.

## 3. Isolation = purity, not privacy

In a dependency DAG, influence flows one way: e and f **cannot** affect d's meaning; if they could, that is a bug (a back-edge or impure evaluation) to fail on loudly, never a reason for private universes. The historical fresh-universe-per-witness guarded against *incidental shared mutable state*, not legitimate influence. The discipline that makes sharing sound: node results are **immutable values keyed by content** — then evaluation order is unobservable in any result, and any counterexample is the cache-impurity failure mode (DESIGN recurring list), walled by the oracles in §6.

Key shape (Merkle): `key(m) = H(source_bytes(m), key(dep_1), …, key(dep_n), resolver_identity)`. Keys ground through `std.content_hash` — and this is the dissolution site for the §3 fnv1a64 dual-surface thread (v1 `atom_identity_hash`/`hash_combine` vs `std.content_hash`: module keys are the v2-carrier consumer that thread was waiting for). `resolver_identity` (compiler binary hash) stays in the key until resolve semantics are explicitly versioned — honest (never serves a stale-semantics artifact) at the cost of cross-commit reuse; removing it is a later, separately-justified step. **In-process sharing (S1, §7) needs none of this** — within one process, one tree snapshot, purity of resolve is the same assumption M1 already shipped on.

## 4. Change surface — v1 (host, `src/v1/stage0`)

| Site | Today | Becomes |
|---|---|---|
| `cli_run.rs run_discovery_corpus_with_options` / `run_discovery_rows` | per-row `resolve_entry_with_index_for_discovery_corpus` — one private closure per entry file (~543/run tree-wide) | rows are leaves: one shared resolution of the union closure; per-entry contexts assemble over shared immutable module facts |
| `cli_run.rs resolve_entry_graph` and callers (`resolve_floor_runner_context`, `floor_git_diff_range`→`floor_diff_observe`, `install_output_policy`, `install_group_syntax`) | 4–5 separate private resolves per `claim_executor` process | consumers of one process-level store; union-resolved once |
| shard threads in `run_discovery_corpus` | re-`build_multi_entry_index` + re-resolve runner context **per thread** | share the immutable store (needs `Rc`→`Arc` on resolved artifacts, or evaluate-prefix-then-fanout; decide early — this is the one real engineering constraint) |
| `claim_executor.rs run_walk` `walk_memo` (M1) | entry-keyed context memo | dissolves into the store (M1's own dissolution trigger) |
| `claim_batch.rs` discovery branch | per-row serial resolves | same store; the local corpus path gets the same collapse |
| `v1_compiler_compile.rs compile_to_resolved` | whole-closure monolithic resolve+infer | **the deep cut (S2 only):** per-module parse+infer consuming dep ModuleInterfaces; `ResolvedGraph` becomes an assembled view. S1 does NOT touch this — it changes how often the monolith runs, not its internals |
| `resolved_graph_cache.rs` (#4867) | dormant entry-keyed JSON cache | deleted, superseded by the per-module store (S2) |
| `v1_interpreter` interior caches | per-context | must key per-context or per-content; covered by the purity oracle |

## 5. Change surface — v2 (substrate, `src/v2` + `dag`)

| Site | Role |
|---|---|
| `std.realization_schedule` | inhabit the staged `RunnableCompile` node (M2's shape) as the module-node workload: `{ module, source_key, dep_keys, profile }`; the resolver plan is a `RealizationPlan` over these — modeled in v2, realized by the v1 host, same seam as the CI floor |
| `v2.workflow.executor` | unchanged — the point is reuse of `parallel_executor_plan_from_dependencies` for module nodes |
| `v2.std.dependency` (`DependencyView`) | module import edges as rows — same vocabulary as floor edges |
| `module_graph` / `decl_index` (v2) | the model-side edge-graph authority; a witness asserts host `build_multi_entry_index` edges == model edges (kills the dual representation) |
| `std.content_hash` | key authority (fnv1a64 convergence thread lands here) |
| `gunbc.ci_floor_measurement` / `ci_budget_tree` | width/memory rows for the module lane: peak = frontier × interface size |
| `v2.std.determinism` (#5941) | gates ONLY the persistent tier (S2b): serving stored artifacts across runs requires proven determinism; in-process S1 does not |

## 6. Oracles & red controls (all execute)

1. **Purity oracle:** graph-major result == request-major result, byte-identical, for a sampled witness set (extend `memo_warm_cold_results_are_identical`). Red control: plant an interior-mutable leak (order-dependent memo) → oracle goes red.
2. **Once-per-node receipt:** evaluation counter == distinct node count for the run; a private re-resolve sneaking back turns the receipt red (extends `memo_deduplicates_resolve_count`).
3. **Collision honesty:** co-residence of previously never-co-resolved modules can surface name collisions (see resolver-type-name-collision-wall.md). The purity oracle catches divergence; a collision is a **loud typed error**, never a silently different resolution.
4. **Planted-staleness (S2b only):** mutate a dep after key computation → keyed lookup must miss, never serve.
5. **Budget receipt:** the `[t+Xs]` floor phase marks (landed 2026-07-04) are the standing measurement; the 10-minute job timeout is the acceptance bar.

## 7. Staging — deliberately NOT a 10-PR arc

**S1 — union resolve, in-process (ship in the current PR line; no substrate migration, no persistence, no determinism dependency).** Stop fragmenting demand: one `claim_executor`/`claim_batch` process resolves the union of everything it will need — prelude entries + all roster rows — once, forward, and every consumer assembles from the shared facts. This is the operator's "walk from a once, then evaluate e, then f" implemented with the existing monolithic resolver (the union closure IS one resolve; the `seen` set already deduplicates inside it). Displacement: N overlapping resolves per process → 1; the corpus's per-entry bill collapses to (union once) + (cheap per-entry assembly). Risks owned: name-collision surfacing (oracle §6.3), `Rc` thread boundary (§4).

**S2a — module nodes + Merkle keys, still in-process.** Split `compile_to_resolved` at module boundaries (ModuleInterface/ModuleBody), schedule via `RunnableCompile` nodes, get frontier-window memory and per-module parallel inference (topo antichains × Altra cores — the 48-minute whole-tree resolve is single-threaded today). The one genuinely deep change (modular inference).

**S2b — persistence across runs (gated on #5941).** Store node artifacts by Merkle key; a PR's run evaluates changed modules + dependents + uncached leaves. This — not S1 — is what makes the 10-minute budget comfortable rather than merely reachable; S1 makes a run cost ~1× union-resolve instead of ~N×, S2b makes warm runs cost ~Δ.

**S3 — shared store across runner slots.** A Realization handler change (local disk → shared), no new semantics.

## 8. Dissolution triggers

- S1 dissolves into S2a when module nodes land (the union monolith becomes the degenerate one-batch plan).
- This doc dissolves into the carriers when: the resolver plan is expressed as `RealizationPlan` rows, the oracles of §6 are enrolled witnesses, and the floor fits its declared budget — at which point DESIGN's open-thread bullet and this file are redundant with the model.
