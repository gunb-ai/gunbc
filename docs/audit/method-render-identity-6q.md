# Method Render Identity 6Q

**Date:** 2026-05-02
**Authority:** `docs/audit/collection-ops-string-ops-map-ops-duplicate-fact.md`
**Scope:** docs/design decision only. No `MethodTemplateContract` carrier change and no Grounding row population.

## Decision

Keep `MethodTemplateContract.dag_method: MethodRef` as the render-row identity for the current row-authoring lane. Do not add `MethodRenderRef` or a `(algebra_id, method_id)` key to render rows in this slice.

The flat `MethodRef` is semantically unique enough for render templates because it identifies the target operation name the renderer must spell. Algebra/profile semantics are already owned elsewhere:

- `dsl/std/methods.dag` declares the flat closed method-name registry. Its module comment explicitly says profile-specific semantics stay in `AlgebraFieldTemplate` and `MethodContract`.
- `src/v3/std/methods.dag` declares `type MethodRef { decl: DeclarationRef }` as the typed handle into that registry.
- `src/v3/std/algebra.dag` declares `MethodContract { algebra_id, method_id, ... }` for target-agnostic cost/complexity metadata.
- `src/v3/std/emit_model.dag` declares `type MethodTemplateContract { dag_method: MethodRef, ... }` as the target-specific render sibling keyed by that handle.
- `src/v3/compiler/tests/integration/method_template_contract_test.rs::method_template_contract_per_target_dag_method_unique` enforces one row per `MethodRef.decl` inside each target row list.

Adding `(algebra_id, method_id)` now would move algebra/profile semantics into the render row before a concrete render conflict exists. That would reintroduce the duplicate-fact problem the CollectionOps/StringOps/MapOps audit is trying to retire, just under a more structural name.

## Rejected Alternative: `MethodRenderRef`

`MethodRenderRef { algebra_id, method_id }` is rejected for the current rows.

It is the right future shape only if a target has two render templates for the same `MethodRef` and the difference is genuinely algebra/profile-owned. No such checked-in `MethodTemplateContract` row set currently needs that split:

- Collection methods that are already rows (`count`, `filter`, `flat_map`, `any`, `all`, `skip`, `take`, etc.) render by method name within a target.
- Map operations have distinct registry names such as `map_get_method`, `map_insert_method`, `map_contains_key_method`, and `map_has_method`; they do not require overloading `get` or `contains` under `MapOps`.
- Construction syntax (`empty_list`, `list_literal`, `cons`, map literals) is not a method-render row until classified by the follow-up substrate/grounding work.

## 6Q

### Q1 - Carrier Invariants

**PASS with no change.** `MethodRef { decl: DeclarationRef }` remains the render-row identity carrier. The existing residual is already documented: the DSL cannot yet express `DeclarationRef<MethodDeclaration>`, so boundary tests enforce that the target resolves to a `MethodDeclaration`.

### Q2 - Index / Handle Types

**PASS.** `MethodRef` is the typed handle into the method registry. A new `MethodRenderRef` would add a second handle before any target row needs two coordinates.

### Q3 - Duplicated Fact

**BLOCKER for adding `(algebra_id, method_id)` now.** Algebra/profile ownership belongs to `AlgebraFieldTemplate` and `MethodContract`. Render templates should not also encode algebra/profile identity unless the renderer has a proven same-method-name conflict.

### Q4 - Coproduct / Product Compression

**PASS.** No sum/product carrier is introduced. The current product `MethodTemplateContract` stays focused on render facts: method identity, runtime template, emit template, result wrapping, and placeholder convention.

### Q5 - Construction Authority

**PASS.** Row producers keep using the per-target `List<MethodTemplateContract>` authorities. The target list carries target identity; `dag_method` carries method identity. No row producer has to infer algebra/profile context from `CollectionOps`, `StringOps`, or `MapOps` record names.

### Q6 - Representation Duality

**PASS if row population respects the trigger below.** Ordinary operation templates should migrate to `MethodTemplateContract` rows keyed by `MethodRef`. Non-method construction render data must get its own home instead of being forced through method rows. If a target later proves two rows with the same `MethodRef` need different templates for different algebra/profile identities, that is the trigger to add `MethodRenderRef` or an equivalent structural coordinate.

## Follow-up Trigger

Add a structural render key only when all of these are true:

1. A target row list needs two render rows for the same `MethodRef`.
2. The difference is not target language, arity, placeholder convention, wrapping, or ordinary higher-order template shape already represented on `MethodTemplateContract`.
3. The difference is owned by algebra/profile identity and cannot be modeled as a distinct method declaration or non-method construction syntax.

Until that trigger fires, flat `MethodRef` is the single render-operation identity. Grounding row-population slices should not recreate `CollectionOps`, `StringOps`, or `MapOps` distinctions through record names or premature algebra/profile coordinates.
