# `algebra.dag` declaration-vs-template reconciliation — findings

**Date:** 2026-04-22
**Lane:** G (from 2026-04-22 dispatch)
**Outcome:** STOP-AND-ESCALATE. Not executing reconciliation; surfacing as separate modeling lane.

## Brief ask

Reconcile `OrderedRing<T>` declaration (`algebra.dag:176-193`, 16 fields) with
`ordered_ring_templates()` (`:447-459`, 9 entries incl. `clamp`); same for
`PartialFunction` (`:315-326` vs `:552-570`). Choose (a) derive templates from
declaration, (b) delete templates, or (c) project templates from declaration.

## Consumer map

`*_templates()` path — executable authority:

- `algebra_templates_for_profile(profile)` fans out to the per-profile
  `*_templates()` list (`algebra.dag:572-582`).
- `enrich_kernel_type` (v2 `04_types.dag:372-382`) maps each kernel type name
  (via `kernel_algebra_profile` data table) → profile → templates →
  `instantiate_algebra_field` → `Conj` product of method field `Node`s. This
  is how `Int`/`Float`/`Bool`/`String`/`List`/`Set`/`Map` acquire their
  methods.
- `resolve_known_method_node` (v2 `04_lookup.dag:290-321`) re-reads the
  matching template to decorate the lookup with `size_effect`, `cost_shape`,
  and `algebra_template` — these feed complexity/cost analysis.

`type OrderedRing<T> { ... }` / `type PartialFunction<K,V> { ... }` path:

- Used as nominal aliases: `type Int64 = OrderedRing<Word64>`
  (`std/integer.dag:31-34`), `type Map<key,value> = PartialFunction<key,value>`
  (`std/types.dag:208`).
- Grep for any walker that enumerates these types' *children* as method
  authority: none. Method resolution for the kernel types flows through the
  profile→template path, not through the alias target's field list.

**Conclusion:** the type-declaration field lists are not compiler-consumed
method authority. Templates are the sole executable authority. The
declarations are prose-shaped-as-code.

## Why they're not redundant

Three surfaces present in `*_templates()` that the algebra type declarations
structurally cannot carry:

1. **Per-method cost/size/callback metadata.** `AlgebraFieldTemplate` carries
   `size_effect: CollectionSizeEffect?`, `cost_shape: CostShape?`,
   `callback_element_position: Int?` (`algebra.dag:425-435`). These are
   declared facts consumed by complexity/cost lenses. The type declaration is
   a pure `fn(T,T) -> T` surface — no slot for these structural contracts.
   Moving metadata onto declared fields would either require field-level
   annotations (violates `feedback_no_annotations.md`) or a new "field with
   attached contract" modeling primitive.

2. **Kernel method-naming synonyms.** `partial_function_templates()` emits
   both the abstract algebra names (`get`, `lookup`, `keys`, `values`,
   `has`) and kernel-call-site synonyms (`map_get`, `map_insert`, `map_merge`,
   `map_has`, `map_contains_key`, `map_keys`, `map_values`). The synonyms are
   used across `std/*.dag` and `src/v2/*.dag` as real call-site names — not
   aliases. The abstract declaration `PartialFunction<K,V>` deliberately does
   not carry the `map_`-prefixed names; they're a realization-layer concern.
   Same split: `PartialFunction` has `insert`/`merge`/`size`;
   templates have `map_insert`/`map_merge`/`length`.

3. **`OrderedRing` operator fields.** Declaration has `sub`, `div`, `eq`, `ne`,
   `lt`, `le`, `gt`, `ge` (per `:168-175` comment, present so v3 §8.9
   inhabitance walk can resolve surface operators `+`, `-`, `==`, `<`, ...).
   Templates omit these — the v2 emission path does not need them as
   distinct methods (it derives `sub` from `add`+`negate`, etc.). Templates
   additionally have `clamp` which is neither in the declaration nor a
   ring-level primitive. The two surfaces serve v3 inhabitance vs v2 emission
   — different consumers, different method sets, both load-bearing.

## Drift inventory (for the record)

**OrderedRing:**
- In declaration, absent from templates: `sub`, `div`, `eq`, `ne`, `lt`, `le`,
  `gt`, `ge` (v3 operator-resolution surface).
- In templates, absent from declaration: `clamp` (v2 emission synonym).

**PartialFunction:**
- In declaration, absent from templates as-is: `empty` (exists in declaration
  as an identity element, not exposed as a method template anywhere);
  `insert` vs template `map_insert`; `merge` vs `map_merge`; `contains_key`
  vs `map_contains_key`; `size` vs template `length`.
- In templates, absent from declaration: `get` (declaration only has
  `lookup`), `map_get`, `map_has`, `map_keys`, `map_values`, `with`,
  `contains`, `length`.

The drift is not accidental divergence. It is two surfaces tracking two
different consumer populations.

## Why none of (a)/(b)/(c) fits cleanly

- **(a) derive templates from declaration.** Blocked: declaration cannot
  carry `size_effect`/`cost_shape`/`callback_element_position` or the
  kernel-synonym name set. Would require extending the type declaration
  with new connectives or violate `no_annotations`.
- **(b) delete templates.** Blocked: consumers (`enrich_kernel_type`,
  `resolve_known_method_node`) need the metadata. Declaration cannot
  supply it.
- **(c) project templates from declaration.** Blocked symmetrically: the
  declaration is the *smaller* surface. You cannot project the larger
  surface from the smaller one.

The only shape that would dissolve the parallel is: lift the template
metadata (size/cost/callback, kernel synonyms) to a modeling primitive that
*extends* the algebra type declaration — i.e., the declaration becomes rich
enough that the templates are pure projection. That's not a reconciliation
edit in `algebra.dag`; it's a substrate-capability lane.

## Recommendation

Escalate as a modeling lane: **"Algebra method contracts — unified authority
design."** It needs to answer:

1. How does an algebra type declaration carry per-field cost/size/callback
   contracts without per-field annotations?
2. How does the realization layer bind abstract algebra method names to
   kernel-call-site synonyms (and are the synonyms themselves a target of
   dissolution — i.e., should `map_insert` go away in favor of `insert`)?
3. Does `OrderedRing`'s v3-operator surface (`sub`, `eq`, `lt`, ...) belong
   in the algebra declaration, or on a sibling "OperatorSurface<OrderedRing>"
   structure that the v3 inhabitance walk reads?

Until those are answered, a local reconciliation edit in `algebra.dag` either
loses information (deleting one side) or creates a third parallel authority
(projection with divergence risk).

## Non-goals honored this lane

- No fields added to or removed from `OrderedRing<T>` / `PartialFunction<K,V>`.
- No templates added, removed, or renamed.
- No consumer refactor.
- No new algebra primitives.

## Suggested ROADMAP entry

Add a debt row:

> **Algebra parallel authority (templates ↔ declaration).** Two surfaces in
> `algebra.dag` track two consumer populations (v2 kernel enrichment +
> complexity lenses vs v3 inhabitance aliases). Dissolution requires a
> modeling primitive for per-field cost/size contracts + a decision on
> kernel-synonym naming. See
> `docs/briefs/algebra-templates-reconciliation-findings-2026-04-22.md`.
