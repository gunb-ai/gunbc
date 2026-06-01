# Python L2 Cross-Target Parity P5 Receipt - 2026-06-01

Scope: PR #4154 / dashboard node `adhoc-5569a75b-c9d`.

This records the P5 / v3 hand-Rust receipt for the narrow Python L2
cross-target behavioral parity bridge in `src/v3/compiler/src/emit_host_bridge.rs`.

## Receipt

The PR adds a temporary hand-Rust verdict/API surface:

- `EmitHostCrossTargetParityVerdict`
- `run_cross_target_mvp2_python_parity_transport`

The change does not claim full T-38 runtime verdict closure. It is an
interim host-transport bridge over the already-modeled `TestClaimRun<Node,
RuntimeValue>` rows in `src/v4/test/claim/nat_semiring/rung_5.dag` and
`src/v4/test/claim/workflow/nat_semiring_rung5_eval.dag`.

Checkable P5 receipt in this PR: the previous integration-test path that
hand-rolled Rust/Python/Go transports and raw stdout equality is deleted from
`v4_emit_host_harness_test.rs`. The test now calls the single
`run_cross_target_mvp2_python_parity_transport` bridge, which requires:

- host setup success,
- host exit witness `Holds`,
- per-target `runtime_value_parse_{rust,python,go}` success,
- then equality of the parsed runtime-value stdout carrier.

That consolidates the duplicate bridge logic instead of adding a second
authority.

## Deferral

The remaining hand-Rust bridge defers to the Runtime/TestClaim + Compiler Spine
runner lane:

- `src/v4/TASKS.md` T-22 (`compiler/05_eval.dag`) - primary execution path
  for modeled eval.
- `src/v4/TASKS.md` T-38 (`TestClaim execution harness`) - structured
  `TestClaimRun` verdict execution in CI.
- `docs/planning/v4-done-predicate-tracker-2026-05-30.md` Predicate 5 -
  blocking receipt is modeled T-22 eval plus structured `TestClaimRun`
  verdicts in CI, replacing shell/hand-host bridge authority.

When T-22/T-38 execute these rows directly, this hand-Rust parity bridge should
be deleted or reduced to an external boundary helper under the modeled runner.
