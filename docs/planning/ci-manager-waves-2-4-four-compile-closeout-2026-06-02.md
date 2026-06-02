# CI Manager Waves 2-4 + Four-Compile Collapse Closeout

> **Status:** CLOSEOUT RECEIPT.
> **Work item:** `node://adhoc-f70d7842-933`.
> **Scope:** CI Manager Waves 2-4 plus #4091 §1.2 four-compile collapse.
> **Gate:** same-path P5(b) receipt in
> `src/v3/compiler/tests/integration/v4_workflow_ci_runner_dag_smoke_test.rs`.

## Disposition

The closeout keeps the existing `src/v4/workflow/ci.dag` authority intact and records the landed
receipt surface that ties the CI-manager waves to the shared-closure collapse:

- **Wave 2:** `CiUpsertStep` rows cover the full in-scope `ci_pipeline_step_ids_shadow`
  universe via `ci_upsert_steps`, `ci_upsert_steps_full_in_scope_step_ids`, and
  `ci_upsert_steps_slice_bijection_ok`.
- **Wave 3:** `CiSelectionReceipt` carries the shadow TestClaim selection partition through
  `CiTestClaimSelection`, `TestgenSlotSelection`, `ci_wave3_shadow_fixture_fail_closed_receipt`,
  and `ci_wave3_shadow_fixture_receipt_ok`. The live GitHub Actions emit remains deferred until
  the modeled whole-program diff/eval entry lands.
- **Wave 4:** active floor-skip wiring stays modeled, not live-wired, through
  `Active { skip_evidence: CiActiveFloorSkipEvidence }` and `ci_floor_held`. No parallel
  `floor_skip` field or GitHub Actions-only skip authority is introduced.
- **Four-compile collapse:** M1 rust emit, phase1 rung gate, lens-CI, and v4 T-15 self-host
  consume `v2_compile_src_v4` through `UpstreamUpsert` edges; T-38 corpus eval consumes the M1
  rust emit receipt rather than owning a host-side compile.

## Non-Goals

- No new compiler stage, substrate type, or target semantics.
- No new GitHub Actions authority outside the existing workflow bridge.
- No promotion of Wave 3 shadow selection to live CI until the whole-program DAG diff input exists.

## Receipt

`v4_workflow_ci_manager_waves_2_4_and_four_compile_closeout_receipt` checks this document against
the live CI DAG and paired workflow smoke harness. The receipt is intentionally same-path: it adds
no new hand-Rust test surface beyond the existing T-PB-B bridge harness, and it dissolves with that
harness when generated workflow/TestClaim execution covers these facts directly.
