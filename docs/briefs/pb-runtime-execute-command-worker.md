# PB-Runtime — `ExecuteCommand` runner extension `(M-L)`

> **Worker brief.** Reports through Zero-Floor Program Manager
> (`stern-swift-335`). Authored 2026-04-25 against shipped main per
> Director ask on [#786](https://github.com/gunb-ai/gunbc/pull/786).
>
> Anchor verification: all read-first sites below verified at HEAD
> before brief authoring (per the discipline lesson from the v1
> PB-Substrate pilot brief #772 + PB-1-b withdrawal #786 — premise
> must match shipped state, not assumed pre-state).

## Read first

- [`docs/design-pure-bootstrap-zero.md`](../design-pure-bootstrap-zero.md) §"New lanes" `PB-Runtime` — your standing program scope (sized M-L; one of the four PB-Runtime files).
- [`TESTING.md`](../../TESTING.md) `:195` — **capability state callout** authored 2026-04-25 in the cascade promotion PR #782. **Live source-of-truth for what's currently allowed vs not.** Read this in full before starting; it documents:
  - ExecuteCommand predicate landed in PR #678 (schema only).
  - M1.5 testgen harness accepts only the tautological allowlist (`command == "true" && args.is_empty() && expect_exit == 0`), panics fail-closed on anything else.
  - Rust `TestRunner` returns `NotYetImplemented` for ExecuteCommand (no match arm).
  - **Full runner support is deferred to this lane** — the cascade-promoted boundary-test migration is structurally expressible but **executing it is blocked** until this PR.
- [`src/v3/std/verification.dag`](../../src/v3/std/verification.dag) `:115-119` — ExecuteCommand schema (the source-of-truth for the data shape):
  ```
  | ExecuteCommand {
      command: String
      args: List<String>
      expect_exit_code: Int
    }
  ```
  Note the dissolution-trigger comment immediately above (lines 109-114) — typed tool/capability references are the eventual durable shape; this brief preserves the current scaffold sum, not the eventual structural form.
- [`src/v3/compiler/src/test_runner.rs`](../../src/v3/compiler/src/test_runner.rs) `:352-388` — match block over predicate variants. ExecuteCommand has no arm; falls through to `other => ClaimResult::NotYetImplemented(format!("TestPredicate::{other} is not wired in the Rust runner yet"))`.
- [`src/v3/compiler/tests/integration/m1_5_testgen_test.rs`](../../src/v3/compiler/tests/integration/m1_5_testgen_test.rs) `:292-294` — current allowlist:
  ```rust
  fn shell_exit_matches_allowlisted(command: &str, args: &[String], expect_exit: i64) -> bool {
      command == "true" && args.is_empty() && expect_exit == 0
  }
  ```
  And `:394-398` — the fail-closed panic when allowlist rejects.

## Frame

The cascade promotion PR #782 retracted TESTING.md's "Post-R2 shape" Rust-residual carve-out under 0-floor. The retraction is structurally honest: `ExecuteCommand` is the cascade-named successor pattern for boundary tests. **But: the retraction is a paper claim** — the migration target exists as a data shape, but the runner cannot execute the data shape beyond the tautological allowlist. ROADMAP T-PB-B's dependency column reads `DB-15 + T-TestGen + PB-Runtime` per #782; this lane is the missing PB-Runtime piece.

This worker lifts the runner from foundation-only to **arbitrary command + args** capability. After landing: a `TestClaim` like *"emit Rust, invoke rustc on output, check exit code"* is structurally expressible AND executable.

## Slice — extend `ExecuteCommand` from tautological to arbitrary

Two surfaces, one PR:

### Surface 1 — Rust `TestRunner` match arm

Add an explicit `ExecuteCommand` match arm at `src/v3/compiler/src/test_runner.rs:352-388`. Today the match block handles `Compiles` / `FailsWithDiagnostic` / `OutputEquals` / `PortHasState` / `CostBounded` / `LensOutputEquals` / `DifferentialEquals` / `AlgebraicLaw` / `MockBackedInvariant`. ExecuteCommand falls to the `other` NotYetImplemented arm. New behavior:

- Extract `command: String`, `args: List<String>`, `expect_exit_code: Int` from the `payload` (the `variant_value` extraction pattern is in the existing arms; mirror).
- Spawn the command via `std::process::Command::new(command).args(args).output()` (or equivalent that captures stdout/stderr + exit code).
- Compare actual exit code to `expect_exit_code`; return `ClaimResult::Pass` if match, `ClaimResult::Fail` with diagnostic-shaped message if mismatch.
- On spawn failure (binary not found, permission denied, etc.), return `ClaimResult::Fail` with a structured message that distinguishes spawn-error from exit-mismatch — distinguishability matters for boundary-test triage.

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
- **STOP-AND-ESCALATE**: if execution surfaces a sandbox/timeout/resource-cap need that the brief doesn't anticipate (e.g., tests must not be able to invoke arbitrary system commands without policy, or runner needs a timeout to avoid hanging CI), surface to manager — sandbox policy is a Director-level discipline call, not worker discretion.

Default expectation: accept the narrowing (the property is already implicit in the cascade's framing). PR description should explicitly cite the narrowed hermetic property + reasoning.

## Acceptance

- [ ] Surface 1: `ExecuteCommand` match arm in `test_runner.rs`; spawns the command; compares exit code to `expect_exit_code`; distinguishable Pass/Fail/spawn-error results.
- [ ] Surface 2: `shell_exit_matches_allowlisted` (or its successor) generalizes from tautological-only to arbitrary; fail-closed panic at `:394-398` retired.
- [ ] Both surfaces share execution mechanism (per manager lean (a)) OR PR description justifies parallel evaluators.
- [ ] **Smoke test**: a TestClaim with `ExecuteCommand { command: "true", args: [], expect_exit_code: 0 }` still passes (preserves the existing behavior at the new allowlist boundary).
- [ ] **Capability test**: a TestClaim with arbitrary command (suggest `ExecuteCommand { command: "echo", args: ["hi"], expect_exit_code: 0 }` and the negative case `expect_exit_code: 1`) demonstrates pass + fail paths.
- [ ] **Boundary-test migration smoke**: at least one existing Rust-side boundary test (e.g., a rustc/python/go invocation) ports to a `TestClaim` ExecuteCommand declaration **end-to-end** — the cascade's claim becomes empirically exercised, not just structurally expressible. PR description names which boundary test ported.
- [ ] `TESTING.md:195` capability-state callout updated to reflect the new state (foundation-only → arbitrary command).
- [ ] `cargo test --workspace --exclude v2-compiler-tests` passes.
- [ ] `cargo clippy --all-targets -- -D warnings` clean.
- [ ] `cargo fmt --all --check` clean.
- [ ] DB-8 `self_host_fixed_point` converges bit-identically.

## STOP-AND-ESCALATE

Surface to Zero-Floor Manager.

- **If the runner needs a timeout / sandbox / resource-cap discipline** to be safe for CI use — STOP. Sandbox policy is Director-level; not absorbable by this lane.
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
- PR description: cite this brief; cite the narrowed hermetic property + reasoning; cite which boundary test ported as the end-to-end smoke; cite TESTING.md:195 capability-state update.
- On merge: Zero-Floor Manager confirms PB-Runtime ExecuteCommand-extension closure to Director; T-PB-B becomes unblocked on its PB-Runtime dependency; broader boundary-test migration can dispatch as separate work post-cascade.

## Cross-manager note

No cross-manager signal needed at brief authoring time. If the migrated boundary test surfaces substrate-shape questions about `Int`'s exit-code semantics or the `ExecuteCommand` schema's expressiveness, surface to manager → Director per established cross-program coordination.
