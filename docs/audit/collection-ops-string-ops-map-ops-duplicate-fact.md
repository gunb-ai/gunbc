# CollectionOps/StringOps/MapOps Duplicate-Fact Ontology

**Date:** 2026-05-01  
**Lane:** T-Ground-LanguageSpec / R3 Grounding  
**Scope:** audit / dissolution spec only. No code changes.

## Summary

PR #1430 Exploratory Finding 7 identified a duplicate-fact shape between the
algebra layer and target-language operation template records:

- `dsl/std/algebra.dag` declares operations on `FreeMonoid<T>` and
  `PartialFunction<K, V>` as typed algebraic facts.
- `dsl/std/languages.dag` declares `CollectionOps`, `StringOps`, and `MapOps`
  records whose fields are operation-shaped render templates.
- `src/v3/std/emit_model.dag` and `src/v3/spec/{rust,python,go}.dag` carry a
  staged v3 `CollectionOps` subset with the same shape.
- `src/v3/std/*_method_template_contracts.dag` already adds registry-backed
  `MethodTemplateContract` rows for many of these methods.

The target record fields are not wrong because they contain render data; they
are wrong because field names such as `fold: String` and `map_get: String` form
a second operation ontology. The dissolution path is to make target render rows
reference method/algebra identity by `DeclarationRef` / `MethodRef`, while the
string template remains target-specific render data attached to that identity.

## Duplicate Operation Catalog

### `FreeMonoid<T>` vs `CollectionOps` / `StringOps`

`dsl/std/algebra.dag` declares `FreeMonoid<T>` as the structural authority for
finite sequences. Its fields have typed signatures. `dsl/std/languages.dag`
declares operation-shaped templates for collections and strings. The overlap is:

| Operation | Algebra-side authority | Target-template side | Mismatch |
|---|---|---|---|
| `concat` | `FreeMonoid<T>.concat: fn(FreeMonoid<T>, FreeMonoid<T>) -> FreeMonoid<T>` | `StringOps.concat: String`; staged v3 `CollectionOps.concat: String` | Algebra names sequence concatenation; target record names a render template under a second `concat` field. |
| `empty` / empty collection | `FreeMonoid<T>.empty: FreeMonoid<T>` | `CollectionOps.empty_list: String`; staged v3 `CollectionOps.empty_list: String` | Algebra identity is `empty`; target record invents list-specific field name. |
| `append` / cons-like construction | `FreeMonoid<T>.append: fn(T) -> FreeMonoid<T>` | staged v3 `CollectionOps.cons: String`; method contracts include `append_method` for Python/Go | Target record uses `cons` / append render shapes without a shared operation key. |
| `length` | `FreeMonoid<T>.length: fn() -> Int` | staged v3 `CollectionOps.length: String`; callable rows target `length`; method registry has `length_method` | v3 record and callable rows overlap; method-template rows currently cover `count`, not `length`. |
| `count` | `FreeMonoid<T>.count: fn() -> Int` | `CollectionOps.count: String` in `dsl/std/languages.dag`; method contracts cover `count_method` for Rust/Python/Go | This is mostly absorbed by `MethodTemplateContract`, but the old record field remains a parallel authority. |
| `first` | `FreeMonoid<T>.first: fn() -> T?` | `CollectionOps.first: String`; method contracts cover Rust/Python `first_method` | Contract coverage exists for Rust/Python; Go method-contract row is absent. |
| `last` | `FreeMonoid<T>.last: fn() -> T?` | `CollectionOps.last: String`; method contracts cover Rust/Python `last_method` | Contract coverage exists for Rust/Python; Go row is absent. |
| `map` | `FreeMonoid<T>.map: fn(fn(T) -> T) -> FreeMonoid<T>` | `CollectionOps.map: String`; staged v3 `CollectionOps.map: String`; method contracts cover Python/Go `map_method` | Algebra signature is endomorphic today; render records encode target syntax, not method identity. Rust contract row is absent at HEAD. |
| `filter` | `FreeMonoid<T>.filter: fn(fn(T) -> Bool) -> FreeMonoid<T>` | `CollectionOps.filter: String`; staged v3 `CollectionOps.filter: String`; method contracts cover Rust/Python/Go `filter_method` | Mostly covered by contracts, but v3 `CollectionOps.filter` remains a second template authority. |
| `fold` | `FreeMonoid<T>.fold: fn(T, fn(T, T) -> T) -> T` | `CollectionOps.fold: String`; staged v3 `CollectionOps.fold: String`; method contracts cover Python/Go `fold_method` | Rust has callable and `CollectionOps` templates but no Rust `MethodTemplateContract` row for `fold` at HEAD. |
| `flat_map` | `FreeMonoid<T>.flat_map: fn(fn(T) -> FreeMonoid<T>) -> FreeMonoid<T>` | `CollectionOps.flat_map: String`; method contracts cover Rust/Python/Go `flat_map_method` | Contract rows exist; old record field remains. |
| `any` | `FreeMonoid<T>.any: fn(fn(T) -> Bool) -> Bool` | `CollectionOps.any: String`; method contracts cover Rust/Python/Go `any_method` | Contract rows exist; old record field remains. |
| `all` | `FreeMonoid<T>.all: fn(fn(T) -> Bool) -> Bool` | `CollectionOps.all: String`; method contracts cover Rust/Python/Go `all_method` | Contract rows exist; old record field remains. |
| `enumerate` | `FreeMonoid<T>.enumerate: fn() -> FreeMonoid<Tuple<Int, T>>` | `CollectionOps.enumerate: String`; method contracts cover Rust/Python `enumerate_method` | Go contract row is absent due the sibling `chars` tokenizer caveat pattern; old record remains. |
| `skip` | `FreeMonoid<T>.skip: fn(Int) -> FreeMonoid<T>` | no `dsl/std/languages.dag` record field; method contracts cover Rust/Python/Go `skip_method` | Already moved through method-contract path for current targets. |
| `take` | `FreeMonoid<T>.take: fn(Int) -> FreeMonoid<T>` | no `dsl/std/languages.dag` record field; method contracts cover Rust/Python/Go `take_method` | Already moved through method-contract path for current targets. |
| `sort_by` | `FreeMonoid<T>.sort_by: fn(fn(T, T) -> Int) -> FreeMonoid<T>` | no `dsl/std/languages.dag` record field; method contracts cover Python/Go `sort_by_method` | Rust contract row is absent at HEAD. |
| `contains` | `FreeMonoid<T>.contains: fn(T) -> Bool` | staged v3 `CollectionOps.contains: String`; method registry has `contains_method`; callable rows target `contains` | No current MethodTemplateContract row for `contains`; v3 record remains the render authority. |

`StringOps.split` and `StringOps.chars` overlap with method registry rows
(`split_method`, `chars_method`) and per-target method-template contracts
partially: Rust/Python cover `chars`; Go intentionally skips `chars` because the
escaped empty-string template is tokenizer-blocked. `StringOps.string_literal`,
`string_interp`, `code_point`, and `from_code_point` are render/literal
concerns rather than direct `FreeMonoid<T>` operation duplicates, though they
still should not become algebra identity fields.

### `PartialFunction<K, V>` vs `MapOps`

`PartialFunction<K, V>` is the algebraic authority for maps. The overlap with
`MapOps` is:

| Operation | Algebra-side authority | Target-template side | Mismatch |
|---|---|---|---|
| `empty` | `PartialFunction<K, V>.empty: PartialFunction<K, V>` | `MapOps.empty_map: String` | Target record carries a second map-empty operation field. |
| `get` / `lookup` | `PartialFunction<K, V>.lookup: fn(K) -> V?`; `get: fn(K) -> V?` | `MapOps.map_get: String` | Algebra distinguishes `lookup` / `get`; target field uses a target-shaped `map_get` name. |
| `insert` | `PartialFunction<K, V>.insert: fn(K, V) -> PartialFunction<K, V>` | `MapOps.map_insert: String` | Same operation as render template, different namespace. |
| `has` / `contains_key` | `PartialFunction<K, V>.has: fn(K) -> Bool`; `contains_key: fn(K) -> Bool` | `MapOps.map_contains: String` | Target field collapses the algebra aliases under a third name. |

The method registry already has `map_get_method`, `map_insert_method`,
`map_has_method`, `map_contains_key_method`, `map_keys_method`,
`map_merge_method`, and `map_values_method`, but the current
`MethodTemplateContract` row sets do not populate map-operation rows. This is a
coverage gap, not a need for a new ontology.

## Ontology Mismatch

The algebra layer answers: **what operation exists and what does it mean?**

- `FreeMonoid<T>.fold` is a typed algebraic operation.
- `PartialFunction<K, V>.get` is a typed partial-map lookup.
- Signature, algebra membership, and future cost/complexity facts attach to the
  operation identity.

The target-language layer should answer: **how does this target render that
operation?**

- Rust may render `fold` as `iter().fold(...)`.
- Python may render it as `functools.reduce(...)` or a helper.
- Go may render it as a runtime helper.

`CollectionOps.fold: String` crosses that boundary. The field name `fold` is
not just render data; it is an operation key independent of the algebra/method
registry. The target record therefore duplicates the operation namespace and can
drift from algebraic authority.

Correct shape:

```text
MethodTemplateContract {
  dag_method: MethodRef { decl: fold_method },
  runtime_template: "...",
  emit_template: ...,
  wraps_result: ...,
  placeholder_convention: ...
}
```

The method reference is the operation identity; the template strings are render
data. For algebra-specific overloads, the missing coordinate should be an
algebra/profile reference on a sibling method contract, not another field name
on `CollectionOps`.

## Method-Template-Contract Bridge Status

The bridge already exists:

- `dsl/std/methods.dag` declares the closed method-name registry with one
  `MethodDeclaration` per unique method name from the algebra template lists.
- `src/v3/std/methods.dag` declares `MethodRef { decl: DeclarationRef }`.
- `src/v3/std/emit_model.dag` declares `MethodTemplateContract` keyed by
  `dag_method: MethodRef`.
- `src/v3/std/{rust,python,go}_method_template_contracts.dag` populate
  target-specific render rows for a large subset.
- #1424 added the ratchet that prevents new non-v2 consumers from reading the
  legacy method-template authorities instead of the row-backed contracts.

Existing row coverage for the duplicate operation set:

| Method family | Covered by current `MethodTemplateContract` rows | Missing or partial |
|---|---|---|
| FreeMonoid collection methods | `count`, `filter`, `flat_map`, `any`, `all`, `skip`, `take` for Rust/Python/Go; `first`, `last`, `enumerate`, `chars` for Rust/Python; `concat`, `map`, `fold`, `sort_by`, `append` for Python/Go. | Rust lacks rows for `concat`, `map`, `fold`, `sort_by`, `append`, `length`, `contains`. Go lacks rows for `first`, `last`, `enumerate`, `chars`. |
| String methods | `join`, `split` for Rust/Python/Go; `chars` for Rust/Python. | Go `chars` intentionally skipped; `string_contains` remains target-only and unclassified in Python/Go notes. |
| PartialFunction / map methods | Registry identity exists for map methods. | No per-target `MethodTemplateContract` rows for `map_get`, `map_insert`, `map_contains_key`, `map_has`, `map_keys`, `map_values`, `map_merge` at HEAD. |
| Empty/list literal/cons construction | v3 `CollectionOps` carries `empty_list`, `list_literal`, `cons`; callable rows also cover list construction strategies. | These may belong in value/type construction syntax or callable realizations, not method-template rows; the audit should split construction render data from operation render data before deletion. |

Conclusion: no new bridge type is needed for ordinary algebra method rendering.
The existing `MethodTemplateContract` bridge should absorb operation templates.
Two refinements are still needed before full dissolution:

1. Contract rows must be populated for the missing operation/target pairs.
2. If one method name has distinct semantics by algebra profile, row identity
   needs an algebra/profile coordinate, or a sibling target render contract
   keyed by `(algebra_id, method_id)`. Do not reintroduce that distinction as
   `CollectionOps` vs `StringOps` vs `MapOps` field names.

## Dissolution Slice Spec

### Slice A: Method Identity Completeness

**Goal:** ensure every operation currently named by `CollectionOps`,
`StringOps`, and `MapOps` has a method declaration identity or an explicit
non-method classification.

Work:

- Audit record fields against `dsl/std/methods.dag`.
- Add missing method declarations only if a record field is truly an algebra
  method and not construction/literal syntax.
- Classify `empty_list`, `list_literal`, and `cons` as construction/render
  syntax or callable realization facts, not method-template contract rows unless
  the algebra owner confirms them as methods.
- Preserve the #1424 ratchet pattern: new consumers must read method-contract
  rows or explicitly classified construction rows, not old template maps.

Owner:

- Substrate Manager for vocabulary / method identity shape.
- Grounding for audit/test ratchet once identities exist.

### Slice B: Row Population for Missing Template Contracts

**Goal:** populate `MethodTemplateContract` rows for duplicate operations still
served by `CollectionOps` / `StringOps` / `MapOps`.

First useful subset:

- Rust: `concat`, `map`, `fold`, `contains`, and possibly `length` if it does
  not remain a callable-only realization.
- Go: `first`, `last`, `enumerate`; `chars` remains blocked until the tokenizer
  escaped-empty-string issue is resolved or the template is expressed without
  that literal.
- Map operations for all targets: `map_get`, `map_insert`,
  `map_contains_key` / `map_has`, plus `map_keys`, `map_values`, `map_merge`
  if/when render templates exist.

Owner:

- T-Ground-LanguageSpec / LanguageSpec population for per-target rows.
- Parser/tokenizer owner only for the existing Go `chars` escaped-empty-string
  blocker.

### Slice C: Consumer Migration + Record Retirement Ratchet

**Goal:** make emit/language consumers resolve operation renderings through
`MethodTemplateContract` rows and retire operation-template fields from
`CollectionOps`, `StringOps`, and `MapOps`.

Work:

- Migrate consumers to lookup by `MethodRef` / target row list.
- Split non-method construction syntax (`empty_list`, `list_literal`, `cons`,
  map literal construction) into value construction or callable-realization
  rows with declaration identity.
- Add a source-level ratchet modeled on #1424:
  - allow v2/bootstrap-wall consumers explicitly;
  - fail on new non-v2 reads of `CollectionOps.<operation>`,
    `StringOps.<operation>`, or `MapOps.<operation>` when a
    `MethodTemplateContract` row exists;
  - allow comments/docs.
- Delete or shrink `CollectionOps`, `StringOps`, and `MapOps` once no
  operation-template consumers remain.

Owner:

- Grounding for consumer migration and ratchet.
- Substrate Manager only if the record deletion requires carrier-shape changes
  or a new construction-syntax carrier.

## Owner Routing

| Slice | Primary owner | Supporting owner | Notes |
|---|---|---|---|
| A: method identity completeness | Substrate Manager (`#1130`) | Grounding audit support | Decides whether remaining fields are methods, construction syntax, or target-only shortcuts. |
| B: target contract row population | T-Ground-LanguageSpec / Grounding | Parser/tokenizer for Go `chars` blocker | Uses existing `MethodTemplateContract`; no new bridge type for ordinary methods. |
| C: consumer migration / ratchet / record shrink | Grounding | Substrate Manager for carrier deletion/refinement | Mirrors #1424 deferral-ratchet discipline. |

## Substrate-Prerequisite Checklist

| Prerequisite | Required shape | Owner |
|---|---|---|
| Method identity for every operation-template field | Either a `MethodDeclaration` / `MethodRef` target or an explicit non-method render-syntax classification. | Substrate Manager (`#1130`) |
| Optional algebra/profile coordinate | Needed only if flat `MethodRef` identity is too coarse for an overloaded method name across `FreeMonoid`, `PartialFunction`, and string-family methods. | Substrate Manager (`#1130`) |
| Construction syntax home | `empty_list`, `list_literal`, `cons`, and map literal construction need a non-duplicative home if they do not become method contracts. | Substrate Manager + Grounding |
| Contract row population | Per-target rows for missing method/target pairs listed above. | T-Ground-LanguageSpec |
| Consumer lookup path | Emit/language consumers can query `MethodTemplateContract` rows by target + `MethodRef` without falling back to record field names. | Grounding |
| Retirement ratchet | A #1424-style ratchet prevents new non-v2 consumers from reading old operation-template records once row-backed alternatives exist. | Grounding |

This audit surfaces no dependency on the current Substrate queue for
`BoundDeclaration`, per-target integer/string inhabitance rows, or canonical
lifetime-axis vocabulary. It does surface one separate Substrate-owned question:
whether flat `MethodRef` is sufficient for all operation render contracts or
whether a target render row needs an explicit algebra/profile coordinate. Route
that question to `#1130` before authoring row-population slices that would
otherwise encode the distinction in record names.

**2026-05-02 follow-up receipt:** `method-render-identity-6q.md` answers that
question for the current row-authoring lane. Keep flat `MethodRef` as the
render identity and add no `MethodRenderRef` / `(algebra_id, method_id)` render
key until a target needs two rows for the same `MethodRef` whose difference is
genuinely algebra/profile-owned.
