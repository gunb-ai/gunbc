# Type environment: single import authority + scope cursor (v1 fix, v2 target)

Status: DRAFT for operator review. Owner: lively-raven-355 (realization / SymbolIndex) — inherited 2026-07-06 when cool-hawk-899 archived; cool-hawk-899 authored §1–§8.
Directive: operator (2026-07-05) — "fix v1, it's worth it; then apply the same to v2." · (2026-07-06) — the intra-process node-level memo must be built as one shard of an eventual content-addressed, distributable realize (map-reduce / RBE); see §5.5.

**Resolver half (companion):** `docs/plans/namespace-resolution-design.md` (loyal-dove-903). This doc is the INDEX/storage half (fill-once `SymbolIndex`, the O(M²) fix); the companion is the SEMANTICS half (`resolve(name, position)`). One index, two `ResolutionPolicy` values over it — `import-scoped` (this doc's §3, behavior-preserving, ships first) and `namespace-only-Y` (the companion's nearest-enclosing-subtree pivot, on top). **One-index invariant (companion §7.5):** the fill stays policy-agnostic (topo prepass fills everything import-DAG-reachable); the policy gates LOOKUP only, never fill — else two policies materialize two indices, the dual-representation Rule 1 forbids.

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

So the O(M²) population is dragging a field only the O(1)-per-function population uses, and is stored
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
encounter. It replaces "start from a requested module and walk backward, repeatedly, flattening ancestry."
Reverse-from-request + per-module flatten is exactly what forced the copies; forward-DFS + one shared index
+ a scope cursor removes the *need* for them.

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

## 5.5 Distributability invariants (map-reduce / RBE endgame — operator 2026-07-06)

The intra-process node-level memo is not a terminal design — it is **shard 0 of a content-addressed, distributable realize** (map-reduce / Bazel-RBE shape: infinitely scalable, across-process/host irrelevant). The whole RBE idea reduces to one shape: *each realize-unit is a pure function of content-addressed inputs → a content-addressed result in a shared CAS*. Given that, **map** (realize the ready frontier) and **reduce** (merge results into the index/CAS) are the same operations whether workers are threads, processes, or hosts — only the CAS transport changes. So "across process/host is irrelevant" becomes true exactly when three invariants hold, and we commit to them NOW so wiring the intra-process memo cannot foreclose the endgame (cheap now, a re-architecture later):

1. **Content-hash-groundable key.** The `SymbolIndex` key is `String` (qualified name) intra-process, but must be *groundable to a content-hash* so results are location-independent and dedupe across processes. The substrate already exists (`std.content_hash`, `std.cache_identity`, `std.cache_interface`, `std.realization_schedule`; the DAG nodes are already content-addressed). Build the index as one shard of the CAS keyed by a projection of that hash — not a name-only map that forks when distributed.
2. **Exact minimal input closure per unit.** A superset digest = spurious cache misses + poisoned dedup. **#6306 already delivers this** ("exact dependency closure, not topological superset") — it is the RBE-critical property and it is landed.
3. **Per-unit determinism.** Each realize-unit a pure deterministic function of its input digests. The whole-compile byte-identical fixpoint (§7 self-host) proves it at *corpus* level; RBE needs it *per-unit* — the live thread is `determinism-mechanism-design.md` (#5941 P1).

The demand-driven shape #6306 already has **is** the map-reduce shape: the set of modules whose deps are all realized is the wavefront. `fold → thread pool → RBE workers` is a scheduler swap; `local Map → content-addressed CAS` is the only backend swap. The reform must keep those two seams clean (a `RealizationBackend` the fill writes through, a scheduler the driver folds through) rather than inlining `HashMap` + a serial fold as load-bearing assumptions.

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
BUT it is the SAME fix: both re-derive the import closure per module. So the `SymbolIndex` single authority
is consumed by BOTH — and the **first realization should target `emit_imports`** (the 78%), then
`build_type_env` (the 17%). The design is unchanged; the ORDER of consumers flips to emit-first.

CAVEAT: 85-module RATIOS ≠ 876-module SCALING. Which stage is O(M²) vs high-linear is settled only by the
scaling curve (100/200/400/800), loyal-heron's next deliverable. Do NOT start surgery until it lands — it
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
