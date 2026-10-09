# The floor's aggregate cost has no authority

Status: proposal. No code yet.

Raised from reading two required-floor runs (35365418267 merge_group,
35366842458 pull_request, 2026-09-18). Those IDs say WHERE the reading was
taken and are deliberately not doing the work of a producer: nothing in this
document quotes a duration, a population or a ratio as a standing quantity.
The producer for a floor run is the `required-witnesses-floor` job of
`gunbc.witness_floor_workflow`, and this proposal's whole point is that the
quantity it is about has no carrier yet -- so where a figure would go, it names
what would emit it instead (DESIGN §6: name the instrument, never transcribe
its output).

## The gap

The floor has a rich **per-item** cost model and no **population** model.

Per item, today: `std.evaluation_budget` bounds one evaluation on two clocks;
`v2.workflow.floor_enrolment_margin` requires a newly enrolled witness to reach
a verdict with derived headroom; `v2.workflow.floor_cost_debt` withholds
identities whose marginal cost exceeds
`v2.workflow.required_floor` `required_floor_per_subject_cpu_line_ms`;
`gunbc.floor_cost_distribution` `implied_clean_run_budget` derives that line
from measured run-to-run inflation as an order statistic.

Every one of those is denominated in ONE witness. None is a statement about the
run. So the floor's total is a quantity no declaration carries, no gate reads,
no refusal can cite, and no change is charged against.

Each added witness is individually justified, individually inside its margin,
individually admitted -- and the aggregate is nobody's finding. That is §5's
externalization at authoring time: the cost is real, it is caused by a specific
change, and it is paid by every session that later waits on CI.

## Why a per-item budget cannot be tightened into a population budget

Tightening `required_floor_per_subject_cpu_line_ms` prices the TAIL, not the
SUM. A per-subject line low enough to bound the sum would withhold a large
population of individually cheap, individually correct witnesses -- a §4d
over-prohibition, forbidding more than the evidence supports. Sum and maximum
are two facts; one authority cannot carry both.

## What is missing, stated as a modeling obligation

Per §3b, a domain whose home does not exist is a modeling obligation, not yet a
conformance row. The obligation is an observation carrier and a disposition.

**The observation must be attributed, not a single wall figure.** A lump total
would blame whichever witness's window happened to contain a shared cost --
the same attribution defect `floor_cost_debt` already discovered per-claim.
The grains are: fixed preparation demand; shared-fill demand; marginal claim
work; orchestration; and an explicit **unattributed remainder**. The remainder
is a declared field rather than a silent residue, because on the measured runs
on the run this proposal was raised from it was the largest single component --
a ONE-OFF reading, taken by summing the `done in` stage lines against the
`[floor-phase]` seam markers in one required run's log, with no entry point that
re-derives it. It is named as a one-off rather than quoted as a quantity, and
the carrier's `unattributed_work` field is what makes the figure a produced
output instead of a remembered one and an authority that cannot say how much it cannot explain is
reporting a number it has not earned. `measurement_completeness` travels with
the observation so a disposition derived from a partial reading is refusable
rather than quietly authoritative.

**The policy carries three facts, not one budget.**

- **desired target** -- 20 minutes.
- **hard steady-state ceiling** -- 30 minutes.
- **declared migration debt** -- NOT a figure carried here. It is read from the
  `required-witnesses-floor` job of `gunbc.witness_floor_workflow` at the revision
  the authority lands, by the producer the observation names. A number transcribed
  into this proposal would be the baseline the whole policy is denominated in,
  decaying without anyone touching either end (DESIGN §6).

The three are separate because the floor is ALREADY over the ceiling. A policy
that refused everything above 30 today would deadlock the repository, and a
policy that silently adopted the observed figure as the budget would let the
temporary baseline replace the target. So during migration the ordinary obligation is
**non-worsening against the declared debt**, while the performance lanes lower
the debt; once the floor is under 30 the ceiling ratchets down and becomes an
ordinary refusal boundary. The target stays visible throughout.

**An increase is a separately authorized act.** An envelope any cost-adding
change may raise is accounting, not governance -- the change cannot grant
itself the room. An amendment names the added capability, its measured cost,
the authorizing operator, and a retirement or offset condition, and it raises
the ceiling without raising the target.

## What this does NOT license

It does not license deleting evidence. §4b(4) requires a repaired class's
discriminating RED and positive control to **remain enrolled**. What it does
not require is one permanent authored file and one independently paid compile
per specimen: evidence may be consolidated into a data-driven matrix, carried
by one stronger discriminating construction, evaluated over a shared prepared
closure, or subsumed by a stronger wall that retains the required RED and
positive control. Consolidation of the physical representation is therefore a
legitimate lane for lowering the debt; retirement of the obligation is not.

If an envelope ever forces the choice, the ladder wins and the envelope is
amended -- which is the whole reason amendment exists.

## An open quantity this proposal deliberately does not assert

Whether, and how steeply, floor runtime grows with authored witnesses. The authored
population is re-derived by `git grep -cE '^\s*test fn ' <rev> -- '*.dag'` over
any two revisions, and it grows steeply. But the topology filters hard: the
floor's own `required-floor: declared=N offered=N routed=N ...` census line
(`v1_compiler.bin.claim_executor`) is the producer for what survives to run, and
it reports a small fraction of the declared population. Declaration count is
therefore NOT the demand unit, and a
claim that a speedup would be consumed by growth needs a measured slope from
added identities to routed closures, compile groups and critical-path time.
That slope is the first instrument this authority needs and is not yet
measured. Until it is, growth is a motive for building the authority and not
evidence for any particular envelope.

## Relation to the performance lane

An aggregate authority does not make the floor faster. It makes the floor's
speed a governed quantity instead of a side effect.
