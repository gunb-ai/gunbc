---
status: draft (wait-window; awaits R3 host restoration before dispatch; operator-bridge expected to ship to PR #1782)
authority parent: R3 Substrate Manager (#1739)
ratification: Director (#828) ratified slate (1) + (2) + (3a) at #issuecomment-4377776495 (2026-05-05)
roadmap row: ROADMAP.md tracked-debt rows under "Tracked debts -- 2026-04 analyses" — `anthropic_messages_200_residual` (Grounding services / response-fidelity)
sibling briefs:
  - r3-coproduct-1-openai-chat-message-full-worker.md
  - r3-coproduct-2-anthropic-tool-result-content-wire-worker.md
---

# R3 Coproduct slice 3 — Anthropic Messages 200 residual

## Context

`dsl/extdeps/llm/anthropic.dag:188` carries
`structural_coverage_gap_anthropic_messages_200_residual` with
closure tag:

```
closure:anthropic_messages_200_residual|AnthropicMessages200Body|json_pending:container|AnthropicMessages200TextBlock|variant_pending:thinking|variant_pending:tool_use|variant_pending:redacted_thinking|variant_pending:web_search|variant_pending:server_tool_use|trigger:content block coproduct plus container-management response surface
```

Two coupled gaps:

1. **`AnthropicMessages200Body.container: Json?`** — the response
   body's `container` field is currently typed as opaque `Json?`,
   delaying the typed surface for container-management response
   data. Closure tag names this as `json_pending:container`.
2. **`AnthropicMessages200TextBlock` content-block coproduct** —
   the modeled response carries a flat list of
   `AnthropicMessages200TextBlock`; five additional content-block
   variants are wire-present but not modeled:
   `thinking | tool_use | redacted_thinking | web_search |
   server_tool_use`.

This row is response-side; pairs with sibling slice 2's
request-side tool_result coverage on the same Anthropic Messages
module surface (Director's "LLM-coherence" rationale at
ratification time).

## Slice

1. **Content-block coproduct sub-slice.** Replace
   `AnthropicMessages200TextBlock` (or the carrier of which it
   is a singleton variant) with a coproduct expressing the five
   missing variants:

   ```dag
   type AnthropicMessages200ContentBlock
     = MessagesText { text: String }
     | MessagesThinking { thinking: String, signature: String }   // sketch
     | MessagesToolUse { id: String, name: String, input: Json }  // sketch
     | MessagesRedactedThinking { data: String }                  // sketch
     | MessagesWebSearch { ... }                                  // sketch
     | MessagesServerToolUse { ... }                              // sketch
   ```

   Concrete variant payloads ground against the Anthropic
   Messages API reference at brief authoring time. The closure
   tag names variants but not payloads; worker reads the API
   spec for each.

2. **Container typed surface sub-slice.** Replace `container:
   Json?` with a typed coproduct expressing the modeled
   container-management response shape. Pre-flight obligation:
   read the Anthropic Messages API reference for what
   `container` carries when present. If the API spec leaves
   `container` open-ended (i.e., it really is opaque from the
   client's standpoint), STOP — `Json?` may be correct and the
   `json_pending:container` closure should narrow rather than
   retire.

3. **Wire-contract row.** Mirror the request-side pattern at
   `anthropic.dag:20-30`: `anthropic_messages_200_content_block_wire_contract:
   CoproductWireContract` with `InternallyTaggedObject` on `type`
   and case-stripped variant naming. The existing
   `rest_request_wire_serde_alignment_receipt` shape at `:194-198`
   gains a `paid:messages_200_content_block_internally_tagged_object`
   row (or equivalent).

4. **Wire ratchets.** Per-variant 200-body deserialization tests
   covering the five new content-block variants and the typed
   container shape. Existing pattern is the
   `openai_chat_message_role_wire_matches_llm_snake_contract`
   shape; new ratchets follow that lockdown form for the
   response side.

5. **Closure-tag retirement.** The closure tag at `:188`
   retires (full closure) or narrows (if `container` legitimately
   stays `Json?` per sub-slice 2 STOP). On full closure, the row
   deletes; on narrow, the closure tag rewrites to name only the
   remaining gap-carried variants/fields.

## Acceptance

- `AnthropicMessages200ContentBlock` (or equivalent renaming)
  expresses all five missing variants with typed payloads.
- `AnthropicMessages200Body.container` is typed (or the closure
  tag narrows with explicit "stays opaque per API spec" residual).
- Wire-contract row authored; receipt row added at
  `rest_request_wire_serde_alignment_receipt`.
- Wire ratchets cover all variants + container shape; existing
  text-block ratchets remain green.
- Closure tag at `:188` retires or narrows with named residual.
- ROADMAP `anthropic_messages_200_residual` row updates per the
  retirement / narrowing outcome.
- `cargo test --workspace --exclude v2-compiler-tests` green.
- `cargo test -p v2-compiler-tests` green; strict-compile diagnostic
  ratchet at 0.
- `cargo clippy --all-targets -- -D warnings` clean.

## STOP-AND-ESCALATE

- **Anthropic API has shifted on response content-block variants**
  since the closure tag was authored. Worker grounds against the
  live API reference; if the closure-tag's five named missing
  variants are not the full set on the wire today, surface — the
  row needs re-scoping, not silent expansion.
- **`container` field is opaque by API design.** If the Anthropic
  spec confirms `container` carries client-opaque data, narrow the
  closure tag rather than fabricating a typed surface.
  Substrate-fact-introduction discipline applies: a typed surface
  must reflect the API's actual shape, not aspirational shape.
- **Existing text-only consumers of `AnthropicMessages200TextBlock`
  break under the rename / coproduct widening.** Per slice 1's
  STOP rule on OpenAI text-message consumers, surface if breakage
  is unavoidable; the slice may need to introduce the full
  coproduct in parallel rather than in place.
- **`thinking` / `tool_use` content variants overlap with
  request-side carriers** (e.g., `AnthropicAssistantContentBlock`
  already has `AssistantTextBlock` / `AssistantToolUseBlock` per
  the wait-window scan of `:25-29`). If response-side and
  request-side variants are in fact the same coproduct shape,
  sharing the type may be honest; surface for the design call —
  do not silently merge or duplicate.

## Authority audit receipt

1. **Substrate exists?** Partial. Modeled response carriers
   exist (`AnthropicMessages200Body`,
   `AnthropicMessages200TextBlock`); five content-block variants
   not modeled; `container` typed as `Json?` placeholder.
2. **Existing brief?** None for this row directly. Adjacent rows:
   `rest_request_wire_serde_alignment_receipt` (paid at `:194-198`),
   sibling slice 2's `anthropic_tool_result_content_wire_shape`
   (`:204`).
3. **Design-doc match?** No design-doc; Anthropic Messages API
   reference + the closure tag's variant enumeration are the
   authority.
4. **Citations live?** `anthropic.dag:188, 194-198` verified at
   HEAD via wait-window grep (2026-05-05); worker re-verifies at
   dispatch with symbol-grep alongside line numbers.
5. **Carrier dissolves the bridge?** Yes for the variant-set
   sub-slice — closure tag's
   `closure:anthropic_messages_200_residual|...variant_pending:...`
   shape names the typed-coproduct dissolution exactly. Container
   sub-slice may narrow rather than retire if the API spec
   confirms client-opaque shape.

## Provenance

Drafted during R3 host-block wait window (2026-05-05) per
Director ratification at #issuecomment-4377776495. Selected over
candidate (3c) `github_pull_review_response_residual` for
LLM-thematic coherence with sibling slices 1 and 2. Honest
disclosure per Director's note: this slate is fresh-authoring
per path (a), not recovery of original "3 newly-routed coproduct
slices" intent. Operator-bridge expected to ship this brief to
PR #1782.
