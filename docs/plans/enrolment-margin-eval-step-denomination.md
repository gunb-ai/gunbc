# Denominating the enrolment margin in eval steps

Governed by DESIGN §5 (fail-closed, oracles, construction over validation) and §4b
(the guarantee ladder). This plan discharges clause (iv) of the §4b(3) drop
`gunbc.rung_drop` `floor_cost_cpu_regression_at_constant_eval_steps`, which is the
only clause of that row gunbc#11195 left standing.

## The defect this closes, stated as the drop already states it

gunbc#11195 moved the required floor's **claim ceiling** off CPU and onto eval steps,
and made CPU observed-only for every claim. It did not move every per-subject budget.
`v2.workflow.floor_enrolment_margin` still derives a 302ms budget from
`required_floor_per_subject_cpu_line_ms` and compares an **observed CPU reading**
against it, so a newly enrolled witness can still be refused on a figure that is a
property of the machine rather than of the claim.

That is exactly what the drop's restoration trigger forbids, in its own words:

> NO PER-SUBJECT BUDGET MAY CONSUME A RUN-LEVEL READING

The trigger is stated at **budget grain and not at denomination grain** deliberately,
and gunbc#11195 records why: an earlier revision quantified over the *primitives* a
cost-fidelity report must cover and said nothing about *which budgets* must stop being
vulnerable, so it could have been discharged by denominating one ceiling while another
per-subject budget kept charging a run-level cost to an arbitrary claim. Denominating
the claim ceiling was one step toward the trigger and is not the trigger.

## Why the first phase is not the enrolment margin

`floor_enrolment_margin` consumes `gunbc.floor_cost_distribution` `ClaimCostReading`
directly — `EnrolmentCostReading = EnrolmentCostMeasured { reading: ClaimCostReading } | …`.
Every arm of that coproduct is CPU-shaped:

```
type ClaimCostReading
  = ObservedCpuReading          { cpu_ms }
  | RightCensoredCpuReading     { cpu_lower_bound_ms, censoring_ceiling_ms }
  | CpuLowerBoundWithoutCeiling { cpu_lower_bound_ms }
```

So the gate **cannot see a step count at all** until the carrier carries one. The step
count exists today only as `FloorCostRow.eval_steps: Int`, a bare scalar beside the
coproduct, and a consumer reaching for it inherits a defect the corpus has already
filed against itself.

## The filed stall this phase discharges

`gunbc.guarantee_stall.eval_steps_outside_the_reading_coproduct_stall` — `current:
OutsideTheLadder`, `ceiling: StructurallyImpossible`, `blocker: ClimbableButUnbuilt`,
population `["gunbc.floor_cost_distribution"]`. Its subject, in its own words: the bare
`Int` means a right-censored row's observed-steps-**before-censor** and a completed
row's **performed** steps share one type and one constructor, so a fold may compare
them for equality and manufacture an equal-work cohort out of two observations that
measured different things — a **half-applied construction**, because the same file
already models the CPU half correctly and one field did not follow the decision its
sibling already made.

Its `next_rung_trigger` is this phase, verbatim: *eval_steps moves INSIDE the
`ClaimCostReading` arms, so an observed step count and an observed-so-far-before-censor
count are reached through different constructors and no fold can compare them.*

This plan therefore **does not invent its own first phase**. It executes a trigger the
corpus declared before this lane existed.

## Phase 1 — the carrier (the model)

```
type ClaimCostReading
  = ObservedCpuReading {
      cpu_ms: Millisecond
      eval_steps: EvalStepCount
    }
  | RightCensoredCpuReading {
      cpu_lower_bound_ms: Millisecond
      censoring_ceiling_ms: Millisecond
      eval_steps_before_censor: EvalStepCount
    }
  | CpuLowerBoundWithoutCeiling {
      cpu_lower_bound_ms: Millisecond
      eval_steps_before_censor: EvalStepCount
    }
```

with two accessors mirroring the CPU pair the file already carries, and named for the
same distinction:

- `observed_eval_steps(reading) -> EvalStepCount?` — `Present` on the observed arm
  only. The point question. A bound cannot reach a site that treats it as performed
  work without the caller writing an `Absent` arm.
- `eval_steps_at_least(reading) -> EvalStepCount` — total over all three arms. The one
  question a censored row can answer soundly: *this claim performed at least N steps*.

`FloorCostRow.eval_steps: Int` is **deleted**, not deprecated. DESIGN §3's replacement
migrations cut over at the root, and a surviving bare `Int` is the attractor that rule
exists to prevent: while it stands, every nearby question is answered in its vocabulary.

### Why inside the arms rather than a sibling coproduct

A sibling `EvalStepReading` beside `cost: ClaimCostReading` would satisfy the trigger's
*letter* — two constructors, no bare `Int` — while leaving the incoherent state
constructible: a row claiming an **observed** CPU reading beside a
**before-censor** step count. Whether the claim reached its end is ONE fact about ONE
occurrence, and it discriminates both readings. Carrying the step count inside the arm
makes disagreement unconstructible, which is the stall's declared ceiling
(`StructurallyImpossible`) rather than one rung below it.

This is not the clock fusion `std.measure` `measure_clock_basis_note` forbids, and the
distinction is worth stating because gunbc#11195 was reworked twice over exactly it.
Fusion is *comparing* magnitudes read from different clocks. This is *co-locating*
three observations of one occurrence under the discriminator they genuinely share. No
arm compares a step count to a millisecond.

### Phase 1 blast radius, measured rather than estimated

Measured on `origin/main` at `3a0f2d4b585`:

| surface | sites |
| --- | --- |
| `ObservedCpuReading {` | 41 |
| `RightCensoredCpuReading {` | 21 |
| `CpuLowerBoundWithoutCeiling {` | 11 |
| `FloorCostRow {` | 41 |

across 7 `.dag` files (`floor_cost_distribution` and its witness test and instrument,
`floor_enrolment_margin` and its test, `floor_cost_debt_admission` and its test) plus
the seed mirror in `required_floor_runner.rs`, `cli_run.rs` and `v1_interpreter.rs`.

The type change admits **no partial state** — every constructor site moves in the same
commit or the corpus does not compile. That is why this is its own phase and why this
PR opens with the plan rather than a half-migration.

**The step value at each site is authored, not mechanical.** A fixture's step count is
part of what the fixture asserts, so these are not a `sed`. Fixtures whose subject is
the CPU half take a step count that is declared as incidental beside the reading they
actually exercise.

### Phase 1 evidence

- **RED** — a fold that compares a performed step count to a before-censor one must
  become unwritable. The discriminating probe is the *source* handed to the compiler by
  a fixture, per §4b: the invalid program is authorable there even once it is
  unwritable in the accepted corpus, and declining the fixture would be
  specification-without-execution.
- **CONTROL** — `observed_eval_steps` answers `Absent` on both bound arms and `Present`
  on the observed one, executing, so the accessor is not a decoration.

## Phase 2 — the enrolment margin consumes it

`floor_enrolment_margin_budget_ms` becomes an `EvalStepCount` derived through the **same**
pinned calibration the claim ceiling uses (`v2.workflow.floor_eval_step_calibration`
`eval_step_budget_for`), from a declared policy in milliseconds of work. It is not a
second calibration and not a literal: §5's oracle rule admits the budget only because it
is grounded in a controlled fixture, and a second rate would be a §3 fork of that
authority.

`EnrolmentMarginStanding` keeps its arms and changes their denomination:
`EnrolmentWithinMargin` / `EnrolmentOverMargin` carry `EvalStepCount`, and the CPU
reading is **carried beside the verdict as an observation** rather than deciding it —
the same shape gunbc#11195 gave the claim ceiling.

`EnrolmentBoundWithoutCeiling` survives unchanged in meaning: a preempted row's steps
are a lower bound with no ceiling, exactly as its CPU is.

### Phase 2 evidence

- **RED** — a claim whose eval steps exceed the enrolment budget refuses.
- **CONTROL** — a claim far over the 302ms CPU line with in-budget steps is admitted,
  with the CPU observation recorded. This is the same pair gunbc#11195 enrolled for the
  claim ceiling, and it must be re-authored here rather than cited: a different budget
  is a different subject.

## Phase 3 — retire the CPU line, or say why it stays

`required_floor_per_subject_cpu_line_ms` exists only because per-subject decisions still
read a CPU clock. Since gunbc#11700 `v2.workflow.floor_cost_debt_admission` compares nothing
(a typed admission is identity and reason); the consumers are both in
`v2.workflow.floor_enrolment_margin`: the margin budget, which must sit below the line, and
the live Roster ground (`enrolment_declared_measured_standing`), which admits only a reading
over it. When Phase 2 denominates the margin, the Roster ground is the remaining consumer, and
Phase 3 decides that one: either it is denominated too and the symbol is deleted with its own
dissolution condition discharged, or the symbol survives with a population of exactly one,
stated.

**Only when no per-subject budget reads a run-level figure does the drop's clause (iv)
retire**, and the drop is retired by its trigger and by nothing else.

## What this plan deliberately does not do

- It does not re-arm a CPU ceiling anywhere. gunbc#11195's evidence stands: three
  prose-only heads measured the same floor at 36.7s / 40.8s / 44.7s of summed CPU, and
  one identity at an identical 2884 eval steps read 34ms, 455ms and 511ms. A CPU line
  refuses on the environment, not on the claim.
- It does not normalise per runner, and does not enrol slow identities as cost-debt
  rows. Both are refused designs of record, for the reasons the floor authority states.
