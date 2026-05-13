# Slow-Test Residual Sweep 2026-05-13

**Scope:** D1 residual paydown after the `TestNodeCostDimension` + JSONL
manifest bridge landed. This is not a new #102 closure claim: the ratchet still
reads `scripts/test-node-wall-clock-ratchet.jsonl` directly until policy is
projected from modeled timing facts.

**Name audit:** `RUSTC_BOOTSTRAP=1 cargo test -p v3-compiler -- --list`
confirmed all 83 manifest names existed in the current v3 test list before this
cut. No stale-name rows were found.

**Timing evidence:** `ctrl-build -- env RUSTC_BOOTSTRAP=1 cargo test -p
v3-compiler -- -Z unstable-options --report-time` captured 391 lib-test timing
rows before stopping on a pre-existing remote-environment helper lookup failure
for `gunbc_execute_command_bootstrap`. The timeout ratchet parser accepted that
partial log and reported no unexpected over-budget tests.

**Retired warn row:** `bootstrap::tests::kernel_bool_path_a_attaches_diagnostic_when_boolean_algebra_unresolvable`
measured **16 ms**, below the 2000 ms Phase-0 budget. Its warn-policy row was
removed from `scripts/test-node-wall-clock-ratchet.jsonl`.

**Rows intentionally left in place:** integration-test manifest rows were not
re-timed by this partial run. Seven lib-test manifest rows still exceeded 2000
ms in the captured log and remain warn-policy backlog.
