# CollectionOps / StringOps / MapOps — algebra contract reframe (T-Ground-LanguageSpec)

**Status:** Phase 1 design + proof-of-shape (Rust `CollectionOps.fold` migrated).  
**Dispatch:** Director throughput sweep / **#828**; ROADMAP **§542** row; ledger **r3-debt-paydown-ledger** (`CollectionOps` / `StringOps` / `MapOps` duplicate operation surfaces).  
**Authority:** `dsl/std/algebra.dag` (`FreeMonoid<T>`, `PartialFunction<K,V>`), `dsl/std/languages.dag` (legacy per-target string ontology), `src/v3/std/emit_model.dag` (`CollectionOps`, `MethodTemplateContract`), `src/v3/std/*_method_template_contracts.dag`.

## Problem

Two parallel ontologies name the same semantic operations:

1. **Algebra substrate** — `FreeMonoid<T>` declares `fold`, `concat`, `map`, … as typed method shapes (`fn` fields on the algebra record). `PartialFunction<K,V>` does the same for map operations.
2. **LanguageSpec template records** — `CollectionOps` / `StringOps` / `MapOps` historically carried **opaque `String` templates per field** (`fold: String`, …), inviting a second normalization table and drift from algebra naming.

Per `feedback_parallel_representation_debt`: when the canonical algebra declaration exists, **consume it**; do not scaffold a parallel string-keyed ontology around it.

## Reframe (identity vs render data)

| Legacy identity | Reframed identity | Render payload (unchanged role) |
|-----------------|-------------------|----------------------------------|
| `fold: String` (“this field is named fold”) | **Realization of** `FreeMonoid<T>.fold` **for this target** — pinned by `MethodTemplateContract.dag_method → fold_method` (`dsl/std/methods.dag`) | `MethodTemplateContract.runtime_template` / `emit_template` (still `String` inside `SingleTemplate` / `HigherOrderTemplates`) |

**Algebra fields drive identity**; **target strings are realization payload only**. The `LanguageSpec` / `CollectionOps` row holds a **`DeclarationRef`** to a `MethodTemplateContract` value (same carrier as `rust_method_template_contracts` rows), not a second invented method name.

## Carrier shape (precedent)

Use existing **`MethodTemplateContract`** (`src/v3/std/emit_model.dag`):

- `dag_method: MethodRef` — must reference the **registry** method (`fold_method`, `concat`/`append` aliases, `map_method`, …) so the parent edge dissolves into a **declaration ref** to the algebra-backed method table, not a free string.
- `runtime_template` / `emit_template` — target render data (placeholders `{recv}`, `{init}`, `{body}`, …).
- `wraps_result` / `placeholder_convention` — closed metadata already used by method-template rows.

**No new meta-markers** and no `__is_fold`-style string flags.

## Per-field migration order (remaining work)

High confidence first (direct **FreeMonoid** hits, arity/shape already aligned with emitters):

1. **`fold`** — monoid-shaped reducer; **done in Phase 1 proof** (`emit_model.CollectionOps.fold_contract` + per-target `MethodTemplateContract`): **Rust** named `rust_language_spec_free_monoid_fold_contract` in `src/v3/spec/rust.dag`; **Python** + **Go** named `python_language_spec_free_monoid_fold_contract` / `go_language_spec_free_monoid_fold_contract` in `src/v3/std/{python,go}_method_template_contracts.dag` (single authority — **not** duplicated in the per-target `List<MethodTemplateContract>` rows; those lists previously carried a conflicting `fold_method` row).
2. **`concat`**, **`length`** (and **`is_empty`** where it mirrors `length` / monoid emptiness) — align to `concat` / `length` / derived emptiness on `FreeMonoid<T>`.
3. **`map`**, **`filter`**, **`flat_map`**, **`any`**, **`all`** — already have rich `MethodTemplateContract` rows on Rust; migrate `CollectionOps` fields to refs that **either** point at those list-backed rows (if made addressable) **or** named contracts that **share** the same templates (dedupe in a later “delete duplicate literal” pass).
4. **List literal / cons / empty_list** — still monoid-shaped surface; likely `empty` / `append` realizations plus list-syntax scaffold.
5. **`StringOps`** — align `concat` to `FreeMonoid<Char>` scalar profile; other fields to monoid / character methods as declared in algebra + `std.methods`.
6. **`MapOps`** — align to `PartialFunction<K,V>` (`lookup`, `insert`, `empty`, …).

**StringOps / MapOps** follow the same pattern as `CollectionOps`: one `DeclarationRef` per field to a `MethodTemplateContract` (or a future thin wrapper if we need arity not expressible in today’s row).

## DSL `languages.dag` (non-v3) note

`dsl/std/languages.dag` still carries the broader `CollectionOps` / `StringOps` / `MapOps` **string** records for dsl-level language facts. **v3 emit authority** lives under `src/v3/std/emit_model.dag` + `src/v3/spec/{rust,python,go}.dag`. Dissolving the dsl duplicate is a **follow-up** once v3 proves the shape end-to-end (avoid two migrations in flight without a consumer for dsl-side refs).

## Emitter contract

At index-build time the emitter **resolves** each `CollectionOps.*_contract: DeclarationRef` → `MethodTemplateContract` value body → **extracts** the `emit_template` `SingleTemplate` string for that site (fail-closed if a higher-order shape appears where a single template is required today).

Runtime render sites use those resolved templates; the old **opaque `String` fields** on `emit_model.CollectionOps` for migrated operations are deleted in favor of contract refs.

### `%Q` vs `placeholder_convention` (Rust today)

The **Rust** emitter applies **`template.replace("%Q", "\"")`** when loading **any** syntax or contract template string that came through the substrate (same path as legacy `CollectionOps` string fields). **`%Q`** is the v3 tokenizer’s stand-in for a literal double-quote inside a `.dag` string literal (see `src/v3/spec/rust.dag` header comment on `{quote}` / `%Q`). It is **not** declared on `MethodTemplateContract` rows today — it is an **emitter-side decoding** step for Rust-shaped carriers, not a second semantic axis.

**Next slices:** either keep documenting `%Q` here (and in any new contract rows that use it) as long as the convention stays Rust-local, or **lift** quote-escaping into a first-class substrate fact (e.g. extend `placeholder_convention` or template-segment substrate) if Python/Go need the same escape channel from shared contract literals. Until then, any new `MethodTemplateContract` text consumed by `rust_target` should treat `%Q` like existing `LanguageSpec` templates.

**Phase-2 receipt (concat / length / is_empty contracts, PR #1602):** lifting `%Q` into `placeholder_convention` (or another substrate fact) stays **deferred** — Python/Go `MethodTemplateContract` literals added for those fields do not use `%Q`; only `rust_target`'s `method_contract_single_emit_template_string` applies the Rust-local decode. Revisit when a **shared** cross-target contract literal needs escaped quotes.

## References

- ROADMAP.md §542 (row text + dissolution).
- `docs/debt/r3-debt-paydown-ledger-2026-05-02.md` (ledger row).
- `docs/audit/collection-ops-string-ops-map-ops-duplicate-fact.md` (ontology debt narrative, if present).
