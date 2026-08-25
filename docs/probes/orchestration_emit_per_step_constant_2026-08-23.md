# The orchestration emit cost is a per-step constant, not a fold shape (2026-08-23)

Filed, not chased. A cost-shape defect was **hypothesised from a correct reading of the source and
refuted by measurement**. What is actually there is a per-step constant of roughly two milliseconds
in the grammar emit pass — a large coefficient on a linear term, which is a different kind of fact
with a different owner, and explicitly **not** something DESIGN §6's always-fix rule reaches.

And that constant turns out **not** to be the dominant term for most of the family that provoked the
measurement: a per-witness fixed cost of about 2.2 seconds, which this probe did not measure, is
larger for twelve of the sixteen rows. Both figures are below.

This row exists because the refuted hypothesis is the more useful half. It read correctly from the
source, it had a byte-safe fix already drafted, and merging that fix would have been indistinguishable
from a real repair afterwards.

## What was hypothesised

`v2.compiler.emit_orchestration` `orch_emit_steps_from` folds a pipeline **left-linearly**, carrying
the accumulated script as a `String` and calling `orch_emit_join2(left: everything_so_far, right:
next_step)` once per step. `orch_emit_join2` is not a string concatenation: it reaches
`orch_emit_from_registry`, which builds a `TargetModel` whose binding spellings carry both operands
verbatim (`v2.extdeps.languages.bash` `orch_seq2_binding_spellings`) and runs the whole grammar
`emit` pass over it.

Read that way, a pipeline of n steps runs n-1 emit passes over payloads growing to the full script
length: quadratic in the emitted bytes with a compiler pass as the constant — the copied accumulator
DESIGN §6 names, and the same shape `std.nat` `nat_range_inclusive` already carries a note about.

**Every sentence above is true.** The mechanism is real and it is described correctly.

## What was measured

A scaling probe: one `test fn` per size, identical trivial `Do`/`Run` steps, **only the count
varying**. Release `claim_batch` on the session's arm64 container, two batches in separate processes.
Figures are **thread CPU** as `claim_batch` reports it; wall was within 1-5ms of CPU on every row, so
the clock choice changes no conclusion here.

| n | cpu | ratio vs previous doubling |
|---|---|---|
| 8 | 48ms | |
| 16 | 61ms | 1.27 |
| 32 | 94ms | 1.54 |
| 64 | 161ms | 1.71 |
| 128 | 308ms | 1.91 |
| *128* | *239ms* | *(second batch, separate process)* |
| 256 | 448ms | 1.87 |
| 512 | 894ms | 2.00 |
| 1024 | 1863ms | 2.08 |

**Linear across two decades, no knee.** A quadratic fold converges to 4.0 per doubling; this
converges to 2.0. The early sub-2.0 ratios are the fixed intercept washing out, not a curve.

Fit: **≈14ms + ≈1.8-2.3ms per step.** The slope differs between the two batches for the same probe on
the same box, which is ambient contention on the shared host — the same roughly 2x spread
`gunbc.witness_row_cost` `witness_row_cost_migration_threshold_note` already records.

**Read the ratio, never the absolute times.** The absolutes move with whatever else is running.

## What this does and does not establish

It establishes that `orch_emit_pipeline`'s cost is linear in step count over 8..1024, so the
accumulator is not the dominant term and rebalancing the fold buys nothing.

It does **not** establish a per-step cost for real deploy pipelines. The probe's steps are uniform
trivial `Do`/`Run` commands; production pipelines mix `Comment`, `Let`, `If` and redirect/capture
forms, which reach different emit paths. The ~2ms/step figure is the probe's, and the agreement with
the live family below is corroboration, not a second measurement.

## The consumer that made this worth measuring

`test.claim.live_deploy.emit` `twin_and_production_configure_disjoint_tailscale_endpoints` was
BUDGET-REFUSED on a required floor run (`Cpu, cost at least 5008ms` against
`v2.workflow.required_floor` `required_floor_claim_cpu_safety_limit_ms`), which reported it as
`interrupted_before_verdict` and reddened main.

Run alone locally it **passes**, at **cpu=5489ms wall=5504ms**. The 5008ms in the floor log is the
**interrupt point, not the row's cost** — the deadline preempted the verdict, exactly as that
diagnostic says. So the row is ~10% over the fail-stop, not the ~0.2% the interrupt figure suggests.

### The family's cost decomposes, and the emit term is not the larger one

The sixteen siblings appear in the same run's `[over-cost]` list at 2830-4109ms. They do **not**
emit a uniform number of scripts -- counted per witness, twelve emit one, three emit two, one emits
three, and the refused row emits four. Regressing the seventeen observed costs on that count:

**~2235ms fixed per witness + ~773ms per emitted script.**

| scripts | rows | observed | predicted |
|---|---|---|---|
| 1 | 12 | 2830-3178ms | 3008ms |
| 2 | 3 | 3841-3979ms | 3781ms |
| 3 | 1 | 4109ms | 4555ms |
| 4 | 1 | 5504ms | 5328ms |

The ~773ms marginal is consistent with the probe's ~2ms/step over roughly 400 emitted lines per
script, so the emit constant explains the **slope**.

It does not explain the **intercept**, and the intercept is the larger term for twelve of the
sixteen. About 2.2 seconds is spent before the first script is emitted, and this probe did not
measure what that is -- it is per-claim preparation, outside `orch_emit_pipeline` entirely.
**Naming it as unmeasured rather than folding it into the per-step figure is the point:** a reader
who takes 5504ms and divides by ~2ms/step concludes the four scripts are ~2750 rendered lines,
about 3.4x the ~800 the slope actually accounts for. An earlier revision of this row did exactly
that division. The fixed term is the bigger target for most of this family, and it has neither a
measurement nor an owner.

## Why this is not a §6 always-fix

DESIGN §6 fixes a **proven** cost-shape defect regardless of the realized n, and is explicit that the
humility is in not trusting your own "negligible here". The symmetric obligation is that a *plausible*
shape is a hypothesis: §6 fires on a wrong complexity class, and there is not one here. A large
constant is priced normally — against the displaced cost, on the ordinary dial.

Reducing it means **memoizing the emit path on declared-input content**, which is the
Realization/content-hash move DESIGN §2 holds up as canonical ("one kernel, N handlers", spanning
resolve-cost through sccache) and which §6 records v2 as still hand-rolling — `ParseTable` is named
there as the standing instance of the same gap. That is compiler-wide work on files DESIGN names as
load-bearing, and it is filed here rather than improvised.

## The class this row is really about

**A mechanism reading is not a cost measurement.** The hypothesis was not sloppy — it named the right
function, the right call, and the right reason that call is expensive. It was still the wrong
complexity class, because whether a real mechanism *dominates* depends on constants that are not
visible in the source. The per-step emit pass swamped the re-join entirely.

The tell is that the fix would have looked principled. `orch_construct_seq2` is `left "\n" right` over
two raw operand tokens (`v2.extdeps.languages.bash` `orch_seq2_tokens`, `orch_seq2_source_text`) — it
wraps nothing and escapes nothing, so it is **associative**, and a balanced or pairwise join tree
renders byte-identical output. The rewrite was available, correct, byte-safe, and pointless. Merged,
it would have been a green rewrite of a load-bearing pipeline stage justified by a story, and
afterwards nothing would distinguish it from a real fix.

So the standing rule this row proposes: **produce the scaling series before invoking §6.** Ratio ≈2
is linear, ≈4 is quadratic; one throwaway probe module doubling n costs minutes and settles it.

## What was deliberately not done

**The witness was not split.** `twin_and_production_configure_disjoint_tailscale_endpoints` asserts
six facts — three about apply, three about retract — and splitting it in two would put each half at
two scripts, which the regression above puts at ~3781ms, comfortably under the fail-stop, with no
assertion weakened and no coverage lost. It is available and it is refused: it would zero this row's failure frequency while leaving the sixteen
siblings at roughly 8x the 500ms **wall** migration threshold
(`gunbc.witness_row_cost` — note the threshold's axis is wall and the fail-stop's is thread CPU;
they are different clocks and comparing them directly is the mismatch that carrier warns about).
A deficit whose frequency is zero by construction never ranks for fixing — the absorbing fallback
DESIGN §5 names, executed at authoring time. The disclosure gate exists to keep that population
visible, and splitting would shrink the visible half.

**The fail-stop was not touched.** `required_floor_claim_cpu_safety_limit_ms` is a fail-stop
protecting the executor, not a tolerance or a budget, and raising it to clear a 10% overshoot converts
a wall into a negotiation.

**No declared-ceiling lane was built.** The `interrupted_before_verdict` diagnostic advises moving
such a row "to a lane declaring its own ceiling"; no such mechanism was found in tree, so that remedy
may not currently exist. Recorded as an observation about the diagnostic, not acted on.

## Standing state

The row stays on the disclosure roster where it is already counted, and it will intermittently
budget-refuse on main until the emit-path memoization lands. **That is named as a real intermittent
red with a known cause, not as an acceptable one.**
