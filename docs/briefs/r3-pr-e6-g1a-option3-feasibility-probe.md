# R3 PR-E E6-G1.a — Option 3 Static Representative Feasibility Probe

**Status:** Director-authorized feasibility probe. This is not an
implementation brief. Do not dispatch implementation from this packet until
Director review accepts the representative.

**Director ratification:** 2026-05-06, Option 3 for the E3/G1.a STOP:
revise the static representative so E6-G1.a can exercise static lens function
fields and report construction over already-evaluator-shaped inputs, while
deferring reflected-program `Dag` folding to a separate carrier question.

**Supersedes for dispatch:** the representative assumption in
[`r3-pr-e6-g1a-static-lens-fold-worker.md`](r3-pr-e6-g1a-static-lens-fold-worker.md)
until this probe either lands a Director-accepted revision or reports STOP.

## Problem

The first E6-G1.a implementation attempt proved that the original static TC1
representative still needed a substrate-shaped `Dag` value at runtime. The only
worker-found path was a local bridge:

- compiled `Dag` to evaluator `Value` reification;
- whole-Dag reflection added through `lens_apply.rs`;
- `DeclarationId` encoded as `Int` because evaluator `Value` has no typed
  declaration-reference inhabitant.

That path is rejected. It creates local authority instead of executing the
existing body-evaluator surface.

## Probe Goal

Find a finite static `Lens<C>` representative that proves the E6-G1.a
evaluator facts that are already in scope:

1. static top-level `data ... : Lens<C>` value consumption;
2. static function-field calls lowered through X1.a to
   `TransformTarget::Callable`;
3. non-function field reads through `TransformTarget::FieldProject` over
   `Value::RecordValue`;
4. `Witness<C>`, `OptionalDiagnostic`, and `DimensionReport<C>` construction
   through declared constructors and existing evaluator `Value` carriers.

The representative must use input values the evaluator can already supply
without compiled-program reflection. Because the live `Lens<C>` carrier fixes
`read` and `validate` to `Dag`-typed inputs, this likely means opaque
already-evaluator-shaped `Dag` / `Behavior` arguments whose structure is not
inspected by the representative.

The representative does not need to prove reflected-program folding. That is
now a separate `ReflectedProgram<T>` / declaration-reference carrier question.

## Hard Scope Bars

The probe and any follow-up implementation brief must not:

- require compiled `Dag` to evaluator `Value` reification;
- add an `eval_substrate_reify.rs`-shaped local authority;
- extend `lens_apply.rs` reflection to whole-Dag or route through
  `lens_apply.rs`;
- encode `DeclarationId` as `Int`, `String`, or any other existing scalar
  carrier;
- introduce new evaluator `Value` variants;
- add parser, lowerer, or substrate carrier changes;
- claim generic `fold_lens<C>` or runtime-sourced callee dispatch.

If one of those is required, STOP. The next action becomes Q-Reification /
reflected-program carrier design, not E3 implementation.

## HEAD Check

`src/v3/std/lens.dag` fixes the `Lens<C>` carrier to:

```text
read: fn(Dag, Behavior) -> Witness<C>
validate: fn(Dag, C) -> OptionalDiagnostic
```

Therefore Option 3 cannot honestly change the representative to a different
subject type such as `MiniSubject` while still claiming to exercise the live
`Lens<C>` carrier. A subject-typed mini lens would be a parallel `LensLike`
fixture, not E6-G1.a.

The only plausible Option 3 probe is narrower: keep the live `Lens<C>`
signature, but choose a static representative whose `read` and `validate`
functions do not inspect the `Dag` or `Behavior` arguments. The evaluator test
would supply already-evaluator-shaped opaque argument values for those ports.
That can prove static lens field calls and report construction, but it does
not prove reflected-program folding.

If Director judges that an argument-opaque representative is too weak for
Path A, this probe should STOP immediately and route Q-Reification.

## Candidate Representative Shape

Prefer a tiny static lens that keeps the live `Lens<C>` signature but avoids
reading reflected-program structure:

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
    Violates { reason: diag, at: beh } =>
      DimensionFail {
        dimension_name: mini_lens.name,
        violations: singleton_diagnostic(diag),
        witnesses: singleton_witness(Violates { reason: diag, at: beh })
      }
  }
```

The implementation test may bind `d` and `b` directly in the evaluator frame
using existing `Value` carriers, but the representative must not inspect either
argument. If any body needs to project fields from `d` or `b`, the probe has
crossed back into reflected-program reification and must STOP.

The `singleton_witness` / `singleton_diagnostic` helpers above are placeholders
for whatever declared list constructor surface is live at implementation time.
The implementation may choose an equivalent already-declared list producer, but
it must not replace the witness list with an unrelated literal or empty list:
the `Witness<C>` returned by `mini_lens.read(d, b)` must flow into the
`DimensionReport<C>`.

## Feasibility Checks

Before authoring an implementation brief, verify at HEAD:

1. The proposed representative keeps the live `Lens<C>` signature from
   `src/v3/std/lens.dag`; no alternate lens carrier or subject-typed fixture.
2. The proposed `data ... : Lens<C>` value lowers static function fields to
   `TransformTarget::Callable` through the existing X1.a path.
3. The proposed opaque `Dag` / `Behavior` arguments can be supplied to
   `evaluate_body` using existing
   `Value::LiteralValue`, `Value::RecordValue`, or `Value::VariantValue`
   carriers only, and no executed body inspects them.
4. Report constructors for `Witness<C>`, `OptionalDiagnostic`, and
   `DimensionReport<C>` execute through E6-G0d constructor runtime support.
5. `mini_report` or its final equivalent calls `mini_lens.read(d, b)` and uses
   the returned `Witness<C>` to choose the report path and populate report
   witnesses/composed/fail fields as appropriate. A report built from only
   literals is not sufficient.
6. The representative avoids `LoopBound::Descent`; any iteration uses
   cardinality-only behavior or no loop.

## Feasibility Outcomes

### Outcome A — Viable Representative Found

Author a revised worker brief that replaces the original `Dag`/TC1-reflection
representative with the accepted argument-opaque representative. Director
review accepted this representative shape as sufficient for E6-G1.a's narrow
mechanism-demonstration scope: static lens value consumption, static
function-field calls, field projection, and report construction.

Acceptance for the implementation brief:

- acceptance gate explicitly says: lens consumer-wiring mechanism
  demonstration; lens-over-`Dag` folding is deferred to
  `ReflectedProgram<T>` / typed declaration-reference carrier work;
- compile-to-evaluate test proves the static lens value is consumed;
- lens function fields execute through `TransformTarget::Callable`;
- the implementation test fails if `mini_lens.read(d, b)` is removed, bypassed,
  or replaced by a literal carrier; the returned `Witness<C>` must feed the
  `DimensionReport<C>`;
- non-function field reads execute through `FieldProject` where the live
  representative uses them, such as `mini_lens.name` or
  `mini_lens.sequential.identity`;
- report values are `Value::VariantValue` / `Value::RecordValue` built through
  declared constructors;
- test fails if it imports `lens_apply`, `eval_substrate_reify`, or reflection
  helpers;
- PR body explicitly says reflected-program `Dag` folding remains deferred;
- cementing-test discipline for static lenses is not part of E6-G1.a Outcome A
  acceptance. It waits behind Q-Reification and the reflected-program carrier
  landing.

Verification V1 cascade routing:

- If Verification can demonstrate TC1 eta-equivalence with the
  argument-opaque representative, V1 and E3 remain paired in the same release
  step.
- If TC1 eta-equivalence requires a real lens-over-`Dag` fold, V1 unpairs from
  this E3 slice and waits for Q-Reification / carrier landing. This does not
  block the E6-G1.a Outcome A mechanism demonstration.

### Outcome B — No Honest Representative Exists

STOP and report the exact blocker:

- Director rejects an argument-opaque `Dag` / `Behavior` representative as too
  weak for Path A; or
- the evaluator cannot bind opaque `Dag` / `Behavior` arguments without
  reflection or new carriers; or
- report construction still requires a typed reference carrier absent from
  evaluator `Value`; or
- static lens function fields cannot be authored without parser/lowerer or
  substrate changes.

This STOP triggers Director's Q-Reification design audit and the
Substrate-side reflected-program carrier calibration.

## Worker Pin Recommendation

Use valiant-carp-10 only after Director accepts a revised implementation brief.
Until then, this is manager-authored feasibility/design work, not a worker
implementation task.
