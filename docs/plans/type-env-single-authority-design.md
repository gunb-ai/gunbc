# Type environment: single import authority + scope cursor (v1 fix, v2 target)

Status: DRAFT for operator review. Owner: cool-hawk-899 (ci-timeout / resolve-axis spine).
Directive: operator (2026-07-05) — "fix v1, it's worth it; then apply the same to v2."

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

- `SymbolIndex` (new): `Map<QualifiedName, Node>` single authority. Filled once by the DFS.
- `Scope` (cursor): local bindings `Map<String, LocalBinding>` + a visibility view (the module's imports).
  `LocalBinding { name, resolved: Node, provenance: SubValueRelation }` — provenance stays HERE.
- `ImportBinding` (or reuse a lean `name → Node`): the import population, provenance-free.
- `TypeEnv` loses `ancestry_str_bindings` (materialized copy) and the parallel String/Int keyings
  (`04_env.dag:34` dissolution). `parents` compositional view is subsumed by the scope + index.

## 5. Migration

1. **Design review** (this doc) — operator signs the model before any load-bearing edit.
2. **v1**: introduce `SymbolIndex` + `Scope`; rewrite `build_type_env`/`typecheck_modules` to fill-once +
   scope-resolve; delete `ancestry_str_bindings` materialization + the `union_parent_type_env_caches` /
   rewire cluster. cool-hawk-899 is owner-of-record for `build_type_env` during this.
3. **v2**: apply the SAME model to `std.type_env` (the L31 target). One model, two realizations — authored
   so v2 is the durable home and v1 is a thin seed of it.

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

## 8. Open questions for operator review

- `provenance` split: confirm no ancestry binding ever carries meaningful (non-Unknown) provenance
  (audit says no; want your confirmation before deleting the field from the import population).
- Does the forward DFS need the shared `SymbolIndex` to be mutable-during-walk (fill as you go), or built
  in a prepass over the import DAG in topo order? (affects the cursor's purity.)
- v2 sequencing: apply to `std.type_env` immediately after v1, or once v1's scaling receipt is green?
