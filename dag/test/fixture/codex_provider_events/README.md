# Captured Codex provider-event receipt

`agent_message_turn_2026-07-29.jsonl` is a **real** `codex exec --json` event stream, captured
2026-07-29 against `@openai/codex` **0.145.0** (`codex-linux-arm64`, aarch64-unknown-linux-musl)
authenticated via ChatGPT, with the prompt `Reply with exactly: ok` under `--sandbox read-only`.

It exists because the `agent_message` item's text field could not be cited from anywhere else:
nothing in the repository recorded it, the shipped CLI is a stripped native binary, and
`learn.chatgpt.com/docs/non-interactive-mode` — the locator `extdeps.llm.cli` already cites —
documents the mode, not the per-item event schema. Modeling the field from memory would be the
guess DESIGN §3 forbids for an `extdeps` surface ("model what the API actually returns"), and
the #7368 design note admits a provider vocabulary only "after an actual provider receipt
demonstrates its grammar." So one was produced.

The load-bearing line is the third:

```json
{"type":"item.completed","item":{"id":"item_0","type":"agent_message","text":"ok"}}
```

`item.text` is therefore evidence, not inference, and `codex_provider_event_projection_filter`
reads that member on that authority.

Two incidental facts the capture settles, kept because they are expensive to re-derive:
`turn.completed` carries a `usage` object with token counts (unread by the projection today — a
candidate carrier if attempt cost is ever modeled), and `thread.started` carries `thread_id`, the
provider-session identity the resume/follow-up lane will need.

The `thread_id` is scrubbed to a zero UUID: it identifies a real provider session and is not a
fact this fixture asserts. Every other byte is as captured.

**Re-capture on:** a Codex major/minor version bump that changes the event envelope. The receipt
is version-stamped so a schema change is a visible re-capture, not silent drift in what the
projection believes.
