# Executor journal and fencing source checkpoint — 2026-09-30

This is local source evidence, not deployment, commissioning or a VM receipt. The production helper, guarded fleet commissioning scope, installation and complete crash recovery remain unfinished. HOLD remains.

## Changes

A protected CAS journal separates submission, invocation binding, accepted readback, observed release and fleet-generation completion. Adjacent transitions preserve the plan, slot, executor, mutation generation and controller invocation. A persisted submission is never another start permit. The existing controller must win the journal CAS before commissioning effects and rechecks ownership during reconciliation.

The lightweight transport accepts settlement recovery without waiting on a predecessor unit that may legitimately have been reused. This is a transport capability; only the unfinished production helper may authorize settlement from actual durable evidence.

The shared fleet publication experiment binds the applied plan hash to the existing generation record. Numeric predecessor records remain readable but cannot establish who applied them. The proposed wire-format cutover makes older numeric-only apply readers refuse, and is identified in the human plan. This is NOT full publication recovery: the latest record is not historical proof after another apply, and interruption of the existing direct write still needs resolution before deployment.

## Local checks

- Eight journal controls passed under 6 GiB/no swap, including canonical decoding, immutable invocation binding, refusal of skipped phases and no repeated start from recorded submission.
- Full controller composition typechecked (783 files including its probe), under 6 GiB/no swap. Reported maximum RSS: 5,235,520 KiB. No controller entry was actuated.
- Twenty-two exported native executor controls passed, including a temporary user-service lifetime check. This used process-boundary doubles, not the privileged production helper.
- Three fleet-publication model controls passed. Twelve cases also exercised the exported native parser: legacy, each supported hash spelling, malformed suffixes, duplicate spaces, extra lines and empty input.
- The fleet plan production closure (704 modules plus a probe) typechecked under 6 GiB/no swap. Its snapshot predates only the added human-plan wire-format notice. This does not qualify the larger CLI/witness closure.
- The full fleet witness closure (1,907 modules) was OOM-killed under 6 GiB/no swap before tests ran. Scope `fleet-generation-integration-20260930.scope` reported `Result=oom-kill`, exit 137. This is a failed qualification, not a passing integrated floor.

Compiler used: existing allocation-vertical-resume release binary, SHA-256 `717e9b4159dc7bc722e94a235325e6760f5acd246e63d8bed14b828dac970243`. These are dependency-snapshot checks, not a rebuilt exact-head binary or CI verdict.

## Outstanding before live apply

Wire the root-owned bounded helper and fleet subject; require actual successful receipt/readback before advancing journal phases; make release/generation settlement recoverable; gate supply on full completion; handle delayed starts and lost publication responses without inferring success. Then qualify the integrated closure, review the exact commissioning plan and run commissioning, SSH, cleanup, same-slot reuse and expiry acceptance.

No srv1 mutation, runner-service change, readiness publication, reservation or guest launch occurred in this checkpoint.
