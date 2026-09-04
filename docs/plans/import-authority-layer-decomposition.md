# Import authority: what actually decides resolution, and in how many layers

Scoping record for the import-declaration cut. It exists because the cut was twice scoped against a
model of the compiler that turned out to be wrong, in both directions — once too narrow, once too
wide — and each correction changed what the cut has to delete. Every claim below is cited to the fold
that carries it, so the next reader can falsify it rather than inherit it.

## The two layers

**L-A — which of a searched module's functions enter the environment.**
`func_env_views_for_import` (`v1.compiler.infer`) returns the parent whole only when `is_all`;
otherwise every view goes through `selective_func_env_view`, which builds its result by folding
**only over the listed names** and setting `parents: []`. So `specific_names` is not a filter applied
to an available population — **it is the population**. `parent_envs` is a `flat_map` over
`resolved_imports` through that function, and `parent_closure_callable_candidates` searches exactly
those parents.

**L-B — which names are then visible, and which candidates are admitted.**
`TypeEnv.source_visible_names` and `TypeEnv.authored_import_names` (`v1.compiler.infer_env`), and
`author_named_visibility` (`v1.compiler.infer_lookup`), whose three states gate whether a builtin may
join the declared candidate population.

L-A is upstream of L-B: L-B masks a population that L-A assembled. Deleting L-B alone removes a mask
over a pool that is still import-assembled, and the failure mode afterwards is **quieter** than
before — a name absent because nobody imported it does not refuse as masked, it is simply not there.

## What is NOT a layer: module discovery

An earlier revision of this record claimed imports also decide *which modules exist to be searched*,
inferring it from the `resolved_imports` recursion inside `realize_module`. **That claim was wrong**,
and it is recorded here rather than quietly removed because it misled two readers before it was
caught.

The recursion is real, but the fold that encloses it iterates `graph.modules` and calls
`realize_module` on **every** module, and `realize_module` early-returns on a name already present in
the index. The recursion therefore realizes a module's dependencies *before* the module itself — a
topological ordering device, not a membership filter. Membership of the final `module_index` is
`graph.modules`, which is containment-derived.

The general lesson, since it is the same one twice: **a true line does not make a true claim.** Both
the original claim and its first confirmation cited a line that really said what was quoted; neither
read the caller that determined what it meant.

## The ordering precondition

`parent_index` is `dep_state.module_index` — the index *as of that moment*, not the final one. When a
module is typechecked, that index holds its own import closure **union an incidental prefix**
determined by position in `graph.modules`. So the parent set a module sees is neither a clean
containment set nor a clean import set; it is order-dependent.

**Today this is latent, not live.** Every resolution-path access is
`map_get(parent_index, imp.module_path)` — keyed by an import path, with no enumeration and no
`map_keys` anywhere. The incidental entries are therefore unreachable, and the order-dependence is
inert *precisely because* every lookup goes through an import path.

It arms the moment lookups stop being keyed by import path. So this is a **precondition of the
population repoint, not a ticket beside it**: the repoint must fix the ordering or seed directly from
`graph.modules`, and if it does neither it silently inherits an order-dependent parent set whose first
symptom is a module resolving differently depending on where it sits in `graph.modules`.

## Consequence for the agreement census

Repointing L-A while leaving L-B in place is intended to make the masks either agree (inert, and
deletable with confidence) or disagree loudly (free census). That reasoning is only valid if the
containment pool is derived independently of the thing the mask was derived from — otherwise
agreement is two views of one source, which is the shared-corruption shape a comparison cannot detect.

`graph.modules` clears the import upstream. It does **not** clear the ordering artifact. So: seed the
containment pool from `graph.modules`, never from `parent_index` or `module_index`.

## The minimum for the repoint

Preserving every name that used to resolve is not sufficient. A pool that answers every previously
answered call while quietly ceasing to refuse something — an ambiguity that used to be an ambiguity,
a located not-found that became a bare miss — has erased a correctness distinction rather than
completed the replacement. Enumerate the refusal classes before the repoint and require each still
refuses after.
