# Format-model reconciliation — record spelling onto single authority

> Record-spelling reconciliation: one concept — how a format spells records — was forked across three models, and no text format routes serialization through any of them today. DESIGN refs: §2 (one concept every scale — decompress the leaf, map to existing carriers, reduce duplicates), §3 (single authority — field-by-field decomposition onto ConfigFormat, CommentSyntax, LayoutProtocol; delete uninhabited scaffolds), §5 (construction over validation — the swap test is execution-grounded), §6 (complementary to [regime-2 shared emission fold](regime2-shared-emission-fold.md), not a parallel ledger).

**Status:** planning tracker · **`.dag` carrier is authority** (§6). Linked from `ROADMAP.md` §6 cross-media band. Carrier facts verified against the live tree 2026-06-30. **Code keystone:** still-wolf-292 / PR #6045 (C1) — this doc captures only; no serializer code here.

## 1. The fork — three models, zero routed serializers

Layout (line/indent/newline) is [regime-2 shared emission fold](regime2-shared-emission-fold.md). **This doc owns record spelling** — how key/value, assign, quoting, and nesting render into text or JSON:

| model | where | role | verdict |
| --- | --- | --- | --- |
| `ConfigFormat` | `dsl/std/languages.dag` | format **identity** (id, name, extensions, comment) | **keep** |
| `FormatModel` | `dsl/std/languages.dag:36` | indent, max_line_width, import_grouping, trailing_newline | **uninhabited dead scaffold** — delete |
| `OutputFormat` | `dsl/std/render.dag:160` | name, indent_unit, kv_separator, list_prefix, comment_prefix, section_separator, trailing_newline | **only live knob record** — sole consumer `gitignore_output_format`; `comment_prefix` is a §3 nickname of `CommentSyntax.line_prefix` |

No text format today routes record serialization through any of these types — emitters hand-roll `concat` / `match` per site.

## 2. The decomposition — map each field to its single authority (§3)

Decompose `OutputFormat` field-by-field onto existing carriers; the irreducible residue is the record-spelling knobs only:

- `name` → `ConfigFormat` (identity already lives there)
- `comment_prefix` → `CommentSyntax.line_prefix` (derive `#` from format comment, never duplicate)
- `indent_unit`, `trailing_newline` → `std.layout.LayoutProtocol` (line-layout half; owned by regime-2)
- **Residue → `SerializationKnobs`** in `std.languages`: assign separator, entry separator, open/close delimiters, quoting policy — a comment-free record-spelling value

**Target pipeline:** a total `serialize_record` fold producing `std.layout.Doc` (recursive — JSON/proto nesting fits). Compose with regime-2 `render(doc, protocol)` for the line half.

```
doc = serialize_record(record, knobs)   // record half (this doc)
text = render(doc, layout_protocol)     // line half (regime-2)
```

### The swap test (acid test)

The **same** record projection renders as manifest text **and** JSON by swapping only the `SerializationKnobs` value — one `serialize_record`, two knob instances, byte-identical witnesses. This is the substrate that makes JSON-as-first-class real under §6 cross-media.

### Relationship to regime-2 (complementary, not duplicate)

- [regime-2 shared emission fold](regime2-shared-emission-fold.md) — **line-layout half**: one `render(doc, protocol)` fold over `std.layout.Doc` for yaml/gitignore/runner-deploy/ci.yml projections.
- **This doc — record-spelling half**: `serialize_record` → `Doc`, then regime-2 renders. Cross-reference only; do not merge the plans.

## 3. Hazard — do not build on `std.render` kv helpers

`std.render` `kv_pair` / `kv_block` are **broken** for real emission: in `.dag` runtime literals bare curly-brace interpolation is live, while the escaped-brace form in `render.dag:119-120` emits literal brace characters, not key=value pairs. `digest_render.dag` and friends must route through `serialize_record`, not these helpers.

## 4. Ordered scope (C1–C6)

1. **C1 (keystone — still-wolf-292 / PR #6045):** `SerializationKnobs` residue in `std.languages` + recursive `serialize_record` → `Doc` + delete `FormatModel`; migrate runner manifest (`dsl/gunbc/runner_deploy_emit.dag` — `manifest_host_text` / `session_host_text` / `operating_row_text`) byte-identically; JSON knobs instance + swap-test witness.
2. **C2 (folded into C1):** runner manifest + JSON as the first proving instance — not a separate roadmap row.
3. **C3:** gitignore de-fork — migrate live `OutputFormat` consumer (`dsl/gunbc/gitignore_emit.dag` + `extdeps/git/gitignore.dag`) onto `SerializationKnobs` + `ConfigFormat`, derive `#` via `CommentSyntax`, then **delete `OutputFormat`** and orphan `extdeps/git/gitignore_render.dag` (declares knobs then ignores them).
4. **C4:** dnsmasq emit — add dnsmasq `ConfigFormat` + knobs; keep cited positional micro-syntax honest (do not force pure key=value).
5. **C5 (low):** digest/accelerator kv blocks (`dsl/gunbc/digest_render.dag` and friends) — route through `serialize_record`; supersedes broken `std.render` kv helpers.
6. **C6 (cosmetic):** CSS declaration blocks — `dsl/gunbc/roadmap_style.dag` `css_rule(selector, props)` has `props` as raw `String` (`css_rule_props_scaffold`); flat `property:value` records (assign `: `, separator `; `) are `serialize_record` candidates; selector nesting stays structural. Plus yaml/markdown/html identity-link cosmetics.

## 5. Audit receipts (live tree 2026-06-30)

- `FormatModel` at `languages.dag:36` — type only, zero `data` rows, not in `LanguageSpec`.
- `OutputFormat` at `render.dag:160` — consumed by `extdeps/git/gitignore.dag` `gitignore_output_format` only.
- `extdeps/git/gitignore_render.dag` — orphan knob declarations, ignored at emit.
- `languages_consumer_census` baselines: 71 total decls, 64 language rows, 7 format rows (`src/v2/lens/languages_consumer_census.dag:9-11`).

## 6. Open / boundaries

- C1 code lands in still-wolf-292 / PR #6045 — this capture PR is docs + roadmap authority only.
- Regime-1 grammar-inverse language emit stays out of scope (v2 `TargetModel` rows).
- `import_grouping` on deleted `FormatModel` — language-emit-only if it survives at all; do not carry into `SerializationKnobs`.

## Dissolution trigger (DESIGN §6)

Delete this doc when all three knob models are collapsed into one: FormatModel deleted, OutputFormat dissolved into ConfigFormat + CommentSyntax + LayoutProtocol + SerializationKnobs, and every record-emitting format routes through serialize_record — byte-identical-witnessed on runner manifest, gitignore, and the manifest-text vs JSON swap test.
