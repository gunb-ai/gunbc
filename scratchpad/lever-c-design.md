# Lever C — stream/evict resolved-module envs: design

## Problem
Whole-tree resolve produces a `ResolvedGraph { modules: Vec<TypedModule> }` (~600
modules). The floor/discovery path then builds an `InterpContext` over it and runs
all claims against it — so the resolved graph is held resident for the *entire* long
claim-execution phase. Each `TypedModule` carries a heavy `type_env: Rc<TypeEnv>`
(per-symbol `bindings` HashMap of resolved type Nodes + provenance) and
`func_env: Rc<ResolvedFuncEnv>` (signature map). This is the sustained floor RSS.

## Key finding (verified by code read)
The interpreter **never reads `type_env` or `func_env`** after construction:
- `InterpContext::with_runtime_options` (v1_interpreter.rs:1044) builds `fn_nodes`,
  `service_ops` from `module.items`, copies the global `item_registry`, and takes
  `source_indices` as an independent param. It then stores
  `modules: graph.modules.clone()`.
- The only eval-time reads of the stored `modules` (v1_interpreter.rs:2686,
  coproduct_reflection.rs:69/466/767, wire_value_serialize.rs:130) touch only
  `module.module`, `module.items`, `module.item_registry`. Zero reads of
  `.type_env` / `.func_env` anywhere in the interpreter or its external readers.

So `type_env`/`func_env` are dead weight in the sustained interpreter context — a
representation that carries data the consumer never consults (DESIGN §2: don't carry
representation the consumer doesn't need).

## Design (minimal blast radius)
In `with_runtime_options`, instead of `modules: graph.modules.clone()` (which keeps
the full heavy modules alive via the InterpContext's Rc), project each `TypedModule`
into a fresh `TypedModule` that keeps `module`/`items`/`item_registry` (same Rcs,
cheap refcount bumps) but points `type_env`/`func_env` at shared **empty** singletons
(`empty_intern_table()` already exists; `ResolvedFuncEnv { signatures: empty }`).

When the caller then drops its `Rc<ResolvedGraph>` (the floor path
`whole_tree_resolved_ctx` drops `graph` at function return), the real
`type_env`/`func_env` payloads are released — freeing whatever is uniquely held there
(resolved type Nodes, provenance, signatures). The Node bodies (`items`) and the two
item registries survive, as the interpreter requires.

## Safety
- Behavior-identical: projected modules preserve `module`/`items`/`item_registry`
  identity; the only mutated fields are provably never read.
- Universally safe: for callers that *keep* the graph alive (e.g. emit also runs),
  the projection just allocates ~600 small structs (no win, no harm); the win
  materializes exactly on the floor path where the graph is dropped.
- No public-type change: `InterpContext.modules` stays `Rc<Vec<Rc<TypedModule>>>`;
  external readers unchanged.

## Scope boundary (honest)
This is the **evict** half of Lever C: it cuts the *sustained* floor RSS during
claim execution. It does NOT cut the *transient* peak during typecheck (all envs are
still built before projection). The deeper "resolve in dependency batches, evict each
env once its dependents are typechecked" (cuts the transient peak) is a restructure
of the load-bearing typecheck pipeline; since v1 is going away, that is documented as
the v2-root follow-on rather than implemented in the doomed seed. The emit path
(needs all envs) is untouched and is not the floor-OOM driver.
