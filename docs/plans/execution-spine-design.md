# The execution spine — primitives → DependencyView → realization/materialization

**Status:** FLAGS A–E SIGNED (operator, 2026-07-09); lane owned by merry-moth-539. Authored clever-koi-282 (2026-07-07). Implementation proceeds per §8 (nothing deleted before its replacement; each step lands with its §9 receipt). Scope ruling (operator, 2026-07-09): the duplicate-work target is **.dag** duplication (dsl/, v2/, all hand-written .dag) — not Rust; Rust is the shrinking seed, so `materialize`'s duplicate-elimination and the recompute-trace both scope to interpreted .dag, never the Rust seed's internals.

**Runner realizer reconciled to emit-on-demand (operator, 2026-07-10).** The runner (increment 2+) is **NOT** gated on making the interpreter thread-safe (`Rc→Arc`) — that gate is **removed**. Per ROADMAP §④, `realize`'s node realizer is **emit-on-demand native** (emit closure → build → run, artifact `Share`d by content-hash), never a parallelized tree-walker: the interpreter is a dissolving scaffold (bootstrap receipts · compile-time eval) and investing in its thread-safety is a §④ review-reject (the `Rc→Arc` refactor would parallelize the *slow* realizer on code scheduled for deletion). So the runner parallelizes **native artifacts** orchestrated by DependencyView independence + resource packing (§6-1) — already-built binaries for the floor's first customer, emit-on-demand for novel closures — because native artifacts are trivially concurrent (process or thread), no interpreter thread-safety is on the path. Consequence: the runner's parallel **coverage is the self-host emit frontier** (it grows as modules become faithfully emittable), a typed frontier rather than a single flip. This *converges* execution-as-realization with self-host — **the parallel runner IS self-hosting** (native artifacts parallelize where the interpreter can't cheaply; the compiler's own 1/128→N win arrives when the compiler self-emits to native, not via `Rc→Arc`). FLAG E re-read accordingly (below).

**Increment-1 progress (merry-moth-539, 2026-07-09) — the ANALYSIS spine, Rc-safe (draft #6375).** The two catamorphisms that need no interpreter thread-safety are composed and green-by-execution: `materialize` (`src/v2/std/materialize.dag`, the content-hash `Recompute|Memoize|Share` dedup — Half B) and `dependency_view`'s level-profile (`src/v2/std/spine.dag` `spine_receipt`: `critical_path_depth` = longest dependency chain / the reduce spine §5, `independence_width` = max nodes at one level / what parallelizes to hardware width §5). `spine_receipt` emits the §9 numbers (node_count, distinct/redundant computation, critical-path depth, independence width) **by computation, not assertion**, with the §9 discriminating RED proven: a single chain `A→B→C` reports depth 3 / width 1 (serial, no recoverable parallelism); a root over N independent leaves reports depth 2 / width N (`src/v2/test/claim/spine/spine_receipt_witness_test.dag`, 9/9; `.../materialize/`, 6/6). The **RUNNER** (actual wall-clock parallel execution — §8 steps 3–4) remains increment 2+, its realizer **emit-on-demand native artifacts** orchestrated by DependencyView independence (NOT a parallelized interpreter — the `Rc→Arc` gate is removed; see the reconciliation note above), gated on emit coverage (the self-host frontier); this increment does not fabricate wall-clock N×, it measures the *achievable* width/critical-path floor the runner would target.

**Thesis (one line):** every program *is* the primitives, the primitives *are* a dependency graph, so `DependencyView` is a **total function over the primitives** and realization/materialization are its only downstream readers — which makes "run this program as fast as the hardware allows" hold **by construction**, and makes "a model connected to nothing" **unwritable**.

This is DESIGN §4 (the closed substrate) turned operational, and DESIGN §5 (construction over validation) applied to *scheduling itself*. It is the missing operational spine whose absence has been fracturing the system.

---

## 0. The problem this fixes (the fracture, verified)

Models are authored and connected to nothing, or to the wrong consumer, because nothing forces them onto one spine. Evidence on `main` today (all verified 2026-07-07):

- **Two scheduling authorities, coexisting.** `std.realization` (`RealizationPlan`/`Runnable`, used by the CI floor) and `std.dependency` (`DependencyView`/`topological_layers`, used by `executor.dag`/`runtime_run.dag`/self-host `compiler_closure_emit.dag`). Neither derives from the other; the floor even imports both and hand-builds a `RealizationPlan`.
- **An entire shard subsystem wired to the wrong purpose.** `dag_compile_clean_shard_{roster,partition,totality,seam,transport}` + 3 witnesses — proven to *totally cover* the corpus — is consumed only for **selection** and **memory-sizing**, never for **parallel execution**. The corpus is partitioned and the partition is never run in parallel.
- **The compiler compiles at 1 of 128 cores.** `v1_compiler_infer.rs` is a serial catamorphism; no `rayon`/`par_iter`/`thread::spawn` in the compile path; whole-tree compile-clean measured >1h serial (frontend+normalize = 3s; the rest is serial typecheck).

These are one disease with four faces: **there is no enforced operational spine.** DESIGN unifies the *computation* layer (Node+Edge DAG + one `fold_node`), but the *execution* layer (order, parallelize, cache, run) has no single fundamental model everything must derive from and cannot bypass.

## 1. The axiom: the DependencyView is a reading of the primitives, not a peer of them

A program is `Node` + `Edge` (DESIGN §4). An edge *is* a dependency. Therefore the dependency structure is not modeled *alongside* the program — it is **derived from it** by a total function:

```
dependency_view : Node -> DependencyView        -- total; every program has exactly one
```

`std.dependency.DependencyView` already carries the right shape — `type/data/effect/resource/module/placement_depends_on` — and `topological_layers : Node -> List<ReadinessLayer>` already derives the ready-frontier from it. The move is to make this **the canonical, total, only** reading, so a program cannot exist without its DependencyView. (Contrast today: `DependencyView` is one model among several, wired by hand.)

**Consequence — the compatibility guarantee.** Because `dependency_view` is total over the primitives, *any* program built from the primitives is automatically compatible with everything downstream of the DependencyView. Compatibility is not a property to check or opt into; it is structural.

## 2. The spine: realization and materialization are the *only* downstream readers

```
run(g)  ≜  realize( materialize( dependency_view(g) ) )
```

- **`dependency_view`** — what depends on what (the fundamental relation; §1).
- **`materialize`** — the `Recompute | Memoize | Share` dimension: because the primitives form a *DAG* (a node is referenced by many parents), `Share` computes each node **once** and reuses it. A tree fold has no sharing; a DAG fold without `Share` recomputes shared subgraphs once-per-parent. (The O(M²) ancestry copy is exactly a *missing Share*.) **`materialize` is also where duplicate-work elimination lives** ([duplicate-work design](duplicate-work-graph-lens-design.md)): today `Materialization`/`reconcile` (`dag/std/realization.dag`) is defined but consumed only by a test, and only over effect `RealizedStep`s. Two coupled halves converge here: **Half A** — extend `materialize` to describe the result of *all* .dag computation, not just `RealizedStep` effects (the spine's job); **Half B** — generalize `reconcile`/`has_collapsible_peer` from its effect-pairwise key to a content-hash `ComputationIdentity` lattice (the qualifier `materialize` keys on to decide `Share` vs `Recompute`). Duplicate-work detection is not a lens beside the spine — it *is* `materialize`'s qualification step.
- **`realize`** — run the materialized DependencyView: schedule unblocked nodes concurrently, respect the dependency edges.

Realization is **downstream of** dependency (operator ruling, this is the intended direction). Today it is a *peer* (the floor hand-builds a `RealizationPlan`); the spine makes it a *reader*.

## 3. How the primitives parallelize — the fold is already map-reduce

`fold_node` (`std/node.dag`) is a catamorphism: `fold(n) = combine(n, [fold(c) for c in children])`. The child folds are independent → they **map** in parallel; `combine` is the **reduce**. A pure fold over a bushy DAG is embarrassingly parallel. Serialism has exactly two sources, and only one is real:

- **False serial dependency — accumulator threading.** A left-fold that threads state across siblings (`foldl(f, acc, children)`) forces siblings serial. This is the "bare fold on a single dimension." It is **removable**: make `combine` associative/monoidal (tree-reduce), or eliminate the threaded state (SymbolIndex resolves each name once instead of copying accumulated ancestry — O(M²)→O(M)).
- **True serial dependency — the critical path.** A real reference edge B→A forces `fold(B)` after `fold(A)`. Irreducible; it is the reduce spine.

**The DependencyView separates these exactly** — an edge is a true serial dependency, its absence is independence. Therefore:

> **The achievable parallel wall-clock of any program is bounded below only by its DependencyView's critical path (longest dependency chain).** Everything else parallelizes to hardware width.

The compiler runs at 1/128 not because folding is serial, but because (a) it is *implemented* serially even where independent, and (b) accumulator-threading adds false chains. Both are removable. **This is what the spine buys: `gunbc run X` approaches the critical-path bound on the modeled hardware, by construction.**

## 3′. The fractal/structural guarantee — why *any* program is atomically parallelizable

The point of §1–§3 is not that *we* parallelized the floor; it is that **parallelism is a theorem of the structure**, holding for any program built from the primitives, at every scale, with zero programmer effort. It rests on three properties, two already true:

- **Fractal — by catamorphism (already true).** `dependency_view` is derived by a fold over the primitives (`topological_layers` *is* `fold_node` with a `NodeFold` algebra, `std/dependency.dag`). A catamorphism applies the same operation at every level, so the DependencyView is **self-similar at every scale by construction** — program ⊃ module ⊃ function ⊃ expression ⊃ atom, each a sub-DependencyView. You cannot define a node whose sub-structure lacks one; the fold is the definition.
- **Parallelizable — by monoid (the one property to enforce).** A catamorphism parallelizes **iff its combine is associative — a monoid** (MapReduce's core law: reduction parallelizes iff the operator is associative). The *map* half (independent children) is always parallel in a pure fold; the *reduce* half is parallel iff associative → a balanced tree-reduce, log-depth. The measured 1/128 serialism is **not a limit** — it is **non-monoidal accumulator-threading** (the O(M²) ancestry copy is a left-fold threading order-dependent state across siblings). Make the combine inhabit `Monoid` (grounds in `std.algebra`/FreeMonoid) and the same fold becomes a parallel tree-reduce at every level, for free.
- **Sound — by edge-completeness (mostly true, one hole).** Parallelizing is safe only if nothing hidden constrains it. The DependencyView edges every dependency kind (`type/data/effect/resource/module/placement_depends_on`) and the substrate is pure/effect-explicit (§4). So no ordering exists that the runner can't see — **except** an effect/resource dependency a node fails to declare as an edge. That is the one hole, and it is closed by a §5 wall: **performing an effect without declaring its resource edge is a hard error** (a hidden dependency is unwritable).

**The three structural enforcements** (this is the answer to "how do we ensure it structurally, without having to think about it"):

1. `dependency_view` is THE canonical catamorphism — single authority, can't-not-use (fractal by construction).
2. every combine inhabits `Monoid` — a non-monoidal/accumulator-threaded fold is flagged as serial-in-disguise; the standard combines the substrate provides are monoidal, so parallelism is inherited, never written.
3. every dependency is an edge — a hidden effect/resource dependency is unwritable (the wall).

Then "atomically parallelizable without worry" is exact: **you write monoids and declare dependencies — both already compelled by the substrate — and realization derives maximum parallelism at every scale.** Parallelism is never annotated; it is a theorem, not a task. (This is also why the floor's batch-2 missing-Share and the compiler's O(M²) copy are *the same law violated* — a non-monoidal fold and a missing `Share` are two faces of one broken guarantee.)

## 4. The single `run` authority — the discipline made structural

Make `run` (§2) the **only** way to execute a graph and **delete the bypasses** (the hand-built `RealizationPlan`, the serial fold, any direct executor). Then, exactly like `bazel run`, there is no unscheduled path *because one cannot be written* — scheduling is implicit in "run."

This is the fix for the **discipline** problem (DESIGN §3/§5): a new capability cannot sit off to the side connected to nothing, because to run *at all* it must be primitives → which are a DependencyView → which `realize` will schedule. Coherence stops depending on anyone remembering to wire it. "A model connected to nothing" becomes **unwritable**, and "is X wired?" collapses to "is X on the spine?" — one question, one execution-backed answer (the feature-liveness readability the operator asked for).

## 5. Sharding: subsume, do not delete in a vacuum

The shard subsystem is a **manual approximation of the DependencyView's independence** — hand-partitioning what the dependency structure gives for free. Under the spine, "a shard" = "an independent region of the DependencyView," and `realize` partitions automatically. So sharding is **subsumed, not deleted standalone**, and the one property worth carrying forward is the shard **totality proof** (shards cover the corpus) → it becomes "the DependencyView covers every node," which is stronger. Delete the manual partition *as the spine replaces it* (§8); deleting it first would leave the floor with nothing.

## 6. Realization scenarios — nail 1–2, the rest is downstream

The spine is realization-agnostic; concrete runners are chosen, not universal. Land two, in order:

1. **Independence-scheduled native artifacts** (the 128-core box) — `realize` runs independent DependencyView nodes concurrently by executing their **emitted native artifacts** (emit→build→run, content-hash `Share`d), NOT by threading a tree-walker. A node already realized as a built binary (the floor's gates) runs as-is; a novel closure is emitted-on-demand and its artifact `Share`d. This is the first target and the one that turns 1/128 into ~critical-path-bounded; because native artifacts are trivially concurrent (process or thread), **no interpreter thread-safety is on the path** — the `Rc→Arc` refactor is off-plan (reconciliation note, top).

   **`realize` is resource-aware — independence is necessary, not sufficient.** The achievable concurrency is bounded by *both* the DependencyView and the resource budget:

   ```
   achievable_concurrency = min( independence_width , resource_budget / per_node_peak_RSS )
   ```

   The DependencyView supplies `independence_width`; per-node **measured** memory profiles + the budget supply the packing (this is the `Placement`/resource dimension). The one trap: **pack on measured peaks, never guessed** — a memory-blind scheduler that packs by count OOMs (the exit-137 failure mode). So a serial node-series can be independence-parallel yet *memory*-serialized; the fix is accurate profiles + pack-to-real-budget, not "crank the width."

   *First live instance — the CI floor's own gate-series.* The floor's gates (layering, extdeps_authority, artifact_drift, emit_host, source_root_ingest, regen_verify) are **dependency-independent** of each other (only compile-clean gates them) yet run **serially** — sequenced by memory profile, not by dependency. Measured floor peak is **4.8 GiB of a 33 GiB budget (~7× headroom)**, so the serialization is over-conservative: re-profile on measured peaks, then pack the independent gates → floor time collapses from **SUM(gates)** to **compile-clean + MAX(gate)**. This is `realize` (resource-aware, §6-1) proving itself on the spine's own CI, and it fits the floor in budget with no budget bump.
2. **Distributed (cross-host) native-artifact placement** — later; `realize` places the same emitted artifacts across hosts by the DependencyView + `Placement` dimension, which already anticipate it. (Still native artifacts, not a parallelized interpreter — the realizer is unchanged from scenario 1; only placement widens.)

Everything past these (heterogeneous placement, remote, accelerators) is genuinely downstream and does not block the spine.

## 7. Lane convergence — the spine is the assembly of work already running

The pieces exist, scattered and partly forked. The spine is what they converge *on*:

| Component | Lane in flight | Role in the spine |
|---|---|---|
| produce the edges (resolve → references) | namespace-only resolution (the pivot) | builds the DependencyView from containment |
| single-authority type-env (per-node compile *shareable*) | SymbolIndex / lively-raven (un-gated: #6348 merged 2026-07-07) | removes the false serial dep (§3); enables `Share` (§2) |
| `DependencyView` (the fundamental relation) | bright-heron #6335 | the spine's §1 |
| `topological_layers` → realization runner | scheduler.dag; crisp-bear's floor | the spine's §2/§6-(1) |

None of these individually reads as "the compiler on DependencyView." **Composed under the spine, they *are* it.** Their current scatter is the fracture; the spine is the coherent target that stops the drift.

## 8. Cleanup sequencing — nothing deleted before its replacement exists

1. `dependency_view` as the canonical total function (§1); `topological_layers` its reader.
2. `realize`/`materialize` as its only downstream readers; introduce `run` (§2).
3. Parallel realize: node-level independence schedules **native artifacts** (emit-on-demand or already-built), not a parallelized interpreter; remove accumulator-threading (SymbolIndex); `Share` for DAG reuse (§3).
4. Native-artifact runner nailed and **measured** (§9): `realize` orchestrates emit-on-demand / already-built artifacts by DependencyView independence — **no interpreter thread-safety on the path** (`Rc→Arc` off-plan; reconciliation note, top).
5. Route the CI floor's compile through `run`; **then** subsume the shard subsystem (§5), carrying totality forward.
5a. **Tracked cleanup task (FLAG C refinement, operator 2026-07-09):** once the floor runs through `run` and DependencyView-covers-all-nodes carries the totality proof, **delete** `dag_compile_clean_shard_{roster,partition,totality,seam,transport}` + the 3 shard witnesses and confirm no consumer references them. This is a first-class `dashboard://work-item` with a dissolution trigger, not a deferred intention — the spine is not "done" while the manual partition coexists with `realize`'s automatic one (§2/§3 no-dual-representation). Deletion is sequenced *after* step 5 (subsume) so it never runs in a vacuum.
6. Retire the serial v1 seed as the native-artifact realize proves out — the compiler's own 1/128→N parallelism arrives via self-emitting to native, not via a parallelized interpreter (this is also §1 get-off-v1).

Delete-before-replace is forbidden (it re-darkens the floor). Each step lands with its receipt (§9). The §2/§3 counter-discipline is equally binding: **subsume-without-delete is itself a violation** — every "subsume" step (5) carries a paired deletion step (5a) so no dual representation survives.

## 9. Acceptance — the north-star, and it is measurable

**Operator's criterion:** when I type `gunbc run X`, it runs as fast as physically possible on the modeled hardware — by construction. Made executable:

- **Core-utilization receipt.** `gunbc run X` on an independence-heavy program reports cores used ÷ cores modeled. Today: **1/128**. Target: near the DependencyView critical-path bound (report both the achieved utilization *and* the critical-path floor, so the gap is visible).
- **Discriminating RED.** A program whose DependencyView is a single chain (all dependent) must run serial (utilization ~1) — the runner must not fabricate parallelism where dependencies forbid it. A program with N independent nodes on ≥N cores must approach N× over serial.
- **Spine-coverage RED (the discipline receipt).** A capability that executes *without* going through `run` is a hard error — the bypass is unwritable, and the check that proves it is itself on the corpus (no inert-lens; DESIGN §6).
- **Byte-identical materialization oracle.** `Share`/`Memoize` results must be byte-identical to `Recompute` (DESIGN §5 purity oracle), so materialization is proven correct by execution, not asserted.

## 10. Flags — SIGNED (operator, 2026-07-09; merry-moth-539 owns the lane)

- **FLAG A — `run` composition. SIGNED.** `run ≜ realize ∘ materialize ∘ dependency_view` (materialize before realize: dedup the DAG, then schedule); the *single* execution authority, all bypasses deleted. Implementation note: `dependency_view` is a **rename/promotion of the existing `dependency_lens` / `topological_layers` catamorphisms** (`src/v2/std/dependency.dag`) — today they exist but are off the run spine (the production path is `claim_executor` over a hand-built `RealizationPlan.schedule` fed hand-authored edges). Signing A means making those catamorphisms the canonical scheduling input and deleting the hand-built-plan path.
- **FLAG B — node granularity. SIGNED: per-declaration.** Matches the DependencyView's natural grain; avoids per-expression scheduling overhead; revisit if the critical path proves coarse. (Distinct from lens-3's *identity* grain, which is content-hash over any subtree — a different concern.)
- **FLAG C — sharding disposition. SIGNED: subsume-THEN-delete, with a tracked cleanup task.** Totality proof migrates to "DependencyView covers all nodes"; the manual partition is deleted *after* the floor routes through `run` (§8-5). **Operator refinement (2026-07-09):** "subsume" must not leave the manual shard subsystem as parallel-representation debt (§2/§3 dual-representation trap). The deletion is a **first-class tracked cleanup task with its own dissolution trigger** (§8 step 5a), not a vague "later" — a `dashboard://work-item` bound to the spine lane whose completion condition is "`dag_compile_clean_shard_{roster,partition,totality,seam,transport}` + the 3 shard witnesses are removed and no consumer references them." The spine is not "done" while both representations coexist.
- **FLAG D — the two authorities. SIGNED.** `std.dependency` is fundamental; `std.realization` is its downstream reader (not a peer); its `Independence`/`Placement`/`Materialization` dimensions are kept as *readings* of the DependencyView. **Resource ruling (operator, 2026-07-09):** memory-budget serialization is **not a graph edge** (not a general graph property), but its host facts **are modeled** as first-class .dag facts that `realize` consumes — not loose parameters. Two distinct, both-modeled things: (i) a resource **ordering/availability** dependency (B needs the resource A produced) → **is** an edge (`ResourceDependsOn`, already in `DependencyView`); (ii) a resource **budget/packing** constraint (independent nodes can't co-run because combined measured RSS > budget) → **not** an edge — it is `realize`'s packing over a modeled host-fact set (budget + per-node **measured** peak RSS + placement), `achievable_concurrency = min(independence_width, budget / peak)` (§6-1). "Model the host facts" ✓ and "not a graph property" ✓ are both satisfied.
- **FLAG E — first scenario. SIGNED 2026-07-09; re-read to in-process (single-host) native-artifact orchestration first** (operator, 2026-07-10: the runner's realizer is **emit-on-demand / already-built native artifacts orchestrated by DependencyView independence**, not a parallelized interpreter — per ROADMAP §④; the `Rc→Arc` interpreter gate is **removed**, see the reconciliation note at top. The original signed label "single-process-multi-thread first" is superseded by this re-read wherever it implied a thread-pooled tree-walker). First concrete win: the floor's own dependency-independent gate-series → `compile-clean + MAX(gate)` instead of `SUM(gates)` (§6-1). Those gates are **already-built binaries**, so the first customer is inherently multi-process — the "multi-process deferred" clause is superseded for the *already-compiled* case; what remains genuinely deferred is novel-closure emit-on-demand latency tuning and *distributed* (cross-host) placement.
