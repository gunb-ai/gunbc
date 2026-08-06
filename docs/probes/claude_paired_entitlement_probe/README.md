# Claude paired entitlement probe (audit §4)

Live experiment mandated by `docs/plans/provider-control-interface-audit.md` section 4
and slice C. Runs **before** any Claude Agent SDK press-trip modeling.

## What it compares

Same explicit credential state root, same model, same prompt:

- **Direct arm:** `claude -p --output-format stream-json --verbose --input-format stream-json`
  with a stream-json `user` frame on stdin (required when `--input-format stream-json` is set;
  a positional prompt alone yields empty stdout).
- **SDK arm:** `@anthropic-ai/claude-agent-sdk` `query()` (0.3.220 by default), with
  `pathToClaudeCodeExecutable` pointed at `claude-wrapper-tee.mjs` so the subprocess
  stream-json is captured **beneath** the SDK parser (audit §4.1).

Dimensions recorded in the receipt JSON:

- account/session identity
- entitlement and rate-limit events
- reset time (when present on `rate_limit_event`)
- Fable / Opus warning mentions (text scan — not a typed decode)
- terminal result shape
- raw event type preservation (which `type` values each arm retained)

## Run

```bash
node docs/probes/claude_paired_entitlement_probe/run-paired-probe.mjs
```

The Node runners are **SCAFFOLD** (see file headers): dissolve when slice C press-trip
witness subsumes this orchestration.

Optional environment:

- `GUNBC_PROBE_SOURCE_HOME` — credential home to copy into an isolated probe state
  (default: `$HOME`). The probe **never** relies on ambient inheritance without recording
  the explicit binding.
- `GUNBC_PROBE_MODEL` — model id (default: `claude-haiku-4-5-20251001`)
- `GUNBC_PROBE_PROMPT` — bounded non-mutating prompt
- `GUNBC_PROBE_SDK_VERSION` — exact SDK pin (default: `0.3.220`)

## Outcome interpretation

The receipt's `comparison.entitlement_probe_verdict` and
`comparison.direct_cli_dissolves_for_realm` are **execution evidence only**.

Neither answer from audit §4 is assumed:

- If the SDK preserves subscription behaviour, direct CLI actuation dissolves for that realm.
- If it demonstrably cannot, direct stream-json becomes an explicit interface — not a fallback.

An unauthenticated host still produces a valid receipt
(`both_arms_auth_failed_before_entitlement_surface`); it does **not** prove or
disprove subscription entitlement windows. That verdict is **failure-path symmetry
only** — neither arm reached code that emits `rate_limit_event`.

The parser-drop finding (`control_response` in subprocess raw, absent from SDK
parsed) is independent of auth and is witnessed in
`dag/test/claim/claude_sdk_parser_drop_witness_test.dag` with receipt
`claude_sdk_parser_drop_receipt.json`.
