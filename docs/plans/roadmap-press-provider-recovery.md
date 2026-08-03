# Roadmap press — attributable turns and provider recovery

**Status:** operator-ratified product requirement, 2026-08-03. Authority for the
`roadmap-press` lane rows in `gunbc.roadmap_authority`. Dissolves into those rows
and the carriers they name as P1–P7 land; this note is not a second registry.

## Product requirement

Every provider execution must end in a typed, subject-bound disposition. A
provider-side failure must either drive a logically valid recovery command or
produce a located escalation. Raw exit codes, opaque error strings, dead
sessions, and generic “Agent failed” states are never sufficient terminal answers.

“Handle all provider errors” means:

1. Every observed terminal event is preserved.
2. Every known shape is decoded into a typed cause.
3. Every unknown shape remains explicitly unclassified and counted.
4. Every typed cause has a separately modeled recovery policy.
5. The belt executes the recovery when policy permits.
6. The UI shows the cause, the decision, and the resulting attempt/turn.
7. No failure silently becomes success, ordinary pending, or an infinite retry.

## Five authorities

| Authority | Owns | Must not own |
| --- | --- | --- |
| `extdeps.llm.{codex,claude,cursor}_stream` | Exact provider wire grammar | Retry / switch / escalate decisions |
| `gunbc.provider_failure` | Provider-neutral business cause | Wire spelling guesses |
| `gunbc.provider_turn_receipt` | Subject-bound turn terminal receipt (`DispatchResolvedExecution` + attempt/turn keys) | Selection policy |
| Binding standing + recovery policy | Derived offer eligibility and one recovery plan | Hand-mutated readiness fields |
| Workflow command + UI projection | New worker turn on same attempt; causal press copy | Treating tmux presence as success |

Decoder selection follows the selected execution binding. Streams are never sniffed.

## Quota specimen pair (P2 handback)

| Specimen | Wire | Business |
| --- | --- | --- |
| Codex usage limit | top-level `error` then `turn.failed`; reset only in prose | `ProviderQuotaExhausted`, reset = `ResetHint` |
| Claude seven-day | `rate_limit_event` / `seven_day` / `rejected` / numeric `resetsAt` / overage org-disabled | `ProviderQuotaExhausted`, window `SevenDay`, reset = `ResetAtUnixSeconds` |

Vendor `rate_limit_event` + HTTP 429 does **not** imply transient `ProviderRateLimited`.
`overageDisabledReason: org_level_disabled` is independent of failure scope.

## Program order

```text
P1 provider-turn-terminal-receipt
  → P2 provider-failure-decoders-r1
  → P3 provider-binding-standing-and-recovery-policy
  → P4 roadmap-worker-turn-command-driver
  → P5 provider-recovery-actuation
  → P6 roadmap-press-recovery-view
  → P7 provider-failure-coverage-expansion
```

Immediate implementation slice: **P1 + P2 as one consumed vertical** — turn-grain
receipt, sanitized fixtures, provider-specific decoders, two quota failures reaching
a typed terminal disposition. Hard rejects (substring cross-provider classifiers,
`retryable: Bool`, poisoning all accounts from one binding, list-order fallback,
new attempt on binding-only change, tmux-as-authority) are enumerated in the
operator ratification that authored this note.
