# PB-Runtime — `ExecuteCommand` runner extension `(M-L)`

> **Worker brief.** Reports through Zero-Floor Program Manager
> (`stern-swift-335`). Authored 2026-04-25 against shipped main per
> Director ask on [#786](https://github.com/gunb-ai/gunbc/pull/786).
>
> Anchor verification: all read-first sites below verified at HEAD
> before brief authoring (per the discipline lesson from the v1
> PB-Substrate pilot brief #772 + PB-1-b withdrawal #786 — premise
> must match shipped state, not assumed pre-state).

> **Landed lane receipt (reconciles “Read first” preflight below):** `ExecuteCommand` is implemented in the Rust `TestRunner` and M1.5 harness with a **bounded execution policy** — `EXECUTE_COMMAND_WALL_TIMEOUT` (30s wall, fail-closed `ClaimResult::Fail` on exceed, child killed), no `output()`-style full capture of child streams; **null** child stdio on the direct path, and on the Linux `unshare(1)` path a **pipe to the wrapper** (bounded read for namespace-setup errors; the exec’d process inherits the stream—see `test_runner.rs`), and distinguishable `Fail` messages for spawn / timeout / signal / exit mismatch. On **Linux** (when permitted), the logical command runs under `unshare(1) -c -f -p` (user + PID namespace) so, if `unshare(1)` succeeds, the workload is tied to that namespace’s “init” exit; on `unshare(1)` `EPERM` the runner **falls back** to the same direct `Child` + wall + pgrp + `sh -c` `&` heuristics as non-Linux. **Non-Linux** uses the direct path only. The prior STOP-AND-ESCALATE “unbounded hang or no resource discipline” condition for *this* lane is **closed** in code; *changing* the default cap is still Director-level policy. Authoritative post-land narrative: [`TESTING.md`](../../TESTING.md) (`:195` callout) and `src/v3/compiler/src/test_runner.rs` (`evaluate_execute_command_exit_code`, `EXECUTE_COMMAND_WALL_TIMEOUT`).

## Read first

- [`docs/design-pure-bootstrap-zero.md`](../design-pure-bootstrap-zero.md) §"New lanes" `PB-Runtime` — your standing program scope (sized M-L; one of the four PB-Runtime files).
- [`TESTING.md`](../../TESTING.md) `:195` — **capability state callout** (updated with this lane). **Live source-of-truth** for what the runner does today. Historically, the 2026-04-25 cascade note also recorded pre-land state:
  - ExecuteCommand predicate: schema PR #678; runner was foundation-only (M1.5 `true` allowlist; `TestRunner` NYI) **until the PB-Runtime `ExecuteCommand` implementation landed.**
- **As implemented:** arbitrary `command` + `args` + `expect_exit_code` with **bounded** host execution; see the **Landed lane receipt** blockquote above and `test_runner.rs`.
- [`src/v3/std/verification.dag`](../../src/v3/std/verification.dag) `:115-119` — ExecuteCommand schema (the source-of-truth for the data shape):
  ```
  | ExecuteCommand {
      command: String
      args: List<String>
      expect_exit_code: Int
    }
  ```
  Note the dissolution-trigger comment immediately above (lines 109-114) — typed tool/capability references are the eventual durable shape; this brief preserves the current scaffold sum, not the eventual structural form.
- [`src/v3/compiler/src/test_runner.rs`](../../src/v3/compiler/src/test_runner.rs) — `TestRunner::run_claim` match: **`ExecuteCommand`** → `eval_execute_command`; shared **`evaluate_execute_command_exit_code`** (null stdio on the direct path; Linux `unshare(1)` wrapper may use a bounded pipe—see `evaluate_execute_command_exit_code` docs), wall timeout, `ClaimResult::Fail` on timeout. Pre-land: no arm → `NotYetImplemented` (retired).
- [`src/v3/compiler/tests/integration/m1_5_testgen_test.rs`](../../src/v3/compiler/tests/integration/m1_5_testgen_test.rs) — **retired** tautological `shell_exit_matches_allowlisted` + allowlist-only panic; harness uses `evaluate_execute_command_m1_5` (exit pass/mismatch as bool; other outcomes panic) on the same underlying `evaluate_execute_command_exit_code` implementation.

## Frame

The cascade promotion PR #782 retracted TESTING.md's "Post-R2 shape" Rust-residual carve-out under 0-floor. The retraction is structurally honest: `ExecuteCommand` is the cascade-named successor pattern for boundary tests. Pre–PB-Runtime, the data shape was ahead of a **foundation** runner; **this lane** lands arbitrary-command execution with **bounded** host policy. ROADMAP / dependencies for T-PB-B are updated when this lane ships — see `ROADMAP.md` and `TESTING.md`.

This worker lifts the runner from foundation-only to **arbitrary command + args** capability. After landing: a `TestClaim` like *"emit Rust, invoke rustc on output, check exit code"* is structurally expressible AND executable.

## Slice — extend `ExecuteCommand` from tautological to arbitrary

Two surfaces, one PR:

### Surface 1 — Rust `TestRunner` match arm

**Implemented:** an explicit `ExecuteCommand` match arm in `test_runner` (`run_claim`); previously `ExecuteCommand` fell through to `NotYetImplemented`. Behavior:

- Extract `command: String`, `args: List<String>`, `expect_exit_code: Int` from the `payload` (the `variant_value` extraction pattern is in the existing arms; mirror).
- Spawn the command via `std::process::Command` with **stdin/stdout/stderr to the null device** (no buffering of child output — P3/P4) and a **wall-clock bound** (fail-closed `ClaimResult::Fail` on exceed; child killed). Compare **exit code** only; do not hang CI on runaway or flood-output children.
- Compare actual exit code to `expect_exit_code`; return `ClaimResult::Pass` if match, `ClaimResult::Fail` with diagnostic-shaped message if mismatch.
- On spawn failure (binary not found, permission denied, etc.), or timeout, return `ClaimResult::Fail` with a structured message that distinguishes spawn-error / **timeout** / exit-mismatch — distinguishability matters for boundary-test triage.

### Surface 2 — M1.5 testgen harness allowlist generalization

Extend `shell_exit_matches_allowlisted` at `m1_5_testgen_test.rs:292-294` from the tautological-only shape to actually invoking the command. Two viable implementations:

- **(a)** Mirror Surface 1 — the harness becomes a thin wrapper over the same `std::process::Command` invocation. Pro: single canonical execution path; harness and runner agree by construction.
- **(b)** Keep the harness as a separate evaluator with its own implementation. Pro: hermetic-test discipline preserved if the harness path needs different sandbox semantics. Con: parallel-implementation risk per `feedback_no_textual_enforcement_bridges` discipline.

**Manager lean: (a)**. Cleanest reuse + zero parallel-representation debt. Surface the choice in PR description.

Update the fail-closed panic at `:394-398` accordingly. The panic was scaffold pending this lane — its dissolution trigger is precisely "M1.5 harness understands arbitrary ExecuteCommand shapes." Retire the panic on landing.

### Hermetic discipline (load-bearing)

Per `TESTING.md` overall framing + `m1_5_testgen_test.rs:289-294` comment: today's allowlist's purpose is *"Hermetic: we do not spawn a host process — the allowlist encodes the only exit semantics this interpreter models."*

Extending to arbitrary command spawning **breaks the literal hermetic property** as written. Two responses:

- **Accept the break, narrow the property.** "Hermetic" reframes from "no host process spawn EVER" to "host process spawn is an explicit, declared boundary expressed via the ExecuteCommand variant; everything outside ExecuteCommand stays hermetic." This is the cascade's framing — the boundary-test migration is exactly *"declarative ExecuteCommand to invoke an external toolchain."*
- **STOP-AND-ESCALATE (timeout resource discipline):** the **landed** runner already applies a fixed wall timeout + null stdio (see **Landed lane receipt**). Escalate only if product needs a **different** default cap, per-claim override, or sandbox *policy* than what ships in `test_runner.rs` — that is Director-level, not a gap in this PR.

Default expectation: accept the narrowing (the property is already implicit in the cascade's framing). PR description should explicitly cite the narrowed hermetic property + reasoning.

## Acceptance

- [x] Surface 1: `ExecuteCommand` match arm in `test_runner.rs`; spawns the command; compares exit code to `expect_exit_code`; distinguishable Pass/Fail/spawn-error/**timeout** results. **Bounded execution (closes prior STOP):** `EXECUTE_COMMAND_WALL_TIMEOUT` + null child stdio + `ClaimResult::Fail` on wall breach (not unbounded `output()` / hang).
- [x] Surface 2: `shell_exit_matches_allowlisted` (or its successor) generalizes from tautological-only to arbitrary; fail-closed panic at `:394-398` retired.
- [x] Both surfaces share execution mechanism (per manager lean (a)) OR PR description justifies parallel evaluators.
- [x] **Smoke test**: a TestClaim with `ExecuteCommand { command: "true", args: [], expect_exit_code: 0 }` still passes (preserves the existing behavior at the new allowlist boundary).
- [x] **Capability test**: a TestClaim with arbitrary command (suggest `ExecuteCommand { command: "echo", args: ["hi"], expect_exit_code: 0 }` and the negative case `expect_exit_code: 1`) demonstrates pass + fail paths.
- [x] **Boundary / migration smoke (structural)**: a landed `TestClaim` + `ExecuteCommand` path in `src/v3/compiler/tests/dag/t_pb_b_1_execute_command_boundary.dag` and `t_pb_b_1_dag_runner_test.rs` — end-to-end **spawn + exit-code** with POSIX `true` / `sh` / `echo` (portable in Linux CI; comments name the class-5 *pattern* paralleled: `m1_4_emit_python_test::python_stdout` host spawn + success via exit, without pulling CPython into every CI run). This is **structural** receipt (same `std::process` shape the brief requires), not a line-for-line retire of a legacy `rustc`/`python3`/`go` test body in default CI.
- [ ] **Empirical class-5 port (deferred)**: the brief’s *example* of a real `rustc` / `python3` / `go` round-trip expressed only as a `.dag` `TestClaim` with no Rust harness — **not** required to close the PB-Runtime *runner* lane. **Bulk** migration of those tests is a separate T-PB-B work item (see `TESTING.md` “Residual **bulk** migration …”); if this box is ever checked, name the ported test in the PR. *(Api-review #792, 2026-04-25: prior checkbox overclaimed; reconciled to structural vs empirical.)*
- [x] `TESTING.md:195` capability-state callout updated to reflect the new state (foundation-only → arbitrary command).
- [x] `cargo test --workspace --exclude v2-compiler-tests` passes.
- [x] `cargo clippy --all-targets -- -D warnings` clean.
- [x] `cargo fmt --all --check` clean.
- [x] DB-8 `self_host_fixed_point` converges bit-identically.

## STOP-AND-ESCALATE

Surface to Zero-Floor Manager.

- **If the runner needs a different default wall-clock / sandbox / resource-cap *policy* than the fixed `EXECUTE_COMMAND_WALL_TIMEOUT` + null-stdio execution now in `test_runner` — STOP (policy is Director-level). The P3/P4 fail-closed unbounded-`output()` / hang gap is **closed** in the landed implementation.
- **If `std::process::Command` semantics differ across platforms** (Windows vs Unix exit codes; signal handling) in ways that the boundary-test migration depends on — STOP. Cross-platform discipline is its own concern.
- **If the `expect_exit_code: Int` field's range is ambiguous** (signed vs unsigned vs platform-specific i32 vs the substrate's Int that maps to i64) — STOP. Substrate type-mapping question deserves explicit resolution.
- **If hermetic-property narrowing reveals a deeper test-discipline gap** (e.g., other tests rely on the literal "no spawn" property in ways the cascade framing didn't anticipate) — STOP.
- **If the migrated boundary test reveals a pattern that the brief didn't anticipate** (stdout/stderr assertions beyond exit-code, environment-variable dependencies, working-directory needs) — STOP. The current `ExecuteCommand` schema only asserts on exit code; richer assertions are out of scope for this lane and may need substrate extension.
- **If pilot scope balloons beyond the two surfaces + capability test + one migrated boundary test** — STOP.
- **If DB-8 fixed-point drifts** — STOP immediately.

## Non-goals

- **Not migrating ALL boundary tests.** One migration as evidence is sufficient; bulk migration is post-cascade T-PB-B work, not this lane.
- **Not extending `ExecuteCommand` schema** with stdout/stderr assertions, env vars, working dir, etc. Schema is at `verification.dag:115-119`; substrate-shape extensions belong in a separate brief.
- **Not implementing the dissolution-trigger target** (typed tool/capability references replacing the current scaffold sum). That's the eventual durable shape per `verification.dag:109-114`; this lane preserves the current scaffold and lifts its runner support.
- **Not extending the substrate's `Int` shape** to disambiguate exit-code semantics. If that surfaces, STOP-AND-ESCALATE.
- **Not changing M1.5 testgen test discipline** beyond the allowlist generalization. Other M1.5 test mechanics stay as-is.

## Reporting

- Single PR. Title pattern: `feat(v3): PB-Runtime — ExecuteCommand runner extension (arbitrary command + args; closes T-PB-B PB-Runtime dependency)`.
- PR description: cite this brief; cite the narrowed hermetic property + reasoning; cite the T-PB-B-1 **structural** migration receipt and that empirical `rustc`/`python3`/`go` **bulk** migration stays deferred (see `TESTING.md`); cite TESTING.md:195 capability-state update.
- On merge: Zero-Floor Manager confirms PB-Runtime ExecuteCommand-extension closure to Director; T-PB-B becomes unblocked on its PB-Runtime dependency; broader boundary-test migration can dispatch as separate work post-cascade.

## Cross-manager note

No cross-manager signal needed at brief authoring time. If the migrated boundary test surfaces substrate-shape questions about `Int`'s exit-code semantics or the `ExecuteCommand` schema's expressiveness, surface to manager → Director per established cross-program coordination.
