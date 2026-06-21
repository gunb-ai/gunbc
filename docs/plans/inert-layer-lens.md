# The inert-layer lens — modeled but unreached

> A lens that reports declared concepts (carriers, fns, whole modules) that are **modeled but unreached
> from any live run-root** — DESIGN §6's "the machinery exists but nothing gates on it," made
> *observable* and eventually fail-closed. The dangerous subset, and the reason for the lens, is the
> **load-bearing-but-unwired** layer: a richly-structured carrier that *looks* like it drives behavior
> and drives none. DESIGN refs: §3 (single authority — generalizes the #5433 inert-lens backstop, does
> not fork it), §5 (fail-closed; construction over validation), §6 (lens as residue; coverage-by-illusion),
> §7 (recursion — the compiler's own dead model is a substrate fact).

## 1. The definition (why reference-count is not enough)

A declared concept is **inert** iff it is **not reachable from a live run-root** over the reference
graph. The subtlety that makes this a real lens and not a grep:

- **Reference-count overstates liveness.** A carrier with N>0 consumers can still be inert if all N
  consumers are themselves inert — a self-referencing cluster. Measured example: `RealizationObjective`
  and `ComputeOffer` each have 4 consumer files, but `RealizationObjective` is **live** (`ci_floor_plan`,
  a run-root, imports `realization_width`→it) while the work-demand cluster around `ComputeOffer` reaches
  nothing that runs. Same count, opposite verdict. **Reachability, not count.**
- **Run-roots ≠ test-roots.** The existing #5433 backstop seeds reachability from *discovered test
  witnesses* — it answers "is this lens **covered**?". Inert-layer detection must seed from the *run*
  roots — what actually executes in production: the CI floor plan, the compiler pipeline driver, the emit
  entry. A module reached only by tests but by nothing that runs is *inert in production yet covered* — a
  distinct, also-interesting state. The lens reports both, labeled.

So: **inert = declared ∧ ¬reachable(run-roots)**. Decidable (graph reachability), a pure Node read.

## 2. The census (the discriminating witnesses the lens must reproduce)

Measured 2026-06-21 over `dsl/**` + `src/v2/**`, reachability cross-checked against run-roots
(`ci_floor_plan`, `scheduler`, the v1 run path). This is what the lens must independently re-derive:

**Fully inert (0 live consumers), load-bearing — the target class:**

| carrier | home | what it models |
| --- | --- | --- |
| `CacheLayerPlan` | `cache_interface.dag` | the L1/L2/L3 cache-layer plan — the whole §2 cache-planner output |
| `WorkDemand` | `compute_fabric.dag` | the work-demand vocabulary (isolation/memory/GPU) — nothing emits one |
| `ParallelismShape` · `IndependentShards` · `PartitionedReduce` | `compute_fabric.dag` | the corpus-sharding demand model |
| `Partitioner` · `SymbolicCost` (forward-stubs `= Node`) | `compute_fabric.dag` | map→reduce execution decomposition |
| `execution_receipt_digest` | `compute_fabric.dag` | the digest the 3-dimension unification rests on — a stub returning `work.id`, consumed nowhere |

**Edge-wired (consumed by a model that is not itself the live source yet):**

| carrier | consumed by | why still inert-at-the-edge |
| --- | --- | --- |
| `ComputeOffer` / the fleet model | `ci_fleet`, `operator_fleet`, `runner_spec_from_offer` | projected to a `RunnerSpec`, but `ci.yml`'s runner block is still a literal — the projection isn't the live emission source (Phase 3 open) |

**Now-wired (recorded so the lens does *not* false-positive these):**

| carrier | reached via |
| --- | --- |
| `RealizationObjective` · `realization_width` · `HardwareThreadCount` (11) · `AxisGoal` | `ci_floor_plan` → `realization_width.width_fold_objective_goals` / `memory_aware_spawn_width` — the memory-aware width landed; the *schedule/width* arm of realization is live |

The reading: the **schedule/width** arm of the realization layer is now wired; the **cache-plan** arm and
the **work-demand / sharding / receipt-digest** arm are the inert load-bearing layers. Exactly the
realization-loop thesis ("shape-complete but input-starved"), now with names.

## 3. Two tiers (what's buildable now vs gated)

**Tier 1 — module-level, buildable now (reuse #5433).** `inert_lens_modules` (`cli_run.rs:2558`) already
computes module-import transitive closure from seed roots and reports unreached `v2.lens.*` modules. The
inert-*layer* lens is the **same machinery with two generalizations**: (a) widen the output filter from
`is_top_level_lens_module` to *all* modules; (b) seed from the **run-roots** (floor plan + pipeline +
emit), not only discovered witnesses. Output: unreached *files/layers*. Reuses the existing BFS over
`module_to_path` + `path_imports`; no new host machinery.

**Tier 2 — symbol-level, gated.** Within reached modules, which declared carriers/fns have zero live
consumers (the §2 census above is symbol-level). This needs **whole-corpus reference (`BindsTo`)
enumeration**, which does *not* exist today — `dependency_lens` is per-declaration, `concept_index`
enumerates *declarations* but not *reference sites*. So Tier 2 is host-fed today (a
`enumerate_all_binds_to_edges()` bridge beside `concept_decl_facts_live()`) and becomes a pure `.dag`
walk on the **same dissolution trigger as `concept_index`** (gunbc#5364 — v2 self-host gains
compile-graph access). Until then Tier 2 is host-fed, Tier 1 is pure.

## 4. The load-bearing ranking (the advisory half)

"Inert" is decidable; **"load-bearing" is a heuristic** — so the lens *decides* inertness and *ranks*
apparent load-bearingness, never gates on the ranking. Rank an inert concept by structural richness:
coproduct arm-count + record field-count + fn return-type richness, plus name signals
(`Plan`/`Account`/`Receipt`/`Schedule`/`Demand`/`Policy`). A 6-arm `ParallelismShape` with 0 consumers
ranks far above an unused 1-line helper. This is the operator's exact ask — "ones that *seem* load-bearing
but are unwired" — surfaced as the ranked head of the inert list.

## 5. Frontier placement (per [expressibility-frontier](expressibility-frontier.md))

- **Inertness is a ① wall candidate.** Reachability is decidable; an inert load-bearing carrier should
  eventually **fail closed** exactly as #5433 does for lenses ("an inert lens is a lie" → "an inert
  load-bearing carrier is a lie"). The honest path: ship as a ② *observing* lens first (a ranked report,
  no gate), promote to a ① wall once the corpus is clean enough that a new inert load-bearing carrier is
  a genuine defect rather than expected staged-ahead modeling.
- **The "staged-ahead" exception is the catch.** Much of the inert set is *deliberately* modeled before
  its consumer (the realization loop is built model-first by design). So a blanket wall would fight the
  project's own just-in-time-after-modeling discipline. The resolution is the #5433 pattern: a
  **named, shrinking exception roster** (carriers modeled ahead of a tracked consumer-PR) that empties as
  the realization loop wires them — the same ratchet-during-migration → wall-when-empty shape as the
  realization-vocabulary guard. Each roster entry names its dissolve-on (the PR that wires it).
- **The ranking is the ② residue**, permanently advisory (judging "load-bearing" needs domain knowledge).

## 6. Reuse map (do not fork — §3)

| need | reuse | file |
| --- | --- | --- |
| transitive reachability BFS | `inert_lens_modules` | `cli_run.rs:2558-2606` |
| enumerate all declared concepts | `concept_index.enumerate_concepts()` | `concept_index.dag:130` |
| use vs structural edge classification | `unused_parameters` `UseRelation` (`BindsTo` = the use authority) | `unused_parameters.dag:22` |
| import/reference edge-walk | `layering_imports` projection | `layering_imports.dag` + `layering_imports_project.rs` |
| run-roots | floor gates + corpus + pipeline/emit entries | `ci_floor_plan.dag:83`, `cli_run.rs:run_discovery_corpus` |

## 7. Wiring + dissolution

- Tier 1 lands as `v2.lens.inert_layer` + a floor witness; runs over the corpus, **reports** (advisory)
  first, ranked, with the exception roster.
- Promote to fail-closed once the roster is small and stable (per §5 above).
- **Load-bearing seed caveat:** Tier 1 touches `cli_run.rs` (the #5433 closure) — a DESIGN-named
  load-bearing file → **escalate before editing**; prefer extending `inert_lens_modules` behind a flag to
  forking it.
- **Dissolution:** the lens itself never dissolves (inertness is a standing property); its *exception
  roster* dissolves to empty as the realization loop wires each carrier, at which point the lens flips
  from advisory ② to fail-closed ① wall.

## 8. Open

- Confirm the run-root set (is `scheduler.dag` the sole runtime, or also the v1 `claim_executor` path?
  the digest census touched both).
- Decide Tier-1-now vs wait for Tier-2's host bridge so the first landing is symbol-granular (the census
  shows the interesting cases are symbol-level — `execution_receipt_digest`, `CacheLayerPlan` — so a
  module-only first cut may under-deliver; weigh against Tier 1's zero-new-machinery cost).
