# Ctrl-Migration Verification Phase 4 Brief

**Status**: READY-FOR-DISPATCH once the Verification Mgr exists.

**Authority**: Ctrl-Migration project plan §6 and §8 from PR #2775.

## Output

Define the parity and cut-over discipline for staged ctrl `.dag` subsystem contracts.

First verification targets:

1. `dsl/ctrl/review_verdict.dag`
2. `dsl/ctrl/inbox.dag`
3. `dsl/ctrl/api_reviewer.dag`

## Parity Contract

For each subsystem, the manager must produce:

- a fixture inventory from current ctrl tests or representative DB/API rows,
- a generated-consumer receipt showing the `.dag` model is exercised structurally,
- a comparison harness that fails closed on missing or ambiguous evidence,
- a cut-over checklist that deletes or generated-only freezes the TS authority.

## Review Verdict Baseline

`dsl/ctrl/review_verdict.dag` is already staged in this Director PR. Its parity tests must cover:

- SHA marker selection with no recency fallback,
- whole-comment fallback as an explicit degraded state,
- approving verdict count by distinct provider,
- request-changes and blocking-finding exclusion from merge readiness,
- actionable feedback routing for `Blocking`, `P0`, and `P1`.

## Acceptance Gates

1. No staged model is marked authority until the generated consumer runs in CI or an equivalent dashboard parity gate.
2. Every cut-over PR deletes the old handwritten TS authority or marks it generated-only.
3. Residual manual steps are named as follow-up work items, not hidden in prose.

