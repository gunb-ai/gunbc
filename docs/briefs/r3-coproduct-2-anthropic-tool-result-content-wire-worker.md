---
status: draft (wait-window; awaits R3 host restoration before dispatch; operator-bridge expected to ship to PR #1782)
authority parent: R3 Substrate Manager (#1739)
ratification: Director (#828) ratified slate (1) + (2) + (3a) at #issuecomment-4377776495 (2026-05-05)
roadmap row: ROADMAP.md "`anthropic_tool_result_full_content_surface`" (P1 / Grounding services)
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

ROADMAP P1 names the broader row "`anthropic_tool_result_full_content_surface`"
under Grounding services. This slice retires both gaps in one
PR (Director-ratified subsumption per
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

2. **Additive-blocks sub-slice.** Extend `ContentBlock` (worker
   greps for live declaration at dispatch — current
   `AnthropicToolResultContent.ToolResultBlocks.blocks`
   consumes it) with the three named missing variants:
   `Document | SearchResult | ToolReference`. Concrete payload
   shapes ground against the Anthropic Messages API reference;
   the closure tag at `:67` names the missing variants but not
   their payloads — worker authors payloads from the API spec.

3. **Wire ratchets.** Wire-emission tests on the v2 Rust
   emitter for `AnthropicToolResultContent` covering both
   scalar-text and block-array shapes; per-variant emission
   tests for the three new `ContentBlock` variants. Existing
   pattern lives in `src/v2/tests/src/pipeline.rs` (re-verify
   surface at dispatch).

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
- `ContentBlock` includes `Document | SearchResult | ToolReference`
  with typed payloads.
- Wire ratchets cover both shapes + the three new variants.
- Both closure tags at `:67` and `:204` retire.
- `rest_request_wire_serde_alignment_receipt` at `:194-198` gains
  the named-paid receipt row.
- ROADMAP row "`anthropic_tool_result_full_content_surface`"
  flips Open → Retired with PR sha and dissolution shape.
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
   exists with two variants (`anthropic.dag:62-65`); `ContentBlock`
   exists (consumed at `:65`). Wire dispatch shape NOT captured;
   three additive `ContentBlock` variants NOT modeled.
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
   sub-slice 2 authors the additive `ContentBlock` variants,
   together retiring both gap rows.

## Provenance

Drafted during R3 host-block wait window (2026-05-05) per
Director ratification at #issuecomment-4377776495. Director's
ratification note explicitly subsumed the `:67` additive-blocks
row into this slice rather than splitting them; honoring that
shape. Operator-bridge expected to ship this brief to PR #1782.
