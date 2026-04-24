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
- **Consumed as executable authority by the v3 operator-resolution
  pipeline.** `infer.rs::read_algebra_field` (`src/v3/compiler/src/infer.rs:3819`)
  walks the algebra declaration's fields, unwraps each field's
  `TypeConnective::Arrow { inputs, output, body }`, substitutes the
  receiver type parameter, and returns the resolved Arrow for operator
  dispatch. `lower.rs` (`:3222-3237`, and the generated mirror at
  `lower_generated.rs:3222-3237`) validates that every
  `OperatorRealization.op` walks to an algebra field declaration whose
  connective is Arrow (e.g. `OrderedRing.add`); non-Arrow or non-field
  targets are rejected with a diagnostic.
- v2 method materialization does not go through the declaration; it goes
  through `kernel_algebra_profile` → templates.

**Conclusion:** both surfaces are compiler-consumed executable authority,
but by *different pipelines for different purposes*. v3 reads the algebra
type declaration's field Arrows to resolve surface operators (`+`, `<`,
...) via `OperatorRealization`. v2 reads templates to materialize method
fields on kernel types and to decorate lookups with cost/size metadata.
Neither pipeline consumes the other's surface.

## Why they're not redundant

Three surfaces present in `*_templates()` that the algebra type declarations
structurally cannot carry:

1. **Per-method cost/size/callback metadata.** `AlgebraFieldTemplate` carries
   `size_effect: CollectionSizeEffect?`, `cost_shape: CostShape?`,
   `callback_element_position: Int?` (`algebra.dag:425-435`). These are
   declared facts consumed by complexity/cost lenses. The type declaration is
   a pure `fn(T,T) -> T` surface — no slot for these structural contracts.
   Moving metadata onto declared fields would either require field-level
   annotations (violates `MODELING.md:19`) or a new "field with
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
- In declaration, absent from templates as-is: `empty` (declared as the
  identity element `empty: PartialFunction<K, V>` — not exposed as a
  method template); `insert` vs template `map_insert`; `merge` vs
  `map_merge`; `contains_key` vs `map_contains_key`; `size` vs template
  `length`.
- In templates, absent from declaration: `map_get`, `map_has`,
  `map_keys`, `map_values`, `with`, `contains`, `length`. (Both
  declaration and templates carry `get`, `lookup`, `has`, `keys`,
  `values`.)

The drift is not accidental divergence. It is two surfaces tracking two
different consumer populations.

## Why none of (a)/(b)/(c) fits cleanly

- **(a) derive templates from declaration.** Blocked: declaration cannot
  carry `size_effect`/`cost_shape`/`callback_element_position` or the
  kernel-synonym name set. Would require extending the type declaration
  with new connectives or violate `MODELING.md:19` ("extend the
  structure rather than add annotations or metadata").
- **(b) delete templates.** Blocked: v2 consumers (`enrich_kernel_type`,
  `resolve_known_method_node`) need the per-method metadata. Declaration
  cannot supply it, and the v3 operator pipeline only validates Arrow
  field shape — it has no slot for cost/size/callback either.
- **(c) project templates from declaration.** Blocked by asymmetric
  information: each surface carries fields the other does not. Declaration
  carries `sub`/`div`/`eq`/`ne`/`lt`/`le`/`gt`/`ge` (OrderedRing) and
  `empty`/`insert`/`merge`/`contains_key`/`size` (PartialFunction) that
  templates omit. Templates carry `clamp` (OrderedRing) and the
  `map_*`-prefixed synonyms + `with`/`contains`/`length`
  (PartialFunction) that the declaration omits, plus metadata the
  declaration cannot hold. Neither is a subset of the other.

The only shape that would dissolve the parallel is: lift the template
metadata (size/cost/callback, kernel synonyms) to a modeling primitive that
*extends* the algebra type declaration, AND fold the v3 operator-surface
fields (`sub`, `eq`, `lt`, ...) into a sibling structure or template
metadata too — so a single enriched declaration drives both the v3
operator walk and v2 kernel materialization. That's not a reconciliation
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
