# Format-model reconciliation — record spelling onto single authority

> Record-spelling reconciliation: one concept — how a format spells records — was historically forked across three models. The live tree has dissolved the two legacy models into `ConfigFormat`, `CommentSyntax`, `LayoutProtocol`, and `SerializationKnobs`; this tracker now owns only the remaining emitter migrations. DESIGN refs: §2 (one concept every scale), §3 (single authority), §5 (execution-grounded swap test), §6 (complementary to [regime-2 shared emission fold](regime2-shared-emission-fold.md), not a parallel ledger).

**Status:** implementation tail · **`.dag` carrier is authority** (§6). Carrier facts re-verified against the live tree 2026-07-31.

## 1. The historical fork and its live resolution

Layout (line/indent/newline) is [regime-2 shared emission fold](regime2-shared-emission-fold.md). **This doc owns record spelling** — how key/value, assign, quoting, and nesting render into text or JSON:

| model | where | role | verdict |
| --- | --- | --- | --- |
| `ConfigFormat` | `std.languages` `ConfigFormat` | format identity plus optional record knobs | **live authority** |
| `FormatModel` | absent from the live tree | legacy layout scaffold | **deleted** |
| `OutputFormat` | absent from the live tree | legacy mixed layout/record knobs | **deleted and decomposed** |

`std.languages` `SerializationKnobs` and `std.serialize` `serialize_record_doc` now carry record spelling. `gunbc.runner_deploy_emit` routes manifest and JSON projections through them; remaining boutique emitters are scoped below.

## 2. The decomposition — map each field to its single authority (§3)

The deleted `OutputFormat` was decomposed field-by-field onto existing carriers; the irreducible residue is the record-spelling knobs only:

- `name` → `ConfigFormat` (identity already lives there)
- `comment_prefix` → `CommentSyntax.line_prefix` (derive `#` from format comment, never duplicate)
- `indent_unit`, `trailing_newline` → `std.layout.LayoutProtocol` (line-layout half; owned by regime-2)
- **Residue → `SerializationKnobs`** in `std.languages`: assign separator, entry separator, open/close delimiters, quoting policy — a comment-free record-spelling value

**Live pipeline:** `std.serialize` `serialize_record_doc` produces `std.layout` `Doc` recursively; compose it with `std.layout` `render` for the line half.

```
doc = serialize_record(record, knobs)   // record half (this doc)
text = render(doc, layout_protocol)     // line half (regime-2)
```

### The swap test (acid test)

`test.claim.config_record_emit_witness` and `test.claim.runner_placement_witness` exercise the same record projection with manifest and JSON knob values. This is the execution-grounded substrate for JSON-as-first-class under §6 cross-media.

### Relationship to regime-2 (complementary, not duplicate)

- [regime-2 shared emission fold](regime2-shared-emission-fold.md) — **line-layout half**: one `render(doc, protocol)` fold over `std.layout.Doc` for yaml/gitignore/runner-deploy/ci.yml projections.
- **This doc — record-spelling half**: `serialize_record` → `Doc`, then regime-2 renders. Cross-reference only; do not merge the plans.

## 3. Boundary — `std.render` kv helpers are presentation utilities

`std.render` `kv_pair` and `kv_block` correctly render caller-supplied separators, but they do not carry format-owned `SerializationKnobs` or recursive record structure. Keep them for presentation-only key/value lists; record emitters such as manifest and JSON projections route through `std.serialize` `serialize_record_doc`.

## 4. Ordered scope (completed foundation, then remaining tail)

1. **C1 complete:** `std.languages` `SerializationKnobs` + recursive `std.serialize` `serialize_record_doc` → `Doc`; `FormatModel` deleted; `gunbc.runner_deploy_emit` manifest fields migrated byte-identically.
2. **C2 complete (folded into C1):** runner manifest + JSON are the first proving instance, witnessed by `test.claim.config_record_emit_witness` and `test.claim.runner_placement_witness`.
3. **C3 complete:** `OutputFormat` and the orphan gitignore renderer are absent. `gunbc.gitignore_emit` derives comments from `std.languages` `gitignore_format` and renders a `Doc` with its `gitignore_protocol`; because gitignore is a line-list rather than a record, it does not acquire fake `SerializationKnobs`.
4. **C4 complete at the honest boundary:** `extdeps.formats.dnsmasq` projects directives to `Doc` with `dnsmasq_protocol`; its positional micro-syntax remains explicit rather than being forced into a key/value record.
5. **C5 (low):** classify digest key/value blocks (`gunbc.digest_render`) by semantic grain: presentation-only lists stay on the correct `std.render` helpers; true record projections route through `std.serialize` `serialize_record_doc` with byte-identical witnesses.
6. **C6 complete:** `gunbc.roadmap_style` now uses typed `CssDecl` rows on shared `BuildRule` machinery; the old `css_rule_props_scaffold` remains only as historical text in its dissolution note.

## 5. Audit receipts (live tree 2026-07-31)

- No `FormatModel`, `OutputFormat`, `gitignore_output_format`, or `gitignore_render` declaration remains in the live tree.
- `std.languages` `ConfigFormat.record` carries optional `SerializationKnobs`; `json_record_knobs` is a live instance.
- `std.serialize` `serialize_record_doc` is the recursive record-spelling fold; `gunbc.runner_deploy_emit` is its live manifest/JSON consumer.
- `v2.lens.languages_consumer_census` baselines: `languages_consumer_census_data_decl_baseline`, `languages_consumer_census_per_language_row_baseline`, `languages_consumer_census_format_row_baseline`.

## 6. Open / boundaries

- The remaining work is emitter migration, not another model declaration.
- Regime-1 grammar-inverse language emit stays out of scope (v2 `TargetModel` rows).
- Language-emission `import_grouping` stays outside `SerializationKnobs`.

## Dissolution trigger (DESIGN §6)

Delete this doc when the C5 digest/accelerator tail is classified by semantic grain and every true record projection routes through `std.serialize` `serialize_record_doc` with byte-identical witnesses; presentation-only lists explicitly remain on `std.render`. The legacy FormatModel/OutputFormat collapse, runner manifest/JSON swap, gitignore line-layout de-fork, dnsmasq boundary, and CSS typed-row migration are already complete.
