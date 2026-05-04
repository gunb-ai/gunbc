# R3 Verification - ExecuteCommand / ForAllTargets Collapse Audit

**Status:** AUDIT RECEIPT - docs-only. This audits the duplicate execution
predicate shape in `src/v3/std/verification.dag`; it does not edit substrate,
the Rust runner, PB-Runtime, or target fixtures.

**Scope:** `TestPredicate::ExecuteCommand`, `TestPredicate::ForAllTargets`, and
their current consumers in:

- `src/v3/std/verification.dag`
- `src/v3/compiler/src/test_runner.rs`
- `src/v3/compiler/src/lens_testgen.rs`
- `src/v3/compiler/tests/`

## Contract Restatement

`verification.dag` currently declares two scaffold variants with the same raw
record shape:

```dag
| ExecuteCommand {
    command: String
    args: List<String>
    expect_exit_code: Int
  }

| ForAllTargets {
    command: String
    args: List<String>
    expect_exit_code: Int
  }
```

The comments already name the intended collapse: one capability-typed execution
fact plus a scope dimension, not two parallel `String / List<String> / Int`
records. The audit question is whether that collapse can be implemented as a
Verification-side rewrite, or whether Substrate/PB/target-spec ownership must
land typed execution capability facts first.

## Current Implementation Audit

### Substrate Surface

`ExecuteCommand` is a singleton host-process scaffold. Its inline comments say
the raw command string is tolerated only until audited tools/capabilities are
bound from the target-spec layer instead of opaque `String` command lines.

`ForAllTargets` is a per-emission-target scaffold. Its inline comments say the
claim source stays on `TestClaim.source` / `TestClaim.file_name`, while the
variant repeats the same raw command triple only as a temporary shell check for
each known target.

The two variants therefore differ in scope, not in execution capability shape.

### Runner Surface

`ExecuteCommand` is live in the Rust runner:

- `parse_execute_command_fields` parses positional or record payloads into
  `(command, args, expect_exit_code)`.
- `run_claim` dispatches the `ExecuteCommand` variant to `eval_execute_command`.
- `eval_execute_command` first requires a clean compile of `TestClaim.source`,
  then runs `evaluate_execute_command_exit_code`.
- The host runner has typed outcomes in `ExecuteCommandHostOutcome` and maps
  policy rejection, spawn failure, setup failure, timeout, signal, and exit-code
  mismatch into `ClaimResult`.
- M1.5 testgen shares the same parser and evaluates `ExecuteCommand` through
  `evaluate_execute_command_m1_5`; only matched exit code vs mismatch is treated
  as propositional true/false.

`ForAllTargets` is not live in the Rust runner:

- `run_claim` has no `ForAllTargets` arm, so it falls to
  `NotYetImplemented("TestPredicate::<name> is not wired in the Rust runner yet")`.
- `m1_5_testgen_test.rs` explicitly treats `ForAllTargets` as runner-deferred.
- `r3_verification_l4_l7_l5_skeleton_test.rs` asserts the L5 fixture reaches
  `NotYetImplemented`.
- `docs/briefs/r3-v-l5-corpus-readiness-audit.md` independently records that
  the current raw command triple is insufficient for strict L5 value
  observation.

## Carrier Collapse Predicate Sketch

The eventual substrate shape should factor the duplicate record into two axes:
execution capability and execution scope.

```dag
type ExecutionScope
  = Once
  | PerEmissionTarget

type ExecutionCapability {
  tool: ToolRef
  args: List<ExecutionArg>
  expect_exit_code: Int
}

type TestPredicate
  = ...
  | Execution {
      capability: ExecutionCapability
      scope: ExecutionScope
    }
```

`ToolRef` / `Capability` is load-bearing. If the collapse keeps
`command: String`, the duplicate record shape shrinks but the opacity bridge
does not: command identity, sandbox policy, target applicability, and allowed
argv shape remain outside substrate.

Required structural facts:

| Fact | Exists today? | Current authority |
|---|---:|---|
| Singleton execution scope | Yes, implicit | `ExecuteCommand` variant label |
| Per-target execution scope | Yes, implicit | `ForAllTargets` variant label |
| Raw command / args / expected exit | Yes | duplicated variant fields |
| Typed host outcome classification | Yes, Rust-only | `ExecuteCommandHostOutcome` in `test_runner.rs` |
| Typed execution capability / tool reference | No | named as dissolution trigger in `verification.dag` |
| Target applicability / target set | Not in this carrier | L5 / LanguageSpec / target-spec work |
| Per-target runner semantics | No | `ForAllTargets` currently NYI |

## Consumer Surface Enumeration

| Consumer | Current dependency | Collapse impact |
|---|---|---|
| `src/v3/std/verification.dag` | Declares both scaffold variants with duplicate fields. | Must replace both variants with one scoped execution carrier, or add successor carrier while old variants remain compatibility scaffolds. |
| `test_runner.rs::parse_execute_command_fields` | Parses the raw triple for `ExecuteCommand`. | Becomes a parser for `ExecutionCapability`, ideally consuming typed tool/capability refs instead of strings. |
| `test_runner.rs::run_claim` | Dispatches `ExecuteCommand`; `ForAllTargets` falls through to NYI. | Needs one `Execution` arm that branches on `ExecutionScope`; `PerEmissionTarget` still requires target enumeration and per-target artifact execution. |
| `test_runner.rs::eval_execute_command` | Singleton clean-compile gate plus host process execution. | Can serve `ExecutionScope::Once`; cannot by itself implement per-target compilation/dispatch. |
| `test_runner.rs::ExecuteCommandHostOutcome` and helpers | Typed host-process outcome carrier. | Reusable for the singleton and each per-target child execution; not a substitute for typed capability identity. |
| `m1_5_testgen_test.rs` | Evaluates `ExecuteCommand`; treats `ForAllTargets` as deferred. | Must update generated predicate construction and interpreter boundary once successor variant exists. |
| `.dag` fixtures using `ExecuteCommand` | PB-Runtime / R1/R1C-E host receipts. | Need mechanical migration to `Execution { scope: Once, ... }` after compatibility decision. |
| `.dag` fixtures using `ForAllTargets` | L5 scaffold fixture only; expected NYI today. | Need migration to `Execution { scope: PerEmissionTarget, ... }`, but strict execution still waits on L5 target semantics. |
| Docs / readiness briefs | Several docs call `ForAllTargets` scaffold and raw command triple insufficient. | Should be updated with whichever successor carrier Substrate/PB ratifies. |

## Conversion Cost Classification

| Classification | Verdict | Rationale |
|---|---|---|
| **(a) Verification-side rewrite-only** | **No for the intended collapse** | Verification could mechanically merge the two variants into `Execution { command: String, args, expect_exit_code, scope }`, but that preserves the raw command identity bridge the current comments explicitly name as a dissolution target. It also would not make `PerEmissionTarget` runnable. |
| **(b) Substrate carrier extension** | **Yes** | A typed `ToolRef` / `Capability` / `ExecutionCapability` carrier is missing. The substrate must decide how commands are named, which args are structural, and how sandbox/target applicability is represented. |
| **(c) Cross-program PB / target-spec coordination** | **Yes** | PB-Runtime owns the host execution runner shape; L5 / target-spec work owns per-emission-target compilation and target set semantics. A single scoped carrier crosses both surfaces. |

## Verdict

**Routing needed before conversion.**

The duplicate record shape is real, but the safe fix is not a Verification-only
enum rewrite. A rewrite that keeps `String` command identity would satisfy the
surface "one variant" shape while preserving the underlying opacity bridge.
Likewise, collapsing the variants without per-target semantics would hide the
fact that `ForAllTargets` is still intentionally `NotYetImplemented`.

Recommended routing:

1. **Substrate / PB-Runtime:** define `ExecutionCapability` with typed
   `ToolRef` / capability identity, argv shape, expected outcome, and host
   policy authority.
2. **Substrate / Verification:** define `ExecutionScope = Once |
   PerEmissionTarget` and the successor `TestPredicate::Execution` shape, with
   an explicit compatibility plan for existing `ExecuteCommand` fixtures.
3. **L5 / target-spec:** define how `PerEmissionTarget` enumerates target
   artifacts and observes per-target results; do not encode that as repeated raw
   shell commands.
4. **Verification follow-on:** after those facts land, migrate runner dispatch
   and fixtures from `ExecuteCommand` / `ForAllTargets` to `Execution`.

## Debt Receipt

This audit does not close a Debt-Paydown row directly.

Debt found + routed: the F12 duplicate-record-shape pattern requires
Substrate/PB/target-spec routing for typed execution capability and scoped
target execution semantics before the two variants can collapse without
preserving the raw command bridge.

## Test Plan

- `git diff --check`
