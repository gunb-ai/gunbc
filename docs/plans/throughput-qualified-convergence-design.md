# Throughput-qualified convergence — one shape, two subjects

Home: DESIGN §3d (selection precedes convergence; a convergence primitive assesses a caller-supplied
goal against an independent observation) and §4b (rung honesty: the reported rung must equal the rung
established by executed evidence). This plan is a design, not a landed capability; each stage names
what it would make executable.

## The defect this exists to close

**A convergence today verifies its CONFIGURATION and calls that success. Neither subject verifies the
RATE the configuration was chosen to deliver.** The configuration is a proxy for the goal, and
asserting the goal from the proxy is the gap.

Two live instances, each found by looking rather than assumed:

- **Serving, Group B, 2026-09-17.** A converge reported success and was independently verified on all
  four hosts: image derived, pin correct, checkpoint shards loading, fp8 admission holding, KV law
  single-sourced. Every one of those is true and none of them is about the collective transport. The
  arm was in fact running every channel over `NET/Socket/0` with no IB device present, which the
  converge could report full agreement through because transport was not in its comparison. Aggregate
  decode measured 51.19 tok/s at 16 streams on that transport.
- **CI, srv1/srv2 (and Mt Collins next).** `docs/plans/ci-humming.md` is the CI-operations authority
  and its throughput section T0-T6 reasons entirely about DERIVING a runner-slot count from a memory
  budget. T1 already carries the right instinct one level down — a cap is not effective until the live
  cgroup is read back (`MemoryMax != infinity`), and T5's converge is set-property → read-effective →
  verdict. The discipline stops at configuration. Nothing measures jobs per hour after the converge,
  so a slot count derived from a budget is believed rather than established.

The shape is identical in both: **a system is converged toward a configuration chosen for a rate, and
the rate is never put to it.**

## Why this is one concept and not two (DESIGN §2, horizontal)

At the right layer there is nothing fundamentally different between "16 decode streams against a
serving arm" and "N concurrent builds against a runner pool". Both are: *within a reserved window,
apply a declared offered load to a subject, derive an aggregate rate from a counter the system itself
keeps, and assess it against a floor that is not a copy of the last reading.* Modelling it twice is
the fork §3 forbids; the parts that genuinely differ are realization handlers, not separate concepts.

**Shared (the interface).** The reserved window; the declared offered load; saturation standing;
completion accounting; rate derivation; subject-bound comparability; the floor and its verdict; the
receipt.

**Per-subject (bound handlers, one per subject, never a fork of the fold).** What a unit of work is
(a decoded token / a completed job); how load is applied (HTTP completions / dispatched builds); which
counter is read (`vllm:generation_tokens_total` / job completion events); what the comparability axes
are.

## The seven parts, each with the failure it prevents

### 1. The window is a lease, not a convention
A throughput probe puts real load on a shared resource. "We reserve a period" is a **lease with a
declared deadline**, and its home already exists: `std.temporal_effect::HeldLease`, with
`std.durable_compare_and_set::CasExpectation` for the linearization where two probes could collide.
Admission refuses when the subject carries foreign in-flight work (`vllm:num_requests_running > 0`;
a runner pool with queued PR jobs). *Prevents:* a probe that measures someone else's contention and
reports it as the subject's capacity, and two probes measuring each other.

### 2. Offered load is declared, never inferred
Concurrency, unit size, and stopping rule are inputs carried in the type. *Prevents:* the defect in
my own hand measurement today — I picked concurrency 8 with no reason when the modelled profile
carries `max_num_seqs: 16`, so the first reading understated aggregate by half. An undeclared load is
a number whose denominator nobody can reconstruct.

### 2b. The offered load itself needs an authority, not a preference
Declaring the load is necessary and not sufficient: the FIGURE must be grounded. `max_num_seqs: 16`
is carried in `glm_native_concurrent_profile` as a chosen number, so a probe at 16 streams inherits
whatever justified it. The probe's concurrency is derived from the subject's admitted concurrency
policy, and that policy owes an observed capacity receipt from the real startup readings rather than
a figure someone liked. *Prevents:* the shape one level up from my concurrency-8 error — not picking
the wrong number, but picking any number without an authority behind it, which makes every rate
measured at it unattributable. (Raised by the side-chat review, 2026-09-17.)

### 3. Saturation standing is part of the reading, and this is the subtle one
If nothing queued, the probe measured the LOAD IT OFFERED, not the subject's capacity. Those are
different facts and collapsing them is the §5 absorbing-fallback shape: a comfortable number that
silently answers a question nobody asked. The reading is three-valued —
`SaturatedAtOfferedLoad` (queue depth > 0 throughout) | `UnsaturatedAtOfferedLoad` (never queued, so
this is a floor on capacity and not a measurement of it) | `SaturationUnread`.
*Applies to today's figures:* per-stream barely moved from 8 to 16 streams (3.41 → 3.20) while
aggregate nearly doubled, so 51.19 tok/s is almost certainly `UnsaturatedAtOfferedLoad` — a lower
bound on what that arm can do on sockets, and it must not be recorded as its capacity.

### 4. Completion accounting is fail-closed
A unit that errors consumes wall clock and contributes nothing. My hand harness divided a counter
delta by wall time, so a failed request would have silently lowered the rate and looked like a slow
system. The reading carries units attempted, completed, and refused; a probe with any refused unit
reports `RateUnderIncompleteRun` rather than a rate. *Prevents:* a broken subject reading as a slow
one — the diagnosis that sends someone tuning a system that is actually erroring.

### 5. The rate is derived, never stored
`gunbc.harness.harness_throughput` already states this law for its own stream: tokens, interval and
count are carried; tokens-per-second is a function of them, and storing it alongside would be a second
representation free to disagree with its inputs. The same law here, cited rather than re-coined.

### 6. The subject is what makes two readings comparable
`gunbc.spark.serving_performance_subject` is this, already landed and already carrying twelve axes
including `AxisCollectiveTransport`, with a join that refuses on a differing axis and keeps `<unread>`
distinct from a value. **The CI subject does not exist yet and is the larger modelling job.** Its axes
are not the serving ones and must be derived from what invalidates a prior number — at minimum: host
class (Mt Collins vs srv1/srv2), runner slot count, per-slot memory cap, jobserver tokens, cpu.weight
class, toolchain revision, and **cache state**, which is the one most likely to be forgotten and the
one most able to produce a meaningless comparison: a cold-sccache run and a warm one are different
subjects, not a regression. Per DESIGN §3's external-decomposition rule these are two subject
authorities inhabiting one shape, not one enum with a product column.

### 7. The floor is not a copy of the last reading
DESIGN §5 is explicit: a merge-blocking test may compare to a numeric literal only when that literal
is grounded in a controlled fixture, an external or versioned authority, an explicit policy budget, or
a monotone debt contract. A floor pasted from this morning's run collapses to `measure() == measure()`
— a change detector whose entire content is the manual update. So the floor is either **a declared
operating budget someone owns** ("Group B on verbs sustains ≥ X aggregate at 16 streams", "srv1
sustains ≥ Y jobs/hour at N slots"), or **a monotone contract** that may only be raised deliberately,
with provenance on the row either way. The verdict itself is `std.goal_assessment` — a caller-supplied
goal assessed against an independent observation, which is exactly this and must not be re-coined.

## Provenance of the converged state is part of the subject

A rate measured against an arm someone converged by hand is not a reading about a gunbc convergence,
and recording it as one would claim an evidence chain that does not exist. The Group B arm serving on
2026-09-17 was converged by a lane outside the modelled transaction, so its standing is
`AdoptedOutOfBandRealization` — observed, admitted as the incumbent, and explicitly NOT
converged-by-gunbc. The distinction matters the moment a floor is set from such a reading: a number
taken from an adopted realization grounds a floor for the SUBJECT, never a claim about the
transaction that did not run. (Raised by the side-chat review, 2026-09-17.)

## Conformance (DESIGN §3b) — the answer is yes, and here are the rows it touches

This is a conformance question, and naming the homes before any worker runs is the printing-press
direction §3b asks for:

- **fabric / compute** — runner slots and serving seats are both capacity. Homes:
  `gunbc.product.capacity.lease::LeaseGrant`, `gunbc.product.fabric.selection::select_supply`. The §3b
  row already covers both compute cells and inference serving and already states the sharing is
  partial, which is the same partial sharing this design has.
- **leasing / locking / grants** — the reserved window. Homes: `std.temporal_effect::HeldLease`,
  `std.durable_compare_and_set::CasExpectation`.
- **process observability / reporting** — the receipt, at occurrence grain. Homes:
  `std.observation::ObservationEvent`, `std.observation::RecordedObservation`.
- **decision / selection** — NOT touched in the first stages, and saying so is part of the design. A
  probe that merely assesses is not choosing a configuration. The moment it ranks candidates (which
  slot count is best, which transport to run), `std.decision` is the home and §3d's laws apply.

## What this does NOT establish (DESIGN §4b, ceiling honesty)

A green probe says the subject delivered a rate under a declared load in a reserved window. It does
not establish behaviour under production traffic shape, and it does not make the configuration
correct — configuration agreement and rate agreement are two readings and a qualified convergence
needs both. The attainable ceiling here is **mechanically preventable**, not structural: the invalid
state (a convergence declared without its rate) stays writable, and safety depends on the probe being
enrolled and executed. Claiming higher would be the rung inflation §4b(1) names.

## Staging

1. **Record the socket baseline as subject-bound rows.** The Group B readings taken 2026-09-17 with
   their twelve axes and their `UnsaturatedAtOfferedLoad` standing. Justified as a receipt rather than
   a transcription (§6) for the same reason the RoCE captures were: once the passthrough is fixed the
   socket arm is gone and that question cannot be put to it again.
2. **The serving probe as a modelled instrument.** extdeps rows for the completions API and the metrics
   counters; the window, load, saturation, completion and rate folds; witness rows per layer on
   supplied inputs, with one inhabitance claim that runs the real endpoint.
3. **The CI subject.** The axis roster above, derived with the same test — does a change to this
   property invalidate a prior number.
4. **The CI probe**, binding the same interface to dispatched builds on Mt Collins.
5. **Qualification.** A converge consumes the assessment, so "converged" means configuration agreed
   AND the declared rate was met in a reserved window — with the drop declared if a subject cannot yet
   be probed, rather than silently qualifying on configuration alone.

Stages 3 and 4 are the ones with real unknowns; 1 and 2 are bounded by what landed today.
