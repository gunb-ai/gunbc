---
status: draft (wait-window; awaits R3 host restoration before dispatch; operator-bridge expected to ship to PR #1782)
authority parent: R3 Substrate Manager (#1739)
ratification: Director (#828) ratified slate (1) + (2) + (3a) at #issuecomment-4377776495 (2026-05-05)
roadmap row: ROADMAP.md "LLM service flattening" (`:393`) — names `structural_coverage_gap_anthropic_tool_result_content_wire_shape` and `structural_coverage_gap_anthropic_tool_result_additive_blocks` as the live tracking rows. The previously-cited `anthropic_tool_result_full_content_surface` row is **RESOLVED 2026-05-03** (ROADMAP `:344`); this slice does NOT reopen it. Open work lives on the two named gap rows only.
sibling briefs:
  - r3-coproduct-1-openai-chat-message-full-worker.md
  - r3-coproduct-3-anthropic-messages-200-residual-worker.md
---

# R3 Coproduct slice 2 — Anthropic tool_result content wire shape

## Context

`dsl/extdeps/llm/anthropic.dag:204` carries
`structural_coverage_gap_anthropic_tool_result_content_wire_shape`
plus the closely-coupled
`structural_coverage_gap_anthropic_tool_result_additive_blocks`
at `:67`. The receipt at `:204` reads:

```
closure:rest_request_wire_serde_alignment|llm.Anthropic.Messages|AnthropicUserContentBlock.UserToolResultBlock.content|residue:tool_result_content_must_serialize_scalar_text_or_content_block_array|trigger:emitted serde for AnthropicToolResultContent serializes ToolResultText as scalar string and ToolResultBlocks as content block array
```

Today's `AnthropicToolResultContent` (`anthropic.dag:62-65`) reads:

```dag
type AnthropicToolResultContent
  = ToolResultText { text: String }
  | ToolResultBlocks { blocks: List<ContentBlock> }
```

Two coupled gaps:

1. **Wire-serialization shape (this row, `:204`)** — the wire
   serializes `ToolResultText` as a scalar string but
   `ToolResultBlocks` as a content-block array. The current
   `CoproductWireContract` rows at `:20-30` cover the outer
   request coproducts (`AnthropicChatMessage`,
   `AnthropicUserContentBlock`,
   `AnthropicAssistantContentBlock`); they don't cover the
   scalar-vs-array dispatch on `tool_result.content`.
2. **Additive missing variants (subsumed row, `:67`)** —
   `ToolResultBlocks.blocks` is `List<ContentBlock>` but the
   closure tag names three missing variants:
   `missing:document | missing:search_result | missing:tool_reference`.

The previously-tracked broader row
`anthropic_tool_result_full_content_surface` is **RESOLVED
2026-05-03** per ROADMAP `:344` — the receipt row was already
deleted; this slice does NOT reopen it. The two open gap rows
named above (`structural_coverage_gap_anthropic_tool_result_content_wire_shape`
at `:204` and `structural_coverage_gap_anthropic_tool_result_additive_blocks`
at `:67`) are tracked under ROADMAP `:393` "LLM service flattening"
as residue from the 2026-05-03 closure. This slice retires both
gap rows in one PR (Director-ratified subsumption per
#issuecomment-4377776495).

Adjacent `CoproductWireContract` pattern: `anthropic.dag:20-30`
(three live wire contracts using `InternallyTaggedObject` with
`StripPrefixSuffixAndSnakeCase`).

## Slice

1. **Wire-shape sub-slice.** Author a `CoproductWireContract` for
   `AnthropicToolResultContent` capturing the scalar-vs-array
   dispatch. The existing pattern (`InternallyTaggedObject`)
   does not fit because `ToolResultText` serializes as a bare
   scalar, not a tagged object. This requires either:
   - Extending `std.serialization::VariantEncoding` with a new
     dispatch shape (e.g., `UntaggedScalarOrTagged` or similar
     — naming open). This is a P1 substrate-fact-introduction
     event; brief STOPs at this point and surfaces to R3
     Substrate Manager for procedure.
   - OR using an existing `VariantEncoding` shape that already
     covers this dispatch — worker greps `std/serialization.dag`
     at dispatch for current variant set and confirms.

   **Sub-slice STOP**: if no existing `VariantEncoding` covers
   the dispatch, the wire-shape sub-slice cannot land without a
   prior P1 procedure on `std.serialization`. Surface; do not
   silently invent a new `VariantEncoding` variant in this
   brief's scope.

2. **Additive-blocks sub-slice.** `ContentBlock` lives at
   `dsl/extdeps/llm/llm.dag:41` as a **shared LLM primitive**
   (cross-provider — both Anthropic and OpenAI consume it).
   Extending it directly with `Document | SearchResult |
   ToolReference` would pollute the shared shape with
   Anthropic-specific variants. Per `INVARIANTS.md` P2 (single
   authority / boundary discipline) and the codex BLOCKING
   finding on the prior brief revision (sha 215f08b5), the
   correct shape is an **Anthropic-specific nested block
   carrier** that wraps shared `ContentBlock` for the cross-
   provider variants and adds the Anthropic-only variants:

   ```dag
   // Anthropic-specific tool-result block; layered atop shared
   // ContentBlock for cross-provider variants (text/image) and
   // adds Anthropic-only nested block kinds. Lives in
   // dsl/extdeps/llm/anthropic.dag, NOT dsl/extdeps/llm/llm.dag.
   type AnthropicToolResultBlock
     = AnthropicSharedBlock { block: ContentBlock }   // delegates to shared primitive
     | AnthropicDocumentBlock { ... }                 // Anthropic-only
     | AnthropicSearchResultBlock { ... }             // Anthropic-only
     | AnthropicToolReferenceBlock { ... }            // Anthropic-only
   ```

   Then `AnthropicToolResultContent.ToolResultBlocks` switches
   from `blocks: List<ContentBlock>` to `blocks:
   List<AnthropicToolResultBlock>`. Cross-provider variants
   reach through the `AnthropicSharedBlock` delegate; the
   shared primitive stays uncontaminated. Worker re-verifies
   at dispatch that `ContentBlock` is in fact cross-provider
   (the wait-window grep confirmed `dsl/extdeps/llm/llm.dag:41`
   is the home and `src/v3/std/anthropic_schema.dag:59` is a
   separate Anthropic-internal `ContentBlock`-named type — if
   those have converged or diverged further, surface).
   Concrete payload shapes for the Anthropic-only variants
   ground against the Anthropic Messages API reference.

3. **Wire ratchets.** Wire-emission tests on the v2 Rust
   emitter for `AnthropicToolResultContent` covering both
   scalar-text and block-array shapes; per-variant emission
   tests for the four `AnthropicToolResultBlock` variants
   (`AnthropicSharedBlock` delegate + `AnthropicDocumentBlock`
   + `AnthropicSearchResultBlock` + `AnthropicToolReferenceBlock`).
   Shared `dsl/extdeps/llm/llm.dag::ContentBlock` is **not**
   re-tested here — it stays unchanged. Existing pattern lives
   in `src/v2/tests/src/pipeline.rs` (re-verify surface at
   dispatch).

4. **Closure-tag retirement.** Both
   `structural_coverage_gap_anthropic_tool_result_content_wire_shape`
   (`:204`) and
   `structural_coverage_gap_anthropic_tool_result_additive_blocks`
   (`:67`) retire when the slice lands. The
   `rest_request_wire_serde_alignment_receipt` at `:194-198`
   gains a `paid:tool_result_content_scalar_or_block_array`
   row (or equivalent named-paid row) reflecting closure.

## Acceptance

- `AnthropicToolResultContent` wire serialization is captured
  by a `CoproductWireContract` row (or equivalent
  substrate-fact-introduction-ratified shape) covering the
  scalar-vs-array dispatch.
- New `AnthropicToolResultBlock` carrier includes
  `AnthropicSharedBlock` (delegating to shared `ContentBlock`)
  + `AnthropicDocumentBlock` + `AnthropicSearchResultBlock` +
  `AnthropicToolReferenceBlock` with typed payloads. Shared
  `dsl/extdeps/llm/llm.dag` `ContentBlock` is **untouched**.
  `AnthropicToolResultContent.ToolResultBlocks.blocks` switches
  from `List<ContentBlock>` to `List<AnthropicToolResultBlock>`.
- Wire ratchets cover both shapes + the three new variants.
- Both closure tags at `:67` and `:204` retire.
- `rest_request_wire_serde_alignment_receipt` at `:194-198` gains
  the named-paid receipt row.
- The two `structural_coverage_gap_*` rows under
  `dsl/extdeps/llm/anthropic.dag` (the closure-tag list
  values at `:67` and `:204`) flip from open coverage gaps
  to deleted (full closure) or narrowed (with named-residual
  closure-tag rewrite). ROADMAP `:393` "LLM service
  flattening" residue prose updates to reflect the closure;
  the previously-tracked row at ROADMAP `:344` is NOT touched
  (already RESOLVED 2026-05-03; this PR is residue cleanup,
  not row resurrection).
- `cargo test --workspace --exclude v2-compiler-tests` green.
- `cargo test -p v2-compiler-tests` green; strict-compile diagnostic
  ratchet at 0.
- `cargo clippy --all-targets -- -D warnings` clean.

## STOP-AND-ESCALATE

- **No existing `VariantEncoding` covers scalar-vs-tagged-array
  dispatch.** Surface — this is the named P1 substrate-fact-
  introduction event and routes through procedure, not silent
  variant-set growth.
- **Anthropic API has shifted on tool_result content shape**
  since the closure tags were authored. Re-scope the slice; do
  not bridge with stale variant lists.
- **`ContentBlock` is consumed by sites outside `tool_result`**
  whose typed expectations are violated by the additive
  variants. The wait-window grep verified `ContentBlock` is
  shared (it appears in `AnthropicToolResultContent.ToolResultBlocks`
  and likely elsewhere — worker enumerates consumers at
  dispatch via `src/v3/DOWNSTREAM_REQUIREMENTS.md` per
  `feedback_enumerate_before_substrate`). If consumer breakage
  is unavoidable, surface to R3 Substrate Manager.
- **Sub-slice 1 + 2 cannot land atomically.** If wire-shape work
  blocks on P1 procedure but additive-blocks work is independent,
  Director may authorize splitting; surface for that decision —
  do not unilaterally split the brief.

## Authority audit receipt

1. **Substrate exists?** Partial. `AnthropicToolResultContent`
   exists with two variants (`anthropic.dag:62-65`); shared
   `ContentBlock` (`dsl/extdeps/llm/llm.dag:41`) exists and is
   consumed at `:65`. Wire dispatch shape NOT captured;
   `AnthropicToolResultBlock` Anthropic-specific carrier and
   its three Anthropic-only variants NOT modeled. Per extdeps
   layering (provider-specific block types live in provider
   modules, not the shared `llm.dag`), this slice authors a
   new Anthropic carrier rather than extending the shared
   primitive.
2. **Existing brief?** None for this row directly. Adjacent
   `rest_request_wire_serde_alignment_receipt` at `:194-198`
   tracks the broader REST-wire authority migration.
3. **Design-doc match?** No design-doc; Anthropic Messages API
   reference + the closure tags' variant enumerations are the
   authority.
4. **Citations live?** `anthropic.dag:20-30, 62-65, 67, 194-198,
   204` verified at HEAD via wait-window grep (2026-05-05).
   Worker re-verifies at dispatch with symbol-grep alongside
   line numbers.
5. **Carrier dissolves the bridge?** Yes — both closure tags
   name the typed-structural dissolution exactly; sub-slice 1
   authors the wire-contract row (or surfaces the P1 event),
   sub-slice 2 authors the new Anthropic-specific
   `AnthropicToolResultBlock` carrier (Anthropic-only variants
   in `dsl/extdeps/llm/anthropic.dag`, NOT in shared
   `llm.dag`), together retiring both gap rows. Extdeps
   layering preserved — shared `ContentBlock` stays cross-
   provider; Anthropic-specific block kinds live on the
   Anthropic-side carrier.

## Provenance

Drafted during R3 host-block wait window (2026-05-05) per
Director ratification at #issuecomment-4377776495. Director's
ratification note explicitly subsumed the `:67` additive-blocks
row into this slice rather than splitting them; honoring that
shape. Operator-bridge expected to ship this brief to PR #1782.
