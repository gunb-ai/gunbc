# The execution spine — primitives → DependencyView → realization/materialization

**Status:** draft for operator + fleet review (clever-koi-282, 2026-07-07). Not yet authority. Flags for sign-off in §10.

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
- **`materialize`** — the `Recompute | Memoize | Share` dimension: because the primitives form a *DAG* (a node is referenced by many parents), `Share` computes each node **once** and reuses it. A tree fold has no sharing; a DAG fold without `Share` recomputes shared subgraphs once-per-parent. (The O(M²) ancestry copy is exactly a *missing Share*.)
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

1. **Single-process, multi-thread** (the 128-core box) — `realize` schedules independent DependencyView nodes across a thread pool (the existing `spawn_width` executor, now driven by node-level independence instead of a flat batch). This is the first target and the one that turns 1/128 into ~critical-path-bounded.
2. **Multi-process, multi-thread** (distributed) — later; the DependencyView + `Placement` dimension already anticipate it.

Everything past these (heterogeneous placement, remote, accelerators) is genuinely downstream and does not block the spine.

## 7. Lane convergence — the spine is the assembly of work already running

The pieces exist, scattered and partly forked. The spine is what they converge *on*:

| Component | Lane in flight | Role in the spine |
|---|---|---|
| produce the edges (resolve → references) | namespace-only resolution (the pivot) | builds the DependencyView from containment |
| single-authority type-env (per-node compile *shareable*) | SymbolIndex / lively-raven (gated on #6348) | removes the false serial dep (§3); enables `Share` (§2) |
| `DependencyView` (the fundamental relation) | bright-heron #6335 | the spine's §1 |
| `topological_layers` → realization runner | scheduler.dag; crisp-bear's floor | the spine's §2/§6-(1) |

None of these individually reads as "the compiler on DependencyView." **Composed under the spine, they *are* it.** Their current scatter is the fracture; the spine is the coherent target that stops the drift.

## 8. Cleanup sequencing — nothing deleted before its replacement exists

1. `dependency_view` as the canonical total function (§1); `topological_layers` its reader.
2. `realize`/`materialize` as its only downstream readers; introduce `run` (§2).
3. Parallel fold: node-level independence drives the thread pool; remove accumulator-threading (SymbolIndex); `Share` for DAG reuse (§3).
4. Single-process-multi-thread runner nailed and **measured** (§9).
5. Route the CI floor's compile through `run`; **then** subsume/delete the shard subsystem (§5), carrying totality forward.
6. Retire the serial v1 seed as the parallel fold proves out (this is also §1 get-off-v1).

Delete-before-replace is forbidden (it re-darkens the floor). Each step lands with its receipt (§9).

## 9. Acceptance — the north-star, and it is measurable

**Operator's criterion:** when I type `gunbc run X`, it runs as fast as physically possible on the modeled hardware — by construction. Made executable:

- **Core-utilization receipt.** `gunbc run X` on an independence-heavy program reports cores used ÷ cores modeled. Today: **1/128**. Target: near the DependencyView critical-path bound (report both the achieved utilization *and* the critical-path floor, so the gap is visible).
- **Discriminating RED.** A program whose DependencyView is a single chain (all dependent) must run serial (utilization ~1) — the runner must not fabricate parallelism where dependencies forbid it. A program with N independent nodes on ≥N cores must approach N× over serial.
- **Spine-coverage RED (the discipline receipt).** A capability that executes *without* going through `run` is a hard error — the bypass is unwritable, and the check that proves it is itself on the corpus (no inert-lens; DESIGN §6).
- **Byte-identical materialization oracle.** `Share`/`Memoize` results must be byte-identical to `Recompute` (DESIGN §5 purity oracle), so materialization is proven correct by execution, not asserted.

## 10. Flags for operator sign-off

- **FLAG A — `run` composition.** `run ≜ realize ∘ materialize ∘ dependency_view`. Confirm the order (materialize before realize: dedup the DAG, then schedule) and that this is the *single* execution authority with all bypasses deleted.
- **FLAG B — node granularity.** Is the realization unit the individual `Node`, the declaration, or the module? Finer = more parallelism + more scheduling overhead. Recommend per-declaration to start (matches the DependencyView's natural grain), revisit if overhead dominates.
- **FLAG C — sharding disposition.** Confirm subsume-not-delete (§5), with the totality proof migrated to DependencyView-covers-all-nodes, and the deletion sequenced *after* the floor routes through `run` (§8-5).
- **FLAG D — scope of the two authorities.** Confirm `std.dependency` is the fundamental authority and `std.realization` is repositioned as its downstream reader (not a peer). The `Independence`/`Placement`/`Materialization` dimensions of `std.realization` are kept — as readings of the DependencyView, not as an independent plan.
- **FLAG E — first scenario.** Confirm single-process-multi-thread is the sole initial runner and multi-process is explicitly deferred.
