# Type environment: single import authority + scope cursor (v1 fix, v2 target)

Status: DRAFT for operator review. Owner: lively-raven-355 (realization / SymbolIndex) — inherited 2026-07-06 when cool-hawk-899 archived; cool-hawk-899 authored §1–§8.
Directive: operator (2026-07-05) — "fix v1, it's worth it; then apply the same to v2." · (2026-07-06) — the intra-process node-level memo must be built as one shard of an eventual content-addressed, distributable realize (map-reduce / RBE); see §5.5.

**Resolver half (companion):** `docs/plans/namespace-resolution-design.md` (loyal-dove-903). This doc is the INDEX/storage half (fill-once `SymbolIndex`, the O(M²) fix); the companion is the SEMANTICS half (`resolve(name, position)`). One index, two `ResolutionPolicy` values over it — `import-scoped` (this doc's §3, behavior-preserving, ships first) and `namespace-only-Y` (the companion's nearest-enclosing-subtree pivot, on top). **One-index invariant (companion §7.5):** the fill stays policy-agnostic (topo prepass fills everything import-DAG-reachable); the policy gates LOOKUP only, never fill — else two policies materialize two indices, the dual representation Rule 1 forbids.

This is reasoned serially (per DESIGN preamble): the problem fixes the axioms, each section a consequence.

## 1. The problem (measured)

`gunbc compile --target dag` = 959s / 876 modules (~1.1s/mod), COMPILED Rust (not interpreted).
The resolve+typecheck half (~710s) is O(M²), and the root is in `build_type_env` (`04_infer.dag:5587`):
each module builds `ancestry_str_bindings` by `map_merge`-copying its parent's transitively-accumulated
ancestry (`5719/5727`). A `std` prelude binding is physically materialized in ~876 descendant maps.
`typecheck_modules` (`6510`) is a serial per-module fold, so the copies sum to O(M²) in map/key churn
(the `resolved: Node` values are Rc-shared, so it is entry/String-key churn, not deep-definition copy).

This was a deliberate B1 tradeoff (`04_env.dag:36` invariant): B1 replaced a per-lookup
`flatten_visible_bindings` (O(depth)/lookup) with the per-module precompute. It traded lookup cost for
materialization cost. It is already marked scaffold: `type_env_compositional_authority_dissolution_trigger`
(`04_env.dag:31`) and `type_env_cache_parallel_repr_dissolution_trigger` (`04_env.dag:34`) name the target
as **v2 `std.type_env`**.

## 2. The root cause: one struct, two concerns (§2/§3)

`TypeBinding { name: String, resolved: Node, provenance: SubValueRelation }` (`04_env.dag:18`) fuses two
populations that have nothing to do with each other:

- **Import/ancestry bindings** — need only `name → resolved: Node`. `provenance` is *always* `SubValueUnknown`
  (a top-level import is never a sub-value of anything). This population is what gets materialized and
  copied N times.
- **Local scope bindings** — params + match-bound fields, function-internal. These need `provenance`
  (`std.induction` descent evidence: `PreservedValue` for params `04_infer.dag:1077`, `compose_sub_value`
  on inductive-field access `3362/3834`) — it is how structural recursion is proven to terminate.

So the O(M²) population drags a field only the O(1)-per-function population uses, and is stored
per-module instead of once. Fusing + materializing is the whole defect.

## 3. The ideal representation (§3 single authority + §5 unwritable-by-construction)

Split the two concerns; the copies then cannot exist:

- **Import resolution → ONE authority, referenced not copied.** A single `qualified-name → Node` index
  (an index over the already-content-addressed DAG; the Nodes exist once). The forward DFS-from-root fills
  it as each definition is first encountered. A module holds NO ancestry copy — it holds a **scope**: its
  import list (already present) says which qualified names are visible. Resolve a name = locals, else
  imports → qualified name → the one index. There is nothing to `map_merge` because nothing is duplicated
  (single authority = the bad state — N stale copies — is unwritable).

- **Local/termination scope → the cursor window.** Params + match-bound sub-values carrying `provenance`,
  pushed as the DFS descends into a body, popped on exit. Small, a stack, O(scope) not O(corpus). The only
  place `provenance` lives.

This is the operator's cursor/windowed model: forward DFS from the DAG root, minimal context, resolve on
encounter, replacing "start from a requested module and walk backward, repeatedly, flattening ancestry."
Reverse-from-request + per-module flatten is what forced the copies; forward-DFS + one shared index + a
scope cursor removes the *need* for them.

### 3.1 CORRECTION — flat direct-import is NOT byte-identical; re-export transitivity is load-bearing (lively-raven-355, 2026-07-07, proven by execution)

The naïve reading of §3 — "resolve a name = locals, else imports → qualified name → the one index" as a *flat direct-import* lookup (a module sees only its direct imports' **own** exports) — is **NOT byte-identical on the live corpus.** A direct-import experiment (own-only module caches, ancestry = direct imports + kernel) was implemented, regen'd, and run against `regen --verify`; it **failed loudly** (the fail-closed fixpoint oracle working as designed) with unresolved `EmitResult` / `parse_with_table` / `default_artifact_plan` / `Rust` in `compile.dag` + `probe_emit_interp.dag`.

**Root cause — re-export transitivity.** Those names *are* directly imported (`compile.dag:29` `import v1.compiler.emit { EmitResult }`) but `v1.compiler.emit` does **not define** `EmitResult` — it imports it from `emit_core_support` (the definition, `emit_core_support.dag:17`) and **re-exports** it. The corpus relies on modules re-exporting their **entire visible surface** (own ∪ ancestry) and importers resolving names *through* the re-exporter. That "re-export everything" **IS** the transitive accumulation — the O(M²) itself.

**The separation that saves the reform (clever-koi adjudication, 2026-07-07):** re-export transitivity and the quadratic are **separable**. The O(M²) is the eager per-module union-**copy** of ancestry maps, *not* the semantics. So **import-scoped resolution = own bindings, else a walk of the import DAG (the re-export chain)** — the walk **memoized and Rc-shared** (per `(module, name)` or lazy per-module name→definer maps), **never unioned into importers**. `SymbolIndex` stores per-module **own** bindings only; the transitive surface is *derived at lookup, stored nowhere* (so increment-2's completeness receipt is just `index == Σ own bindings`, and the dual-representation risk drops out). Cycle-safety = §7.6 inv-1 (import DAG acyclic). **Precedence crux:** when a name is reachable via >1 branch, the walk's winner must reproduce the current union's exact `(fold-order over resolved_imports) × (map_merge overwrite direction)`; byte-identity (`regen --verify` + corpus emit) is the oracle.

**Staging (PR-2 vs PR-4):** import-scoped = own + re-export-chain walk is **PR-2's byte-identical contract** (changes nothing about what compiles). The corpus migration to **import-from-definer** (each `import` names the *defining* module, eliminating re-export reliance) belongs to **PR-4, the namespace-only-Y pivot** (`namespace-resolution-design.md`), where the semantic change always lived — the refutation above hands PR-4 its first concrete census rows: `EmitResult`, `parse_with_table`, `default_artifact_plan`, `Rust`. Operator holds veto on this staging (namespace design is operator-signed).

## 4. New/changed types (model-before-implement — these land first)

The split is BY CONSTRUCTION, not validation (operator 2026-07-05: "why guess — design it this way").
We do not trust that import bindings carry `SubValueUnknown`; we make a stray provenance *unwritable* by
giving the import population a type with no provenance field.

- `SymbolIndex` (new): `Map<QualifiedName, Node>` single authority. Filled once by the DFS prepass.
- `ImportBinding { name: String, resolved: Node }` — the import population. **No `provenance` field** — a
  provenance on an import is not "always Unknown," it is *unrepresentable*. (§5 correctness-by-construction.)
- `ScopeBinding { name: String, resolved: Node, provenance: SubValueRelation }` — the cursor window (params,
  match-bound sub-values). The ONLY carrier of provenance.
- `TypeEnv` loses `ancestry_str_bindings` (materialized copy) and the parallel String/Int keyings
  (`04_env.dag:34` dissolution). `parents` compositional view is subsumed by the scope + index.

### 4.1 `SubValueRelation` cleanup (operator: "Unknown isn't helpful — categorize further")

`SubValueRelation` (`std/induction.dag:60`) is already a lattice (`StrictSubValue`, `IteratedSubValue`,
`ArithmeticDescent`, `PreservedValue`, `NonIncreasingValue`, `StrictAxisErased`, `MixedTop`,
`SubValueUnknown`). The reason `SubValueUnknown` reads as unhelpful is that it fuses **two unrelated
states** (the §5 state-space-conflation antipattern):
1. **not-applicable** — a top-level/import definition, which is not a value with a descent relation at all;
2. **undetermined** — a local value whose descent relation the analysis genuinely could not compute.

The §4 provenance split ELIMINATES (1) by construction: imports no longer carry provenance, so nothing
top-level can land in `SubValueUnknown`. What remains is (2), the honest fail-closed "undetermined" — which
`std/termination.dag` already models as `DescentUnknown` (the fail-closed bottom). So after the split,
`SubValueUnknown` means exactly one thing.

FOLLOW-UP (needs termination-soundness review — `SubValueRelation` is std/load-bearing): if the remaining
`undetermined` cases are themselves distinguishable (e.g. "unanalyzed — recursion shape not yet handled" vs
"analyzed — provably no descent axis"), split them into named variants too. Not in the v1 hot path; a
separate std-induction PR so the termination checker's soundness is reviewed on its own.

## 5. Migration

1. **Design review** (this doc) — operator signs the model before any load-bearing edit.
2. **v1**: introduce `SymbolIndex` + `Scope`; rewrite `build_type_env`/`typecheck_modules` to fill-once +
   scope-resolve; delete `ancestry_str_bindings` materialization + the `union_parent_type_env_caches` /
   rewire cluster. lively-raven-355 owns this (inherited).
3. **v2**: apply the SAME model to `std.type_env` (the L31 target). One model, two realizations — authored
   so v2 is the durable home and v1 is a thin seed of it.

### 5.1 Status + dependency-ordered work (2026-07-06)

- **DONE** — #6306 demand-driven realize (module walk, exact closure §5.5-inv-2, memoized). `SymbolIndex` /
  `ImportBinding` / `ScopeBinding` / `Scope` + ops MODELED (`src/v1/04_env.dag:25-66`) but **inert**
  (`symbol_index_lookup`/`insert` called nowhere in `04_infer`). Content-addressed substrate exists in std
  (§5.5-inv-1). Interim `ancestry_cache_sharing` (borrow parent Rc-shared cache) landed (#6304/#6310).
- **GATE (blocks surgery — a measurement, not code)** — the scaling receipt (`reference_carrier_witness_test`
  @ 100/200/400/800 modules) + whole-corpus compile-clean wall-clock baseline, loyal-heron's deliverable.
  §7.5 profile says **emit_imports (78%) is the first consumer, `build_type_env` (17%) second** — so the
  receipt decides emit-first vs both-at-once AND confirms the target is superlinear not a fat constant.
  Second gate: the namespace pivot sign-off (the index is shared with the resolver companion, §header).
- **DEPENDENCY (parked — re-dispatch on the node/algebra defork)** — the index keys on `String`, which *works
  for the surgery* (the `entries: Map<String, Node>` / `symbol_index_lookup(qualified_name: String)` params are
  even named `qualified_name`: String is a projection of the real key, not the real key). The `QualifiedName`
  grounding — the §5.5-inv-1 content-hash-groundable / containment-path key — is v2.std-only today
  (`src/v2/std/qualified_name.dag`) and is **gated on the node + algebra two-std deforks** (operator-owned,
  `dag/gunbc/plans/dag_v2_defork_audit.dag`): `QualifiedName = FreeMonoid<Symbol>`, and `FreeMonoid` is the live
  `algebra` fork-census item while `Symbol`/`symbol_intern_lexeme` are v2-only, so **naive promotion re-creates
  the `dag/std/algebra.dag:115` shadow** (the Empty/Cons variant-drop when both trees co-occur) — doing it now is
  actively wrong, not merely low-value. Not blocking the perf wiring. Re-dispatch when that defork lands and a
  clean `std.qualified_name` home exists. (smart-ant-466 was tied only to this; archived without producing a
  branch/PR — the item is parked, not half-done.)
- **SURGERY (Phase 2, load-bearing — DESIGN-named; higher bar, escalate on doubt)** — fill the index
  (topo-prepass per §8, or fill-as-you-go per #6306 — receipt decides); replace `build_type_env`'s
  per-module `ancestry_str_bindings` materialization (`04_infer.dag:5726-5752`) with index lookups; wire the
  `Scope` cursor; delete `ancestry_str_bindings` + `union_parent_type_env_caches` + rewire cluster. **Note:
  its own validation (byte-identical fixpoint) is currently #6239-wall-blocked — the surgery cannot be proven
  green-by-execution until the wall is manageable or emit_imports is cut first.**
- **VALIDATION (§6)** — scaling curve N²→N; byte-identical emit fixpoint; `cargo test --workspace` + witnesses.
- **THEN → §5.5 runway** — concurrent memo + in-progress markers (intra-process parallel); content-hash key
  + per-unit determinism → the CAS (inter-process / RBE).

### 5.2 Floor-memory evidence and placement (sleek-ibex-207, cost lane; 2026-10-01)

**The §5.1 GATE is discharged at class grain.** The gate asked for a receipt that the ancestry
materialization is superlinear, not a fat constant. `//gunbc/instruments:typed-graph-exclusive-bytes`
and `//gunbc/instruments:typed-graph-exclusive-bytes-floor-subject` (gunbc#12850) read each
`TypedModule` class's EXCLUSIVE bytes leave-one-out at two closure sizes: the whole-tree strict closure
and the required floor's nominal prepared subject. The scored result sits on gunbc#12850 beside the
prediction written before the runs (DESIGN §6b). What it establishes, re-derivable by the two labels:
- `type_env` is the largest exclusive class at both sizes, and no other class is close.
- `type_env`'s exclusive bytes per `ancestry_str_bindings` entry agree across the two sizes (the
  structural prediction held), so its bytes ARE the per-module ancestry unions this plan targets.
- Both the bytes and the entry sum grow superlinearly in module count (the linear falsifier did not
  fire). The numeric band predicted for the exponent missed and is recorded as a miss there.
- `interface` and `emit_graph_info` were candidates on the sequential split
  (gunbc#12774 `typed_graph_byte_attribution`) and own almost nothing exclusively. gunbc#12832 cut
  `emit_graph_info` and the floor-memory-qualification A/B found the peak unmoved, so they are not
  this lever.

**Why it matters now.** The required floor's strict prepare is v1's `compile_to_resolved` over the
prepared subject, and its typed graph is the floor's peak (`gunbc.floor_demand`
`floor_phase_attribution`, the prepare-closure-resolve phase). At the larger subjects
(gunbc#12381, #12799) that peak pins the CI leaf at `memory.high` and the floor refuses
`MemoryStallRefusedPageThrash`. The residual after the demand-side lifetime fixes (gunbc#12774) is
this representation.

**Placement (the question this lane was asked).**
- **v1, PR-2's contract is a pure representation change.** §3.1 already established that flat
  direct-import resolution is NOT byte-identical (re-export transitivity is load-bearing), and that
  own bindings plus a memoized, Rc-shared walk of the re-export chain IS. That contract keeps every
  typed result and diagnostic. Locals > kernel > direct-selected > transitive union is preserved by
  construction, with the union's winner and the `binding_forks` ledger unchanged. So it is admissible
  under `gunbc.v1_maintenance_standing` `v1_seed_standing` on PURPOSE (it removes the floor's dominant
  superlinear memory on the v2 self-host path), the same admission as gunbc#12832.
  - Its proof: the stage0 regen fixed point (`claim_executor --regen-round-cost`), plus a floor
    differential (same subject digest, every claim outcome equal) on the real pool.
  - The PR-4 import-from-definer migration is the SEMANTIC change and stays with the namespace-cut
    program (`namespace-cut-replacement-plan.md`). This lane does not take it.
- **The 2026-07-06 one-level-read invariant** (`04_env.dag`: `flatten_visible_bindings` and
  `merge_envs` deleted) forbade a per-lookup walk standing as a FALLBACK beside the materialized map.
  PR-2's memoized walk REPLACES the map as the single representation, so it is that invariant's
  intent (no second representation), not its violation. Because it re-introduces a walk, it needs the
  operator's explicit re-ruling, and this addendum asks for it.
- **v2 does not inherit the representation.** v2's typecheck path resolves through
  `v2.compiler.name_resolve`'s shared `ResolutionContext` and the namespace containment tree. There is
  no per-module union, and the `v2.std.type_env` target named in §5 step 3 does not exist. So the fix
  is v1-only, and its cost is justified by v1's remaining lifetime as the required floor's strict
  typechecker. That lifetime ends when the floor's prepare runs on the v2 resolve: the self-host
  frontier for `v2.compiler.compile`, with no scheduled date. Until then every floor at #12799 size
  pays it.

**PR-2's scope is ONE union with TWO holders, and both go.** The per-module import union is
materialized as `TypeEnv.ancestry_str_bindings` AND as the module's `TypeEnvCache`.
`build_type_env` sets `cache_str_bindings = map_merge(ancestry_str_bindings, str_bindings)`, and
`union_parent_type_env_caches` builds every importer's union by folding its parents' `interface.cache`.
So the cache is the vehicle of transitivity, across the four maps `union_base_choice_note` names:
`str_bindings`, `deps_map`, `variant_locals` and `cycle_set_str`.
- Deleting the ancestry map alone would leave its nodes alive through the merged cache, which shares
  its spine. That is gunbc#12832's lesson: removing one holder of bytes another still holds saves
  nothing.
- So PR-2 dematerializes the ancestry map and all four flattened cache unions TOGETHER, behind the one
  walk. This is the same union the operator's 2026-10-01 re-ruling covers, not a second change.
- The saving to predict is therefore what that SET holds jointly. The two-size leave-one-out reading
  of the joint class (gunbc#12850's prediction 2) re-bases the figures below before any PR-2 code.

**The re-export walk is a materialization, so its identity is stated** (DESIGN §2,
`std.materialization_ladder`). This is what tells it apart from a cache placed at a symptom.
- **Value.** The binding a name resolves to through ONE module's export surface: its own bindings,
  then its imports' surfaces in import order, under the declared precedence.
- **Key.** (pool identity, module identity, name).
  - Pool identity is the one strict resolve (one `compile_to_resolved` over one source set, one
    kernel cache).
  - Module identity is its module path AND its source file. The census lesson of gunbc#12774 applies:
    the diff base's checkout is a different module under the same path.
- **Why the key is complete.** Within one pool, a module's export surface is a function of exactly:
  its source (own declarations and its import list, with each import's order, `is_all` and
  `specific_names`), its imports' surfaces (inductively, the same key one level down), and the
  pool's kernel cache. The pool identity fixes the source set and the kernel cache, and the module
  identity fixes the source. No other input reaches it. The exporter-count non-hermeticity
  (gunbc#12815) lives in the rewire's pass 2, which runs after assembly and is not an input here.
- **Scope.** Every reader of a module's surface within one prepared pool. That is NOT only the
  typecheck: evaluation reads the same map. `v1_interpreter` `resolve_coproduct_type_node` reaches
  `v1_compiler_infer_env` `lookup_type_by_name` -> `lookup_binding_by_name` ->
  `lookup_binding_on_chain`, which reads `ancestry_str_bindings`. So the least common ancestor of
  the demands is the prepared repository (the floor's `prepared`, held by `run_required_floor`
  through claim evaluation), not the resolve. The walk is never shared across pools or graphs until
  gunbc#12815's declared-scope fix makes surfaces graph-independent.
- **Retention.** The prepared pool's lifetime: the same lifetime the materialized maps have today, and
  no longer. The memo therefore lives through evaluation, so its size is the figure that decides the
  saving. The 10-20% assumption below is unmeasured, and the A/B reads it directly.

**The union winner and the fork ledger stay identical, and the differential checks both.**
`union_parent_type_env_caches` folds a module's imports IN IMPORT ORDER through
`merge_type_env_cache_guarded`. The later import wins, the kernel overlay is skipped for kernel
names, and the direct-selected overlay applies last.
- **The walk must use the same order and the same overlay rule**, so the winner for every
  (module, name) is the materialized map's winner.
- **The `binding_forks` ledger stays EAGER.** Forks are recorded while the union is folded, so a lazy
  walk that resolves only the names actually looked up would record FEWER forks and silently thin
  the ledger. PR-2 therefore keeps a per-module conflict pass over the imports' surfaces, in import
  order, that records the same rows in the same order without retaining the union map. The memory
  is saved; that pass's time is not, and the note does not claim it.
- **The floor differential compares the `binding_forks` rows (contents and order) for every module**,
  as well as the subject digest and every claim outcome. Equal claim outcomes alone would not catch a
  thinned ledger.

**Re-based after gunbc#12850's prediction 2 (the joint class at two sizes). The shape premise did not hold.**
Earlier versions of this note argued PR-2's case from a SUPERLINEAR shape: deeper modules carry
larger import closures, so the union's bytes grow faster than the module count. The measurement
falsifies that premise for bytes.
- The ancestry ENTRY count is still superlinear between the two sizes (the `ancestry_entries` field
  of both labels' lines).
- The joint holder's BYTES (`type_env+type_env_cache+interface`, read jointly by
  //gunbc/instruments:typed-graph-exclusive-bytes and its -floor-subject row) scale close to
  linearly. Bytes per entry therefore FALL as the closure grows, so the extra entries a deeper module
  adds are increasingly shared spine. The two-size exponent and the per-module figure are on the
  scoring comment for PREDICTION 2 on gunbc#12850, re-derivable by those two labels.
- So the case no longer rests on shape. It rests on SIZE: the joint holder is the majority of the
  typed graph at both sizes (the joint class's `exclusive` over `graph_total`, same labels), and it
  grows with the corpus about linearly.
  The operator's 2026-10-01 re-ruling of the one-level-read invariant was argued from the
  superlinear shape, so it is returned for re-confirmation on this basis before any PR-2 code.

**Predicted saving, re-based, stated before any build. These are inferences from the two-size
reading (base: the joint class's `exclusive` at both labels, extrapolated at the two-size exponent),
and are labelled as such.**
- The joint figure extrapolates to about 7.3 GB at a ~2.7k-module subject. At the whole tree the
  base is the -whole-tree label's joint reading itself.
- The walk's memo is assumed to hold 10-20% of that. The assumption is not measured, and an AFTER
  saving under 75% of the joint figure falsifies it.
- So the saving is about 5.9-6.6 GB at ~2.7k modules, and about 17-19 GB at the whole tree.
- **At the floor's peak, the relevant figure is smaller and conditional.** The leave-one-out reading is
  taken at the compile peak, where `TypeEnvCache` is still present. gunbc#12774 drops the cache
  before evaluation, so at the evaluation-time peak the saving is the ancestry map's share, about
  the joint figure less the cache's and interface's single exclusives: about 5.7-6.4 GB at ~2.7k.
  That figure lands at the run peak only IF the prepared typed graph is live at the seam where
  the peak now sits. After gunbc#12890 that seam is claim evaluation or discovery authority. Whether
  the graph is live there is NOT yet read from the chain. It is the first thing PR-2's chain read
  establishes.
- **Predicted AFTER at the gunbc#12381 subject** (about 2.7k modules; peak measured at
  //gunbc/instruments:floor-memory-qualification after gunbc#12890): the measured peak less
  5.7-6.4 GB IF the graph is live at that seam, and unchanged otherwise.
- **Falsifiers.**
  - An AFTER peak saving under 4 GB at that subject, with the graph shown live at the seam, falsifies
    the claim that the union is what PR-2 removes.
  - A graph not live at the peak seam means PR-2 does not move the floor's peak at all. Its case
    would then be the compile-time peak only, which re-opens whether the work is warranted.

**What PR-2 does NOT do, and what is left. (Written against the earlier prediction; the re-based
figures above supersede its arithmetic, and this list is kept as the candidate set only.)** By the earlier prediction, PR-2 alone probably did NOT
retire the 41G floor slot class. So the stopgap's trigger is not met by the planned work alone.
Closing the remaining ~1-3 GB at ~2.7k modules has these candidates, each needing its own chain read
before it is proposed:
- **`occurrence_transport`** is the next class by exclusive bytes that may be typecheck-time only.
  Its readers outside inference are the resolve input (`v1_compiler_resolve`) and
  `v1_compiler_frontend_observation`. If nothing on the floor's route reads it after the strict
  resolve, it is a lifetime cut of the same shape as gunbc#12774's `TypeEnvCache` drop. Unverified.
- **`module_nodes` and `func_env`** are read by evaluation, so they are not lifetime cuts.
- **The shared remainder** (the leave-one-out `shared_or_unlisted`) is not yet broken down, so it is
  not yet a candidate.
If those do not close the gap, the 41G floor class becomes a STANDING cost of v1's remaining lifetime
as the floor's typechecker. A stopgap with no reachable trigger is no longer a stopgap, so that
re-classification is the operator's decision, not this lane's.

**Order of work:** the uncensored BEFORE (floor-memory-qualification under MemoryMax=96G at the
#12799 subject) runs FIRST, because it may move the whole estimate. Then this sign-off chain, then
PR-2.

**Sign-off chain** (a load-bearing v1 typecheck representation): jolly-boar-500 → neat-boar-16 → the
operator, who also re-rules the one-level-read invariant above. deep-ferret-305 reviews the floor-side
consequences. No code lands before that chain; the first code is the uncensored BEFORE measurement.

## 5.5 Distributability invariants (map-reduce / RBE endgame — operator 2026-07-06)

The intra-process node-level memo is not a terminal design — it is **shard 0 of a content-addressed, distributable realize** (map-reduce / Bazel-RBE shape: infinitely scalable, across-process/host irrelevant). RBE reduces to one shape: *each realize-unit is a pure function of content-addressed inputs → a content-addressed result in a shared CAS*. Then **map** (realize the ready frontier) and **reduce** (merge results into the index/CAS) are the same operations whether workers are threads, processes, or hosts — only the CAS transport changes. So "across process/host is irrelevant" holds exactly when three invariants hold; we commit to them NOW so wiring the intra-process memo cannot foreclose the endgame (cheap now, a re-architecture later):

1. **Content-hash-groundable key.** The `SymbolIndex` key is `String` (qualified name) intra-process, but must be *groundable to a content-hash* so results are location-independent and dedupe across processes. The substrate already exists (`std.content_hash`, `std.cache_identity`, `std.cache_interface`, `std.realization_schedule`; the DAG nodes are already content-addressed). Build the index as one shard of the CAS keyed by a projection of that hash — not a name-only map that forks when distributed.
2. **Exact minimal input closure per unit.** A superset digest = spurious cache misses + poisoned dedup. **#6306 already delivers this** ("exact dependency closure, not topological superset") — it is the RBE-critical property and it is landed.
3. **Per-unit determinism.** Each realize-unit a pure deterministic function of its input digests. The whole-compile byte-identical fixpoint (§7 self-host) proves it at *corpus* level; RBE needs it *per-unit* — the live thread is the determinism construction gate (`src/v2/lens/determinism.dag` / DESIGN.md §5 open thread; #5941 P1 → #6594).

The demand-driven shape #6306 already has **is** the map-reduce shape: the modules whose deps are all realized are the wavefront. `fold → thread pool → RBE workers` is a scheduler swap; `local Map → content-addressed CAS` the only backend swap. The reform must keep those two seams clean (a `RealizationBackend` the fill writes through, a scheduler the driver folds through) rather than inlining `HashMap` + a serial fold as load-bearing assumptions.

## 6. Validation (§5 prove-by-execution)

- **Scaling curve**: `--target dag` time on module subsets (~100/200/400/800) must go from ~N² to ~N.
  This is the discriminating receipt — a flat-per-module curve is the goal, O(M²) is the red control.
- **Byte-identical emit fixpoint** (`bootstrap_fixed_point`): behavior-preserving — the reform must not
  change a single emitted byte.
- `cargo test --workspace` green; the resolve/infer witnesses green.

## 7. Coordination (build_type_env is multi-lane contention)

- **Subsumes #6239 (loyal-heron, P3/P5)**: it optimizes the per-module ancestry SCANS — the very structure
  this reform DELETES. Merging it is throwaway. RECOMMEND: pause #6239; redirect loyal-heron to (a) finish
  the empirical profile (confirms the O(M²) magnitude before we invest) and (b) implement this reform with
  its infer context.
- **Parallel, non-conflicting**: the emit-axis lanes — ownership de-fork (snappy-newt #6249), restoration
  → gen-2 (proud-moth), get-fix/hotfixes — touch emit/regen, not the type-env model. They continue.
  Landing order coordinates only at regen.
- **v1-burndown (sharp-deer)**: this reform IS v1-burndown-aligned (it's the resolve model the burndown
  wants gone) — coordinate so v2's `std.type_env` is the shared target.

## 7.5 Profile redirection (loyal-heron partial profile, 2026-07-05)

85-source self-compile subset: Tokenize 274ms · Parse 3.19s · **Resolve 63ms · Reconcile 12.46s · Emit
58.34s** · Total 74.3s. So **emit ≈78%, typecheck/reconcile ≈17%, resolve <1%.** This CONTRADICTS the
earlier "~710s resolve+typecheck dominates" grounding (likely stale / different fixture / pre-#6242).

IMPLICATION: the dominant O(M?) cost is **emit — `emit_imports`** (the per-module transitive re-export
closure recompute, audit finding #1), NOT `build_type_env`'s ancestry materialization (reconcile, the 17%).
BUT it is the SAME fix: both re-derive the import closure per module, so the `SymbolIndex` single authority
is consumed by BOTH — the **first realization should target `emit_imports`** (the 78%), then
`build_type_env` (the 17%). Design unchanged; the ORDER of consumers flips to emit-first.

CAVEAT: 85-module RATIOS ≠ 876-module SCALING. Which stage is O(M²) vs high-linear is settled only by the
scaling curve (100/200/400/800), loyal-heron's next deliverable. Do NOT start surgery before it lands — it
decides emit-first vs both-at-once and confirms the target is superlinear, not a fat constant.

## 7.6 Preserve-invariants the reform MUST NOT break (loyal-heron infer review)

These are RED controls — each must stay green through the reform (existing witnesses noted):
1. **Import-DAG cycles**: resolve topo-sorts and EXCLUDES cycle participants (acyclic_resolved only). The
   `SymbolIndex` prepass must use the same `ModuleGraph.modules` order; do NOT index cycle modules as if
   parents exist; mirror `import_diags` / missing-parent fail-closed.
2. **Forward refs within a module**: NOT solved by inter-module `SymbolIndex`. `build_type_env` still needs
   local `detect_type_cycles_kahn` + `topo_resolve_types` over the local `deps_map`. The scope cursor
   carries local `str_bindings`; `SymbolIndex` is qualified cross-module lookup ONLY.
3. **Multi-import overlay**: today ancestry is single-parent copy when |imports|==1 else
   `merge_type_env_cache` union. Reform must preserve overlay semantics (kernel + import overlay-wins), not
   assume a single parent chain.
4. **std.types filter**: `type_env_for_import` strips type-variable names — easy to miss in a flat index;
   needs an explicit rule or qualified entries.
5. **Single-exporter canonical pick** (`rewire_...`: last `TypedModule` in list wins): reform must not
   silently change import-order authority; multi-exporter names defer to overlay-wins.
6. **variant_surfaces re-export** (P3+P5 incremental, own-wins): keep or subsume; the `E = A | B` re-export
   chain witness (kept by #6239) is the RED control for re-export semantics.

## 8. Resolved decisions (operator, 2026-07-05)

- **provenance split**: design it unwritable (§4), do not audit-and-trust. DONE in this doc.
- **SubValueUnknown**: categorize further; the split removes the not-applicable case for free, residual
  undetermined refinement is a separate termination-reviewed std PR (§4.1). DONE.
- **`SymbolIndex` fill**: **topo-order prepass over the import DAG**, keeping the cursor PURE (the latter
  option) — assuming the prepass is not itself a perf problem on large graphs. GUARD: the prepass is one
  pass over the import DAG (O(V+E)); the scaling receipt (§6) must show the prepass stays linear, else
  reconsider fill-as-you-go. The cursor never mutates the index — it only reads it + pushes/pops local scope.
- **#6239 (loyal-heron)**: NOT force-paused. Operator: "if they finish it and we delete it, it's fine."
  Inform loyal-heron the reform subsumes it; let them choose to finish or pivot. Their profile is still
  wanted regardless.
- **v2 sequencing**: apply to `std.type_env` after v1, **gated on v1's green scaling receipt** — "see what
  happens with v1" first, then port the proven model.

## PR-2 site census (chain read, 2026-10-01, at main)

The union reaches PR-2 in FIVE representations, not two. All five live on every module's
`TypedModule`, so all five are inside the joint class the leave-one-out reading measured.
1. **`TypeEnv.ancestry_str_bindings`** (04_env `TypeEnv`). It is built by `build_ancestry_precedence`
   (import union -> kernel overlay -> `overlay_direct_import_exports`).
   - Point readers: `lookup_binding_on_chain`, which typecheck reaches through `lookup_binding_by_name`
     and evaluation reaches through `v1_interpreter` `resolve_coproduct_type_node`.
   - Whole-map readers: the rewire pass `rewire_type_env_import_str_binding_identity` (enumerates every
     key, then `map_merge`s its `rewrites` back into the map), `type_env_for_import` (std.types only,
     filters type variables out), and `source_visible_names` (enumerates an is_all parent's ancestry keys).
2. **`TypeEnvCache.str_bindings`** = `map_merge(ancestry, own)`, handed to importers as
   `interface.cache` and folded by `union_parent_type_env_caches`. This is the vehicle of
   transitivity.
3. **`TypeEnvCache.deps_map`**, the union of every ancestor's dependency rows. Merged with local rows
   into `all_deps_map`, it feeds `detect_type_cycles_kahn` and `resolve_env_bindings`.
4. **`TypeEnvCache.cycle_set_str`**, the union of every ancestor's cycle names. It is re-materialized
   per module as `recursive_types`, `recursive_type_set` and the inductive set, keyed by intern id.
5. **`TypeEnv.source_visible_names`**: for each is_all import, the parent's own AND ancestry names, as a
   key set.

**Per-site disposition. This is the design PR-2 implements; each line is a claim the differential checks.**
- (1) and (2) become ONE walk, keyed by the identity stated above.
  - Point lookups call it.
  - The rewire's `rewrites` stay a per-module OVERLAY consulted before the walk. They are not merged
    into it, because a rewrite is a per-module fact and the walk serves every importer.
  - The std.types filter becomes a predicate applied at the walk's answer for that one module path.
  - The rewire's key enumeration and `source_visible_names`' ancestry enumeration become an ENUMERATING
    walk. It is transient: computed where it is demanded and not retained. Its time is not saved and the
    note does not claim it. The evaluation-step gate measures it.
- (3) The deps union is consumed only by cycle detection and binding resolution, which run inside
  `build_type_env`. It becomes a walk demanded there and is never retained on the module.
- (4) and (5) are union-sized SETS. They are in the same class and are NOT named by the 2026-10-01 ruling's
  wording ("map and cache unions"). Including them is a scope question for the lane manager. Excluding
  them leaves part of the joint class's bytes in place, so the predicted saving must then be measured
  against the narrower deletion.
- The `binding_forks` ledger stays eager (see above): the per-module conflict pass runs inside
  `build_type_env` over the imports' surfaces, in import order.

**Iteration constraint.** `04_env.dag` and `04_infer.dag` are emitted into stage0 Rust
(`v1_compiler_infer_env.rs` and its siblings, "Generated by v1 compiler"). The regen fixed point cannot be
produced on a session host or on the remote runner (HostBudgetUnreadable); it has so far run only on srv1.
So PR-2 is authored in `.dag` with the emitted Rust mirrored by hand for compile-and-test iteration, and the
srv1 regen is the adjudicator: its output replaces the hand mirror wherever they differ.

## PR-2 representation: exporter index plus closure reachability (B), with its equality argument

The ruling names one representation with the unions deleted. A MEMOIZED walk has no realization in v1:
the typechecker is pure `.dag` emitted into stage0, and no fold there consumes a materialization
provider. A walk without a memo revisits shared ancestors across diamonds, so each miss costs
O(closure). This section therefore replaces the memo with structure that makes most lookups O(1) and
bounds the rest. It is a representation change within the ruling's intent (one representation, the
unions deleted), decided by the lane manager and neat-boar-16, 2026-10-01.

### The fold being replaced, stated exactly (`04_env.dag`, `04_infer.dag` at main)
Write `own(P)` for module P's own `str_bindings` at interface build, BEFORE the rewire pass. Write
`surf(P)` for `P.interface.cache.str_bindings` = `map_merge(anc(P), own(P))`, so own wins. For a
module M with resolved imports I_1..I_k in import order:
- **U(M)**, the import union `union_parent_type_env_caches`: a left fold over surf(I_1..I_k) through
  `guarded_union_str_bindings`.
  - If the authorities DIFFER (`binding_same_authority` is false, i.e. different declaration span and
    structurally unequal), the overlay (later import) wins and a fork row is recorded.
  - If the authority is the SAME, the retained side's copy is kept. Which side is retained depends on
    the size-based orientation, but the declaration is the same.
- **K(M)**: `merge_type_env_cache(U, kernel)`. The kernel wins every key it carries.
- **anc(M)**: `overlay_direct_import_exports(K.str_bindings, ...)`.
  - For each import J in order, the selected names are: if `is_all`, J's export-surface names minus
    type variables (and, for std.types, the type-variable filter of `type_env_for_import`); otherwise
    `specific_names`.
  - Each selected name is inserted from J's EXPORT SURFACE `str_bindings`, which is own(J).
  - Names in `overlay_skips_kernel_name` are skipped.
  - Later J wins.
- **Lookup at M** (`lookup_binding_on_chain`): rewired own(M) first, then anc(M) with the rewire's
  `rewrites` merged over it.

### The new representation
- **E**: the pool's EXPORTER INDEX, name -> the set of modules whose own(P) carries the name. Its
  size is the sum of OWN entries.
- **The kernel** is a distinguished exporter.
- **R(M)**: per-module CLOSURE REACHABILITY, the set of modules in M's import closure, as module
  indices. It is built in topological order as R(M) = U over j of ({I_j} plus R(I_j)). This is a set of
  module ids, not names.
- **A surface node per module**: own(M) (pre-rewire), its ordered imports with their selection data
  (`is_all`, `specific_names`, the std.types flag), R(M), and the rewire overlay. These are shared by
  reference, never copied.
- **Lemma (presence).** n is in keys(surf(P)) iff n is a kernel name, or some E in E[n] lies in
  {P} plus R(P).
  - Proof: by induction on the topological order. surf(P) = own(P) plus anc(P); keys(anc(P)) =
    keys(U) plus kernel plus the selected direct names; and every selected direct name comes from
    own(J) with J in R(P). So keys(surf(P)) = own(P), plus kernel, plus the union over j of
    keys(surf(I_j)). The induction hypothesis on each I_j gives the claim.
  - The std.types filter only removes names from the DIRECT overlay. The union still carries them,
    so presence is unaffected.

### The winner, and the proof sketch per case
`win(M, n)` is the binding anc(M) carries for n, computed as follows:
1. If n is in `overlay_skips_kernel_name` and n is a kernel name: the kernel binding. (K wins, and the
   direct overlay skips it.)
2. Else, if some direct import selects n and own(J)[n] exists (after the std.types filter for J =
   std.types): own(J)[n] for the LAST such J in import order.
3. Else, if n is a kernel name: the kernel binding.
4. Else, the union winner. Take the LAST import I_j (scanning k down to 1) with n in keys(surf(I_j)),
   tested by the presence lemma, and return sv(I_j, n) = own(I_j)[n] if present, else win(I_j, n).
- **Fast path:** if n is not a kernel name and E[n] = {E} is a single exporter, then win(M, n) =
  own(E)[n] whenever E is in R(M). Every surface containing n carries own(E)[n], or a same-authority
  copy of it, since there is no other source. So the descent can only end there.

Cases:
- **Diamond** (I_a and I_b both reach a common ancestor C that exports n). U's fold takes the later
  import's surface value. Step 4 takes the LAST I_j by presence and descends into it. Both arrive at
  C's binding or at the same-authority copy, so the authority is equal. The VALUE copy can differ only
  when the fold retained the earlier side for a same-authority key (orientation). That is the existing
  orientation case below.
- **A re-export of a re-export** (n is declared at depth d and reaches M only through intermediate
  surfaces). surf(I_j) carries n by the lemma. sv descends I_j -> ... and at each level selects that
  module's own winner by the same rule, which is exactly how surf(I_j) was folded. So, by induction on
  depth, sv(I_j, n) = surf(I_j)[n].
- **A name exported at several depths** (two exporters E1 and E2 both in R(M)). The fold's winner is
  the value carried by the last import whose surface has n; inside that import, recursively, the same
  rule. Step 4 is that rule. No 'nearest exporter' heuristic is used: depth never decides, only import
  order at each level, as in the fold.
- **Kernel shadowing.** K = U overlaid by the kernel, so for a kernel name the union is irrelevant
  unless the direct overlay applies. That is steps 1 and 3. A kernel name in `overlay_skips_kernel_name`
  ignores even the direct overlay (step 1). Other kernel names take the direct overlay first (step 2).
  Both match `build_ancestry_precedence`.
- **Local shadowing.** Lookup at M reads rewired own(M) first and anc(M) second, unchanged. surf(P)
  for importers reads pre-rewire own(P) first. That is `map_merge(anc, own)`: own wins.
- **is_all parents.** The selection in step 2 is J's export-surface names minus type variables, read
  from own(J) and filtered. specific imports use `specific_names`. Both are carried on the surface
  node. The union in step 4 ignores selections because U uses whole surfaces, which the lemma reflects.
- **The rewire overlay.** `rewrites` are consulted at M's lookup only, before win(M, n), and are
  never visible to importers. The current code also builds importers' surfaces from the pre-rewire
  interface.
- **Orientation (the only value-level freedom).** When two same-authority copies exist, the current
  fold keeps the retained side's copy, chosen by map SIZE. The walk returns the copy on its descent
  path. These are equal at identity grain (same declaration span) by `binding_same_authority`'s
  definition. The differential therefore compares at identity grain AND reports every
  structurally-unequal same-span pair as its own row, so a case where the copies would typecheck
  differently is surfaced rather than equated.

### Fork rows
A row arises at M when U's fold meets n in surf(I_k) with a different authority than the
accumulated winner from imports before k. By the fast path, a name with one non-kernel exporter has
one authority everywhere, so it produces no row.
- Rows therefore come only from MULTI-EXPORTER names: |E[n]| > 1, or the kernel plus at least one
  module.
- PR-2 computes them eagerly per module: for each such name present in two or more of M's imports
  (presence lemma), fold over imports in order with the current winner, and emit
  (name, import_path, existing site, incoming site, span, same_tree) exactly where the guarded union
  would.
- The rows are identical as a MULTISET. Row ORDER today follows `map_keys` over a hash map, which is
  host-ordered and already not deterministic across processes (see #12895's RFM row). The ledger's
  consumers are count- and partition-grain (`union_base_choice_note`). So the differential compares
  the multiset, and sorted order is the canonical presentation.

### The other three representations
- **deps_map union.** The same winner rule (overlay wins when different, no rows), over a deps
  exporter index keyed like E. It is demanded inside `build_type_env` only (cycle detection,
  `resolve_env_bindings`) and never retained on the module.
- **cycle_set_str union.** A presence predicate: n is a cycle name at M iff n is in the compiler
  recursive set, or some E with n in newcycles(E) lies in {M} plus R(M). The derived `recursive_types`,
  `recursive_type_set` and inductive set have union-grain consumers. Each consumer is converted to the
  predicate where it only tests membership. Where one enumerates, the enumeration is computed
  transiently from the cycle exporter index restricted to R(M). Cycle names are few, so this is small.
- **source_visible_names.** A membership predicate built from the same lemma: n is visible iff it is
  local, a kernel name, selected by a specific import, or present in an is_all parent's surface. Its
  readers (`map_has` and `map_is_empty` in the visibility wall) are membership tests.
- **interface.env and interface.cache** carry the same union again. The floor's strict-refused byte attribution (the
  `[floor-heap] bytes label=strict-refused` lines that `typed_graph_byte_attribution` prints, read on
  #12900's CI run of 2026-10-01) frees a large `interface` class after `te.ancestry_str_bindings`. Both are replaced by the
  surface node: importers read the node, not a copied map.

### The differential that checks it
On the real pool, for both representations:
- every (module, name) that the strict typecheck AND evaluation actually look up, recorded at
  `lookup_binding_on_chain`, compared at identity grain (declaration span plus module path), with
  structurally-unequal same-span pairs reported;
- every module's fork rows as a multiset;
- the deps winners and cycle membership for every name the cycle detection demands;
- every claim verdict, and the producer-grain warm digests, which are deterministic after #12895.
The lookup set is recorded by an armed probe in the BEFORE arm, then replayed against the AFTER
representation, so the comparison covers exactly what the program reads, not a sample.

### Cost bound per lookup
- An own(M) hit is unchanged: one map get.
- A single-exporter name (the common case): one E[n] get plus one R(M) membership test.
- A multi-exporter or kernel name: O(D x k x X) membership tests in the worst case, where D is the
  import-chain depth descended, k the imports scanned per level, and X = |E[n]|. It stops at the first
  level whose own map carries n.
- These are map operations in emitted Rust (the typecheck) and evaluation steps in the interpreter
  (`resolve_coproduct_type_node`). The A/B reports both against the BEFORE arm's single map get, as
  the lane manager's gate (3) requires.
- FALSIFIER: if multi-exporter lookups dominate and push the strict resolve's or the floor's step
  total beyond noise, the representation fails the gate as stated. The remedy is then a per-module
  table for MULTI-EXPORTER names only (homonyms plus kernel names, a fraction of the union), named as
  its own decision, never added silently.

### Conditions of confirmation (neat-boar-16 via the lane manager, 2026-10-01)
1. **One authority for "who exports n".** The exporter index is derived from own entries in ONE
   function, built with the pool and rebuilt only when the pool is rebuilt. Every other reader (the
   fast path, presence, fork detection, the deps and cycle indexes) consumes it and never re-derives it.
2. **The winner reuses the fold's rule.** Kernel and local shadowing, re-export, is_all selection and
   the rewire overlay are evaluated by functions shared with today's construction:
   `overlay_skips_kernel_name`, the direct-selection predicate (extracted from
   `overlay_direct_import_exports`), `binding_same_authority`, `type_env_for_import`'s std.types
   filter, and the rewire's overlay. The PR cites each one. Only the ITERATION changes, from folding
   whole maps to descending for one name. Wherever a rule cannot be shared, the PR says why beside it.
3. **Ambiguity refuses exactly as today.** The replay differential compares winners AND refusals at
   identity grain, including the `listed_import_required_bare_call_blocked` refusal and the
   ambiguous-bare cases.
4. **Fork-row order: one canonical order.** Rows sort by their content fields compared
   byte-lexicographically: name, import_path, existing_site, incoming_site, then the span. That is the
   order #12895's `portable_value_cmp` gives a record of those fields (strings by bytes, fields in
   declared order). A third ordering is not introduced. If the map-rendering fix adopts a different
   comparator, both are reconciled to one before PR-2 lands.
5. **Reachability size.** One bit per (module, module) pair: about 7.1k² bits, about 6.3 MB, at the
   whole tree, and about 0.65 MB at a 2.3k-module subject. It is built in ONE pass over the closure in
   topological order (R(M) = the union over j of ({I_j} plus R(I_j))). If `.dag` lacks a dense bitset,
   the per-module set of module indices is the fallback, and its size is measured and reported, not
   assumed.
6. **The falsifier gets a control in the A/B.** A synthetic pool whose shape maximizes descent:
   - a deep chain of re-exports with a homonym exported at every level;
   - k imports per module, all of them reaching it;
   - the lookup count and membership-test count measured at two depths.
   Membership tests that grow faster than D x k x X between the two depths red the control.

### Scope added by the lane manager (2026-10-01): one declarer pool, two namespaces
The runtime interpreter needs the same relation PR-2 builds: which modules declare a name. Its scope
indexes (`v1_interpreter` `PreparedScopeIndexes`) are materialized over a frame's whole closure,
and a pure-producer warm reads a small share of them (the frame-demand reading on gunbc#12890). So
PR-2's pool is keyed by (name, NAMESPACE) and admitted in ONE place, `surface_pool_admit`, with two
namespaces:
- **type bindings:** each module's own `str_bindings`, as above;
- **runtime items:** each module's authored items, the population `derive_module_scope_fragment`
  walks (services, data and transports included; parameters never).
The interpreter's scope then resolves each DEMANDED name from that pool: filtered by scope membership,
ordered by the scope's precedence, and memoized per frame at the size of demand. It never
materializes a whole-closure index.
- That retires the frontier row `gunbc.resolver_cost_frontier` (path `required_floor_runner.rs`),
  the CPU residual gunbc#12890 accepted as a stated trade.
- `bare_item_registry`'s host-ordered last-write-wins leaf projection becomes the declared scope
  precedence. That is a semantic change fixing nondeterminism, so it joins the population of
  `gunbc.recurring_failure_mode` `portable_value_map_order_is_process_random`. Every leaf with more
  than one in-scope declarer is reported as a differential row, and any differing claim verdict
  stops the change and is reported by name.
- The A/B for this part is paired thread CPU (two runs per arm), plus a differential at identity
  grain of every name the BEFORE arm looked up.

## PR-2 implementation status (2026-10-01, at wind-down)
Branch `sleek-ibex-207/type-env-surface-walk` (no PR yet), stacked on gunbc#12895. Part (a) is
implemented:
- `04_env.dag`: `SurfaceImport`, `ModuleSurface`, `SurfacePool` (exporter index, multi-exporter set,
  ordinal map), `AncestryView`, `ancestry_lookup`, `ancestry_winner` (one descent),
  `surface_fork_rows` (canonical order, with a closure-declarer prefilter), and `ancestry_names`.
- `04_infer.dag`: the realize loop admits each module into the pool. The final pool is attached
  before the rewire (`modules_with_final_surface_pool`, which recomputes each view's closure bitset).
- Encoders strip the pool, decoders re-attach it, and the format versions are bumped (resolved-graph
  cache 6, shared typecheck store 2).
State:
- **Regen fixed point:** holds (`first_generation_equal=true`, `changed=0`).
- **Declaration-grain differential:** BEFORE (`sleek-ibex-207/pr2-diff-before`) vs AFTER
  (`sleek-ibex-207/pr2-diff-after`) over two closures (634 modules): 0 differing ancestry digests,
  0 differing fork rows. The whole-tree run (~7k modules) is requested on srv1
  (`pr2_whole_tree_differential_probe`, `GUNBC_ANCESTRY_DIGEST=1`).
- **Permanent controls:** `cli_run` `surface_view_controls` (a diamond lattice lookup is linear in
  depth; a view re-pointed across pools recomputes its closure). Both are red without their fixes.
- **Still owed before a PR:**
  - part (b) (the deps union) and part (c) (`source_visible_names`);
  - the seed-growth receipt for the controls;
  - a merge with main (the base predates gunbc#12850 and gunbc#12890), which needs a regen round;
  - the A/B against the lane manager's gates (1)-(4).
  The runtime-item namespace, the demand-sized frame and the cycle set are PR-2b.
**Regen recipe used (BuildBuddy, one round per dispatch under the 1 h free-tier cap):**
`CTRL_BUILD_RUNNER_EXEC_PROPERTIES="EstimatedMemory=60GB" ctrl-build --remote --timeout 58m -- bash -lc '<script>'`.
The script, in order:
1. Check out the pin.
2. `cargo build --release --bin claim_executor`.
3. Bind a 48 GiB cgroup v2 leaf (mkdir `/sys/fs/cgroup/gunbcrun`, write `memory.max`, then move the
   subshell into it before exec).
4. `claim_executor --required-regen --source-root dag --source-root src/v2`.
5. Install every `target/stage0-regen-candidate/src/*.rs` that differs into `src/v1/stage0/src`.
6. Emit `git diff -- src/v1/stage0/src` as gzip+base64, to apply locally.
Bootstrap rule: hand Rust that calls newly emitted functions is withheld for one round, and a
hand-mirrored generated change is dev-only (the regen replaces it).
**Late result (neat-boar-16, srv1, whole tree): read before resuming.** The whole-tree differential was
VOID. Both arms stopped on the same 9 blocking diagnostics already present on main, before any digest,
so declaration-grain equality is established at 634 modules only. The probe must digest every module
that resolved and report the blocked ones as a counted, named set rather than panic. The AFTER arm also
took about 6.3x the BEFORE arm's wall to reach the same point, with no memory gain visible there. That
is a cost red flag for representation B at whole-tree scale, and possibly this note's own falsifier
(membership tests going superlinear). PR-2 is not opened until it is attributed by thread CPU per
phase and fixed or ruled.
Candidate causes of the 6.3x, all unmeasured, each to be attributed by thread CPU per phase on the same
subject before PR-2 is opened:
- the fork-row walk (`surface_fork_rows`): multi-exporter and kernel candidates times each module's
  imports, even with the closure-declarer prefilter;
- the `ancestry_names` enumerations in the rewire pass and in `source_visible_names`, which
  re-enumerate every closure module's own names per module;
- presence tests over long exporter lists for widely declared names (`surface_has` walks every
  exporter);
- per-module pool snapshots during typecheck (persistent-map path copies on every admission).

## PR-2 slowdown: attribution (calm-pike-525, 2026-10-02)
**Where the figures live.** This section keeps the structural findings, the predictions and their
verdicts, and the rulings. It does not copy figures (DESIGN §6). The probe that measured them is an
ignored test that needs a whole-tree resolve, and it lives on probe branches, not on main. So the
figures are recorded only where their pinned commits are recorded too: the receipts in #13008's
description and the scoring messages to the lane manager. A durable instrument row is a possible
follow-up. It is not part of this note.

**Instrument shape.**
- **Attribution.** Each emitted function under test gets a thread-CPU guard at its head, charged at its
  outermost activation only. The hot point lookups are counted, not timed. A cumulative table is
  printed every 500 typechecked modules, so arms can be compared at equal module counts.
- **Arms.** BEFORE is PR-2's base. AFTER is PR-2, then PR-2 with each repair added.
- **Differential.** It digests every module that resolved. Modules owning a blocking diagnostic are
  reported as a counted, named set rather than stopping the run, so the differential can no longer
  void.
- **Fork ledgers.** A per-module fork-row digest, recomputed over the final pool, compares the ledgers.

**Attribution.** The dominant term was none of the four listed candidates. It was the point lookups
during typecheck, at identical lookup demand in every arm.
- Every lookup that passed presence descended.
- Each descent level ran a presence test over every import.
- Each presence test walked the name's POOL-wide exporter list.
- So the cost per lookup grew with the tree, not with the module's closure.
- The fork-row walk was the second term, sized by the same pool-wide grain.
- The design's single-exporter fast path had not been implemented. My first fix keyed it pool-wide and
  missed: in a whole tree almost every name has a homonym somewhere. That miss is labelled.

**Predictions, stated before the run.**
- P1, the fork-row walk, structural: HIT. Its magnitude: MISS, because it is the second term, not the
  first.
- P2, the `ancestry_names` enumerations: MISS. They are larger than predicted.
- P3, the presence tests: MISS. They are part of the dominant term.
- P4, the pool snapshots: MISS. They are small.

**Repairs on #13008.**
- The closure-grain classification (`closure_sole_declarer`).
- Closure-grain fork candidates (`closure_contested_names`).
- The contested-name table below.

**The 8 apparent differences between PR-2 and BEFORE are not resolution differences.** They are byte
spans moved by PR-2's own edit to `gunbc.namespace_cut_subject_roster`. The full-entry mode
(`GUNBC_ANCESTRY_DIGEST=full`) separates an edited span from a different winner, and here it finds no
different winner.

### Ruling and the contested-name table (calm-boar-904, 2026-10-02)
**The table is admitted** as sharing at the least common ancestor. `surface_contested_walk` produces
the fork ledger and each contested name's winner in one walk per module. Its derivation is written
beside it:
- identity (view, name), and why it is complete;
- least common ancestor: the module's `build_type_env`;
- retention: the module's `TypeEnv`;
- one producer: `ancestry_winner` and `surface_union_winner` are deleted.

**The receipt protocol** was set by the ruling.
- All three arms are built on ONE runner and run interleaved, with two runners in counter-order.
- The 1-hour cap of the BuildBuddy free tier forced a fixed subject for the timing figures:
  `GUNBC_PROBE_EXCLUDE=dag/test/,src/v2/test/`. That EXCLUDES THE TEST TREES, and the subject is closed
  under importers using the assembler's own `ExclusionOrphansImporter` refusal.
- Equality is whole-tree.

**Verdicts.**
- **Equality.** Against #13008 alone, the table is equal at declaration grain and in every module's fork
  rows.
- **Overall prediction, HIT.** The table does not bring PR-2 to BEFORE, and the residual is the rewire
  pass's `ancestry_names` re-enumeration.
- **P-a, PARTIAL.** Typecheck including env construction reaches BEFORE. Typecheck excluding env
  construction stays above it, because of the exporter-list walk in the classification.
- **P-b, HIT.** The fork walk falls by more than predicted.
- **P-c, HIT.** The rewire is the residual.

**Follow-up ruling.** Both remaining terms are cost-shape defects to be fixed: the rewire's
re-enumeration, and the classification's exporter walk. That work is HELD behind the memory question
below.

### PR-2's landing gate is memory (neat-boar-16, 2026-10-01), and it is open
**The gate.** PR-2 lands only if the floor-subject peak falls by at least 3 GB. The process peaks
measured above moved far less than that.

**Chain read on main: LIVE.** The ancestry unions are live at the floor's peak seam
(`SeamDiscoveryAuthority`).
- `run_required_floor` holds `prepared.graph` through discovery authority.
- `prepared_graph_without_typecheck_caches` drops only `type_env_cache`.
- gunbc#12890 bounded the warm seam's frames. It did not project the graph.

So PR-2 could move that peak in principle.

**Why it has not, so far.** This branch removes only part (a), the string-binding union. The cache's
deps, cycle-name and variant-local unions, and `source_visible_names` (parts b and c), are still
materialized. This section's own warning applies: removing one holder of a union that another still
shares saves nothing.

**Pre-registered expectation.**
- (ii) PRIMARY: the bulk sits in the unremoved unions.
- (i) CONTRIBUTING: the string union is small exclusively, because of HAMT sharing.
- (iii) SMALL: new holders (contested tables, pool snapshots) offset the saving.

**Discriminator.** `//gunbc/instruments:typed-graph-exclusive-bytes-floor-subject`, main against
PR-2 merged with main.

**Decision rule.**
- (ii) means re-scope to a+b+c and measure that against the gate.
- (i) means withdraw.
- (iii) means fix the new holders first.

### Is the floor's whole-closure reconcile demanded? (C2 of `floor-time-attribution-2026-10-02.md`; chain read at main, 2026-10-03)
`compile.reconcile` is `reconcile_with_census_extra`. That is the whole typecheck
(`typecheck_with_census_extra`) plus the three rewire passes. A resolved `ModuleGraph` carries no
per-module environment, registry or inferred type. Every planned-claim category reads facts that only
reconcile produces:
- **Claim evaluation, wet lanes, enrolment, and the discovery and authority frames.** All of them go
  through `claim_scope_for_with_memos` into the interpreter.
  - The scope's closure IS `func_env.parents`.
  - The interpreter's dispatch guards read `item_registry`.
  - Evaluation reads the typed `items`: `.inferred` on match scrutinees, map literals, and op return
    types, plus the `ground_kernel_views` folds.
  - The coproduct fallback (`resolve_coproduct_type_node`) reads `type_env`.
- **The non-fold-residue verdict** reads the typed `items` and `type_env` of the touched, interface-consumer
  and NFR-row modules.

So no planned claim can run on a resolved-only closure. Reconcile is demanded, not redundant, and
the C2 repair class "delete redundant demand" does not apply to it as a whole.

**The whole-closure strict verdict is itself a floor deliverable.** A blocking type diagnostic anywhere in
the prepared closure refuses the floor (`resolved_graph_from_sources`, Strict gate). `checker_module_seeds`
widens preparation for typechecking alone ("Claim planning does not read this: only preparation widens").
The judged identities are printed for required CI.

**The only narrowing left.** It would typecheck only the union of the planned scopes, the NFR scope and
the authority frames, and stop typechecking the closure modules that no claim reaches (gate-prefix
modules with no witness, checker-change seeds). That gives up the whole-closure strict verdict, which is
a safety property. It is therefore a declared §4b(3) rung drop that needs an explicit ruling, not a
demand cut. Nothing in the code or docs has ruled on it. The size of that population has not been
counted: it is `prepared.graph.modules` minus the union of `claim_scope_for` modules over the planned
claims, the wet lane and the authority frames.

**Where reconcile's time can honestly go down: its interior.** The floor runner's own comment
(`required_floor_runner.rs`, "RECONCILE'S INTERIOR") records that over half of reconcile's wall time
allocates nothing, and that which of its six operations owns which region is unattributed. That is a
cost-shape question inside a demanded computation, and the next attribution step there is per-operation
trace marks in `04_infer.dag`.

### Outcome: PR-2 is withdrawn (calm-boar-904, 2026-10-03)
- **The gate became unreachable.** neat-boar-16 measured the rebased PR-2 against main with #13076 on srv1. The figures are with the pinned receipts on #13008. PR-2's saving in class bytes was well under the 3 GB gate, and answers were unchanged (equal `ancestry_entries`). The floor-peak runs at those pins are NOT a gate read: every run refused at the end of strict preparation on a main red (two String-type-mismatch diagnostics, routed to the String lane), so their peak is the prepare seam of runs that stopped there.
- **Main had already moved.** Main's own `type_env` fell by about the size of the gate between the two baseline pins. That makes the gate unreachable for PR-2 on current main, whatever released the bytes.
- **Not kept on single-authority grounds.** On main the import union is computed once, by `build_ancestry_precedence`'s fold, and then materialized in several holders. That is one authority, not a DESIGN §3 fork, and PR-2 was justified by memory alone. Its derived view adds a surface pool, closure bitsets, derived views and contested tables that its remaining saving does not pay for.
- **Attributed: #13076 released main's `type_env` bytes** (neat-boar-16, srv1, 2026-10-03). The instrument `typed-graph-exclusive-bytes-floor-subject` was run at #13076's parent `ca79f2c` and at its merge `82558fb`, over the same subject with equal `ancestry_entries`. The whole drop belongs to #13076: `type_env`, the joint `type_env` + cache + interface, and `graph_total` each fell by about the gate's size, and every other class was unchanged.
  - **So the chartered memory win was delivered by DELETING the identity rewire, not by PR-2.**
  - **It reverses this note's earlier reading:** the rewire's RSS rise was LIVE retention, not transient allocation. Its per-module rewrite path-copied nearly every ancestry key and broke the structural sharing between modules' maps. The two-versions probe compared against the rewritten maps themselves, which were the cost.
  - #13076's body is corrected by a comment, and the receipt is on `gunbc.recurring_failure_mode` `name_canonicalisation_depends_on_the_co_compiled_graph`.
- **What carries forward:**
  - the closure-grain classification and the shared contested-name walk, which are on the PR-2 branch (kept as provenance);
  - the reconcile-interior attribution;
  - the deletion of the identity rewire, which landed as #13076.

### Warm-route attribution: both hypotheses falsified (neat-boar-16, srv1, 2026-10-03)
- **The run.** Pin `558cecb` (MAIN representation), entry `dag/std/decision.dag`, cold and warm as separate processes, both rc=0. The instrument is the sequential class drop of `typed_graph_byte_attribution` on the warm-route probe branch `calm-pike-525/warm-route-bytes-probe`. Figures are in neat-boar-16's report and are not copied here.
- **H1 FALSIFIED** (the warm delta lives in the type_env union maps). Those maps free essentially nothing in either arm.
- **H2 FALSIFIED** (decoded Node trees lose their structural sharing). No graph class shows a warm/cold difference.
- **What the miss shows.** In BOTH arms, dropping the entry's whole typed graph frees a tiny fraction of the process heap. So the probe measured the wrong population: neither the warm-route delta nor most of the cold heap is held by the entry's graph. What holds it is process-lifetime state beside the graph: the shared index, the persisted-store decode, the resident pools. Both hypotheses shared the premise that the cost sits in the entry's graph, and that premise is what the run refuted.
- **Next.** Attribute the process-lifetime residency by the same sequential drop, at the same pin and entry: shared-index generations, resident pools, and the typed-store decode. Predictions are stated before that run. Until then the warm-route lead is unattributed, and no repair is proposed.
- **Residency run (pin `6e25707`): P1 and P2 HIT.**
  - **P1:** the shared index costs the same cold and warm.
  - **P2:** the whole warm delta is loaded inside the warm RESOLVE step, and it survives dropping the entry's graph. So it is state the warm resolve route retains for the process lifetime.
  - **Next (pin `ede7021`, P3/P4 pre-registered):** the typed-store counters, to tell breadth (decoding far beyond the entry closure) from per-decode expansion.
- **Counters run (pin `ede7021`, neat-boar-16): P4 HIT; P3 FALSIFIED on breadth, true on bytes.**
  - The warm route reads only TWO store entries, but they are near whole-pool size.
  - Cold encodes are corpus-sized per module.
- **Structural reading (from the code; not yet measured):**
  - `TypecheckModuleResult` → `TypedModule.type_env: Rc<TypeEnv>` (and `interface.env`) → `TypeEnv.parents: Rc<Vec<Rc<TypeEnv>>>`, and all of it derives `serde::Serialize`.
  - serde writes an `Rc` by value, so ONE module's typed snapshot serializes every ancestor's environment again, with no sharing. Decoding rebuilds them unshared.
  - This is the same class as the identity rewire: structural sharing lost at a copy boundary, here serialization.
- **Earliest unjustified boundary (proposed):** the snapshot's grain.
  - A module-grain record should carry its parents as module identities, resolved at decode against environments already decoded, not as their contents.
- **Discriminator, pre-registered before any repair:**
  - Per-module encoded bytes grow with the size of the module's import closure, approaching proportionality.
  - A module with no imports encodes small.
  - If encoded size is flat in closure size, this reading is falsified.
- **Snapshot-grain discriminator (pin `e479390`, neat-boar-16, srv1, cold): CONFIRMED.**
  - Per module, the encoded `type_env` size is a near-exact linear function of the number of reachable environments, independent of the module's item count.
- **Per-field reading (pin `c2e50a8`): CONFIRMED.**
  - The per-environment constant is `symbol_index`, identical across modules and pool-scoped.
  - `intern_table` and `source_indices` are smaller and also identical.
  - The module's own fields are KB-sized.
  - So parents-by-identity alone would not have been the repair. Figures are in those runs' logs.
- **Outcome: no repair built (calm-boar-904 ruling).**
  - The persisted store that produced the warm-route delta was deleted from main by #13064, as a regression of the same size per entry; that is this class, fixed by deletion.
  - The same encoder survives only in the in-memory cross-worker store. On main it is armed only in a `cfg(test)` module and in `run_discovery_corpus_with_options_inner`, whose sole entry has a unit test as its only caller.
  - No production consumer demands typed-snapshot sharing (DESIGN §2's demand-first rule), so nothing is built.
  - The class is filed as `gunbc.recurring_failure_mode` `typed_snapshot_serializes_shared_structure_by_value`, a declared frontier: the row lands with #13163 and does not resolve until then. Its next-rung trigger carries the build conditions for when a consumer appears.
- **This closes the type_env attribution:**
  - PR-2 withdrawn;
  - the chartered memory win delivered by #13076;
  - the warm-route cost attributed to the persisted store's snapshot grain, already deleted by #13064.
