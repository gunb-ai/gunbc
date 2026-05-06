---
status: draft (wait-window; awaits R3 host restoration before dispatch; operator-bridge expected to ship to PR #1782)
authority parent: R3 Substrate Manager (#1739)
ratification: Director (#828) ratified slate (1) + (2) + (3a) at #issuecomment-4377776495 (2026-05-05) over candidates surfaced at #828 #issuecomment-4377521226
roadmap row: ROADMAP.md "`openai_chat_message_full_coproduct`" (P1 / Grounding services)
sibling briefs:
  - r3-coproduct-2-anthropic-tool-result-content-wire-worker.md
  - r3-coproduct-3-anthropic-messages-200-residual-worker.md
---

# R3 Coproduct slice 1 — OpenAI chat message full coproduct

## Context

`dsl/extdeps/llm/openai.dag:97-100` declares the narrow modeled
shape `OpenAiChatMessage { role, content: String }` with
`OpenAiChatMessageRole = System | Developer | User | Assistant`
(`:94`). The structural-coverage-gap row at `:198` carries the
closure tag:

```
closure:openai_chat_message_full_coproduct|OpenAiChatMessage|missing:tool_role_with_tool_call_id|missing:function_role|missing:multimodal_content_array
```

Three named missing variants:

1. **Tool role with `tool_call_id`** — Chat Completions tool
   messages carry both a `role: "tool"` discriminator and a
   `tool_call_id` field; today's narrow row cannot express either.
2. **Function role** — pre-tool-API function-call messages still
   on the wire surface; not modeled.
3. **Multimodal content array** — `content` widens from string to
   an array of typed parts (text / image / etc.); today's row
   forces `content: String`.

ROADMAP P1 names this row "`openai_chat_message_full_coproduct`"
under Grounding services. Adjacent `CoproductWireContract`
pattern: not yet declared on the OpenAI side
(`dsl/extdeps/llm/openai.dag` does not import
`std.serialization::CoproductWireContract`); the equivalent rows
on the Anthropic side live at `dsl/extdeps/llm/anthropic.dag:20-30`.

## Slice

1. Extend `OpenAiChatMessageRole` with `Tool` and `Function`
   variants. The narrow text-message subset retains
   `System | Developer | User | Assistant`; the full coproduct
   adds the two missing wire roles.
2. Widen `OpenAiChatMessage` to a coproduct that expresses the
   three missing variants. **Critical layering rule:** the role
   discriminator is **variant-determined** — it must NOT live as
   a free `role: OpenAiChatMessageRole` field on each variant
   (which would make `OpenAiChatMessageTool { role: User, ... }`
   structurally representable, an illegal-state-by-construction
   violation per `feedback_state_space_vs_behavioral_invariants`).
   Either drop the `role` field from the variant payloads
   entirely (variant identity ⇒ role) and project it via an
   accessor function, OR carry per-variant singleton role tags
   keyed by the variant tag itself.

   ```dag
   type OpenAiChatMessageContent
     = OpenAiChatMessageText { text: String }
     | OpenAiChatMessageParts { parts: List<OpenAiChatMessagePart> }

   type OpenAiChatMessagePart
     = OpenAiChatMessageTextPart { text: String }
     | OpenAiChatMessageImageUrlPart { image_url: OpenAiChatMessageImageUrl }
     // additive parts (audio, file, ...) gap-carried separately if not
     // observed in current wire surface

   // Variant identity determines role; role NOT carried as field.
   // Wire serializer emits the role discriminator from variant tag
   // via `CoproductWireContract` / role accessor.
   type OpenAiChatMessage
     = OpenAiChatMessageSystem { content: OpenAiChatMessageContent }
     | OpenAiChatMessageDeveloper { content: OpenAiChatMessageContent }
     | OpenAiChatMessageUser { content: OpenAiChatMessageContent }
     | OpenAiChatMessageAssistant { content: OpenAiChatMessageContent, tool_calls: List<OpenAiChatMessageToolCall>? }
     | OpenAiChatMessageTool { content: OpenAiChatMessageContent, tool_call_id: String }
     | OpenAiChatMessageFunction { content: OpenAiChatMessageContent, name: String }

   // Role projection (variant tag ⇒ role) — accessor, not field.
   fn role_of(message: OpenAiChatMessage) -> OpenAiChatMessageRole {
     match message {
       OpenAiChatMessageSystem { .. } => OpenAiChatMessageRole.System,
       OpenAiChatMessageDeveloper { .. } => OpenAiChatMessageRole.Developer,
       OpenAiChatMessageUser { .. } => OpenAiChatMessageRole.User,
       OpenAiChatMessageAssistant { .. } => OpenAiChatMessageRole.Assistant,
       OpenAiChatMessageTool { .. } => OpenAiChatMessageRole.Tool,
       OpenAiChatMessageFunction { .. } => OpenAiChatMessageRole.Function,
     }
   }
   ```

   Concrete variant shapes are sketches; worker grounds against
   the OpenAI Chat Completions API reference at brief authoring
   time and preserves whatever discriminator structure rustc /
   serde can ratify at the wire layer. The variant-determined-role
   rule is **not** negotiable — it's the load-bearing invariant
   per the codex BLOCKING finding.
3. Author `openai_chat_message_wire_contract: CoproductWireContract`
   in the same file, mirroring the Anthropic pattern at
   `anthropic.dag:20-23` (internally-tagged-object on `role`,
   case-stripped variant name). Add `import std.serialization {
   CoproductWireContract, VariantEncoding }` if not already
   present.
4. Wire ratchets in `src/v2/tests/src/pipeline.rs` (or the
   current test surface — re-verify at dispatch) covering the
   three new variants. The existing pattern is named
   `openai_chat_message_role_wire_matches_llm_snake_contract` /
   `openai_chat_message_row_json_matches_chat_completions_wire_tags`
   (cited at `openai.dag:90-92`); new ratchets follow that shape.
5. Retire the closure tag at `openai.dag:198` once the structural
   surface lands. The `structural_coverage_gap_openai_chat_message_full_api_surface`
   row either deletes (full closure) or narrows to remaining
   gap-carried subshapes (e.g., audio parts not on the modeled
   wire surface).

## Acceptance

- `OpenAiChatMessage` expresses all three missing variants with
  typed fields (variant-determined role per the load-bearing
  invariant; no `role` field on payloads).
- `OpenAiChatMessageRole` includes `Tool` and `Function`.
- **Practice 4 classification receipts on the live declarations
  (mandatory).** Per
  `docs/modeling-discipline.md#4-coproduct-dissolution`
  (Practice 4) — *"Any new Rust enum with N ≥ 2 variants must
  have a checkpoint comment naming its classification (🟢/🟡/🔴),
  with a ledger entry if GREEN or a named trigger if YELLOW."*
  Every new sum-typed declaration carries an inline 🟢/🟡/🔴
  doc comment. For this slice: `OpenAiChatMessage`
  (the widened coproduct) and the new `OpenAiChatMessageContent`
  / `OpenAiChatMessagePart` sums all carry classification marks
  with named dissolution triggers where 🟡 (e.g., gap-carried
  additive parts like audio / file → tracking gate to update on
  next API spec extension). The widened `OpenAiChatMessageRole`
  also gains a Practice 4 mark if its variant set isn't already
  TERMINAL. Marks must be present on the `.dag` source in
  `openai.dag` itself, not only in the brief or PR body.
- `openai_chat_message_wire_contract` exists and emits expected
  serde attributes for the new variants.
- Wire ratchets land covering all variants; existing
  text-message-row ratchets remain green.
- Closure tag at `openai.dag:198` retires (or narrows with named
  residual sub-shapes).
- ROADMAP row "`openai_chat_message_full_coproduct`" flips Open →
  Retired with PR sha and dissolution shape.
- `cargo test --workspace --exclude v2-compiler-tests` green.
- `cargo test -p v2-compiler-tests` green; strict-compile diagnostic
  ratchet at 0.
- `cargo clippy --all-targets -- -D warnings` clean.

## STOP-AND-ESCALATE

- **OpenAI API discriminator structure has shifted** since the
  closure tag was authored (e.g., new variants merged or
  consolidated). Worker grounds against the live API reference;
  if the closure-tag's three named missing variants are not the
  full set on the wire today, surface to R3 Substrate Manager —
  the row needs re-scoping, not silent expansion.
- **`CoproductWireContract` cannot express OpenAI's variant
  discriminator** (e.g., the role tag and content shape jointly
  determine the variant in a way the contract types don't
  cover). STOP — this is a substrate-fact-introduction event for
  `std.serialization` and routes through P1 procedure
  (`INVARIANTS.md#p1-modeling-faithfulness`).
- **Existing text-message-only consumers break under the
  coproduct widening.** Per the row's narrow-text-row receipt at
  `openai.dag:90-92`, the narrow row is 🟢 TERMINAL for a
  modeled subset; widening must preserve text-only callers'
  read paths. If consumer breakage cannot be avoided, surface —
  the slice may need to introduce the full coproduct in
  parallel rather than in place.

## Authority audit receipt

1. **Substrate exists?** Partial. `OpenAiChatMessage` /
   `OpenAiChatMessageRole` exist as a narrow text-only modeled
   subset (`openai.dag:94, 97-100`); the closure-tag at `:198`
   names exactly which variants the narrow row excludes.
2. **Existing brief?** None. The structural-coverage-gap row at
   `:198` is the live tracking authority.
3. **Design-doc match?** No design-doc; OpenAI Chat Completions
   API reference + the closure tag's variant enumeration are
   the authority.
4. **Citations live?** `openai.dag:90-92, 94, 97-100, 198` verified
   at HEAD via wait-window grep (2026-05-05). Worker re-verifies
   at dispatch. Cite by symbol name (`OpenAiChatMessage`,
   `OpenAiChatMessageRole`, `structural_coverage_gap_openai_chat_message_full_api_surface`)
   alongside line number to make drift recoverable.
5. **Carrier dissolves the bridge?** Yes — the closure tag's
   `closure:openai_chat_message_full_coproduct|...` shape names
   the typed-coproduct dissolution exactly; this slice authors
   that coproduct and the wire-contract row, retiring the closure
   tag.

## Provenance

Drafted during R3 host-block wait window (2026-05-05) per Director
ratification at #issuecomment-4377776495 of the slate (1) + (2) +
(3a) surfaced at #828 #issuecomment-4377521226. Honest disclosure
per Director's note: this slate is fresh-authoring per path (a),
not recovery of original "3 newly-routed coproduct slices" intent
(unrecoverable from session memory). Operator-bridge expected to
ship this brief to PR #1782 alongside the existing seven briefs.
