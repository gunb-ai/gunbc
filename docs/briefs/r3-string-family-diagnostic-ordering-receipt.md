# R3 String-Family Diagnostic Ordering Receipt

**Status:** READINESS RECEIPT (docs-only, no row population, no reader implementation).  
**Owning manager:** Substrate Manager (R2 -> R3 continuation).  
**Lane:** T-Ground-LanguageSpec / string-family diagnostic-ordering precursor.  
**Authored:** 2026-05-02.

This receipt records the honest precursor requested after the STOP on per-target string-family row population: define the row host and schema contract, keep the current `String*Axis` namespaced direction from #1465, and leave Grounding reader attachment for a later consumer PR.

## Authority audit receipt

1. **Substrate exists?** Yes for axis vocabulary, no for row host. `src/v3/std/emit_model.dag` already owns the four landed declarations at HEAD:
   - `type StringOwnershipAxis = Owned | Borrowed` (`src/v3/std/emit_model.dag:141-149`)
   - `type StringLifetimeAxis = SelfContained | Caller` (`src/v3/std/emit_model.dag:151-157`)
   - `type StringGrowabilityAxis = Growable | Fixed | NotApplicable` (`src/v3/std/emit_model.dag:159-165`)
   - `type StringEncodingAxis = Utf8FreeMonoidChar` (`src/v3/std/emit_model.dag:167-172`)
2. **Existing brief?** Yes for adjacent readiness and vocabulary. `docs/audit/lifetime-axes-canonical-vocabulary-spec.md` and `docs/audit/grounding-tests-stratum-b-scaffold-readiness.md` already hold the #1465 axis split and the “rows later, reader later” boundary, but neither one defines a row host/schema contract for string-family diagnostic ordering.
3. **Design-doc recommendation matches?** Yes. The live audit prose says string-family rows should reference `String*Axis` structurally and that `LanguageSpec` / row authority is the place to attach future candidate facts (`docs/audit/lifetime-axes-canonical-vocabulary-spec.md:104-126`, `:180-203`). No shared non-namespaced layer is required in this slice.
4. **Citations live?** Yes at current `origin/main` / HEAD. `TypeRealization` is the existing carrier authority at `src/v3/std/emit_model.dag:8-24`. `LanguageSpec` is the existing target-spec carrier at `src/v3/std/emit_model.dag:390-405`. The string-family axis sums are declared at `src/v3/std/emit_model.dag:141-172`.
5. **Carrier dissolves the bridge?** Yes, as a contract only. The point of the precursor is to define the row host that later consumers will read structurally, without reusing `TypeRealization` as a proxy for candidate ordering and without adding a parallel shared-axis layer.

## Host contract

The row host for string-family diagnostic ordering should be a `LanguageSpec`-owned sibling carrier, not `TypeRealization`.

Proposed shape:

```text
type StringFamilyInhabitanceRow {
  language: DeclarationRef
  target_type: DeclarationRef
  type_realization: DeclarationRef
  ownership: DeclarationRef
  lifetime: DeclarationRef
  growability: DeclarationRef
  encoding: DeclarationRef
}
```

The exact field names can still be refined, but the authority split should not change:

- `language` pins the owning `LanguageSpec` declaration.
- `LanguageSpec` stays the host container.
- `StringFamilyInhabitanceRow` stays a sibling row carrier.
- `TypeRealization` remains the realization carrier for target primitives and their field bindings.

## Why the host is not `TypeRealization`

`TypeRealization` already means “how a target carrier realizes a type” and carries carrier/field/cost facts. The string-family row slice needs a different unit of authority:

- candidate identity for a target type family,
- canonical axis references for ownership / lifetime / growability / encoding,
- a later fold/read path that can select among rows structurally.

That is candidate-ordering authority, not realization authority. Reusing `TypeRealization` here would mix two distinct facts into one carrier and force a parallel encoding of row semantics inside a realization record.

## Why there is no shared non-namespaced axis layer in this slice

#1465 already chose the namespaced surface: `StringOwnershipAxis`, `StringLifetimeAxis`, `StringGrowabilityAxis`, `StringEncodingAxis`. This slice preserves that decision.

No `Ownership` / `LifetimeScope` / `Growability` shared layer is introduced here because that is a separate substrate design call on the `#1130` path. This receipt records the row contract against the existing namespaced authority; it does not reopen the namespace-vs-shared question.

## How rows reference the canonical axes

Row values should be structural references, not lowercase strings, and the row should be typed to a specific `LanguageSpec` via `language`.

The row fields above should point at the named axis declarations by `DeclarationRef`, using the landed `String*Axis` values:

- `Owned` / `Borrowed` via `StringOwnershipAxis`
- `SelfContained` / `Caller` via `StringLifetimeAxis`
- `Growable` / `Fixed` / `NotApplicable` via `StringGrowabilityAxis`
- `Utf8FreeMonoidChar` via `StringEncodingAxis`

That keeps Grounding from inventing local normalization tables and keeps the row vocabulary aligned with the substrate authority already landed in `emit_model.dag`.

## Where Grounding projection readers attach later

The reader belongs on the consuming side, after the row host exists. The later Grounding slice should:

- read `LanguageSpec` structurally,
- walk the `StringFamilyInhabitanceRow` carrier,
- and project those rows into fold / selection behavior.

This receipt does **not** implement that reader. It only defines the substrate contract the future reader will consume.

## Scope boundary

- No row population.
- No Grounding projection reader.
- No shared non-namespaced axis layer.
- No `TypeRealization` reuse as a row host.

## Next step

Once this receipt lands, the next honest slice is the row-population brief that can attach concrete per-target string-family rows to this host without inventing a new authority split.
