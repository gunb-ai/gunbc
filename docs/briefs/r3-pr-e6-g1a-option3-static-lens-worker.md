# R3 PR-E E6-G1.a — Option 3 Static Lens Mechanism Worker

**Status:** worker implementation brief. Dispatch only after
[`r3-pr-e6-g1a-option3-feasibility-probe.md`](r3-pr-e6-g1a-option3-feasibility-probe.md)
is present on `main`.

**Worker pin:** valiant-carp-10.

**Scope:** implement the E6-G1.a Outcome A mechanism demonstration: a static
`Lens<Int>` value whose declared function fields are consumed by the evaluator,
whose non-function fields are projected through the evaluator, and whose
`Witness<Int>` result is lifted into the existing `DimensionReport<Int>`
carrier.

This is not a reflected-program fold. Lens-over-`Dag` folding is deferred to
follow-on consumer-wiring work (`Dag` IS the reflected program per Q-Reification
Option A ratified 2026-05-07 [#2096](https://github.com/gunb-ai/gunbc/pull/2096);
the deferred work is **wiring lens-fold consumers through `.dag` body authority
via the Evaluator**, NOT a separate `ReflectedProgram<T>` carrier).

## Context

The previous E6-G1.a vehicle STOPed because it needed compiled-`Dag` to
evaluator-`Value` reification and encoded declaration references through scalar
carriers. That path is forbidden.

Director ratified the Option 3 replacement: use an argument-opaque static
representative that keeps the live `Lens<C>` signature from
`src/v3/std/lens.dag`:

```text
read: fn(Dag, Behavior) -> Witness<C>
validate: fn(Dag, C) -> OptionalDiagnostic
```

The representative may accept `Dag` and `Behavior` arguments, but its executed
bodies must not inspect them. The slice proves evaluator consumer wiring, not
semantic analysis of a reflected program.

## Required Shape

Author the smallest static representative that compiles and evaluates through
the existing body evaluator:

```text
fn mini_read(d: Dag, b: Behavior) -> Witness<Int> =
  Inhabits(1)

fn int_add(a: Int, b: Int) -> Int = a + b
fn int_max(a: Int, b: Int) -> Int = if a > b then a else b
fn mini_iterate(c: Int, bound: LoopBound) -> Int = c
fn mini_validate(d: Dag, c: Int) -> OptionalDiagnostic =
  NoDiagnostic

data mini_lens: Lens<Int> = {
  name: "mini_static",
  read: mini_read,
  sequential: { op: int_add, identity: 0 },
  branch: int_max,
  iterate: mini_iterate,
  validate: mini_validate
}

fn mini_report(d: Dag, b: Behavior) -> DimensionReport<Int> =
  match mini_lens.read(d, b) {
    Inhabits(c) =>
      match mini_lens.validate(d, c) {
        NoDiagnostic =>
          DimensionOk {
            dimension_name: mini_lens.name,
            composed: c,
            witnesses: singleton_witness(Inhabits(c))
          }
        SomeDiagnostic { value: diag } =>
          DimensionFail {
            dimension_name: mini_lens.name,
            violations: singleton_diagnostic(diag),
            witnesses: singleton_witness(Inhabits(c))
          }
      }
    Violates { reason: _reason, at: _behavior } =>
      STOP
  }
```

The exact names may change to match local fixture style. The structural
requirements may not change:

- `mini_report` must call `mini_lens.read(d, b)`;
- the returned `Witness<Int>` must select the report path;
- the `Inhabits(c)` payload must flow into both `DimensionOk.composed` and the
  report witness list;
- `mini_lens.name` or an equivalent live non-function lens field must be read
  through `TransformTarget::FieldProject`;
- `mini_lens.read` and `mini_lens.validate` must execute through
  `TransformTarget::Callable`;
- `Witness`, `OptionalDiagnostic`, and `DimensionReport` values must be built
  through declared constructors and existing evaluator `Value` carriers.

## Read-Violation Rule

Do not lift `Witness::Violates.reason` into `DimensionFail.violations`.
`Witness::Violates` carries `reason: String` and `at: Behavior`, while
`DimensionFail.violations` carries `List<Diagnostic>`.

Outcome A may include a read-channel violation path only if the implementation
identifies and exercises an existing declared `Diagnostic` construction path.
If no such path exists, keep the representative on the `Inhabits` path and add
a fail-closed assertion or documented STOP that any read-channel violation
lifting is deferred.

## Hard Bars

Do not:

- add `eval_substrate_reify.rs` or any equivalent reification helper;
- route this slice through `lens_apply.rs` or extend whole-Dag reflection;
- encode `DeclarationId`, `DeclarationRef`, or any declaration handle as
  `Int`, `String`, or another scalar carrier;
- add evaluator `Value` variants;
- add parser, lowerer, or substrate carrier changes;
- claim generic `fold_lens<C>`;
- claim runtime-sourced / `Indirect` callable dispatch;
- add runner or `BinaryDimensionReportEquals` behavior;
- import reflection helpers into the implementation test.

If any hard bar becomes necessary, STOP and report the exact missing authority.

## Acceptance

The implementation PR must demonstrate:

1. A top-level `data mini_lens: Lens<Int>` or equivalent static `Lens<Int>`
   instance is consumed by evaluator execution.
2. Static function-field calls execute through the existing
   `TransformTarget::Callable` evaluator path.
3. At least one non-function lens field read executes through
   `TransformTarget::FieldProject`.
4. `mini_report` evaluates to a real `DimensionReport<Int>` represented by
   `Value::VariantValue` / `Value::RecordValue`, not by a host-only fixture.
5. A focused test fails if `mini_lens.read(d, b)` is bypassed or replaced by a
   literal-only report construction.
6. A focused guard fails or STOPs if the implementation tries to lift
   `Violates { reason: String, at: Behavior }` into `List<Diagnostic>` without
   an existing declared diagnostic-construction path.
7. The test proves no `lens_apply`, `eval_substrate_reify`, or reflection
   helper is imported or called.
8. PR body explicitly says: this is a lens consumer-wiring mechanism
   demonstration; lens-over-`Dag` folding is deferred to follow-on
   consumer-wiring (`Dag` IS the reflected program per Q-Reification Option A
   ratified 2026-05-07 [#2096](https://github.com/gunb-ai/gunbc/pull/2096); the
   deferred work is wiring fold consumers through `.dag` body authority via the
   Evaluator, NOT a separate `ReflectedProgram<T>` carrier).

Validation should be the narrowest relevant test target plus any repository
format/check command normally required for touched files.

## Non-Goals

- Cementing tests for static-lens semantic equivalence.
- Verification V1 / TC1 eta-equivalence implementation.
- Generic `fold_lens<C>`.
- Whole-Dag traversal.
- Descent loop execution.
- New list, diagnostic, or reflected-program carriers.

Verification V1 remains cascade-routed separately: if TC1 can use this
argument-opaque representative, V1 may pair with E3 in narrow form; if TC1
requires real lens-over-`Dag` folding (consuming `Dag` via `.dag` body
authority through Evaluator per Q-Reification Option A ratified 2026-05-07
[#2096](https://github.com/gunb-ai/gunbc/pull/2096); `Dag` IS the reflected
program — no separate carrier lands), V1 waits for that consumer-wiring work
to land, NOT for a carrier-introduction.
