# Format-model reconciliation — one layout authority for regime-2, render, and languages

> Plan doc for the **§6 format-model reconciliation** lane. Merges the parallel layout/format authorities in `std` so regime-2 emission, generated-artifact headers, and the `languages.dag` surface share one grounded model. DESIGN refs: §2 (one concept every scale — indent/comment/newline spelled once), §3 (single authority — `FormatModel` vs `OutputFormat` vs the `Language` / `MarkupFormat` / `ConfigFormat` trio is a nickname fork), §4 (regime-1 grammar-inverse vs regime-2 pure projection stay distinct regimes; this doc owns only the shared layout knobs both consume), §5 (construction over validation — merge types before adding another lens), §6 (DFS before mint; price in displaced drift pain, not elegance).

**Status:** planning tracker · **`.dag` carriers are authority** (§6). Linked from `ROADMAP.md` §6. Carrier facts verified against the live tree 2026-06-30 — re-check receipts before acting.

## 1. The fork — four parallel spellings of the same concept

Layout and surface-format facts (indent, comment prefix, line width, trailing newline, list/kv separators) are modeled in **four places** that should be one horizontal concept:

| authority | where | fields / role | live consumers |
| --- | --- | --- | --- |
| `FormatModel` | `dsl/std/languages.dag` | `indent`, `max_line_width`, `import_grouping`, `trailing_newline` | **none** — type only, not in `LanguageSpec`, zero `data` rows |
| `OutputFormat` | `dsl/std/render.dag` | `indent_unit`, `kv_separator`, `list_prefix`, `comment_prefix`, `section_separator`, `trailing_newline` | `gitignore_output_format` (`extdeps/git/gitignore.dag`) |
| regime-2 **protocol** | [regime2-shared-emission-fold](regime2-shared-emission-fold.md) §2 | per-format global knobs passed to `render(doc, protocol)` | **planned** — yaml/gitignore/runner_deploy emitters hand-roll layout today |
| `Language` / `MarkupFormat` / `ConfigFormat` | `dsl/std/languages.dag` | three parallel carriers sharing `CommentSyntax` + extensions | `std.render` headers; `languages_consumer_census` tracks **7** `*_format` rows separately from **64** language rows |

Downstream but **not** the same layer: `src/v2/extdeps/formatters/*` (rustfmt, clang-format, …) are **cited upstream formatter configs** — realization handlers for post-emit pretty-print, not the seed layout model. Reconciliation maps a *subset* of `FormatModel` onto them; it does not merge the full rustfmt surface into `std`.

## 2. The reconciliation target — one layout authority, two regimes

Keep the [regime-1 vs regime-2 split](regime2-shared-emission-fold.md) — do **not** collapse grammar-inverse language emit with forward-only config projection. Reconcile only the **shared layout substrate** both regimes read:

- **`std.layout` `Doc` IR** (net-new, DFS-confirmed) — the structural document tree (`text` / `line` / `nest` / `concat` / `sep`); distinct from `std.markup` (HTML tags) and from v2 `serialize_target` (grammar rows entangled). Owned by [regime-2 shared emission fold](regime2-shared-emission-fold.md); this doc names the protocol type that parameterizes `render`.
- **`FormatProtocol` (working name)** — the reconciled successor merging `FormatModel` and `OutputFormat`: the thin per-medium record of global spelling knobs (indent unit, comment prefix, newline, trailing_newline, kv/list separators where applicable). Regime-2 `render(doc, protocol)` and `std.render` header helpers both take this type — one home.
- **`SurfaceFormat` coproduct** — `LanguageSurface` or `MarkupSurface` or `ConfigSurface` (or a type parameter on one carrier) replacing the parallel `Language` / `MarkupFormat` / `ConfigFormat` trio; `CommentSyntax` lives once. Migration tracked by `languages_consumer_census` format-row ratchet shrinking toward 0.

```
render(doc: Doc, protocol: FormatProtocol) -> String     // regime 2
format_header(protocol: FormatProtocol, gen: GeneratorHeader) -> String  // generated-artifact headers
```

### Discriminator — no 4th hand-fold

Same rule as regime-2: **zero format-specific branches in `render`.** Format-specific text prep stays in `project_X_to_doc`; global knobs stay in `FormatProtocol`. If a new format forces a `render` special-case, the `Doc` IR is wrong — stop and remodel.

## 3. Audit receipts (live tree 2026-06-30)

- `FormatModel` is inert — defined at `languages.dag:36`, absent from `LanguageSpec`, no `data` row.
- `OutputFormat` is the only live std layout record (`render.dag:160`); `gitignore_output_format` is its sole external consumer.
- `languages_consumer_census` baselines: **71** total `data` decls, **64** per-language rows, **7** `*_format` rows — the format/language split is the §3 fork made countable.
- Regime-2 serializers (`serialize_yaml`, `serialize_gitignore`, `expected_runner_deploy_manifest`) each re-implement indent/comment/line-join — the §2 violation [regime2-shared-emission-fold](regime2-shared-emission-fold.md) addresses; **this doc** is the type-level prerequisite so their shared `protocol` is not a fourth nickname.

## 4. Sequencing (dependency-ordered)

1. **DFS + merge `FormatModel` → `FormatProtocol`** — absorb `OutputFormat` fields; deprecate `OutputFormat` with a typed alias or migration shim; witness: `gitignore_output_format` unchanged at the byte level after regen.
2. **Mint `std.layout` `Doc` + format-agnostic `render` fold** — [regime-2 step 1](regime2-shared-emission-fold.md); `render` parameter is `FormatProtocol` from step 1.
3. **Migrate regime-2 emitters** — gitignore + runner_deploy (trivial), then yaml/ci.yml (heavy); byte-identical witnesses; public `expected_*()` surfaces unchanged.
4. **Unify `Language` / `MarkupFormat` / `ConfigFormat`** — parameterized `SurfaceFormat` (or extend `LanguageSpec` with a `format: FormatProtocol` field); ratchet `languages_consumer_census_format_row_baseline` down as rows merge. Not blocked on regime-2 but sequenced after the protocol type exists so the merge target is stable.
5. **Map cited formatters** — optional rows mapping `FormatProtocol` subsets → `v2.extdeps.formatters.*` configs where post-emit pretty-print applies; extdeps stays cited-upstream, std owns only the agnostic shape.

## 5. Cross-arc edges

- **Enabler for** [regime-2 shared emission fold](regime2-shared-emission-fold.md) — supplies the `protocol` type; regime-2 lands the `Doc` IR + `render` fold.
- **Cited by** [emission-ingestion-inverse](emission-ingestion-inverse.md) §5.3 — regime-2 is the ② lens-residue slice; this doc is the §3 single-authority kill for its layout knobs.
- **Ratchet** `languages_consumer_census` — format-row count is the migration-progress witness (consolidation ratchet per [self-applying-lenses](self-applying-lenses.md), not an auto-applier).
- **Out of scope** — v2 `TargetModel` grammar-inverse layout (regime 1); `std.markup` HTML tree; full rustfmt config surface (extdeps cited authority).

## 6. Open / boundaries

- Whether `import_grouping` (`StdExternalLocal | NoGrouping`) belongs in `FormatProtocol` or stays language-emit-only — decide at merge time; do not carry unused fields into the reconciled type.
- Whether yaml eventually crosses to regime 1 (round-trip-gated) is [fenced in regime-2](regime2-shared-emission-fold.md); `FormatProtocol` must not assume ingest exists.
- Name bikeshed (`FormatProtocol` vs reusing `FormatModel`) resolves at construction — the merge witness is the point, not the identifier.

## Dissolution trigger (DESIGN §6)

Delete this doc when `FormatModel` and `OutputFormat` are merged into one live `FormatProtocol` (or renamed successor) consumed by both `std.render` header helpers and regime-2 `render(doc, protocol)`, the seven `*_format` rows in `languages.dag` are ratcheted into a unified `SurfaceFormat` carrier with a shrinking `languages_consumer_census` format-row baseline, and byte-identical witnesses cover gitignore/ci.yml/runner-manifest — at which point the fork is a witnessed property and this prose tracker is redundant.
