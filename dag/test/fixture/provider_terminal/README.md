# Provider terminal fixtures

Sanitized exact provider terminal streams used by the roadmap-press provider-recovery
program (`docs/plans/roadmap-press-provider-recovery.md`). Fixture bytes are evidence;
provider decoders are authority; common semantic mapping lives in `gunbc.provider_failure`.

## Files

| Path | Provider | Condition |
| --- | --- | --- |
| `codex/usage_limit_reset_hint.jsonl` | Codex CLI `--json` | Usage / credits limit; reset only in prose |
| `claude/seven_day_limit_overage_disabled.stream.jsonl` | Claude Code `stream-json` | Seven-day limit rejected; org overage disabled |

## Provenance (private / PR record)

Do **not** commit account emails, tokens, or personal session content into these
files. Thread / session identifiers are zeroed.

### Codex — `usage_limit_reset_hint.jsonl`

- Provider: Codex CLI (`codex-cli`), observed as `@openai/codex` 0.145.0 class
- Invocation: `codex exec --json -s workspace-write --dangerously-bypass-approvals-and-sandbox`
- Prompt: `Reply with exactly: ok`
- Host: session container (keen-wolf-740 worktree), 2026-08-03
- Process exit: 0 (provider emitted terminal failure events; process exit alone is not the cause)
- Account binding label: `codex-default` (sanitized)
- Stream digest: sha256 of fixture bytes as committed
- Confirmation: no token, secret, email, or personal session content remains; `thread_id` zeroed

### Claude — `seven_day_limit_overage_disabled.stream.jsonl`

- Provider: Claude Code CLI stream-json
- Invocation class: `-p --verbose --output-format stream-json --permission-mode bypassPermissions`
- Observed condition: weekly / seven-day limit (`rateLimitType: seven_day`, status `rejected`)
- Capture authority: live receipt on srv1, 2026-08-03 (operator session); envelope
  fields reconstructed into this sanitized fixture with zeroed session identity
- Account binding label: `claude-root` (sanitized; not a personal email)
- Confirmation: no token, secret, email, or personal session content remains

**Re-capture on:** provider major/minor changes that alter the terminal envelope.
