# Captured Codex provider-event receipt

`agent_message_turn_2026-07-29.jsonl` is a **real** `codex exec --json` event stream, captured
2026-07-29 against `@openai/codex` **0.145.0** (`codex-linux-arm64`, aarch64-unknown-linux-musl)
authenticated via ChatGPT, with the prompt `Reply with exactly: ok` under `--sandbox read-only`.

It exists because the `agent_message` item's text field could not be cited from anywhere else.
Nothing in the repository recorded it, the shipped CLI is a stripped native binary rather than
readable source, and `learn.chatgpt.com/docs/non-interactive-mode` — the locator
`extdeps.llm.cli` already cites — documents the mode, not the per-item event schema. Modeling
the field from memory would have been exactly the guess DESIGN §3 forbids for an `extdeps`
surface ("model what the API actually returns"), and the #7368 design note makes the same rule
explicit for provider grammars: a vocabulary is admitted only "after an actual provider receipt
demonstrates its grammar." So one was produced.

The load-bearing line is the third:

```json
{"type":"item.completed","item":{"id":"item_0","type":"agent_message","text":"ok"}}
```

`item.text` is therefore evidence, not inference, and `codex_provider_event_projection_filter`
reads that member on that authority.

Two incidental facts the capture also settles, recorded because they are cheap to keep and
expensive to re-derive: `turn.completed` carries a `usage` object with token counts (unread by
the projection today — a candidate carrier if attempt cost ever wants modeling), and
`thread.started` carries `thread_id`, which is the provider-session identity the resume/follow-up
lane will need.

The `thread_id` is scrubbed to a zero UUID: it identifies a real provider session and is not a
fact this fixture is asserting. Every other byte is as captured.

**Re-capture on:** a Codex major/minor version bump that changes the event envelope. This receipt
is version-stamped precisely so a schema change is a visible re-capture rather than a silent
drift in what the projection believes.
