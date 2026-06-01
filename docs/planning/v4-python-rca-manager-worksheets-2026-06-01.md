# v4 Python RCA Manager worksheets - L1/L2 release-minimum runway

> **Status:** **WORKSHEET APPROVED** — Modeling DFS Arbiter §8 sign-off 2026-06-01 (`proud-fox-405`). Worksheet A: **READY-FOR-WORKER-DISPATCH** (static). Worksheets B/C: **ARBITER-APPROVED — BLOCKED-ON-RUNTIME/TESTCLAIM-ACCEPTANCE** (manager checklist item below still open).
> **Date:** 2026-06-01
> **Dispatch anchor:** `docs/planning/v4-predicate-dependency-graph-2026-06-01-eod.md` Section 11.8, Python row.
> **Current rung:** L0 complete via #4117 (R1 + R2a + R2b + R3-external).
> **Target rung:** L1 fixture-scale + L2 cross-target behavioral parity receipts versus Rust and Go.
> **Manager role:** Python RCA Manager. Python is release-minimum parity with a weaker static verifier; runtime and cross-target behavior carry more weight than binary-like self-host framing.

## Mechanical rules

- Each Python class below has a separate Section 10.0 worksheet.
- Shared facts route through the Modeling DFS Arbiter before implementation.
- Runtime/TestClaim owns execution receipts. Python RCA identifies the missing fact and acceptable falsification shape; it does not create a parallel runner authority.
- Static Python tools (`pyright`, `mypy`) are useful evidence only when their invocation, configuration, expected diagnostics, and advisory-vs-blocking role are modeled.
- Do not copy a Rust worksheet unless the shared authority is named. Rust `rustc` rejection, Go `go build`, Python `py_compile`, Python static analyzers, and Python runtime execution are distinct authorities.

## Existing receipts consumed

| Receipt | Surface | Python RCA interpretation |
| --- | --- | --- |
| #4117 | `src/v4/lens/leaf_model_verification.dag` + `src/v4/test/claim/language_model/python_r*.dag` + `scripts/v4-leaf-model-python-r*-verify.sh` | L0 complete. Python falsifications route through `TargetPythonExecRejected` where CPython accepts compile but rejects execution. |
| #4081 Wc L5 | Phase 1 `nat_semiring` Rust/Python/Go cross-target first-fire, recorded in planning snapshot | Baseline L2 pattern: compare release-minimum target behavior on the same small fixture rather than treating Python static checks as equivalent to Rust compile. |
| `scripts/v4-phase1-nat-semiring-rung-gate.sh` | Fixture-scoped rungs 0-2 over Rust/Python/Go | Existing L1-ish fixture-scale host gate: Python currently uses `python3 -m py_compile` for R0/R2. pyright/mypy must extend this as modeled static evidence, not replace runtime parity. |

## Worksheet A - Python static structural checks (`pyright` / `mypy`)

```text
Class: PY-L1-STATIC-STRUCTURAL
Representative symptom:
  Python emitted code can pass `python3 -m py_compile` while still carrying
  annotation/name/member drift that pyright or mypy can detect earlier than
  runtime. Conversely, CPython runtime failures from #4117 prove that
  py_compile alone is not a semantic verifier.
Immediate local patch:
  Add ad hoc pyright/mypy shell invocations to the phase1 fixture gate or CI
  with grep-based diagnostic expectations.
Why that patch is forbidden:
  It creates a second verifier authority outside TestClaim/TargetInvocation,
  bakes tool configuration into CI text, and leaves "advisory vs blocking"
  semantics implicit. It would also make Python look rustc-like when the
  operator framing says Python has a weaker static verifier.
DFS path:
  std authority:
    - `v4.std.leaf_model_verification.TargetInvocation` is currently coarse
      Symbol scaffolding for target exercise.
    - `v4.std.leaf_model_verification.TargetPythonExerciseVerdict` distinguishes
      compile and execution for L0 leaf-model fixtures only.
    - `v4.std.host_run` and `v4.std.test_claim_falsification` already model
      host process receipts for runtime-facing checks.
  extdeps/language authority:
    - `v4.extdeps.languages.python` owns Python primitive/model facts.
    - `v4.extdeps.formatters.black` models a Python-adjacent tool, but there is
      no Python static analyzer model for pyright or mypy.
  compiler/test consumer:
    - `scripts/v4-phase1-nat-semiring-rung-gate.sh` uses py_compile only.
    - L0 Python leaf-model scripts exercise py_compile + python exec.
Deepest unsound boundary:
  Python static analyzer results have no modeled invocation profile, version
  policy, config authority, diagnostic-code carrier, or verdict role. Treating
  them as "compile" would collapse CPython compile, third-party static analysis,
  and runtime execution into one false authority.
Systemic fix:
  Model a shared `TargetStaticAnalysisInvocation` / `TargetStaticAnalysisVerdict`
  shape (name subject to Arbiter) once, with per-tool profiles for pyright and
  mypy. The row must carry tool identity, version/config policy, input artifact,
  diagnostic-code namespace, and whether the check is advisory or blocking for
  the rung. Python rows consume that shape; TypeScript `tsc --noEmit` or other
  static analyzers can reuse it only if the Arbiter confirms the authority is
  genuinely shared.
Non-goals:
  - Replacing runtime execution parity with pyright/mypy.
  - Treating pyright/mypy diagnostics as CPython compile diagnostics.
  - Adding per-CI-step grep expectations without modeled diagnostic carriers.
  - Inventing Python-only static-verdict vocabulary if a shared target-static
    analysis carrier is approved.
Falsification probes:
  F1. A fixture with a known missing symbol/member is rejected by pyright/mypy
      under the modeled profile while `py_compile` still accepts; verdict records
      static rejection, not CPython compile rejection.
  F2. A dynamically valid fixture that the selected analyzer cannot prove is
      recorded as analyzer advisory/non-blocking or as a modeled expected
      diagnostic, never as target runtime failure.
  F3. Changing the modeled analyzer profile/config changes the invocation
      receipt without touching Python language primitive facts.
Metric allowed only as secondary:
  Count of emitted Python files passing pyright/mypy over the fixture roster.
```

## Worksheet B - Python runtime fixture execution

```text
Class: PY-L1-L2-RUNTIME-FIXTURE-EXECUTION
Representative symptom:
  #4117 showed Python R1/R2a/R3-external falsifications that compile under
  CPython but fail during execution (`NameError`, `AttributeError`, `TypeError`).
  L1/L2 therefore need a runtime receipt for Python fixtures, not only static or
  parse receipts.
Immediate local patch:
  Add one-off `python3 emitted.py` calls to each fixture script and compare stdout
  or stderr strings in shell.
Why that patch is forbidden:
  Shell stdout/stderr matching would become a parallel TestClaim runner. It
  would not carry typed host exit, logical stdout, runtime-value parse, or
  expected-failure authority, and it would not compose with Rust/Go receipts.
DFS path:
  std authority:
    - `v4.std.host_run` models host exit/logical run.
    - `v4.std.test_claim_falsification` models `TestClaimRun` host evidence.
    - `v4.std.runtime` owns runtime values.
    - `v4.std.leaf_model_verification.TargetPythonExerciseVerdict` is a
      Python-specific L0 bridge, not the final L1/L2 runtime carrier.
  extdeps/language authority:
    - `v4.extdeps.languages.python` owns the target model facts under exercise.
  compiler/test consumer:
    - L0 leaf-model scripts already prove py_compile+exec for hand-authored
      snippets.
    - `scripts/v4-phase1-nat-semiring-rung-gate.sh` stops at py_compile for
      Python R0/R2 and has no Python runtime cell.
Deepest unsound boundary:
  Python runtime execution is the meaningful verifier for many Python target
  facts, but current fixture-scale gates stop before runtime. L0 has a
  Python-specific bridge; L1/L2 need the shared TestClaim runtime surface.
Systemic fix:
  Extend the fixture roster so Python target artifacts can be executed through
  the same modeled host-run / runtime-value receipt path used for cross-target
  parity. The first accepted implementation should add Python rows to the
  runtime roster for a small fixture set and record exit, stdout, parse, and
  expected value as typed evidence. L0 `TargetPythonExerciseVerdict` remains a
  leaf-model bridge until dissolved by the shared runner.
Non-goals:
  - Python binary self-host claims.
  - Grep-only stderr comparison as the final receipt.
  - Runtime execution of the whole compiler before fixture-scale rows are green.
Falsification probes:
  F1. A Python artifact that passes py_compile but raises at execution is
      reported as runtime rejected with typed host evidence.
  F2. A Python artifact that exits 0 but prints an unparsable runtime value is
      rejected at runtime-value parse, not accepted on exit alone.
  F3. A Python artifact that prints a value different from the model/Rust/Go
      expected value is a behavioral mismatch, not a toolchain setup failure.
Metric allowed only as secondary:
  Number of Python fixture artifacts with typed runtime receipts.
```

## Worksheet C - Cross-target behavioral parity on small fixtures

```text
Class: PY-L2-CROSS-TARGET-BEHAVIORAL-PARITY
Representative symptom:
  Rust, Python, and Go can each compile/parse a fixture while disagreeing on the
  observable runtime value. Python's release-minimum value comes from agreeing
  behaviorally with Rust/Go and the model, not from py_compile alone.
Immediate local patch:
  Compare stdout strings from emitted Rust/Python/Go in a shell script for
  `nat_semiring` or another small fixture.
Why that patch is forbidden:
  It duplicates #4081's L5 pattern outside the TestClaim surface and loses the
  typed distinction between host setup failure, target execution failure,
  runtime-value parse failure, and actual behavior mismatch.
DFS path:
  std authority:
    - `v4.std.test_claim_falsification.TestClaimRun<Node, RuntimeValue>` and
      `v4.std.verdict` provide the shared verdict vocabulary.
    - `v4.std.runtime` provides the runtime-value subject.
  extdeps/language authority:
    - `v4.extdeps.languages.rust`, `python`, and `go` own target facts.
  compiler/test consumer:
    - #4081 Wc L5 is the concrete first-fire pattern for release-minimum
      Rust/Python/Go parity on `nat_semiring`.
    - `scripts/v4-phase1-nat-semiring-rung-gate.sh` currently renders rungs 0-2,
      but not an L2 runtime parity row.
Deepest unsound boundary:
  Cross-target parity lacks a Python-manager-owned worksheet that says which
  facts and receipts make Python "done" for L2. Without it, workers can confuse
  target-specific compile checks with behavioral equivalence.
Systemic fix:
  Reuse the #4081 Wc L5 pattern for a small fixture roster. Rows must bind the
  same fixture subject, run each release-minimum target through modeled host
  execution, parse each output into the same `RuntimeValue`, and compare all
  targets against the model/evaluator expectation. The release-minimum target
  set is Rust + Python + Go per Section 11.8; additional targets are non-blocking.
Non-goals:
  - Adding TypeScript/C++ to the release-minimum parity set.
  - Counting emitted-file compile success as L2.
  - Per-target expected values that are not derived from the same fixture/model
    subject.
Falsification probes:
  F1. Mutate only the Python emitted behavior for the fixture; the parity row
      fails as Python-vs-Rust/Go mismatch.
  F2. Break Python host setup; the row records setup/toolchain failure and does
      not claim behavioral mismatch.
  F3. Produce unparsable Python stdout; the row fails at parse receipt before
      equality comparison.
Metric allowed only as secondary:
  Fixture count with all three release-minimum targets passing behavioral parity.
```

## Worksheet D - Python self-compile framing

```text
Class: PY-SELF-COMPILE-FRAMING
Representative confusion:
  Rust/Go can be framed as binary-ish compiler paths. Python should not be
  judged by the same binary self-host shape. The operator framing is compiler
  execution parity before binary-like self-host.
Immediate local patch:
  Declare Python L3/L4 done when emitted Python files pass py_compile or when a
  Python package entrypoint exists.
Why that patch is forbidden:
  It claims self-compile without proving that the emitted Python compiler slice
  executes the same compile function as Rust/Go or produces the same outputs.
Systemic frame:
  - L1: emitted Python compiler subset is syntactically/static-clean under the
    approved Python static/runtime verifier profile.
  - L2: emitted Python compiler subset runs on small `.dag` fixtures and matches
    Rust/Go/model behavior.
  - L3: Python-emitted compiler slice reaches a fixed-point equivalent artifact
    under a declared normalization/comparison policy. Source equivalence or
    structured artifact equivalence is acceptable; binary identity is not the
    Python target shape.
  - L4: Python target becomes a credible compiler execution path only after L2
    and L3 receipts compose through the shared TestClaim/runtime surface.
Non-goals:
  - CPython binary production.
  - Treating `py_compile` as self-host.
  - Bit-identical executable output as the Python success criterion.
Falsification probes:
  F1. Python-emitted compiler slice compiles but produces different output from
      Rust/Go on the same fixture; L2 remains failed.
  F2. Python-emitted compiler slice produces semantically equivalent but
      formatting-different source; L3 uses the declared artifact comparison
      policy rather than raw byte equality if that policy is approved.
  F3. A Python runtime failure during compiler-slice execution blocks self-compile
      even if static checks pass.
```

## Dispatch routing

| Worksheet | Primary owner after Arbiter approval | Shared-fact review required |
| --- | --- | --- |
| PY-L1-STATIC-STRUCTURAL | Python RCA + Runtime/TestClaim for runner integration | Yes: static analyzer invocation/verdict may be shared with TypeScript and other targets. |
| PY-L1-L2-RUNTIME-FIXTURE-EXECUTION | Runtime/TestClaim | Yes: dissolve Python-specific L0 bridge into shared host-run/TestClaim receipt. |
| PY-L2-CROSS-TARGET-BEHAVIORAL-PARITY | Runtime/TestClaim + language RCA managers | Yes: release-minimum target set and runtime-value parity carrier. |
| PY-SELF-COMPILE-FRAMING | Python RCA, then Runtime/TestClaim and Compiler Spine by rung | Yes: fixed-point/artifact comparison policy must not be Python-only unless proven target-specific. |

## §8 Modeling DFS Arbiter approval checklist — CLOSED 2026-06-01

### Worksheet A (PY-L1-STATIC-STRUCTURAL)

- [x] **Shared carriers:** `TargetStaticAnalysisInvocation` + `TargetStaticAnalysisVerdict` in `v4.std.leaf_model_verification` — **not** `TargetPythonExerciseVerdict` extension
- [x] Per-tool profiles: `v4.extdeps.typecheckers.pyright` + `mypy` (advisory vs blocking on row)
- [x] Reject: CI-only pyright/mypy grep; Python-only static verdict vocabulary
- [x] **READY-FOR-WORKER-DISPATCH** (`proud-fox-405`)

### Worksheet B (PY-L1-L2-RUNTIME-FIXTURE-EXECUTION)

- [x] **Shared runtime carrier:** `TargetRuntimeExerciseVerdict` (additive, shared) for L1/L2 fixture execution
- [x] `TargetPythonExerciseVerdict` remains L0 leaf-model bridge only until dissolved by shared runner
- [x] Receipt path: `host_run` + `test_claim_falsification` — reject stdout/stderr shell compare as final authority
- [x] **ARBITER-APPROVED — BLOCKED-ON-RUNTIME/TESTCLAIM-ACCEPTANCE** (`proud-fox-405`; Runtime/TestClaim coordinates roster)

### Worksheet C (PY-L2-CROSS-TARGET-BEHAVIORAL-PARITY)

- [x] Reuse #4081 Wc L5 pattern; release-minimum set **Rust + Python + Go** per §11.8
- [x] Shared `RuntimeValue` + `TestClaimRun` verdict vocabulary — no per-target expected-value tables
- [x] **ARBITER-APPROVED — BLOCKED-ON-RUNTIME/TESTCLAIM-ACCEPTANCE** (`proud-fox-405`; depends on worksheet B runner surface + Runtime/TestClaim gate)

### Worksheet D (PY-SELF-COMPILE-FRAMING)

- [x] Policy frame ratified: L1 static/runtime → L2 behavioral parity → L3 artifact equivalence → L4 compiler execution path
- [x] Non-goals affirmed: no CPython binary self-host; `py_compile` ≠ self-host
- [x] **FRAME-ONLY** — no impl dispatch from this section

---

## Manager checklist

- [x] L0 consumed from #4117 rather than redefined.
- [x] pyright/mypy framed as modeled static evidence, not CPython compile.
- [x] Runtime execution framed as the meaningful Python verifier.
- [x] Cross-target parity extends #4081 Wc L5 instead of creating a new stdout shell authority.
- [x] Python self-compile framed as compiler execution parity before binary-like self-host.
- [x] Modeling DFS Arbiter approval.
- [ ] Runtime/TestClaim owner accepts runner-surface dispatch.
